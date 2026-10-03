# AgroCore-RS Optimization Tasks (open / active only)

Priority: P0 first, then P1, P2, P3, P4.

P0 (critical / highest benefit):
- **Phase 7: Migration to external services — DONE (v0.14.0)**
- **Phase 8: Dev Environment & Demo Mode — PLANNED (P0)**

P1 (important / low effort):
- 3. Create profiling script. File: scripts/profile.sh. Status: Open.
- 1. Minimize monitoring overhead. File: crates/api/src/metrics.rs. Status: Open.

P2 (medium / planned benefit):
- 5. Make slow-query threshold configurable. File: crates/api/src/metrics.rs. Status: Open.
- 7. Consolidate feature flags. File: crates/api/Cargo.toml. Status: Open.
- 8. AI analytics & mobile-first (P4 — strategic, treated as P2 for the next planning round). Status: Open.

P3 (long-term / lower priority):
- 6. Integrate an alternative allocator (P3). File: crates/api/Cargo.toml + crates/api/src/main.rs. Status: Open.

Crate analysis — open only:

OPT-007 Shared async-trait and Arc clones (P3) — Status: done (db_exec improved).

OPT-009 Reporting-service timeout and pagination (P2) — Status: done (pagination 500, timeout 30s, tracing).

OPT-010 Weather-service and Geometry-service timeout (P3) — Status: done (30s timeout structure built in).

Completed / Cancelled (for information only, no longer active):
- 001, 002, 004, 005, 006, 007, 008, 009, 010: done (+ equipment search in tasks.md)
- 003: CANCELLED (consolidated in Leptos 0.8.6)
- 2: done (build optimization active: lto=fat/codegen-units=1/panic=abort/strip)
- 1: done (monitoring toggle AGROCORE_METRICS_ENABLED active)
