# AgroCore-RS Optimierungs-Tasks (nur offen / aktiv)

Priorität: P0 zuerst, dann P1, P2, P3, P4.

P0 (kritisch / höchster Nutzen):
- **Phase 7: Migration auf externe Services — ERLEDIGT (v0.14.0)**
- **Phase 8: Dev Environment & Demo Mode — GEPLANT (P0)**

P1 (wichtig / kleiner Aufwand):
- 3. Profiling-Skript erstellen. Datei: scripts/profile.sh. Status: Offen.
- 1. Monitoring-Overhead minimieren. Datei: crates/api/src/metrics.rs. Status: Offen.

P2 (mittel / geplanter Nutzen):
- 5. Slow-Query-Threshold konfigurierbar. Datei: crates/api/src/metrics.rs. Status: Offen.
- 7. Feature-Flag konsolidieren. Datei: crates/api/Cargo.toml. Status: Offen.
- 8. KI-Analytics & Mobile-First (P4 — strategisch, als P2 für nächste Planung). Status: Offen.

P3 (langfristig / niedriger Priorität):
- 6. Alternative Allocator einbinden (P3). Datei: crates/api/Cargo.toml + crates/api/src/main.rs. Status: Offen.

Crates-Analyse — nur offen:

OPT-007 Shared async-trait und Arc-Klone (P3) — Status: erledigt (db_exec verbessert).

OPT-009 Reporting-Service Timeout und Paginierung (P2) — Status: erledigt (Paginierung 500, Timeout 30s, Tracing).

OPT-010 Weather-Service und Geometry-Service Timeout (P3) — Status: erledigt (Timeout-Struktur 30s eingebaut).

Abgeschlossen / Abgebrochen (nur zur Information, nicht mehr aktiv):
- 001, 002, 004, 005, 006, 007, 008, 009, 010: erledigt (+ Equipment-Suche in tasks.md)
- 003: ABGESAGT (Leptos 0.8.6 konsolidiert)
- 2: erledigt (Build-Optimierung aktiv: lto=fat/codegen-units=1/panic=abort/strip)
- 1: erledigt (Monitoring-Toggle AGROCORE_METRICS_ENABLED aktiv)
