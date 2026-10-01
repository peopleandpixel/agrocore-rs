# Agrocore-RS Open Tasks

Letztes Update: 2026-10-01

Nur offene Arbeit. Erledigte Phasen und Module sind entfernt; die Historie steht im
CHANGELOG. Reihenfolge nach Priorität, innerhalb einer Priorität nach Abhängigkeit.

Legende: **P0** blockiert den Betrieb · **P1** MVP · **P2** wichtig · **P3** Komfort ·
**P4** Zukunft.

---

## P0 — Kritisch

### Notification-Kanäle vervollständigen

Aus Phase 8 Abschnitt 3. Der Dispatcher ist produktiv, aber die Kanalliste ist unvollständig.

- [ ] Push-Kanäle: Firebase (FCM), APNs, WebPush (`crates/messaging/src/notification/channel.rs`)
- [ ] Inbound-Webhooks für den Empfang (WhatsApp, Telegram, Email-Reply)
- [ ] Weitere E-Mail- und SMS-Provider: Postmark, Vonage, Plivo, Sms77
- [ ] Tenant-User-Preferences (welcher Kanal für wen, Ruhezeiten, Eskalation)

### Backup-Service abschließen

Aus Phase 8 Abschnitt 4. Betrifft `crates/backup-service/`.

- [ ] SFTP-Host-Key-Pinning gegen eine `known_hosts`-Datei — `check_server_key` akzeptiert aktuell jeden Schlüssel (`sftp_backend.rs`), das ist ein offener MITM-Risiko
- [ ] Cloud-Downloads speicherschonend machen — `object_store` 0.11 liefert keinen asynchronen Byte-Stream, `GetResult::bytes()` lädt das Objekt komplett. Betrifft S3, Azure und GCS; Local, SFTP und WebDAV streamen bereits
- [ ] Integrationstests gegen echte Cloud-Instanzen (S3/MinIO, Azure Blob, GCS) in CI
- [ ] Integrationstests gegen echte SFTP- und WebDAV-Server
- [ ] NATS-Progress-Events (0–100 %) durch Integrationstests absichern
- [ ] Age- und KMS-Verschlüsselung — aktuell nur AES-256-GCM (`encryption.rs`)
- [ ] Disaster-Recovery-Runbook: RTO < 15 Min für eine 50-GB-Datenbank, Single Tenant

---

## P1 — MVP

### Kunden & Verkauf

- [ ] CSA-Verwaltung: Abonnement-Boxen, Lieferplanung
- [ ] Großhandelsaufträge: Staffelpreise, Lieferplanung
- [ ] Direktverkauf: Onlineshop, Zahlungsabwicklung

### Tierhaltung

- [ ] Zucht-Records: Brunft, KI, Kalbung, Paarung
- [ ] Futteraufnahme-Tracking: Ration, Verschwendung, Futterwert
- [ ] Milchproduktions-Tracking: Tagesproduktion, Butterfett, Protein
- [ ] Bewegungsdokumentation: Geburten, Todesfälle, Käufe, Verkäufe
- [ ] Weide-Management: Flächenrotation, Ruheperioden, Tierstandort

### Finanzen

- [ ] Buchhaltungs-Integration: QuickBooks, Xero, doppelte Buchführung
- [ ] Budgetierung und Prognosen: geplant versus tatsächlich
- [ ] Feldkalkulation: Eingangs- versus Ausgangswerte
- [ ] Umsatz-Tracking pro Kultur
- [ ] Geldfluss-Management: Verbindlichkeiten und Zahlungseingänge planen
- [ ] Steuerberichtswesen: Schedule F, Abschreibungen, Düngerkosten-Abzug
- [ ] Eingangskosten-Tracking: Saatgut, Dünger, Chemikalien, Kraftstoff, QR-Scans
- [ ] Abschreibung in den Finanzbericht integrieren — die Berechnung (`depreciation.rs`) und der monatliche Timer laufen, aber `/financial/reports` wertet sie noch nicht aus

### Katalog-Import

- [ ] `scripts/import_catalog.py`: vollständige Kataloge als CSV erzeugen und nach `varieties`/`breeds` importieren. Quellen: VIVC-Rebsorten (>12k), Oliven-DB (>260), FAO-Tierrassen. Lazy-Load-Suche für die Admin-UI vorbereiten

---

## P2 — Wichtig

### Monitoring & Observability

Aus Phase 4, bisher nicht begonnen.

- [ ] Query-Dauer-Monitoring (sqlx-Middleware oder `sqlx-metrics`)
- [ ] Pool-Auslastung: aktive und idle Verbindungen
- [ ] Slow-Query-Erkennung mit Logging
- [ ] Span-Attribute: Tenant-ID, User-ID, Operationstyp
- [ ] Tracing über alle Service-Grenzen hinweg standardisieren

### Business Metrics

- [ ] Aktive Geräte pro Tenant
- [ ] Übertragene Telemetrie-Nachrichten pro Stunde
- [ ] Erfolgreich verarbeitete Import-Dateien

### Mobile First

- [ ] Offline-first Mobile-App mit Sync bei wiederhergestellter Konnektivität
- [ ] Barcode- und QR-Code-Scanner: Equipment, Inventar, Feld-ID
- [ ] GPS-Feld-Grenzen (Boundary Recording)
- [ ] Mobile Zeiterfassung: Clock-In/Clock-Out mit GPS
- [ ] Sprach-zu-Text-Notizen
- [ ] Foto-Dokumentation: Anhänge an Tasks, Probleme, Inspektionen
- [ ] Push-Benachrichtigungen: Wetterwarnungen, Task-Erinnerungen
- [ ] Feldaktivitäten-Recording in Echtzeit: Pflanzen, Spritzen, Ernten
- [ ] Ernte-Daten-Import von Combine-Harvestern
- [ ] Drohnen- und UAV-Integration: NDVI-Bilder

---

## P3 — Komfort

### Präzisionslandwirtschaft

- [ ] Variable Düngungspläne: VRA für Sämaschinen, Spritzer, Streuer
- [ ] GPS-Autosteuerung: Anbindung an Lenksysteme
- [ ] Drohnen-Spritzen-Integration: Management und Steuerung
- [ ] Automatisierte Bewässerungssteuerung über IoT-Ventile

### Nachhaltigkeit & Compliance

- [ ] Kohlenstoff-Gutschriften-Tracking: CO₂-Sequestrierung messen und berichten
- [ ] Wasser-Nutzungs-Monitoring: Bewässerungseffizienz, regulatorische Compliance
- [ ] Chemische-Anwendungs-Logs: REI, beschränkte Verwendung
- [ ] Biologische Zertifizierung: Eingangs-Tracking, Pufferzonen, Inspektionen
- [ ] Nachhaltigkeits-Metriken: Bodengesundheit, Biodiversität, Düngerreduktion

### Business-Features

- [ ] Multi-Betriebs-Management: Haltereien, Pachtverträge, Mieter
- [ ] Vertragslandwirtschaft: Erzeuger-Verträge, Qualitätsprämien
- [ ] Lohnarbeits-Management: Gehälter, Zertifizierungen, Planung
- [ ] Equipment-Sharing: Vermietungsmarktplatz zwischen Betrieben
- [ ] Versicherungs-Integration: Schadensdokumentation, Risikobewertung

---

## P4 — Zukunft

### KI-Analytics (Modul 17)

- [ ] Anforderungen für Ertragsprognosen definieren
- [ ] Anforderungen für Krankheits- und Schädlingsfrühwarnsysteme definieren
- [ ] Anforderungen für Bewässerungs-, Düngungs-, KPI-, Satelliten-Monitoring- und generatives Reporting definieren
- [ ] Computer Vision: Pflanzenkrankheiten, Unkrauterkennung
- [ ] KI-Beratungsassistent: Chat-Interface für agronomische Fragen
- [ ] Generative KI für die betriebliche Planung
- [ ] Implementierung erst nach Stabilisierung der Produktionsmodule und der Sync-Grundlagen

### KI & Robotik

- [ ] Roboter-Krähen: autonome Steuerung und Monitoring
- [ ] Autonome Geräte: Flotten-Management für selbstfahrende Traktoren

### Fortgeschrittene Technologien

- [ ] Augmented Reality: Feld-Daten-Overlay auf der Live-Kamera
- [ ] Digital Twin: virtuelles Betriebsmodell für Szenario-Planung
- [ ] Blockchain-Rückverfolgbarkeit: Lieferkettentransparenz