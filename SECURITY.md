# Security Policy

## Supported Versions

We provide security updates for the following versions:

| Version | Supported | End of Support |
|---------|-----------|----------------|
| 0.13.x  | ✅ Yes    | Until 0.14.0   |
| 0.12.x  | ✅ Yes    | Until 0.13.1   |
| 0.11.x  | ⚠️ Critical only | 2026-09-30 |
| < 0.11  | ❌ No     | —              |

**Policy**: We backport critical security fixes to the two most recent minor release lines. Upgrade to the latest patch version for full protection.

---

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities via public GitHub issues.**

### Preferred: Private Disclosure

| Method | Details |
|--------|---------|
| **Email** | `security@peopleandpixel.com` (PGP key: `0x4A3B2C1D` on keyservers) |
| **GitHub Security Advisories** | [Report a vulnerability](https://github.com/peopleandpixel/agrocore-rs/security/advisories/new) (private) |

### What to Include
- Description of the vulnerability
- Steps to reproduce (minimal proof-of-concept)
- Affected versions/components
- Potential impact (data exposure, RCE, DoS, privilege escalation, etc.)
- Suggested fix (if any)

### Response Timeline
| Phase | Target |
|-------|--------|
| **Acknowledgment** | ≤ 48 hours |
| **Initial Assessment** | ≤ 5 business days |
| **Fix Development** | ≤ 30 days (critical: ≤ 7 days) |
| **Coordinated Disclosure** | After fix released + 14 days grace period |

We credit reporters in the release notes (unless anonymity requested).

---

## Security Architecture

### Authentication & Authorization
- **JWT** – RS256, 30-minute expiry, rotating keys via JWKS
- **Passwords** – Argon2id (memory-hard, configurable params)
- **Rate Limiting** – 120 req/min/IP (actix-governor, per-route configurable)
- **Roles** – `admin`, `manager`, `worker`, `viewer` (RBAC via middleware)

### Transport Security
- **TLS 1.3** – enforced in production (Docker: Traefik/Caddy termination)
- **HSTS** – `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- **CSP** – restrictive Content-Security-Policy headers
- **X-Frame-Options** – `DENY`

### Data Protection
- **At Rest** – AES-256-GCM for backups (envelope encryption with Age)
- **In Transit** – TLS 1.3 for all external connections (NATS, S3, DB, APIs)
- **Secrets** – Never in images/logs; injected via Docker secrets / env files
- **PII** – Minimal collection; tenant isolation at DB row level

### Supply Chain
- **cargo-audit** – runs in CI on every push
- **cargo-deny** – checks advisories, licenses, bans, sources
- **Dependencies** – pinned versions in `Cargo.lock`; `dependabot` alerts enabled
- **SBOM** – generated on release (`cargo cyclonedx`)

### Infrastructure
- **Non-root containers** – all Docker images run as `appuser` (UID 1000)
- **Read-only rootfs** – where possible
- **Capability drop** – `CAP_DROP=ALL` in compose/prod
- **Seccomp/AppArmor** – default profiles

---

## Secure Configuration Checklist (Production)

> **Run this before deploying to production**

### Environment Variables
- [ ] `JWT_SECRET` – 256-bit random, rotated quarterly
- [ ] `DATABASE_URL` – TLS mode `require`, cert verification
- [ ] `NATS_URL` – TLS + user/pass or NKey auth
- [ ] `BACKUP_ENCRYPTION_KEY` – Age/GPG key, stored in secret manager
- [ ] `S3_*` / `AZURE_*` / `GCS_*` – least-privilege IAM roles
- [ ] `AGROCORE_LOG__OTLP_ENDPOINT` – if using distributed tracing

### Docker / Orchestration
- [ ] `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d`
- [ ] Secrets via `docker secret` / Kubernetes `Secret` (not `.env`)
- [ ] Network policies: API ↔ DB/NATS only; Admin UI ↔ API only
- [ ] Resource limits: CPU/memory requests + limits on all containers
- [ ] Health checks: `/health` (liveness), `/ready` (readiness)

### Database
- [ ] `pg_hba.conf` – TLS + scram-sha-256 only
- [ ] Row-level security policies for multi-tenancy
- [ ] Automated backups + point-in-time recovery tested quarterly
- [ ] Connection pooling (PgBouncer) for production scale

### Monitoring & Alerting
- [ ] Prometheus scraping `/metrics` (API, backup, scheduler, reporting)
- [ ] Alert rules: failed backups, auth anomalies, rate limit hits, disk space
- [ ] Log aggregation (Loki/ELK) with `agrocore_logging` structured fields
- [ ] Distributed tracing (OTLP → Tempo/Jaeger) enabled

---

## Known Security Considerations

| Area | Risk | Mitigation |
|------|------|------------|
| **LPIS WFS** | External HTTP calls to gov endpoints | Timeouts (30s), retry with backoff, response size limits, no auth credentials in URLs |
| **File Uploads** | Shapefile/GeoJSON import | Size limits, validation via `geozero`, sandboxed parsing |
| **Backup Encryption** | Key management | Envelope encryption (Age), keys never in config, rotation documented |
| **NATS** | Unauthenticated in dev | `allow_anonymous` only in dev; prod requires TLS + auth |
| **Admin UI (WASM)** | Client-side only | All authz enforced server-side; UI hides but doesn't enforce |

---

## Security Hardening Guide (For Deployers)

### 1. Network Segmentation
```yaml
# docker-compose.prod.yml snippet
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # no internet access

services:
  api:
    networks: [backend, frontend]
  admin-ui:
    networks: [frontend]
  postgres:
    networks: [backend]
  nats:
    networks: [backend]
```

### 2. Reverse Proxy (Traefik Example)
```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.api.rule=Host(`api.farm.local`)"
  - "traefik.http.routers.api.tls=true"
  - "traefik.http.routers.api.tls.certresolver=letsencrypt"
  - "traefik.http.middlewares.security.headers.stsseconds=31536000"
  - "traefik.http.middlewares.security.headers.stsincludesubdomains=true"
  - "traefik.http.middlewares.security.headers.contentsecuritypolicy=default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:;"
```

### 3. Database Hardening
```sql
-- Run as superuser after migrations
REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO agrocore_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO agrocore_app;

-- Enable RLS on tenant tables
ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON sites USING (tenant_id = current_setting('app.current_tenant')::uuid);
```

---

## Cryptographic Standards

| Purpose | Algorithm | Library |
|---------|-----------|---------|
| Password Hashing | Argon2id (m=65536, t=3, p=4) | `argon2` crate |
| JWT Signing | RS256 (2048-bit RSA) | `jsonwebtoken` |
| Backup Encryption | AES-256-GCM + Age (X25519) | `age` / `aes-gcm` |
| TLS | TLS 1.3, X25519, AES-256-GCM | `rustls` |
| API Keys | SHA-256 hashed storage | `sha2` |

---

## Incident Response Plan

1. **Detect** – Monitoring alert / user report / audit finding
2. **Triage** – Severity (Critical/High/Medium/Low), affected components
3. **Contain** – Revoke tokens, block IPs, disable features, rotate keys
4. **Eradicate** – Deploy fix, rebuild images, rotate all credentials
5. **Recover** – Verify fix, restore from clean backup if needed, monitor
6. **Postmortem** – Root cause, timeline, action items, publish advisory

---

## Security Resources

| Resource | Link |
|----------|------|
| **Rust Secure Code Guidelines** | https://anssi-fr.github.io/rust-guide/ |
| **OWASP Top 10** | https://owasp.org/www-project-top-ten/ |
| **Cargo Audit** | https://github.com/RustSec/cargo-audit |
| **Cargo Deny** | https://github.com/EmbarkStudios/cargo-deny |
| **GitHub Security Advisories** | https://github.com/peopleandpixel/agrocore-rs/security/advisories |

---

## Contact

| Purpose | Channel |
|---------|---------|
| **Vulnerability Report** | `security@peopleandpixel.com` or [GitHub Security](https://github.com/peopleandpixel/agrocore-rs/security/advisories/new) |
| **General Security Questions** | [GitHub Discussions](https://github.com/peopleandpixel/agrocore-rs/discussions/categories/security) |
| **Commercial Security Review** | `info@peopleandpixel.com` |

---

**Last Updated:** 2026-09-04  
**Next Review:** 2026-12-04 (quarterly)

*This policy follows GitHub's recommended security policy format and Rust ecosystem best practices.*