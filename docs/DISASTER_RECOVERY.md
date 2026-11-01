# Disaster Recovery Runbook — Agrocore

## Overview

This runbook describes the procedures for recovering the Agrocore system after a catastrophic failure. The target **Recovery Time Objective (RTO)** is **< 15 minutes** for a 50 GB database.

## Prerequisites

- Access to backup storage (S3, SFTP, or local)
- PostgreSQL credentials for the new instance
- AWS/GCP/Azure credentials if using KMS encryption
- Age private key if using Age encryption

## Recovery Procedures

### 1. Assess the Damage

```bash
# Check if PostgreSQL is running
systemctl status postgresql

# Check disk space
df -h

# Check if backup storage is accessible
aws s3 ls s3://agrocore-backups/
# or for SFTP:
sftp -i ~/.ssh/backup_key backup@backup-server:/backups/
```

### 2. Provision New Infrastructure

#### Option A: Docker Compose (Fastest)

```bash
# Clone the repository if not present
git clone https://github.com/agrocore/agrocore-rs.git
cd agrocore-rs

# Start PostgreSQL
docker-compose -f docker-compose.postgres.yml up -d postgres

# Wait for PostgreSQL to be ready
sleep 10
```

#### Option B: Cloud Database

```bash
# AWS RDS
aws rds create-db-instance \
  --db-instance-identifier agrocore-recovery \
  --engine postgres \
  --engine-version 16 \
  --db-instance-class db.r6g.large \
  --master-username agrocore \
  --master-user-password "$(cat /secure/password)"

# GCP Cloud SQL
gcloud sql instances create agrocore-recovery \
  --database-version=POSTGRES_16 \
  --tier=db-custom-2-7680 \
  --region=europe-west1
```

### 3. Restore Database Backup

#### Find Latest Backup

```bash
# List recent backups
aws s3 ls s3://agrocore-backups/daily/ --recursive | sort -k 1 | tail -5

# For SFTP
sftp backup@backup-server << 'EOF'
ls -lt /backups/daily/
EOF
```

#### Download and Restore

```bash
# Download latest backup
LATEST=$(aws s3 ls s3://agrocore-backups/daily/ --recursive | sort -k 1 | tail -1 | awk '{print $4}')
aws s3 cp "s3://agrocore-backups/daily/$LATEST" ./restore.sql.gz

# If encrypted with Age:
age --decrypt -i ~/.config/agrocore/age-key.txt < restore.sql.gz > restore.sql

# If encrypted with KMS (AWS):
aws kms decrypt \
  --ciphertext-blob fileb://restore.sql.gz \
  --output text \
  --query Plaintext | base64 -d > restore.sql

# Decompress if needed
gunzip -c restore.sql.gz > restore.sql 2>/dev/null || true

# Restore to PostgreSQL
PGPASSWORD="$DB_PASSWORD" pg_restore \
  --dbname=agrocore \
  --host=localhost \
  --port=5432 \
  --username=agrocore \
  --no-owner \
  --no-privileges \
  --jobs=4 \
  restore.sql
```

### 4. Verify Recovery

```bash
# Check database connectivity
PGPASSWORD="$DB_PASSWORD" psql -h localhost -U agrocore -d agrocore -c "SELECT count(*) FROM farms;"

# Run health check
curl http://localhost:8080/health

# Check application logs
journalctl -u agrocore-api --since "5 minutes ago"
```

### 5. Restart Application

```bash
# Docker Compose
docker-compose up -d

# Systemd
systemctl restart agrocore-api
systemctl restart agrocore-admin-ui

# Verify
curl http://localhost:8080/health
```

### 6. Post-Recovery Checklist

- [ ] Database restored and accessible
- [ ] Application health check passes
- [ ] All tenants can authenticate
- [ ] Backup schedule is running
- [ ] Monitoring alerts are active
- [ ] DNS points to new instance (if applicable)
- [ ] SSL certificates are valid
- [ ] NATS connection is established
- [ ] Notification channels are configured

## RTO Targets

| Component | Target | Method |
|-----------|--------|--------|
| PostgreSQL | < 10 min | pg_restore with --jobs=4 |
| Application | < 2 min | Docker Compose / systemd |
| DNS | < 5 min | Low TTL pre-configured |
| **Total** | **< 15 min** | |

## Backup Verification Schedule

- **Daily**: Automated backup integrity check (checksum verification)
- **Weekly**: Test restore to staging environment
- **Monthly**: Full disaster recovery drill

## Contact Information

| Role | Name | Contact |
|------|------|---------|
| On-Call | TBD | TBD |
| DBA | TBD | TBD |
| Infrastructure | TBD | TBD |
