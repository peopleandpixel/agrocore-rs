# <img src="docs/agrocore_RS.png" width="48" height="48" alt="AgroCore Logo"> AgroCore RS

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Rust CI](https://github.com/peopleandpixel/agrocore-rs/actions/workflows/ci-cd.yml/badge.svg?branch=main)](https://github.com/peopleandpixel/agrocore-rs/actions/workflows/ci-cd.yml)
[![Version](https://img.shields.io/badge/version-0.15.0-green.svg)](CHANGELOG.md)
[![GitHub commit activity](https://img.shields.io/github/commit-activity/m/peopleandpixel/agrocore-rs)](https://github.com/peopleandpixel/agrocore-rs/commits/main)
[![Tests](https://img.shields.io/badge/tests-153%2B-brightgreen.svg)](https://github.com/peopleandpixel/agrocore-rs/actions/workflows/ci-cd.yml)
[![Crates](https://img.shields.io/badge/crates-15-blue.svg)](#repository-layout)
[![Docker](https://img.shields.io/badge/docker-ghcr.io-blue.svg)](https://github.com/peopleandpixel/agrocore-rs/pkgs/container/agrocore-rs)
[![Coverage](https://img.shields.io/badge/coverage-lcov.info-orange.svg)](lcov.info)

> **Farm operations platform** – fields, tasks, livestock, finance, compliance, weather, equipment & administration in one **local-first**, **open-source** system. **8 European LPIS providers** (SIGPAC, BRP, RPG, iLPIS, SIAN, LPIS-DE, LPIS-PL, INVEKOS) built-in. Rust 2024 · Actix Web · Leptos · PostgreSQL/PostGIS · NATS.

---

## 🎯 Why AgroCore?

Built for farm offices that need to **record, check, and evaluate** operational data without fighting the software.

- **One system** for the entire office: fields, jobs, livestock, resources, compliance, finance, equipment
- **Fast capture** – setup-driven workflows, clean system starts clean
- **Multilingual UI** – 10 languages (DE, EN, ES, FR, PT, IT, PL, RO, UK, NL)
- **Local-first, cloud-optional** – data stays on your farm; distribute services only if you want
- **Automated Backup & Recovery** – encrypted, multi-target (S3, MinIO, Azure, GCS, Local, SFTP, WebDAV), GFS retention
- **Built-in Scheduler** – recurring worker tasks + one-time appointments via `agrocore-scheduler`
- **LPIS Import for 8 countries** – Schlagdaten in 2 Klicks importieren (SIGPAC/ES, BRP/NL, RPG/FR, iLPIS/PT, SIAN/IT, LPIS/DE, LPIS/PL, INVEKOS/AT)

---

## 🌍 LPIS Providers (8 Countries)

| Country | System | Provider | Status |
|---------|--------|----------|--------|
| 🇪🇸 Spain | SIGPAC | `sigpac` | ✅ Full WFS |
| 🇳🇱 Netherlands | BRP | `brp` | ✅ Full WFS |
| 🇫🇷 France | RPG | `rpg` | ✅ Full WFS |
| 🇵🇹 Portugal | iLPIS | `ilpis` | ✅ Full WFS |
| 🇮🇹 Italy | SIAN | `sian` | ✅ Full WFS |
| 🇩🇪 Germany | LPIS | `lpis_de` | ✅ Full WFS |
| 🇵🇱 Poland | LPIS | `lpis_pl` | ✅ Full WFS |
| 🇦🇹 Austria | INVEKOS | `invkos` | ✅ Full WFS |

All providers share a unified **BaseClient** with caching (Memory/Redis), rate limiting (Governor), retry logic, and configurable timeouts.

---

## 🖥️ Admin UI Preview

> **Screenshots coming soon** – the Admin UI (Leptos 0.8 / WASM) includes:
> - Dashboard with system status & business metrics
> - Site/Field management (CRUD, crop, variety, area)
> - Tasks & Orders (recurring, worker assignment, GPS clock-in/out)
> - Livestock (animals, treatments, movements)
> - Weather integration (Open-Meteo + station fallback)
> - Finance (PAC applications, cost centers)
> - Equipment (maintenance, fuel, depreciation)
> - Compliance tracking
> - Import Wizard (SIGPAC, BRP, RPG, iLPIS, GeoJSON, Shapefile)
> - Settings (LPIS providers, backup targets, scheduler jobs)
> - Dark/Light theme, PWA support, full i18n

---

## 🛠 Technology Stack

| Layer | Technology |
|-------|------------|
| **Language** | Rust 2024 Edition |
| **API** | Actix Web 4, JWT (30 min), Argon2, Governor rate limiting (120 req/min) |
| **Frontend** | Leptos 0.8 (WASM), reactive, SSR-ready, Tailwind CSS |
| **Database** | PostgreSQL 16 + PostGIS, 15 migrations (34 tables), sqlx compile-time checked |
| **Messaging** | NATS 2.10 + JetStream, request-reply, pub/sub |
| **Infrastructure** | Docker Compose (prod-ready), systemd units, K8s manifests |
| **Observability** | Prometheus metrics, structured logging (`agrocore-logging`), OTLP-ready |
| **Testing** | 153+ tests (unit + integration), cargo-audit, cargo-deny |

---

## 📦 Repository Layout

```
crates/
├── api              # HTTP API & request handlers
├── admin-ui         # Leptos WASM Admin UI
├── asset-registry   # Equipment, buildings, trees, varieties, breeds
├── backup-service   # Backup & Recovery (pg_dump streaming, AES-256-GCM, GFS)
├── dashboard        # TUI Dashboard (service control, sparklines)
├── domain           # Business logic & entities
├── geometry-service # Geospatial operations (PostGIS)
├── infrastructure   # Persistence & integrations (Postgres repos)
├── i18n-shared      # Shared i18n keys (10 languages)
├── lpis-providers   # 8 LPIS providers (BaseClient pattern)
├── logging          # Unified structured logging (tracing bridge)
├── messaging        # NATS publisher/subscriber traits
├── reporting-service# Excel/GeoJSON/PAC-SIP export worker
├── scheduler        # Reusable scheduler (Cron + OneTime jobs)
├── shared           # Common types, config, macros
└── weather-service  # Weather data ingestion (Open-Meteo)
```

---

## 🚀 Quick Start

### Prerequisites
- Rust 2024 (`rustup install stable`)
- Docker & Docker Compose
- PostgreSQL 16 + PostGIS (or use Docker)

### Local Development (host-based, no container rebuilds)

```bash
# 1. Start infrastructure + API + Admin UI
./scripts/dev.sh

# 2. Or start services individually:
cargo run -p agrocore-api          # API on :8080
cd crates/admin-ui && trunk serve  # Admin UI on :3000
```

### Docker Compose (Production-Ready)

```bash
docker compose up -d
```

**Services started:**
- `postgres` – PostgreSQL 16 + PostGIS (auto-migrations, 15 files)
- `nats` – NATS 2.10 with JetStream
- `api` – Actix Web API on port 8080
- `backup-service` – Backup & Recovery sidecar
- `reporting-service` – Export worker (Excel, GeoJSON, PAC-SIP)
- `weather-service` – Weather ingestion worker
- `admin-ui` – Leptos Admin UI on port 3000

All services include **health checks** and proper **dependency ordering**.

### Docker Images (GHCR)

```bash
docker pull ghcr.io/peopleandpixel/agrocore-api:latest
docker pull ghcr.io/peopleandpixel/agrocore-admin-ui:latest
docker pull ghcr.io/peopleandpixel/agrocore-backup-service:latest
# ... etc for all services
```

---

## 🧪 Testing & Quality Gates

```bash
# Full test suite (153+ tests)
cargo test --workspace --features=mocks

# Quality gates (run before every commit!)
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace --features=mocks
cargo audit
```

**CI/CD Pipeline** runs on every push/PR:
1. **Quality Gates** – fmt, check, clippy, api contract validation
2. **Tests** – unit + integration with PostgreSQL/PostGIS
3. **WASM Build** – Admin UI compilation
4. **Docker Build** – multi-stage images pushed to GHCR
5. **Security Audit** – cargo-audit + cargo-deny
6. **Release** – automatic on git tags

---

## 🔒 Security

- **JWT** – 30 min expiry, secure defaults
- **Passwords** – Argon2id hashing
- **Rate Limiting** – 120 req/min per IP (actix-governor)
- **Headers** – X-Frame-Options, CSP, HSTS
- **Dependencies** – `cargo audit` + `cargo deny` in CI

**Report vulnerabilities:** See [SECURITY.md](SECURITY.md)

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [CHANGELOG.md](CHANGELOG.md) | Keep a Changelog format, all versions |
| [docs/tasks.md](docs/tasks.md) | Open tasks, roadmap, phase tracking |
| [docs/optimizations.md](docs/optimizations.md) | Performance optimization tasks (P0–P4) |
| [POSTGRES_MIGRATION_CHECKLIST.md](POSTGRES_MIGRATION_CHECKLIST.md) | Migration verification checklist |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guide, LPIS provider how-to |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Contributor Covenant 2.1 |

---

## 🤝 Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Development setup & workflow
- **Adding new LPIS providers** (step-by-step guide)
- Code style & quality gates
- Testing requirements
- DCO sign-off

---

## 💬 Support & Commercial Services

AgroCore is **open-source** (GPL v3) built for the agricultural community.

| Need | Contact                                                                                                                   |
|------|---------------------------------------------------------------------------------------------------------------------------|
| **Bug reports / Feature requests** | [GitHub Issues](https://github.com/peopleandpixel/agrocore-rs/issues)                                                     |
| **Questions / Discussion** | [GitHub Discussions](https://github.com/peopleandpixel/agrocore-rs/discussions)                                           |
| **Financial Support** | [GitHub Sponsors](https://github.com/sponsors/peopleandpixel) \| [Open Collective](https://opencollective.com/agrocore)   |
| **Commercial Support** | Custom Rust integration, hardware adapters, enterprise deployment → [lda@peopleandpixel.pt](mailto:lda@peopleandpixel.pt) |

---

## 📄 License

This project is licensed under the **GNU GPL v3.0 or later** – see [LICENSE](LICENSE).

> **tl;dr:** You can use, modify, distribute commercially, but modifications must stay open source under GPL v3. No warranty.

---

## 🙏 Acknowledgments

- Built with practical experience from professional farm software projects
- Inspired by real farm office workflows in Portugal, Germany, Spain, France
- Thanks to the Rust, Actix, Leptos, PostGIS, and NATS communities

---

**Made with 🌱 for farmers, by a developer who listens to them.**

*Questions? Open a [Discussion](https://github.com/peopleandpixel/agrocore-rs/discussions) or [Issue](https://github.com/peopleandpixel/agrocore-rs/issues/new/choose).*