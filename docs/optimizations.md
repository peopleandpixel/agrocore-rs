# AgroCore-RS Optimierungsvorschläge

Dieses Dokument enthält Vorschläge zur Verbesserung der Performance, Sicherheit und Code-Qualität des AgroCore-RS Projekts.

## 1. Performance-Optimierungen

### 1.1 Datenbank-Interaktionen
*   **Batch-Updates & N+1 Problematik:** In den Repositories (z.B. `PgUserRepo::update`) werden assoziierte Daten wie `user_sites` in einer Schleife einzeln gelöscht und eingefügt. Dies führt zu N+1 Datenbank-Aufrufen.
    *   *Lösung:* Verwendung von `UNNEST` oder Batch-Insert-Statements, um alle Datensätze mit einer einzigen Query zu aktualisieren.
*   **Subqueries vs. Joins:** Bei der Abfrage von Listen (z.B. `find_all` bei Usern) wird für jeden Datensatz eine Subquery ausgeführt, um assoziierte IDs zu sammeln (`json_agg`).
    *   *Lösung:* Einsatz von `LEFT JOIN` und Aggregation auf Datenbankebene, um die Anzahl der Abfragen zu reduzieren.
*   **Pool-Konfiguration:** Die Datenbankverbindung nutzt Standardeinstellungen.
    *   *Lösung:* Explizite Konfiguration des `PgPoolOptions` (z.B. `max_connections`, `min_connections`, `idle_timeout`, `max_lifetime`), angepasst an die Lastprofile der Dienste.

### 1.2 Speicher- und Ressourcenmanagement
*   **Repository-Instanziierung:** In `Database`-Methoden wird bei jedem Aufruf ein neues `Arc::new(Repo::new(pool))` erstellt.
    *   *Lösung:* Vorab-Instanziierung der Repositories im `PostgresDb`-Struct, da diese zustandslos sind und nur den Pool halten.
*   **String-Allokationen im Messaging:** In den Messaging-Clients (NATS/MQTT) werden Themenpfade (`subjects/topics`) oft bei jedem Senden neu allokiert (`to_string()`).
    *   *Lösung:* Verwendung von `Cow<'static, str>` oder vorberechneten Strings für statische Themenpfade.

## 2. Sicherheits-Verbesserungen

### 2.1 API-Sicherheit
*   **CORS-Konfiguration:** Aktuell wird `Cors::permissive()` verwendet, was alle Origins erlaubt.
    *   *Lösung:* Implementierung einer expliziten Whitelist für erlaubte Origins in der Produktionsumgebung.
*   **Abhängigkeiten:** Einige sicherheitsrelevante Crates nutzen Vorabversionen (z.B. `argon2 = "0.6.0-rc.8"`).
    *   *Lösung:* Wechsel auf stabile Versionen, um unentdeckte Bugs in Release Candidates zu vermeiden.

### 2.2 Infrastruktur-Sicherheit
*   **MQTT Verschlüsselung:** Der MQTT-Client unterstützt aktuell kein TLS (nur als Kommentar vorbereitet).
    *   *Lösung:* Implementierung der TLS-Unterstützung in `MqttClient`, um Telemetriedaten sicher zu übertragen.
*   **Fehlerbehandlung bei Tokens:** Fehler beim Aktualisieren von Refresh-Tokens werden aktuell mit `let _ = ...` ignoriert.
    *   *Lösung:* Korrektes Error-Handling, um sicherzustellen, dass ungültige Zustände (z.B. alter Token noch aktiv, neuer nicht gespeichert) vermieden werden.

## 3. Code-Qualität & Wartbarkeit

### 3.1 Architektur & Patterns
*   **Vereinheitlichung der Retry-Logik:** Die Retry-Logik für Datenbank- und NATS-Verbindungen ist fast identisch implementiert, aber dupliziert.
    *   *Lösung:* Extraktion in eine generische `with_retry`-Hilfsfunktion oder Verwendung eines spezialisierten Crates wie `backoff`.
*   **Repository Boilerplate:** Es gibt eine hohe Anzahl an Repositories mit viel repetitivem Code.
    *   *Lösung:* Einführung von Basis-Traits oder Makros, um Standard-CRUD-Operationen zu vereinheitlichen.
*   **Modern Rust Async Traits:** Das Projekt nutzt `Pin<Box<dyn Future<Output = Result<T>> + Send>>` für async Methoden in Traits.
    *   *Lösung:* Da Rust 1.75+ (und Edition 2024) native Unterstützung für `async fn` in Traits bietet, könnte dies den Code erheblich vereinfachen und die Lesbarkeit verbessern.

### 3.2 Docker & Deployment
*   **Healthcheck im Dockerfile:** Der Docker-Healthcheck verwendet `curl`, welches im `runtime`-Image (debian-slim) standardmäßig nicht installiert ist.
    *   *Lösung:* Entweder `curl` im Runtime-Image installieren oder den Healthcheck auf eine interne Methode (z.B. ein spezialisiertes Health-Binary) umstellen.
