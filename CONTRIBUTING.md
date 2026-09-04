# Contributing to AgroCore RS

Thank you for your interest in contributing! This project is built **for farmers, by developers who listen to them**. Every contribution – code, docs, translations, bug reports, ideas – helps make farm office work easier.

---

## 🚀 Quick Start for Contributors

```bash
# 1. Fork & clone
git clone https://github.com/YOUR-USERNAME/agrocore-rs.git
cd agrocore-rs

# 2. Install Rust 2024
rustup install stable
rustup component add rustfmt clippy

# 3. Run quality gates (MUST pass before commit)
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace --features=mocks

# 4. Start developing
./scripts/dev.sh  # starts infra + API + Admin UI
```

---

## 📋 Development Workflow

### Branch Naming
- `feat/<short-description>` – new feature
- `fix/<short-description>` – bug fix
- `docs/<short-description>` – documentation
- `refactor/<short-description>` – code restructuring
- `chore/<short-description>` – maintenance

### Commit Messages (Conventional Commits)
```
feat: add INVEKOS provider for Austria
fix: backup-service encryption key rotation
docs: update LPIS provider guide
refactor: consolidate BaseClient caching logic
chore: bump version to 0.13.0
```

### Pull Request Checklist
- [ ] Quality gates pass locally (`cargo fmt/check/clippy/test`)
- [ ] Tests added/updated for new functionality
- [ ] CHANGELOG.md updated (Keep a Changelog format)
- [ ] Documentation updated if API/UI changed
- [ ] DCO sign-off: `git commit -s` (Developer Certificate of Origin)

---

## 🏗 Project Structure Deep Dive

```
crates/
├── api              # Actix Web HTTP API, handlers, middleware, OpenAPI
├── admin-ui         # Leptos 0.8 WASM frontend (Tailwind, i18n, PWA)
├── asset-registry   # Equipment, Building, Tree, Variety, Breed entities
├── backup-service   # pg_dump streaming, AES-256-GCM, GFS, 9 backends
├── dashboard        # TUI (Ratatui) – service control, metrics, sparklines
├── domain           # Core entities, repository traits, business logic
├── geometry-service # PostGIS operations, geospatial queries
├── infrastructure   # PostgreSQL repositories (pg_repo! macro), DB wiring
├── i18n-shared      # 10-language translation keys (fluent-rs)
├── lpis-providers   # 8 country providers + BaseClient (caching, rate-limit)
├── logging          # Unified logging (tracing bridge, OTLP, macros)
├── messaging        # NATS Publisher/Subscriber traits, request-reply
├── reporting-service# Excel/GeoJSON/PAC-SIP export worker
├── scheduler        # Cron + OneTime jobs, NATS events, handler registry
├── shared           # Config, macros, error types, common utilities
└── weather-service  # Open-Meteo ingestion, station fallback
```

### Key Architectural Patterns

| Pattern | Where | Purpose |
|---------|-------|---------|
| **Repository Trait** | `domain/src/repositories.rs` | Abstract persistence, enable mocking |
| **pg_repo! Macro** | `infrastructure/src/postgres/` | Boilerplate-free Postgres repos |
| **BaseClient** | `lpis-providers/src/base.rs` | Unified caching/rate-limit/retry for all LPIS |
| **ServiceContext** | `logging/src/context.rs` | Automatic span enrichment (tenant, request, user) |
| **JobDefinition** | `scheduler/src/lib.rs` | Declarative job config (Cron/OneTime/Builtin) |

---

## 🌍 Adding a New LPIS Provider (Step-by-Step)

This is a **high-impact contribution** – each provider unlocks a new country for farmers.

### 1. Understand the Target System
Research the country's LPIS:
- **WFS endpoint** (GetCapabilities, DescribeFeatureType, GetFeature)
- **Authentication** (API key, OAuth, none, IP allowlist)
- **Schema** (feature type names, geometry column, CRS, key fields)
- **Rate limits** / pagination / max features
- **Language** of field names / enums

### 2. Create Provider Module
```
crates/lpis-providers/src/
├── <country_code>.rs    # e.g., sigpac.rs, brp.rs, ilpis.rs
└── mod.rs               # add: pub mod <country_code>;
```

### 3. Implement `LpisProvider` Trait
```rust
// In crates/lpis-providers/src/<country_code>.rs
use agrocore_shared::lpis::{LpisProvider, LpisParcel, LpisCountry, LpisError};
use crate::base::BaseClient;
use async_trait::async_trait;

pub struct <Country>Provider {
    client: BaseClient,
}

impl <Country>Provider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            client: BaseClient::new(config),
        }
    }
}

#[async_trait]
impl LpisProvider for <Country>Provider {
    async fn search_parcels(&self, request: SearchRequest) -> Result<Vec<LpisParcel>, LpisError> {
        // 1. Build WFS GetFeature request (CQL filter, bbox, pagination)
        // 2. Call self.client.get_with_retry(url).await?
        // 3. Parse GML/GeoJSON response → Vec<LpisParcel>
        // 4. Normalize to common schema (parcel_id, geometry, crop_code, area_ha, ...)
    }

    async fn get_parcel_by_id(&self, id: &str) -> Result<Option<LpisParcel>, LpisError> { ... }

    fn country(&self) -> LpisCountry {
        LpisCountry::<CountryCode>  // e.g., LpisCountry::ES
    }

    fn provider_name(&self) -> &'static str {
        "<COUNTRY> <System>"  // e.g., "Spain SIGPAC"
    }
}
```

### 4. Register in Registry
```rust
// In crates/lpis-providers/src/lib.rs → create_default_registry()
registry.register(Arc::new(<country_code>::<Country>Provider::new(
    crate::config::ProviderConfig {
        base_url: "https://<official-wfs-endpoint>/wfs".to_string(),
        ..Default::default()
    },
)));
```

### 5. Add to Admin UI (i18n + Country Dropdown)
- `crates/i18n-shared/src/locales/<lang>.ftl` – add `lpis-country-<code>` key for all 10 languages
- `crates/admin-ui/src/components/lpis_import.rs` – country dropdown auto-populates from registry

### 6. Test with Real Data
```bash
# Set env vars for the provider
export AGROCORE_LPIS_<CODE>_BASE_URL="https://..."
export AGROCORE_LPIS_<CODE>_ENABLED=true

# Run integration test
cargo test --package lpis-providers --features=mocks -- <country_code>::tests --nocapture
```

### 7. Document
- Update README.md LPIS table
- Add entry in CHANGELOG.md
- Optional: blog post / demo screenshots

---

## 🧪 Testing Standards

| Test Type | Location | Command |
|-----------|----------|---------|
| **Unit** | `crates/*/src/*_test.rs` or `tests/` subdir | `cargo test --package <crate>` |
| **Integration** | `crates/api/tests/` (PostgreSQL) | `cargo test --workspace --features=mocks` |
| **Contract** | `crates/api/tests/api_contract_test.rs` | `cargo test --test api_contract_test --features=mocks` |
| **LPIS Providers** | `crates/lpis-providers/src/*/tests.rs` | `cargo test --package lpis-providers` |

### Test Rules
- **Tests in separate `tests/` directories** (not inline `#[cfg(test)]`)
- **Mock external services** (NATS, HTTP, DB) via `mocks` feature
- **No `unwrap()`/`expect()` in tests** – use `?` and `assert!`
- **Test names**: `test_<function>_<scenario>_<expected>`

---

## 🌐 Internationalization (i18n)

**10 Languages**: DE (default), EN, ES, FR, PT, IT, PL, RO, UK, NL

### Adding Translation Keys
1. Edit `crates/i18n-shared/src/locales/de.ftl` (source language)
2. Run extraction: `cargo run -p i18n-extract` (if tool exists) or manually copy to other `.ftl` files
3. Keys follow pattern: `feature-section-element` (e.g., `sites-form-area-hectares`)

### Adding a New Language
1. Copy `de.ftl` → `<lang>.ftl` in `crates/i18n-shared/src/locales/`
2. Translate all keys
3. Add to `crates/i18n-shared/src/lib.rs` supported languages list
4. Add to Admin UI language selector

---

## 📝 Documentation Standards

- **README** – user-facing, practical, screenshots
- **CHANGELOG** – Keep a Changelog, every version
- **Doc comments** – `///` for public APIs, include examples
- **Architecture decisions** – `docs/adr/<number>-<title>.md` (if needed)

---

## 🔧 Code Style & Quality

Enforced by CI – run locally before pushing:

```bash
# Format
cargo fmt --all

# Check (compiles, no warnings)
cargo check --workspace

# Lint (pedantic, nursery, style -D warnings)
cargo clippy --workspace -- -D warnings \
  -W clippy::pedantic \
  -W clippy::nursery \
  -W clippy::style \
  -A clippy::module_name_repetitions \
  -A clippy::too_many_arguments

# Test
cargo test --workspace --features=mocks

# Security
cargo audit
cargo deny check advisories licenses bans sources
```

### Rust Edition & MSRV
- **Edition**: 2024 (enforced in `Cargo.toml`)
- **MSRV**: Latest stable Rust (tracked via `rust-toolchain.toml`)

### Forbidden Patterns
- ❌ `unwrap()` / `expect()` in production code (use `?`, `anyhow`, `thiserror`)
- ❌ `todo!()` / `unimplemented!()` / stubs – **complete implementations only**
- ❌ Direct `tracing` imports in business logic – use `agrocore_logging` macros
- ❌ Direct `async_nats` in business logic – use `agrocore_messaging` traits
- ❌ Direct `tokio::time::interval` for recurring tasks – use `agrocore_scheduler`

---

## 🏷️ Release Process

1. Update version in `Cargo.toml` (workspace.package.version)
2. Update `CHANGELOG.md` with `[Unreleased] → [X.Y.Z] - YYYY-MM-DD`
3. Commit: `chore: release X.Y.Z`
4. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. CI creates GitHub Release with WASM artifact + changelog

---

## 🤝 Community Guidelines

- **Be respectful** – farmers, developers, hobbyists all welcome
- **No gatekeeping** – questions are contributions too
- **Practical over perfect** – solves real farm problems first
- **Local-first mindset** – cloud is optional, never required

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for full policy (Contributor Covenant 2.1).

---

## 💬 Getting Help

| Channel | Purpose |
|---------|---------|
| [GitHub Discussions](https://github.com/peopleandpixel/agrocore-rs/discussions) | Questions, ideas, show & tell |
| [GitHub Issues](https://github.com/peopleandpixel/agrocore-rs/issues/new/choose) | Bugs, feature requests, tasks |
| Email | Commercial: `info@peopleandpixel.com` |

---

## 📜 Legal

By contributing, you agree that your contributions will be licensed under **GPL v3.0 or later** (same as the project). You certify that you have the right to submit the work (DCO sign-off via `git commit -s`).

---

**Happy coding! 🌾**  
*Every line of code here helps a farmer spend less time on paperwork and more time in the field.*