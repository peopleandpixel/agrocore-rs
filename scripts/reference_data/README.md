# Reference data

Datasets consumed by `scripts/build_catalogue_seed.py` to generate
`scripts/catalogue_seed.sql`.

## Why these files are checked in

The generator reads them at run time. An earlier version read from `/tmp`, which meant a
fresh checkout produced a seed with **zero** varieties and still applied cleanly to the
database — no error, no warning, just an empty catalogue. The generator now resolves
paths relative to its own location and exits with a message if a file is missing or
empty.

## Provenance

All datasets are public reference data or permissively licensed encyclopaedia content.
Each row in the generated seed carries `source` and `source_ref` columns, so any figure
can be traced back to the document it came from.

| File | Source | Licence |
|------|--------|---------|
| `eu_grapes_annex1a.json` | GrapeGen06 "Grapevine European Catalogue", Annex 1A — the grape varieties registered in EU Member States, with VIVC accession number, species, sex, berry colour and allowed use per country. Published by the INRA GrapeGen06 project, distributed via VIVC (Julius Kühn-Institut). | Public research data |
| `olive_cultivars.json` | Wikipedia, "List of olive cultivars" — name, synonyms and country of origin for 87 cultivars. | CC BY-SA 4.0 |
| `breeds_wikipedia.json` | Wikipedia breed articles — country of origin and reference weights, harvested from the structured breed infoboxes only. Disambiguation pages and articles without an infobox are excluded. | CC BY-SA 4.0 |
| `us_grapes_nass.json` | USDA National Agricultural Statistics Service, *California Grape Acreage Report 2024 Summary* — every variety with bearing and non-bearing acres, plus the report's own synonym table. | US public domain |
| `ar_grapes_inv.json` | Argentina, Instituto Nacional de Vitivinicultura, Resolución 18/2022 (Boletín Oficial 12 July 2022) and Resolución 20/2024 — the varieties recognised as suitable for quality wine, grouped as the resolution groups them. | Public regulation |
| `cl_grapes_sag.json` | Chile, Servicio Agrícola y Ganadero, Catastro Vitícola Nacional — planted area per variety. Only the varieties the catastro names are included; Chile has no full public varietal list in English. | CL public data |

## National lists outside the EU

`eu_grapes_annex1a.json` now carries 882 cultivars across 27 countries. The EU catalogue
(Annex 1A) supplied the original 699; the rest were merged in from:

| Country | Source | Cultivars |
|---------|--------|----------|
| Australia | Wikipedia "List of Australian wine grape varieties" | 179 |
| United States | USDA NASS California Grape Acreage Report 2024 | 80 |
| New Zealand | Wikipedia "New Zealand wine" | 43 |
| Argentina | INV Resoluciones 18/2022 and 20/2024 | 19 |
| Chile | SAG Catastro Vitícola Nacional | 8 |
| South Africa | Wikipedia "South African wine" | 5 |

Countries are stored on the cultivar row rather than as separate rows, because a cultivar
grown in Portugal, California and Australia is one cultivar with three registrations, and
splitting it would make "which varieties may I grow in my market" unanswerable by
counting.

### Matching is by VIVC number, then name, then synonym

The GrapeGen06 catalogue uses the VIVC prime name, which is frequently not the everyday
name: Chardonnay is listed as *Chardonnay Blanc* with *Chardonnay* among its synonyms, Syrah
as *Syrah* with *Shiraz*. Matching on name alone produced two rows for one cultivar and
left plain "Chardonnay" carrying only its South African registration. All three levels are
needed.

The document also prints a bare section sign `¤` where a variety has no synonym. 666 of the
699 rows carried it; the parser now drops it, since a placeholder in a name field will be
read as a variety name by whatever queries it.

## Regenerating

The datasets are snapshots. To rebuild them, fetch from the sources above and keep the
same JSON shape:

- `eu_grapes_annex1a.json` — one object per cultivar: `name`, `prime_name`, `vivc_no`,
  `species_key`, `sex`, `berry_colour`, `countries` (list), `uses` (list), `synonyms`
  (list), `source`, `source_ref`.
- `olive_cultivars.json` — `name`, `synonyms`, `origin`.
- `breeds_wikipedia.json` — `name`, `page`, `species_group`, `species_key`,
  `scientific_name`, `origin`, `use`, weights, `source_ref`.

Note on `species_group` in the breeds file: it is derived from what each article
declares, never from the file name. An earlier version matched on filename substrings
and fell through to `chicken`, which put Lusitano, Appaloosa and Holstein Friesian under
poultry.

## Values that are deliberately NULL

`eggs_per_year`, `wool_kg_per_year` and the weight columns are NULL where the source
does not state them for that breed. A plausible-looking number in a catalogue is worse
than an empty field, because it will be used in a calculation.