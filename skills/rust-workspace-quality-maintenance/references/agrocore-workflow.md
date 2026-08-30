---
description: Agrocore-rs domain patterns: entities, migrations, groups, varieties, breeds
triggers:
  - Neue Entities (Livestock, Tree, Building, Group, Variety, Breed)
  - Migrations mit plot_id, group_id, parent_group_id
  - CSV-Referenzdaten für Varieties und Breeds
version: 1.0.0
---
# Agrocore Domain Patterns

Wenn neue Features hinzugefügt werden: IMMER ALLES anlegen (Domain-Model, Migration, AdminUI, Locale, Tests, Seed/CSV-Referenz). Keine halben Zustände.

Referenzen:
- references/varieties.csv
- references/breeds.csv
- Migrations: 001-008 (livestock, trees, buildings, groups, varieties, breeds, seeds)
