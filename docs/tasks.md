# Agrocore-RS Open Tasks

Letztes Update: 2026-08-21

Tasks sind nach Priorität und geschätztem Implementierungsaufwand geordnet.
Erledigte Arbeit ist weggelassen; ein Modul ohne offene Punkte ist als erledigt markiert.

## Module 17 — KI-Analytics

**Status:** Geplant · **Priorität:** P4 · **Aufwand:** 8–15 Tage pro Feature

- [ ] Definition der Daten- und Evaluierungsanforderungen für Ertragsprognosen.
- [ ] Definition von Krankheits- und Schädlingsfrühwarnsystemen.
- [ ] Definition von Bewässerungs-, Düngungs-, KPI-, Satelliten-Monitoring- und generativen Reporting-Features.
- [ ] Implementierung erst nach Stabilisierung der Produktionsmodule und Sync-Grundlagen.

---

## Admin-UI Status & Missing Features

### Was bereits funktioniert (kompiliert & hat echte API-Integration)

- [x] Login & Setup-Workflow
- [x] Dashboard mit Systemstatus
- [x] Site-Management (CRUD: Site-Typ, Kultur, Sorte, Fläche)
- [x] Aufträge & Tasks (wiederkehrende Tasks, Mitarbeiter-Zuweisung, Status-Verfolgung)
- [x] Worker-Tasks-Ansicht (eigene Aufgaben für Arbeiter)
- [x] Tierhaltung (Tierschau, Behandlungen, Bewegungen)
- [x] Wetterintegration (Open-Meteo Geocoding + aktuelle Wetterdaten)
- [x] Finanzen (PAC-Anträge, Kostenstellen)
- [x] Equipment-Verwaltung
- [x] Compliance-Tracking
- [x] Ressourcenverwaltung (Wasser, Dünger)
- [x] Einstellungen
- [x] Import-Wizard (SIGPAC, GeoJSON, Shapefile)
- [x] Audit-Log
- [x] Kartenansicht
- [x] 10-Sprachunterstützung (DE, EN, ES, FR, PT, IT, PL, RO, UK, NL)
- [x] Dark/Light-Theme
- [x] PWA-Unterstützung

### Gefundene Stubs & Placeholders

**Alle 6 Stubs wurden in v0.9.1 durch funktionale Inhalte ersetzt:**

- [x] **Analytics Profitability Chart** — `"—"` durch echte Revenue/Kosten/Marge-Visualisierung aus `financial_records` API ersetzt (v0.9.1)
- [x] **Analytics Harvest Prediction** — API-Aufruf existiert, Ergebnis-Anzeige um Forecast-Reference aus `weather_data` erweitert (v0.9.1)
- [x] **Equipment Maintenance Scheduling** — UI vorhanden, Wartungsdaten/Termine aus API integriert (v0.9.1)
- [x] **Formularvalidierung** — Forms haben jetzt sinnvolle Beispielwerte, Labels und i18n-Keys (v0.9.1)
- [x] **Pagination** — Listen-Komponenten laden dynamische Daten statt statischer Stubs (v0.9.1)
- [x] **Search/Filter** — Site-Management, Equipment-Selects sind jetzt dynamisch aus API-Daten (v0.9.1)

### Kritisch fehlend (MVP-Level)

#### Inventory Management
- [x] Multi-Lagerverwaltung (Silo, Scheune, Werkstatt) — locations table + UI
- [x] Lot-/Chargen-Tracking (Rückverfolgbarkeit von Saatgut, Dünger, Medikamenten)
- [x] Ablaufdatum-Tracking mit Warnungen
- [x] FEFO/FIFO-Logistik
- [x] Bestandsbewertung (FIFO, Durchschnittskosten, Act-Cost)

#### Kunden & Verkauf
- [x] Kunden-CRM (Kontakte, Bestellhistorie, Vorliebe) — Customer entity + CustomerRepository + PgCustomerRepo + API routes + migration table
- [ ] CSA-Verwaltung (Abonnement-Boxen, Lieferplanung)
- [ ] Großhandelsaufträge (Staffelpreise, Lieferplanung)
- [ ] Direktverkauf (Onlineshop, Zahlungsabwicklung)

#### Job & Arbeitskräfte-Management
- [x] Arbeitszeit-Erfassung (Clock-In/Clock-Out mit GPS)
- [x] Arbeitskosten-Tracking (Stundensatz pro Arbeiter)
- [x] Arbeitskräfte-Zuteilung zu Aufträgen

#### Equipment Management
- [x] Wartungsplanung (basierend auf Betriebsstunden, Kalender, Nutzungsschwellen)
- [x] Wartungskosten-Tracking (Teile, Arbeitszeit, Ausfallkosten)
- [x] Kraftstoffverbrauch (Liter/Stunde, pro Operation, pro Feld)
- [x] Nutzung-Logging (wer hat was, wann, wie lange)
- [x] Abschreibung (automatische Amortisation für Finanzberichte) — Timer (monatlich, ~30 Tage) eingebaut (`database.rs` tokio::spawn); Modul `depreciation.rs` erstellt (`calculate_straight_line`, `double_declining`, `schedule`); Finanzbericht-Integration: offen (Endpoint `/financial/reports` noch nicht erweitert).
- [x] Equipment-Suche und Filterung — Plan umgesetzt: Backend-Filter (`find_all_filtered` + 2 neue Felder), API-DTO erweitert, Handler aktualisiert. Status: erledigt.

#### Tierhaltung (Erweiterung)
- [ ] Zucht-Records (Brunft, KI, Kalbung/Meerschweinchenpaarung)
- [ ] Futteraufnahme-Tracking (Ration, Verschwendung, Futterwert)
- [ ] Milchproduktions-Tracking (Tagesproduktion, Butterfett/Protein)
- [ ] Bewegungsdokumentation (Geburten, Todesfälle, Käufe, Verkäufe)
- [ ] Weide-Management (Flächenrotation, Ruheperioden, Tierstandort)

#### Finanzen (Erweiterung)
- [ ] Buchhaltungs-Integration (QuickBooks, Xero, Doppelte Buchführung)
- [ ] Budgetierung & Prognosen (geplant vs. tatsächlich)
- [ ] Feldkalkulation (Eingangs- vs Ausgangswerte)
- [ ] Umsatz-Tracking pro Kultur
- [ ] Geldfluss-Management (Verbindlichkeiten/Zahlungseingänge planen)
- [ ] Steuerberichtswesen (Schedule F, Abschreibungen, Dünger-Kosten-Abzug)
- [ ] Eingangs-Kosten-Tracking (Saatgut, Dünger, Chemikalien, Kraftstoff, QR-Code-Scans)

#### Wetter & Umwelt
- [x] Anbindung diverser Wetterdienste kostenlos und kostenpflichtig (z.B. OpenWeather, Weather Underground)
- [x] Hyperlokaler Wetterstation-Integration (Davis, Campbell Scientific)
- [x] Bodenfeuchtigkeits-Monitoring (IoT-Sensor-Integration mit Bewässerungssteuerung)
- [x] Wachstumsgradtag-Tracking (Kartoffelernte-Prognose)
- [x] Fröst-/Einfrierwarnungen
- [x] Krankheits-/Schädlings-Risiko-Modell

### Mittel- / Langfristig

#### Mobile First (P2)
- [ ] Offline-first Mobile-App (Sync bei Wiederherstellung der Konnektivität)
- [ ] Barcode/QR-Code-Scanner (Equipment, Inventar, Feld-ID)
- [ ] GPS-Feld-Grenzen (Boundary Recording)
- [ ] Mobile Zeiterfassung (Clock-In/Clock-Out mit GPS)
- [ ] Foto-Dokumentation (Anhänge an Tasks, Probleme, Inspektionen)
- [ ] Sprach-zu-Text-Notizen
- [ ] Push-Benachrichtigungen (Wetterwarnungen, Task-Erinnerungen)
- [ ] Feldaktivitäten-Recording (Pflanzen, Spritzen, Ernten in Echtzeit)
- [ ] Ernte-Daten-Import (Combine-Harvester)
- [ ] Drohnen/UAV-Integration (NDVI-Bilder)

#### Präzisionslandwirtschaft (P3)
- [ ] Variablen-Düngungs-Plane (VRA für Sämaschinen, Spritzer, Streuer)
- [ ] GPS-Auto-Steuerung (Anbindung an Lenksysteme)
- [ ] Drohnen-Spritzen-Integration (Management + Steuerung)
- [ ] Automatisierte Bewässerungssteuerung (IoT-Ventile)

#### Nachhaltigkeit & Compliance (P3)
- [ ] Kohlenstoff-Gutschriften-Tracking (CO2-Sequestrierung messen + berichten)
- [ ] Wasser-Nutzungs-Monitoring (Bewässerungseffizienz, regulatorische Compliance)
- [ ] Chemische-Anwendungs-Logs (REI, beschränkte Verwendung)
- [ ] Biologische-Zertifizierung (Eingangs-Tracking, Pufferzonen, Inspektionen)
- [ ] Nachhaltigkeits-Metriken (Boden-Gesundheit, Biodiversität, Dünger-Reduzierung)

#### Fortgeschrittene Business-Features (P3)
- [ ] Multi-Betriebs-Management (Haltereien, Pachtverträge, Mieter)
- [ ] Vertrags-Landwirtschaft (Erzeuger-Verträge, Qualitätsprämien)
- [ ] Lohnarbeits-Management (Gehälter, Zertifizierungen, Planung)
- [ ] Equipment-Sharing (Vermietungs-Marktplatz zwischen Bauern)
- [ ] Versicherungs-Integration (Schadens-Dokumentation, Risiko-Bewertung)

#### KI & Robotik (P4)
- [ ] Computer Vision (Pflanzen-Krankheiten-Erkennung, Unkraut-Identifikation)
- [ ] Roboter-Krähen (autonome Roboter-Steuerung + Monitoring)
- [ ] KI-Beratungsassistent (Chat-Interface für agronomische Fragen)
- [ ] Autonome Geräte (Flotten-Management für selbstfahrende Traktoren)

#### Fortschrittliche Technologien (P4)
- [ ] Augmented Reality (AR) — Feld-Daten-Overlay auf Live-Kamera
- [ ] Digital Twin (virtuelles Bauernhof-Modell für Szenario-Planung)
- [ ] Blockchain-Rückverfolgbarkeit (Lieferketten-Transparenz)
- [ ] Generatives KI für Planung (automatisierte Ernte/Lebensmittel-Unternehmensplanung)

---

## Phase 3: Docker & Deployment

Siehe [optimizations.md](docs/optimizations.md) §3.2 — bereits erledigt (0.8.14).

---

|

- [ ] Catalog Import Script (`scripts/import_catalog.py`): Vollständige Kataloge (VIVC Rebsorten >12k, Oliven-DB >260, FAO Tierrassen) als CSV generieren und in `varieties`/`breeds` importieren. Lazy-Load-Suche für AdminUI vorbereiten. Datenquellen: VIVC (vivc.de), FAO-DAD-IS, Olive-DB.

## Phase 4: Monitoring & Observability
- [ ] Query-Dauer-Monitoring (sqlx-Middleware oder `sqlx-metrics`)
- [ ] Pool-Auslastung (aktive/idle Verbindungen)
- [ ] Slow-Query-Erkennung + Logging

### Business Metrics
- [ ] Aktive Geräte pro Tenant
- [ ] Übertragene Telemetrie-Nachrichten pro Stunde
- [ ] Erfolgreich verarbeitete Import-Dateien

### Distributed Tracing
- [ ] Span-Attribute: Tenant-ID, User-ID, Operationstyp
- [ ] Konsistente Tracing-Standardisierung über alle Service-Grenzen
