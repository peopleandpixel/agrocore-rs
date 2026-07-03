# <img src="docs/agrocore_RS.png" width="48" height="48" alt="AgroCore Logo"> AgroCore RS

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

AgroCore RS is a farm operations platform built from scratch in this repository. The code, structure, naming, and product decisions were created here independently, while the implementation benefits from practical experience gained in earlier professional work.

The goal is simple: give the office a single system that stays usable under real farm conditions. AgroCore brings together field management, tasks, weather, livestock, finance, compliance, equipment, and administration in one workflow instead of scattering daily work across isolated tools.

## Why AgroCore?

AgroCore is designed for teams that need to record, check, and later evaluate operational data without fighting the software.

- One place for office-side farm administration
- Fast capture of fields, jobs, livestock, resources, and compliance data
- Setup-driven workflows so a clean system starts clean
- Internationalized UI for multilingual teams
- Practical admin screens instead of a marketing shell
- Local development flow that avoids unnecessary container rebuilds

## What Makes It Useful

- Field and site management with a focus on real operational data
- Task and order handling for office and planning workflows
- Weather and phenology views, including fallback weather lookup when no station exists
- Livestock records and treatments
- Finance, cost centers, PAC-related workflows, and reporting
- Compliance and certification tracking
- Equipment and resource management
- A reactive Admin UI for day-to-day office work

## Technology

- Rust 2024
- Actix Web for the API
- Leptos for the Admin UI
- MongoDB for persistence
- NATS for messaging
- Docker for local infrastructure

## Repository Layout

- `crates/api`: HTTP API and request handlers
- `crates/admin-ui`: Web UI
- `crates/domain`: Business logic and entities
- `crates/infrastructure`: Persistence and integrations
- `crates/messaging`: Event and messaging support

## Local Development

The repository includes a host-based development flow so the services can run without rebuilding containers on every change.

```bash
./scripts/dev.sh
```

That script starts the local infrastructure, runs the API, and serves the Admin UI.

### Direct Startup

```bash
cargo run -p agrocore-api
```

```bash
cd crates/admin-ui
trunk serve
```

## Docker Compose

Docker Compose is still available for a fully containerized run, but the local development script is the better choice for day-to-day work.

## License

This project is licensed under the GNU GPL v3.0 or later.
