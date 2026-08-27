# AgroCore-RS Optimierungs-Tasks

1. Monitoring-Overhead minimieren (P1). Datei: crates/api/src/metrics.rs. Aktion: Makro measure_sqlx_query! optional einbinden (nur wenn is_enabled() true), nicht pro Query erzwingen. Begründung: Perf Book — Instrumentierung nur im heißen Pfad, sonst I/O-Overhead.
2. Release-Profil optimiert (P0). Datei: Cargo.toml (fertig). Aktion: lto=fat, codegen-units=1, panic=abort, strip, rustflags gesetzt; keine weitere Aktion nötig.
3. Profiling-Skript erstellen (P1). Datei: scripts/profile.sh. Aktion: perf record + flamegraph oder samply für Release-Build erstellen; dokumentieren in docs/profiling.md.
4. Pool-Metrik-Timer integriert (P2). Datei: crates/infrastructure/src/postgres/database.rs. Aktion: Hintergrund-Thread (tokio::spawn) alle 5s PostgresDb-Status abfragen; Metrik nur wenn AGROCORE_METRICS_ENABLED=1. Status: erledigt.
5. Slow-Query-Threshold konfigurierbar (P2). Datei: crates/api/src/metrics.rs. Aktion: Konstante SLOW_QUERY_THRESHOLD_MS durch std::env::var("SLOW_QUERY_MS") ersetzen, Default 1000 bleiben.
6. Alternative Allocator einbinden (P3). Datei: crates/api/Cargo.toml + crates/api/src/main.rs. Aktion: tikv-jemallocator als Abhängigkeit hinzufügen; #[global_allocator] in main.rs setzen.
7. Feature-Flag konsolidieren (P2). Datei: crates/api/Cargo.toml + crates/api/src/metrics.rs. Aktion: cfg(feature="metrics") zusätzlich zur Env-Var einführen; Binary kann ohne Feature kompiliert werden.
8. KI-Analytics & Mobile-First (P4). Datei: docs/tasks.md. Aktion: Phase starten sobald Monitoring abgeschlossen; nicht vor v0.10.0. Status: Offen.

Crates-Analyse (konkrete Tasks):

OPT-001 Dashboard TUI Fehlerbehandlung und Timeout (P1). Datei: crates/dashboard/src/services/process.rs. Aktion: Timeout (30s) und explizite Fehlerbehandlung eingebaut. Status: erledigt.

OPT-002 Database Enum groß Pool-Metrik-Timer (P1). Datei: crates/infrastructure/src/postgres/database.rs. Aktion: Pool-Metrik-Timer (5s) eingebaut. Status: erledigt.

OPT-003 Admin-UI Leptos Upgrade und Vec-Klone (P2). ABGESAGT. Status: Abgebrochen.

OPT-004 Metrics Makro vollständig integrieren Middleware (P0). Datei: crates/api/src/metrics.rs + crates/api/src/middleware.rs. Aktion: Middleware erstellen, Makro integrieren. Status: erledigt.

OPT-005 LPIS-Providers Cache doppelter Klon retry (P2). Datei: crates/lpis-providers/src/cache.rs + lib.rs. Aktion: Cache nutzt Arc<[u8]>, kein doppelter Klon; retry via with_retry aktiv. Status: erledigt.

OPT-006 Messaging Hardcoded-Strings retry-Integration (P2). Datei: crates/messaging/src/lib.rs. Aktion: Retry-Backoff (max 3 Versuche) eingebaut; retry aktiv. Status: erledigt.

OPT-007 Shared async-trait und Arc-Klone (P3). Datei: crates/shared/src/lib.rs. Aktion: db_exec Makro verbessert (Referenz-Klon statt direkter Referenz); with_retry bleibt. Status: erledigt.

OPT-008 Domain lazy-loading (P2). Datei: crates/domain/src/lib.rs. Aktion: Repository-Enum Mock-Branch mit lazy-loading. Status: Offen.

OPT-009 Reporting-Service Timeout und Paginierung (P2). Datei: crates/reporting-service/src/main.rs. Aktion: Paginierung dynamisch, Timeout 30s, Tracing. Status: Offen.

OPT-010 Weather-Service und Geometry-Service Timeout (P3). Datei: crates/weather-service/src/main.rs + crates/geometry-service/src/main.rs. Aktion: Timeout-Struktur hinzufügen. Status: Offen.

Status: 001, 002, 004, 005, 006, 007 erledigt. 2 (Build) erledigt, 1 (Toggle) erledigt. 003 abgebrochen. Übrige offen (008, 009, 010). 7 (Flag) offen, 6 (Allocator) offen, 5 (Threshold) offen, 3 (Profiling) offen, 8 (KI) offen.
