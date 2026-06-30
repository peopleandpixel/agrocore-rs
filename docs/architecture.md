# AgroCore Architektur: Microservices-Strategie

## Aktueller Stand
Das Projekt ist als **Monorepo** mit mehreren **Crates** (Rust-Paketen) organisiert:
- `agrocore-domain`: Enthält die Geschäftslogik, Entitäten und Service-Traits.
- `agrocore-infrastructure`: Implementiert Repositories, Datenbank-Zugriffe und externe Adapter.
- `agrocore-api`: Das REST-Backend (Actix-web), das die Domain-Services nutzt.
- `agrocore-shared`: Gemeinsam genutzte Hilfsmittel (Pagination, Events).

## Bewertung der Microservice-Tauglichkeit
Die aktuelle Struktur ist **Microservices-Ready**, da die Domänen-Logik bereits sauber in Services und Entitäten getrennt ist. Ein Splitting in eigenständige Services ist jederzeit möglich.

### Empfohlene Service-Grenzen
Basierend auf der `transfer.md` und der funktionalen Kohäsion ergeben sich folgende logische Services:

1. **Asset & Master Data Service (AssetRegistry)**
   - Entitäten: `Site`, `Animal`, `Crop`.
   - Aufgaben: Verwaltung der physischen Assets.
   
2. **Calculation & Nutrition Service (AgronomyService)**
   - Aufgaben: Pflanzenschutz-Berechnungen, Düngebedarfsplanung, Nährstoffbilanzen.
   - Grund: Diese Logik ist oft zustandslos oder benötigt nur temporär Zugriff auf Asset-Daten.
   
3. **Order Management Service**
   - Entitäten: `Order`, `Task`.
   - Aufgaben: Workflow-Steuerung, Statusübergänge.
   
4. **Resource Service**
   - Entitäten: `Equipment`, `Staff`.
   - Aufgaben: Planung von Maschinen und Personal.

## Umsetzungsstrategie
Es wird empfohlen, zunächst beim **Modular Monolith** zu bleiben, solange die Last und die Teamgröße dies zulassen. Die Vorteile sind:
- Einfacheres Deployment.
- Typsicherheit über Crates hinweg ohne Netzwerk-Overhead.
- Konsistente Datenhaltung.

### Schritte zum Splitting (falls erforderlich)
1. **API-Gateways:** Einführung eines Gateways, das Anfragen an die entsprechenden Module (später Services) routet.
2. **Event-Bus:** Die bereits genutzte Event-Struktur (`GlobalEvent`) über einen echten Message Broker (z.B. RabbitMQ oder NATS) verteilen.
3. **Database per Service:** Jedes Crate bekommt seine eigenen Tabellen/Datenbanken.

## Fazit
Die Services `CalculationService` und `NutritionService` sind ideale Kandidaten für einen gemeinsamen "Agronomy-Microservice", da sie spezialisierte Berechnungen bündeln, die unabhängig vom restlichen System skalieren können.
