# AgroCore-RS Optimierungs-Tasks

1. Monitoring-Overhead minimieren (P1). Datei: crates/api/src/metrics.rs. Aktion: Makro measure_sqlx_query! optional einbinden (nur wenn is_enabled() true), nicht pro Query erzwingen. Begründung: Perf Book — Instrumentierung nur im heißen Pfad, sonst I/O-Overhead.
2. Release-Profil optimiert (P0). Datei: Cargo.toml (fertig). Aktion: lto=fat, codegen-units=1, panic=abort, strip, rustflags gesetzt; keine weitere Aktion nötig.
3. Profiling-Skript erstellen (P1). Datei: scripts/profile.sh. Aktion: perf record + flamegraph oder samply für Release-Build erstellen; dokumentieren in docs/profiling.md.
4. Pool-Metrik-Timer integrieren (P2). Datei: crates/infrastructure/src/postgres/database.rs. Aktion: Hintergrund-Thread (tokio::spawn) alle 5s PostgresDb::update_pool_metrics() aufrufen; Metrik-Status nur wenn AGROCORE_METRICS_ENABLED=1.
5. Slow-Query-Threshold konfigurierbar (P2). Datei: crates/api/src/metrics.rs. Aktion: Konstante SLOW_QUERY_THRESHOLD_MS durch std::env::var("SLOW_QUERY_MS") ersetzen, Default 1000 bleiben.
6. Alternative Allocator einbinden (P3). Datei: crates/api/Cargo.toml + crates/api/src/main.rs. Aktion: tikv-jemallocator als Abhängigkeit hinzufügen; #[global_allocator] in main.rs setzen.
7. Feature-Flag konsolidieren (P2). Datei: crates/api/Cargo.toml + crates/api/src/metrics.rs. Aktion: cfg(feature="metrics") zusätzlich zur Env-Var einführen; Binary kann ohne Feature kompiliert werden.
8. KI-Analytics & Mobile-First (P4). Datei: docs/tasks.md. Aktion: Phase starten sobald Monitoring abgeschlossen; nicht vor v0.10.0.

Crates-Analyse (konkrete Tasks):

OPT-001 Dashboard TUI Fehlerbehandlung und Timeout (P1). Datei: crates/dashboard/src/services/process.rs. Aktion: process.rs Command::status() mit Timeout (Duration::from_secs(30)) umschließen; Fehler als anyhow::Error zurückgeben statt ignorieren. Datei: crates/dashboard/src/services/build.rs. Aktion: BuildStatus::run_check() mit Timeout und Tracing-Log bei Fehler. Datei: crates/dashboard/src/services/log.rs. Aktion: LogBuffer::push() auf Overflow prüfen und alte Einträge rotieren.

OPT-002 Database Enum groß Pool-Metrik-Timer (P1). Datei: crates/infrastructure/src/postgres/database.rs. Aktion: PostgresDb::connect() initialisiert Metrik-Registry; spawn-Task ruft alle 5s pool_active_connections und pool_idle_connections ab und ruft update_pool_metrics() auf. Enum Database bleibt; kein Refactoring nötig, nur Timer hinzufügen.

OPT-003 Admin-UI Leptos Upgrade und Vec-Klone (P2). Datei: crates/admin-ui/Cargo.toml. Aktion: Leptos auf 0.9.0-beta aktualisieren (User explizit gewünscht, NIE downgraden). Datei: crates/admin-ui/src/main.rs. Aktion: Auth-Path Signal-Klone prüfen — unnötige .clone() auf große Vec entfernen; stattdessen Arc<Signal> oder Lazy-Signal nutzen.

OPT-004 Metrics Makro vollständig integrieren Middleware (P0). Datei: crates/api/src/metrics.rs + crates/api/src/middleware.rs. Aktion: measure_sqlx_query! in alle SQL-Handler integrieren (mindestens PostgresDb-Queries). Datei: crates/api/src/middleware.rs. Aktion: Middleware erstellen, die is_enabled() prüft und Metriken registriert; als actix-web Middleware einbinden.

OPT-005 LPIS-Providers Cache doppelter Klon retry (P2). Datei: crates/lpis-providers/src/cache.rs + lib.rs. Aktion: Cache-Implementierung prüfen: Vec<u8> nicht doppelt zwischen Memory und Redis klonen; stattdessen Arc<[u8]> oder String mit Referenz nutzen. Retry-Logik (with_retry aus shared) aktiv in Provider-Requests einbinden.

OPT-006 Messaging Hardcoded-Strings retry-Integration (P2). Datei: crates/messaging/src/lib.rs. Aktion: Webhook-Event-Handler mit retry-Backoff (exponential, max 3 Versuche) ergänzen. NATS-Subject-Strings als Konstante belassen; retry für alle publish-Operationen aktivieren.

OPT-007 Shared async-trait und Arc-Klone (P3). Datei: crates/shared/src/lib.rs. Aktion: with_retry() auf native async fn in Trait umstellen (nur wenn Trait nicht als dyn genutzt). Makro db_exec prüfen: Pool-Klon vermeiden — stattdessen Referenz (&PgPool) oder Arc-Pool-Referenz nutzen.

OPT-008 Domain lazy-loading (P2). Datei: crates/domain/src/lib.rs. Aktion: Repository-Enum Mock-Branch mit lazy-loading versehen — Mock-Repositories nur initialisieren, wenn tatsächlich genutzt (lazy_static oder OnceLock).

OPT-009 Reporting-Service Timeout und Paginierung (P2). Datei: crates/reporting-service/src/main.rs. Aktion: Paginierung dynamisch (Standard 100, max 500) statt fest 1000. Timeout für Excel-Generierung (30s) setzen. Tracing-Log bei Timeout.

OPT-010 Weather-Service und Geometry-Service Timeout (P3). Datei: crates/weather-service/src/main.rs + crates/geometry-service/src/main.rs. Aktion: Identische Timeout-Struktur wie Reporting-Service hinzufügen; NATS-Worker-Integration mit retry und Timeout versehen.

Status alle: Offen, außer 2 (Performance-Build erledigt) und 1 (Monitoring-Toggle aktiv).
