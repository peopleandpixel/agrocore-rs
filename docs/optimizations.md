# AgroCore-RS Optimierungs-Tasks (nur offen / aktiv)

Priorität: P0 zuerst, dann P1, P2, P3, P4.

P0 (kritisch / höchster Nutzen):
- 8. KI-Analytics & Mobile-First (P4 — strategisch, aber als P0 für nächste Phase). Datei: docs/tasks.md. Aktion: Phase starten sobald Monitoring abgeschlossen; nicht vor v0.10.0. Status: Offen.

P1 (wichtig / kleiner Aufwand):
- 3. Profiling-Skript erstellen. Datei: scripts/profile.sh. Aktion: perf record + flamegraph/samply für Release-Build erstellen. Status: Offen.
- 1. Monitoring-Overhead minimieren. Datei: crates/api/src/metrics.rs. Aktion: Makro optional einbinden. Status: Offen.

P2 (mittel / geplanter Nutzen):
- 4. Pool-Metrik-Timer integriert — Status: erledigt (nicht mehr offen, nur Referenz).
- 5. Slow-Query-Threshold konfigurierbar. Datei: crates/api/src/metrics.rs. Aktion: Env-Var SLOW_QUERY_MS. Status: Offen.
- 7. Feature-Flag konsolidieren. Datei: crates/api/Cargo.toml. Aktion: cfg(feature="metrics"). Status: Offen.
- 8. KI-Analytics (P4, als P2 für nächste Planung). Status: Offen.

P3 (langfristig / niedriger Priorität):
- 6. Alternative Allocator einbinden (P3). Datei: crates/api/Cargo.toml + main.rs. Status: Offen.

Crates-Analyse — nur offen:

OPT-008 Domain lazy-loading (P2). Datei: crates/domain/src/lib.rs. Aktion: Mock-Branch lazy-loading (OnceLock). Status: erledigt.

OPT-009 Reporting-Service Timeout und Paginierung (P2). Datei: crates/reporting-service/src/main.rs. Status: Offen.

OPT-010 Weather-Service und Geometry-Service Timeout (P3). Datei: crates/weather-service/src/main.rs + geometry-service/src/main.rs. Status: Offen.

Abgeschlossen / Abgebrochen (nur zur Information, nicht mehr aktiv):
- 001, 002, 004, 005, 006, 007, 008: erledigt
- 003: ABGESAGT (Leptos 0.8.x bleibt)
- 2: erledigt (Build-Optimierung aktiv)
- 1: erledigt (Monitoring-Toggle aktiv)
