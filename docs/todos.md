# agrocore-rs — Projekt-Roadmap

## Phase 1: Basis-API (MVP) — ABGESCHLOSSEN

### Erledigt
- [x] Workspace-Struktur (4 Crates: shared, domain, infrastructure, api)
- [x] Domain Entities (Tenant, User, Site, Order, TaskData)
- [x] Infrastructure (MongoDB Repositories, Password Hashing, JWT)
- [x] DTOs für API-Responses
- [x] JWT Auth Middleware (placeholder)
- [x] API Routes (Sites, Orders, Users, Tasks, Auth/Login)
- [x] Cargo.toml mit Dependencies
- [x] actix-web 4 als Framework (gewählt wegen Handler-Trait-Kompatibilität)
- [x] Alle Dependencies auf neuesten Stand (geo 0.33, geojson 1.0, argon2 0.6, rand 0.9)
- [x] API kompiliert fehlerfrei

### Nächste Schritte
- [ ] Server main.rs fertigstellen (actix-web HttpServer)
- [ ] CORS aktivieren (actix-cors)
- [ ] MongoDB Index-Erstellung
- [ ] Integrationstests
- [ ] JWT Auth Middleware implementieren

---

## Phase 2: Compliance & GAP-Nachweis

- [ ] Pflanzenschutz-Mittel-Buchführung (gesetzlich vorgeschrieben DE/ES/PT)
- [ ] GAP-Checklisten (EU-GAP Nachweis)
- [ ] Audit-Log / Compliance-Protokolle
- [ ] Dünge-Bilanz pro Parzelle (Nitrat-Richtlinie)
- [ ] Pflanzenschutz-Schein-Tracking (Carné de Aplicador)

---

## Phase 3: Spezialkulturen Wein & Oliven

- [ ] Wein: DOC-Gebiete (Douro, Alentejo, Vinho Verde)
- [ ] Wein: Vintage/Jahresgang-Tracking pro Parzelle
- [ ] Wein: Qualitäts-Tracking (Brix, pH, Säure bei Ernte)
- [ ] Wein: Kelter-Logistik (Trauben-Lieferung mit Gewicht/Chargen-Nr)
- [ ] Oliven: Ölgüte-Tracking (Säuregehalt, Peroxid-Zahl, Sensorik)
- [ ] Oliven: Mühlen-Lieferketten (Los-Nummern)
- [ ] Oliven: Klassifizierung (Extra Vergine, Vergine, Lampante)

---

## Phase 4: Wassermanagement

- [ ] Wassermengen-Erfassung (m³/ha)
- [ ] Bewässerungs-Quellen (Brunnen, Bach, Reservoir)
- [ ] Wasser-Rechte / Lizenzen (Concesión de aprovechamento hídrico)
- [ ] Comunidades de Regantes (Wassergemeinschaften, Mitgliedschaft & Quoten)
- [ ] Trocken-Stress-Indikatoren

---

## Phase 5: Arbeitergesetzgebung

- [ ] Jornada-Arbeitszeit (Spanisches Gesetz, Überstunden, Ruhezeiten)
- [ ] Temporeros-Saisonarbeiter (contrato temporal)
- [ ] Sprachen-Unterstützung für Arbeiter (Marokko, Rumänien)
- [ ] Schulungsnachweise-Tracking

---

## Phase 6: Finanzen & EU-Beihilfen

- [ ] PAC (Política Agrícola Común) — EU-Agrarzahlungen pro Parzelle
- [ ] Kreislaufwirtschaft-Beihilfen (Eco-esquemas in ES)
- [ ] Kosten-Stellenrechnung (pro Parzelle, pro Kulture, pro Arbeitsgang)
- [ ] SIGPAC-Parzellen-IDs (offizielle EU-Flurstücks-IDs)
- [ ] REGEPAC (Registro de Explotaciones Agrarias)

---

## Phase 7: Ernte-Logistik & Qualität

- [ ] Los-Tracking / Chargen-Nummern
- [ ] Wiegung (Brutto/Netto-Gewicht pro Ernte-Lieferung)
- [ ] Kühlketten-Protokolle
- [ ] Ernte-Jahresgang-Verknüpfung

---

## Phase 8: Wetter & Klima

- [ ] Wetter-Stationen-Integration (Temperatur, Niederschlag, Luftfeuchte)
- [ ] Frost-Warnungen
- [ ] BBCH-Phänologie-Modell mit Vorhersage

---

## Phase 9: Reporting & Export

- [ ] PAC-Antragstellung (einheitliches Antragsformular SIP)
- [ ] Excel-Export (rust_xlsxwriter)
- [ ] GeoJSON-Export für Karten
- [ ] OpenAPI/Swagger-Dokumentation (utoipa)

---

## Technische Debt

- [ ] actix-web Server main.rs fertigstellen
- [ ] CORS aktivieren
- [ ] MongoDB Index-Erstellung
- [ ] JWT Auth Middleware implementieren
- [ ] Tests schreiben
- [ ] alte src/ Verzeichnis entfernen
- [ ] utoipa/utoipa-swagger-ui wieder einbinden
