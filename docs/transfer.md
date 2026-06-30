# Transfer-Dokumentation: VineyardCloud Legacy -> AgroCore Rust (Generische Landwirtschaft)

Dieses Dokument dient dazu, den Migrationsstatus der Anwendungsfälle vom alten Java-Backend (`vc-backend-new`) zum neuen Rust-Backend (`agrocore-rs`) zu verfolgen, mit dem Ziel einer universellen landwirtschaftlichen Plattform.

## Fortschrittsübersicht

- [x] Pflanzenschutz-Logik (Vollständig portiert, Fokus: Generalisierung)
- [x] Stammdaten (Flächen/Kulturen/Weiden)
- [x] Tierhaltung (Neu: Weidemanagement, Bestandsführung)
- [ ] Auftragsmanagement (Generische Aufgaben - Teilweise portiert)
- [ ] Benutzer- & Mandantenverwaltung
- [ ] Equipment & Kostenstellen

---

## Strategie: Intelligente Migration & Generalisierung

Anstatt die weinbauspezifische Java-Logik 1:1 zu portieren, folgen wir diesen Prinzipien:
1.  **Domain-Driven Design (DDD):** Abstraktion von Weinbau zu generischen Entitäten (z.B. `Site` statt `Vineyard`, `Crop` statt `Vine`).
2.  **Microservices-Ready:** Modularisierung nach Agrar-Sparten (Wein, Ackerbau, Viehzucht).
3.  **Multi-Crop & Livestock Support:** Unterstützung verschiedener Kulturen (Oliven, Kork, Getreide) und Tierarten durch flexible Attribut-Systeme.
4.  **Event-Driven:** Entkopplung der Sparten über ein gemeinsames Event-Bus-System.
5.  **Type Safety:** Nutzung von Enums und Traits zur Abbildung unterschiedlicher landwirtschaftlicher Logiken.

---

## 1. Pflanzenschutz & Düngung (Plant Protection & Fertilization)
Ziel: Vollständige Abbildung der Flächenberechnungen, Materialmengen und Nährstoffbilanzen für alle Kulturen.

### Anwendungsfälle / Provider
- [x] Basis-Berechnungslogik (`PlantProtectionAreaMethod`)
- [x] Netto-Fläche (`NetArea`)
- [x] Brutto-Fläche (`GrossArea`)
- [x] Laubwandfläche gemessen (`LeafWallMeasured`)
- [x] Laubwandfläche Brutto (`LeafWallGross`)
- [x] Bodenschutz (`GroundProtection`)
- [x] Herbizidschutz (`HerbicideProtection`)
- [x] Steillagen-Faktoren (Weinbau-spezifisch, aber generisch konfigurierbar)
- [x] Spezialkulturen-Logik (z.B. Baumkronen-Volumen für Oliven/Kork)
- [x] Materialmengenberechnung (`MaterialAmountProvider` -> `MaterialCalculationService`)

### Microservices-Splitting
- **`CalculationService`**: Erweitert um Formeln für unterschiedliche Anbausysteme (Reihenkulturen, Streuobst, freie Flächen) und Tierhaltung (Futterbedarf). [Integriert in `agrocore-domain::services::calculation`]
- **`NutritionService`**: Neuer Service für Düngebedarfsplanung und Nährstoffbilanzen (relevant für Ackerbau und Tierhaltung/Gülle). [ERLEDIGT]

### Konkrete Schritte
1.  **Stammdaten-Update:** `Site` Entität um `gross_area`, `total_strike_length` und `average_lane_width` ergänzen. [ERLEDIGT]
2.  **Berechnungs-Logik vervollständigen:** `PlantProtectionAreaMethod` in `crates/domain` finalisieren (GrossArea, LeafWallGross, GroundProtection). [ERLEDIGT]
3.  **Calculation Service:** Erstellen eines zentralen `CalculationService` in `crates/domain/src/services`, der alle agrarwirtschaftlichen Formeln bündelt. [ERLEDIGT]
4.  **API Integration:** Handler in `crates/api/src/handlers/specialized.rs` erstellen, der die Domain-Services nutzt. [ERLEDIGT]
5.  **API-Konsolidierung:** Spezialisierte Endpunkte für verschiedene Kulturen in `specialized.rs` zu einem generischen `/specialized/sites` Endpunkt zusammengefasst. [ERLEDIGT]

---

## 2. Stammdaten (Master Data / Sites / Assets)
Ziel: Verwaltung aller physischen und biologischen Assets.

### Anwendungsfälle
- [x] Flächen-Verwaltung (`SiteRetriever` - ehemals Vineyard)
- [x] Kultur-Management (Sorte, Pflanzjahr, Unterlage - erweiterbar für Oliven/Kork)
- [x] Tierbestands-Management (Art, Alter, Gruppierung)
- [ ] Geometrie-Mapping (Weideflächen, Ackerschläge, Waldparzellen)
- [ ] Gebiets-Hierarchien (Mandant -> Betrieb -> Sektor -> Schlag/Weide)

### Microservices-Splitting
- **`GeometryService`**: Spezialisiert auf GIS-Operationen, inklusive Topografie-Analysen für verschiedene Kulturen.
- **`AssetRegistry`**: Ein zentraler Service zur Verwaltung der "Inventory"-Items (Bäume, Tiere, Maschinen).

### Konkrete Schritte
1.  **Domänen-Generalisierung:** Umbenennung von `Vineyard` zu `Site` und Einführung von `SiteType` (Weinberg, Olivenhain, Weide, Acker). [ERLEDIGT]
2.  **Attribut-System:** Implementierung eines flexiblen `Property`-Systems für `Site`, um kulturspezifische Daten (z.B. Baumabstand bei Oliven, Grasart bei Weiden) ohne Schema-Änderung zu speichern. [ERLEDIGT]
3.  **Livestock Integration:** Definition einer `Animal` Entität und deren Verknüpfung mit `Site` (Weidefläche). [ERLEDIGT]

---

## 3. Auftragsmanagement (Order Management)
Ziel: Reaktives Workflow-System.

### Anwendungsfälle
- [ ] Auftragserstellung & Modifikation (`OrderModifier`)
- [ ] Auftrags-Workflow-Status (`OrderWorkflow`)
- [ ] Folgeaufträge (`FollowUpOrderWorkflow`)
- [ ] Stammdaten für Aufträge (`OrderMasterData`)
- [ ] Task-Präsentation (`TaskAsDtoPresenter`)

### Intelligenter Ansatz
- **Workflow Engine:** Anstatt hunderter `if-else` Statements für Statusübergänge, nutzen wir ein State-Machine-Pattern (z.B. mit dem `statig` Crate oder einem einfachen Enum-Dispatch).
- **Messaging:** Statusänderungen werfen Events (`OrderCompleted`), auf die andere Services (z.B. `ReportingService`) reagieren.

### Konkrete Schritte
1.  **State Machine:** Definition der `OrderState` Transitionen in `crates/domain/src/entities/order.rs`. [ERLEDIGT]
2.  **Command Pattern:** `OrderModifier` durch Command-Handler ersetzen (z.B. `CompleteOrderCommand`, `StartOrderCommand`). [ERLEDIGT]

---

## 4. Benutzer & Mandanten (User / Tenant)
Ziel: Authentifizierung, Autorisierung und Mehrmandantenfähigkeit.

### Anwendungsfälle
- [ ] Mandanten-Konfiguration (`TenantConfigurationRetriever`)
- [ ] Benutzer-Events (`UserEventHandler`)
- [ ] Sichtbarkeitsregeln (`VisibilityAwareEntity`)
- [ ] Token-Rollen (`TokenRole`)

### Intelligenter Ansatz
- **RBAC (Role-Based Access Control):** Integration von feingranularen Berechtigungen direkt in die Repository-Ebene (Visibility-Filter), anstatt sie manuell in jedem Handler zu prüfen.
- **Tenant Isolation:** Nutzung von PostgreSQL Row Level Security (RLS) oder striktem Filtering über Domain-Services.

---

## 5. Equipment & Kostenstellen
Ziel: Ressourcenplanung und Kostenkontrolle.

### Anwendungsfälle
- [ ] Ausrüstungs-Verwaltung (`EquipmentRetriever`)
- [ ] Kostenstellen-Zuordnung (`CostCenterRetriever`)
- [ ] Maschinen-Stammdaten

### Microservices-Splitting
- **`ResourceService`**: Verwaltung von Equipment und Personal als gemeinsam genutzte Ressourcen für die Auftragsplanung.

---

## 6. Tierhaltung (Livestock Management)
Ziel: Management von Weidegang und Tiergesundheit.

### Anwendungsfälle
- [ ] Belegungsplanung (Welche Gruppe ist auf welcher Weide?)
- [ ] Futterbedarfsberechnung (Basierend auf Tierzahl und Weidequalität)
- [ ] Behandlungsregister (Medikamente, Impfungen - analog zu Pflanzenschutz)

### Microservices-Splitting
- **`LivestockService`**: Eigenständiger Service für Tier-Stammdaten und Ereignisse (Geburt, Verkauf, Tod).
- **`GrazingService`**: Optimierung der Weiderotation zur Vermeidung von Überweidung.

---

## Technische Schulden / Differenzen
- Abkehr von der harten Kopplung an `Vineyard`-Id hin zu einer generischen `Asset`- oder `Site`-Id.
- Die Java-Logik für "Steillagen" muss in eine generische "Erschwernis-Zulage" umgewandelt werden, die auch für andere Kulturen definierbar ist.
