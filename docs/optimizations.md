# AgroCore-RS Optimierungsvorschläge

Dieses Dokument enthält Vorschläge zur Verbesserung der Performance, Sicherheit und Code-Qualität des AgroCore-RS Projekts.

> **Hinweis:** Die folgenden Aufgaben wurden bereits erledigt und sind aus diesem Dokument entfernt worden:
> - **Task 4 Quick Wins** (alle 5): DecodingKey Caching, PgPoolOptions Konfiguration, Repository Factory Macro, Messaging Topic Precomputation, Rollen-Mapping Optimierung — alle ✅ in v0.8.2
> - **Task 1.1** (Batch-Updates & N+1, Subqueries vs. Joins, Pool-Konfiguration) — ✅ in v0.8.4
> - **Task 1.1c erweiterte Pool-Konfiguration** (acquire_timeout, connect_timeout) — ✅ in v0.8.5
> - **Task 1.1c Repository Pre-instantiation** (Pre-instanziierung aller Repos im PostgresDb-Struct) — ✅ in v0.8.6
> - **Task 2.1 CORS-Konfiguration** — ✅ in v0.8.5

## 1. Performance-Optimierungen

### 1.1c Repository Pre-instantiation (Resolved ✓)
*   ~~**Repository-Instanziierung:** In `Database`-Methoden wurde bei jedem Aufruf ein neues `Arc::new(Repo::new(pool))` erstellt.~~
*   ~~*Lösung:* Vorab-Instanziierung der Repositories im `PostgresDb`-Struct, da diese zustandslos sind und nur den Pool halten.~~ ✅ **Resolved (0.8.6):** Alle 32 Repositories werden in `PostgresDb::connect()` und `from_pool()` einmalig als `Arc`-Fields initialisiert. Repository-Zugriffe geben nun `self.xxx_repo.clone()` (Refcount-Inkrement) zurück statt `Arc::new(Repo::new(pool.clone()))` (Heap-Allocation) auf jedem Aufruf.

### 1.2 Speicher- und Ressourcenmanagement (Resolved ✓)
*   ~~**Repository-Instanziierung:** In `Database`-Methoden wurde bei jedem Aufruf ein neues `Arc::new(Repo::new(pool))` erstellt.~~
*   ~~*Lösung:* Vorab-Instanziierung der Repositories im `PostgresDb`-Struct, da diese zustandslos sind und nur den Pool halten.~~ ✅ **Resolved (0.8.6):** Siehe Task 1.1c oben — alle 32 Repositories werden einmalig in `PostgresDb::connect()`/`from_pool()` initialisiert und als gecachte `Arc`-Fields gehalten.

### 1.3 DTO & Serialisierung
*   ~~**Selektive Validierung:** Die neuen IoT-DTOs verwenden `validator::Validate` für alle Felder bei jedem Request. Bei hohen Durchsatzraten könnte die Validierung bestimmter Felder (wie bereits validierte UUIDs über Routing) überflüssig sein.~~
*   ~~*Lösung:* Einführung von Validierungsgruppen oder bedingter Validierung abhängig vom Endpunkttyp und Datenherkunft.~~ ✅ **Resolved (0.8.7):** `IoTCommandRequestDto` erhählt `#[validate(length(...))]`/`#[validate(range(...))]` Attribute; `send_command` Handler ruft `.validate()` auf. UUIDs validiert durch serde; `serde_json::Value` lässt sich nicht mit `#[validate]` validieren.
*   ~~**OpenAPI Dokumentation Größe:** Die `ApiDoc` Struktur in `crates/api/src/lib.rs` ist sehr groß geworden und enthält alle Endpunkte. Für große Anwendungen könnte eine Aufteilung nach Modulen die Kompilierzeiten verbessern.~~
*   ~~*Lösung:* Modulare OpenAPI-Dokumentation mit separaten `OpenApi`-Structs pro API-Bereich die zur Laufzeit kombiniert werden.~~ ✅ **Resolved (0.8.8):** Extracted into `crates/api/src/openapi.rs` — 13 per-module `OpenApi` structs merged at runtime via `utoipa::OpenApi::merge()`.

## 2. Sicherheits-Verbesserungen

### 2.1 API-Sicherheit
*   **Abhängigkeiten:** Einige sicherheitsrelevante Crates nutzen Vorabversionen (z.B. `argon2 = "0.6.0-rc.8"`).
*   *Lösung:* Wechsel auf stabile Versionen, um unentdeckte Bugs in Release Candidates zu vermeiden.

### 2.2 Infrastruktur-Sicherheit
*   **MQTT Verschlüsselung:** Der MQTT-Client unterstützt aktuell kein TLS (nur als Kommentar vorbereitet).
*   *Lösung:* Implementierung der TLS-Unterstützung in `MqttClient`, um Telemetriedaten sicher zu übertragen.
*   **Fehlerbehandlung bei Tokens:** Fehler beim Aktualisieren von Refresh-Tokens werden aktuell mit `let _ = ...` ignoriert.
*   *Lösung:* Korrektes Error-Handling, um sicherzustellen, dass ungültige Zustände (z.B. alter Token noch aktiv, neuer nicht gespeichert) vermieden werden.

### 2.3 Authentifizierung & Autorisierung
*   **Token-Revocation:** Aktuell gibt es kein Mechanismus zur sofortigen Widerruf von kompromittierten Tokens außerhalb der natürlichen Ablaufzeit.
*   *Lösung:* Implementierung eines Token-Blacklists mittels Redis oder Datenbank-Tabelle mit kurzen TTL für widerrufene Tokens.
*   **Rate Limiting Differenzierung:** Aktuell gilt das gleiche Rate-Limit für alle Endpunkte unabhängig von ihrer Sensitivität oder Ressourcenintensität.
*   *Lösung:* Implementierung von differenzierten Rate-Limits basierend auf Endpunkt-Typen (z.B. strengere Limits für Auth-Endpunkte, höhere Limits für Lese-Operationen).

## 3. Code-Qualität & Wartbarkeit

### 3.1 Architektur & Patterns
*   **Vereinheitlichung der Retry-Logik:** Die Retry-Logik für Datenbank- und NATS-Verbindungen ist fast identisch implementiert, aber dupliziert.
*   *Lösung:* Extraktion in eine generische `with_retry`-Hilfsfunktion oder Verwendung eines spezialisierten Crates wie `backoff`.
*   **Repository Boilerplate:** Es gibt eine hohe Anzahl an Repositories mit viel repetitivem Code.
*   *Lösung:* Einführung von Basis-Traits oder Makros, um Standard-CRUD-Operationen zu vereinheitlichen.
*   **Modern Rust Async Traits:** Das Projekt nutzt `Pin<Box<dyn Future<Output = Result<T>> + Send>>` für async Methoden in Traits.
*   *Lösung:* Da Rust 1.75+ (und Edition 2024) native Unterstützung für `async fn` in Traits bietet, könnte dies den Code erheblich vereinfachen und die Lesbarkeit verbessern.
*   **Configuration Management:** Konfiguration ist derzeit über verschiedene Wege verteilt (Umgebungsvariablen, Hardcoded Werte, einzelne Config-Funktionen).
*   *Lösung:* Zentralisierte Konfiguration mittels eines `Config`-Structs mit automatischem Laden aus Environment, .env-Dateien und optionalem Hot-Reload während der Entwicklung.

### 3.2 Docker & Deployment
*   **Healthcheck im Dockerfile:** Der Docker-Healthcheck verwendet `curl`, welches im `runtime`-Image (debian-slim) standardmäßig nicht installiert ist.
*   *Lösung:* Entweder `curl` im Runtime-Image installieren oder den Healthcheck auf eine interne Methode (z.B. ein spezialisiertes Health-Binary) umstellen.
*   **Multi-Stage Build Optimierung:** Aktuelle Dockerfiles könnten durch bessere Nutzung von Build-Caching und kleineren Basis-Images optimiert werden.
*   *Lösung:* Überprüfung der Dockerfile-Schichten für bessere Cache-Nutzung und Verwendung von `distroless` oder ähnlichen minimalen Basis-Images für Produktions-Builds.

### 3.3 Monitoring & Observability
*   **Detailliertere Database Metrics:** Neben den bestehenden Prometheus-Metrics könnten Query-Dauer, Pool-Auslastung und Slow-Query-Erkennung hinzugefoben werden.
*   *Lösung:* Implementierung von benutzerdefinierten SQLx-Middleware oder Nutzung von `sqlx-metrics` Crate für tiefere Einblicke.
*   **Business Metrics:** Zusätzlich zu technischen Metrics sollten domain-spezifische Metrics wie aktive Geräte pro Tenant, übertragene Telemetrie-Nachrichten pro Stunde, erfolgreich verarbeitete Import-Dateien etc. hinzugefügt werden.
*   *Lösung:* Einführung von eigenen Metrics zur Erfassung von Geschäftsprozessen und KPIs.
*   **Distributed Tracing Integration:** Verbesserung der bestehenden Tracing-Integration um mehr Span-Attributes für bessere Debugbarkeit hinzuzufügen (z.B. Tenant-ID, User-ID, Operationstyp).
*   *Lösung:* Standardisierung dessen, was in Tracing-Spans aufgezeichnet wird über alle Service-Grenzen hinweg.
