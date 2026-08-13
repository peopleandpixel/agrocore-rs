# AgroCore-RS Optimierungsvorschläge

Dieses Dokument enthält Vorschläge zur Verbesserung der Performance, Sicherheit und Code-Qualität des AgroCore-RS Projekts.

## 1. Performance-Optimierungen

### 1.1 Datenbank-Interaktionen
*   **Batch-Updates & N+1 Problematik:** In den Repositories (z.B. `PgUserRepo::update`) wurden assoiierte Daten wie `user_sites` in einer Schleife einzeln gelöscht und eingefügt. Dies führte zu N+1 Datenbank-Aufrufen. ✅ **Resolved (0.8.4):** Batch-INSERT via `SELECT $1, unnest($2::uuid[])` ersetzt die Schleife durch eine einzige Query.
*   *Lösung:* Verwendung von `UNNEST` oder Batch-Insert-Statements, um alle Datensätze mit einer einzigen Query zu aktualisieren.
*   **Subqueries vs. Joins:** Bei der Abfrage von Listen (z.B. `find_all` bei Usern) wurde für jeden Datensatz eine Subquery ausgeführt, um assoiierte IDs zu sammeln (`json_agg`). ✅ **Resolved (0.8.4):** Alle `PgUserRepo`-Queries verwenden nun `LEFT JOIN user_sites` + `GROUP BY u.id` + `json_agg` als einzige Query, keine correlated Subqueries mehr.
*   *Lösung:* Einsatz von `LEFT JOIN` und Aggregation auf Datenbankebene, um die Anzahl der Abfragen zu reduzieren.
*   **Pool-Konfiguration:** Die Datenbankverbindung nutzt Standardeinstellungen. ✅ **Resolved (0.8.2 + 0.8.4):** `PgPoolOptions` konfigurierbar via Env-Variablen (`DATABASE_MAX_CONNECTIONS`, `DATABASE_MIN_CONNECTIONS`, `DATABASE_IDLE_TIMEOUT_SECS`, `DATABASE_MAX_LIFETIME_SECS`, `DATABASE_ACQUIRE_TIMEOUT_SECS`, `DATABASE_CONNECT_TIMEOUT_SECS`).
*   *Lösung:* Explizite Konfiguration des `PgPoolOptions` (z.B. `max_connections`, `min_connections`, `idle_timeout`, `max_lifetime`), angepasst an die Lastprofile der Dienste.
*   **Repository Factory Pattern:** Alle Repository-Methoden in `PostgresDb` folgen dem identischen Muster `Arc::new(PgXyzRepo::new(self.pool.clone()))`, was zu Boilerplate-Code führt. ✅ **Partially Resolved (0.8.2):** `repo!` Makro in shared crate reduziert Boilerplate für Repository-Instanziierung.
*   *Lösung:* Einführung einer generischen Repository-Factory oder eines Makros zur Reduktion des Boilerplates und zentralen Fehlerbehandlung.

### 1.2 Speicher- und Ressourcenmanagement
*   **Repository-Instanziierung:** In `Database`-Methoden wurde bei jedem Aufruf ein neues `Arc::new(Repo::new(pool))` erstellt.
*   *Lösung:* Vorab-Instanziierung der Repositories im `PostgresDb`-Struct, da diese zustandslos sind und nur den Pool halten.
*   **String-Allokationen im Messaging:** In den Messaging-Clients (NATS/MQTT) wurden Themenpfade (`subjects/topics`) oft bei jedem Senden neu allokiert (`to_string()`).
*   *Lösung:* Verwendung von `Cow<'static, str>` oder vorberechneten Strings für statische Themenpfade. ✅ **Resolved (0.8.2):** Statische NATS-Subjects als Konstanten vordefiniert.
*   **DecodingKey Caching:** In `AuthExtractor` wurde bei jeder Anfrage ein neuer `DecodingKey` aus dem JWT-Secret erstellt, was bei hoher Request-Rate ineffizient ist. ✅ **Resolved (0.8.2):** `DecodingKey` via `OnceLock` gecacht.
*   *Lösung:* Einführung eines gecachten `DecodingKey` das nur bei Secret-Change aktualisiert wird.
*   **Rollen-Mapping Optimierung:** Die Konvertierung von String-Rollen zu `UserRole` Enums erfolgt bei jedem Aufruf der `roles()` Methode in `AuthenticatedUser`. ✅ **Resolved (0.8.2):** Rollen werden während Token-Entschlüsselung direkt als `UserRole` Vector gespeichert.
*   *Lösung:* Memoisierung des konvertierten Vectors oder direkte Speicherung als `UserRole` Vector im Claims während der Token-Entschlüsselung.

### 1.3 DTO & Serialisierung
*   **Selektive Validierung:** Die neuen IoT-DTOs verwenden `validator::Validate` für alle Felder bei jedem Request. Bei hohen Durchsatzraten könnte die Validierung bestimmter Felder (wie bereits validierte UUIDs über Routing) überflüssig sein.
*   *Lösung:* Einführung von Validierungsgruppen oder bedingter Validierung abhängig vom Endpunkttyp und Datenherkunft.
*   **OpenAPI Dokumentation Größe:** Die `ApiDoc` Struktur in `crates/api/src/lib.rs` ist sehr groß geworden und enthält alle Endpunkte. Für große Anwendungen könnte eine Aufteilung nach Modulen die Kompilierzeiten verbessern.
*   *Lösung:* Modulare OpenAPI-Dokumentation mit separaten `OpenApi`-Structs pro API-Bereich die zur Laufzeit kombiniert werden.

## 2. Sicherheits-Verbesserungen

### 2.1 API-Sicherheit
*   **CORS-Konfiguration:** Aktuell wird `Cors::permissive()` verwendet, was alle Origins erlaubt.
*   *Lösung:* Implementierung einer expliziten Whitelist für erlaubte Origins in der Produktionsumgebung.
*   **Abhängigkeiten:** Einige sicherheitsrelevante Crates nutzen Vorabversionen (z.B. `argon2 = \"0.6.0-rc.8\"`).
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
*   *Lösung:* Da Rust 1.75+ (und Edition 2024) native Unterstützung für `async fn` in Traits bietet, könnte dies den Code erheblich vereinfachen und die Lesbarkeit verbessern. ✅ **Partially Resolved (0.8.4):** Das Projekt verwendet bereits Edition 2024, aber das Refactoring der Repository-Traits von `Pin<Box<dyn Future>>` zu nativen `async fn` ist noch offen.
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

## 4. Schnell umsetzbare Quick Wins (≤ 1 Stunde jeweils)

1. **~~DecodingKey Caching in Middleware~~** ✅ - Introduce cached DecodingKey in AuthExtractor
2. **~~Einführung von PgPoolOptions Konfiguration~~** ✅ - Expose connection pool configuration via environment variables
3. **~~Repository Boilerplate Reduktion durch einfaches Makro~~** ✅ - Create a repository factory macro
4. **~~Überprüfung der Messaging-Topic Erstellung~~** ✅ - Identify and precompute static topics in messaging clients
5. **~~Rollen-Mapping Optimierung~~** ✅ - Memoize or pre-convert role strings to UserRole enums