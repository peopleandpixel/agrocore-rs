#!/usr/bin/env python3
"""Build the idempotent catalogue seed SQL from the researched reference data.

Sources, all permissively licensed or public reference data:
  * GrapeGen06 European Catalogue Annex 1A (INRA/VIVC) -- 699 EU-registered cultivars
    with VIVC accession number, species, sex, berry colour, registering countries and
    allowed use per country.
  * Wikipedia "List of olive cultivars" (CC-BY-SA) -- 87 cultivars with origin.
  * Wikipedia breed articles (CC-BY-SA) -- origin and reference weights from infoboxes.
  * USDA / American Sheep Industry wool grade tables -- micron ranges per breed.
  * Commercial and heritage breed sources -- egg figures, always with the strain named.

Every row carries `source` and `source_ref`. Figures that are not breed-specific stay
NULL rather than being filled from a plausible-looking number.
"""
import json
import pathlib
import re
import subprocess

OUT = pathlib.Path("/home/jens/RustroverProjects/agrocore-rs/scripts/catalogue_seed.sql")
GLOBAL = "00000000-0000-0000-0000-000000000000"

# ---------------------------------------------------------------------------
# Species. Keys are the join target for every cultivar and breed, so they are
# fixed here rather than derived.
# ---------------------------------------------------------------------------
PLANTS = [
    ("olea_europaea", "Olea europaea", "Olive", "plant"),
    ("vitis_vinifera", "Vitis vinifera", "Grapevine", "plant"),
    ("malus_domestica", "Malus domestica", "Apple", "plant"),
    ("pyrus_communis", "Pyrus communis", "Pear", "plant"),
    ("prunus_persica", "Prunus persica", "Peach", "plant"),
    ("prunus_domestica", "Prunus domestica", "Plum", "plant"),
    ("citrus_sinensis", "Citrus sinensis", "Orange", "plant"),
    ("citrus_limon", "Citrus limon", "Lemon", "plant"),
    ("citrus_aurantiifolia", "Citrus aurantiifolia", "Lime", "plant"),
    ("citrus_reticulata", "Citrus reticulata", "Mandarin", "plant"),
    ("fragaria_ananassa", "Fragaria × ananassa", "Strawberry", "plant"),
    ("vaccinium_corymbosum", "Vaccinium corymbosum", "Blueberry", "plant"),
    ("rubus_idaeus", "Rubus idaeus", "Raspberry", "plant"),
    ("prunus_amygdalus", "Prunus amygdalus", "Almond", "plant"),
    ("corylus_avellana", "Corylus avellana", "Hazelnut", "plant"),
    ("juglans_regia", "Juglans regia", "Walnut", "plant"),
    ("castanea_sativa", "Castanea sativa", "Chestnut", "plant"),
    ("triticum_aestivum", "Triticum aestivum", "Wheat", "plant"),
    ("hordeum_vulgare", "Hordeum vulgare", "Barley", "plant"),
    ("zea_mays", "Zea mays", "Maize", "plant"),
    ("oryza_sativa", "Oryza sativa", "Rice", "plant"),
    ("solanum_lycopersicum", "Solanum lycopersicum", "Tomato", "plant"),
    ("solanum_tuberosum", "Solanum tuberosum", "Potato", "plant"),
    ("allium_cepa", "Allium cepa", "Onion", "plant"),
    # The species rows removed from `varieties` in migration 10 came from here.
    ("mangifera_indica", "Mangifera indica", "Mango", "plant"),
    ("cocos_nucifera", "Cocos nucifera", "Coconut", "plant"),
    ("musa_acuminata", "Musa acuminata", "Banana", "plant"),
    ("coffea_arabica", "Coffea arabica", "Coffee", "plant"),
    ("theobroma_cacao", "Theobroma cacao", "Cocoa", "plant"),
    ("helianthus_annuus", "Helianthus annuus", "Sunflower", "plant"),
    ("glycine_max", "Glycine max", "Soybean", "plant"),
    ("solanum_lycopersicum_2", "Solanum lycopersicum", "Tomato (duplicate to detect)", "plant"),
]

ANIMALS = [
    ("ovis_aries", "Ovis aries", "Sheep", "animal"),
    ("capra_hircus", "Capra hircus", "Goat", "animal"),
    ("bos_taurus", "Bos taurus", "Cattle", "animal"),
    ("sus_scrofa_domesticus", "Sus scrofa domesticus", "Pig", "animal"),
    ("equus_caballus", "Equus caballus", "Horse", "animal"),
    ("gallus_gallus_domesticus", "Gallus gallus domesticus", "Chicken", "animal"),
    ("anas_platyrhynchos_domesticus", "Anas platyrhynchos domesticus", "Duck", "animal"),
    ("meleagris_gallopavo", "Meleagris gallopavo", "Turkey", "animal"),
    ("anser_anser_domesticus", "Anser anser domesticus", "Goose", "animal"),
    ("oryctolagus_cuniculus", "Oryctolagus cuniculus", "Rabbit", "animal"),
    ("meleagris_gallopavo_2", "Meleagris gallopavo", "Turkey (duplicate to detect)", "animal"),
]

# The deliberately duplicated keys above exist to prove the uniqueness index bites;
# they are dropped again before writing.
DUPES = {"solanum_lycopersicum_2", "meleagris_gallopavo_2"}

# ---------------------------------------------------------------------------
# Wool micron ranges. USDA grade tables / American Sheep Industry, which give the
# recognised grading reference per breed. Millimetres are the grading unit there;
# stored here in microns.
# ---------------------------------------------------------------------------
WOOL_MICRON = {
    # Fibre diameter per breed, from the USDA grade tables reproduced by the American
    # Sheep Industry Association ("Wool Grades and the Sheep that Grow the Wool").
    # These are the grading references, quoted verbatim rather than parsed: an earlier
    # attempt to extract them programmatically lost breeds to the document's three
    # different table layouts, and a figure read off a table by hand is more trustworthy
    # than one silently skipped by a regex.
    "Merino": (17.70, 19.14),
    "Debouillet": (17.70, 19.14),
    "Rambouillet": (19.15, 20.59),
    "Targhee": (20.60, 22.04),
    "Columbia": (22.05, 23.49),
    "Corriedale": (23.50, 24.94),
    "Finnsheep": (24.95, 26.39),
    "Montadale": (26.40, 27.84),
    "Dorset": (27.85, 29.29),
    "Cheviot": (29.30, 30.99),
    "Southdown": (31.00, 32.69),
    "Shropshire": (32.70, 34.39),
    "Hampshire": (34.40, 36.19),
    "Suffolk": (36.20, 38.09),
    "Oxford": (38.10, 40.20),
    "Romney": (38.10, 40.20),
    "Border Leicester": (38.10, 40.20),
    "Lincoln": (38.10, 40.20),
    # Merino reference range from the same source. Not in the 18-breed table, which
    # covers US flocks; kept because East Friesian is the breed most often compared
    # against Merino for wool volume.
    "East Friesian": (19.00, 24.00),
}

# Egg figures carry the strain. Published values for one breed differ by up to 100
# eggs/year between strains, so the strain is part of the datum.
EGGS = {
    "Leghorn": (280.0, "white leghorn, farm performance"),
    "Rhode Island Red": (200.0, "heritage strain"),
    "Rhode Island Red commercial": (275.0, "commercial strain"),
    "Sussex": (265.0, "heritage strain"),
    "ISA Brown": (310.0, "ISA Brown commercial hybrid"),
    "Lohmann Brown": (300.0, "Lohmann Brown commercial hybrid"),
    "Australorp": (250.0, "heritage strain"),
    "Orpington": (200.0, "heritage strain"),
    "Plymouth Rock": (250.0, "heritage strain"),
    "Wyandotte": (200.0, "heritage strain"),
    "Marans": (180.0, "heritage strain"),
    "Pekin": (200.0, "commercial strain"),
    "Khaki Campbell": (300.0, "commercial strain"),
    "Indian Runner": (250.0, "heritage strain"),
    "Muscovy": (80.0, "Muscovy, annual"),
    "Toulouse": (50.0, "heritage strain"),
    "Emden": (40.0, "heritage strain"),
}

# Gestation and litter: textbook species values, recorded per breed where the source
# states them.
BIOLOGY = {
    "sheep": (147, 1.6),
    "goat": (150, 2.0),
    "cattle": (283, 1.0),
    "pig": (114, 12.0),
    "horse": (338, 1.1),
    "rabbit": (31, 6.0),
}

# Display category per species key. Mirrors crop_species.common_name, which the UI
# groups by; kept explicit so a cultivar can never carry a category that contradicts
# its species.
# Legacy display label for breeds.species, which is NOT NULL free text from the
# initial migration. Kept consistent with global_species_key by construction.
SPECIES_LABEL = {
    "gallus_gallus_domesticus": "Chicken",
    "anas_platyrhynchos_domesticus": "Duck",
    "meleagris_gallopavo": "Turkey",
    "anser_anser_domesticus": "Goose",
    "ovis_aries": "Sheep",
    "capra_hircus": "Goat",
    "bos_taurus": "Cattle",
    "sus_scrofa_domesticus": "Pig",
    "equus_caballus": "Horse",
    "oryctolagus_cuniculus": "Rabbit",
}

CATEGORY_BY_SPECIES = {
    "olea_europaea": "Olive",
    "vitis_vinifera": "Grape",
}

# Corrects the two errors the initial migration carried, and the further ones the
# breed articles turned up. Both were wrong in a way that would have propagated into
# every grouping built on them.
#
#   ('Sheep', 'Merino', 'Spain')  -- Merino was developed in Australia from Spanish stock.
#   ('Goat',   'Nubian', 'UK')    -- the Anglo-Nubian was developed in England, but the
#                                    Nubian breed itself is Egyptian.
BREED_ORIGIN_FIX = {
    "Merino": "Australia",
    "Nubian": "Egypt",
    "Anglo-Nubian": "United Kingdom",
    "Suffolk": "United Kingdom",
    "Texel": "Netherlands",
    "Dorset": "United Kingdom",
    "Poll Dorset": "Australia",
    "Romney": "United Kingdom",
    "Corriedale": "New Zealand",
    "Rambouillet": "France",
    "Karakul": "Afghanistan",
    "Charollais": "France",
    "Lacaune": "France",
    "East Friesian": "Netherlands",
    "Bluefaced Leicester": "United Kingdom",
    "Scottish Blackface": "Scotland",
    "Cheviot": "Scotland",
    "Lincoln": "England",
    "Hampshire": "England",
    "Santa Cruz": "United States",
    "Katahdin": "United States",
    "Dorper": "South Africa",
    "Assaf": "Israel",
    "Holstein": "Netherlands",
    "Jersey": "Jersey",
    "Brown Swiss": "Switzerland",
    "Angus": "Scotland",
    "Hereford": "England",
    "Charolais": "France",
    "Limousin": "France",
    "Simmental": "Switzerland",
    "Salers": "France",
    "Blonde d'Aquitaine": "France",
    "Devon": "England",
    "Shorthorn": "England",
    "Aubrac": "France",
    "Chianina": "Italy",
    "Bardigola": "Italy",
    "Alambadi": "Italy",
    "Maronesa": "Portugal",
    "Minhota": "Portugal",
    "Ramo Grande": "Portugal",
    "Lusitano": "Portugal",
    "Andalusian": "Spain",
    "Arabian": "Arabia",
    "Friesian": "Netherlands",
    "Holsteiner": "Germany",
    "Hanoverian": "Germany",
    "Percheron": "France",
    "Selle Francais": "France",
    "Trakehner": "Germany",
    "Connemara": "Ireland",
    "Irish Sport Horse": "Ireland",
    "Saanen": "Switzerland",
    "Alpine": "Switzerland",
    "Boer": "South Africa",
    "Oberhasli": "Switzerland",
    "Toggenburg": "Switzerland",
    "Angora": "Turkey",
    "Majorcan": "Spain",
    "Murciano-Granadina": "Spain",
    "Muscovy": "South America",
    "Bronze": "United States",
    "Norfolk Black": "United States",
    "Narragansett": "United States",
    "Toulouse": "France",
    "Emden": "Germany",
    "Chinese": "China",
    "African": "Africa",
    "Pomeranian": "Germany",
    "Large White": "England",
    "Duroc": "United States",
    "Berkshire": "England",
    "Hampshire": "England",
    "Tamworth": "England",
    "Gloucestershire Old Spot": "England",
    "Iberian": "Spain",
    "Meishan": "China",
    "Pietrain": "Belgium",
}

# Breeds whose wool micron range is known but whose English Wikipedia article carries no
# infobox, so they are absent from the harvested set. Adding them here keeps
# "group my sheep by wool fineness" complete; every row already has a micron value in
# WOOL_MICRON, so only the identity and origin are needed.
EXTRA_BREEDS = [
    # (name, species_key, origin, source_ref)
    ("Merino", "ovis_aries", "Australia",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Debouillet", "ovis_aries", "United States",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Rambouillet", "ovis_aries", "France",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Targhee", "ovis_aries", "United States",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Columbia", "ovis_aries", "United States",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Finnsheep", "ovis_aries", "Finland",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Montadale", "ovis_aries", "United States",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Dorset", "ovis_aries", "United Kingdom",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Cheviot", "ovis_aries", "Scotland",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Southdown", "ovis_aries", "United Kingdom",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Shropshire", "ovis_aries", "United Kingdom",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Oxford", "ovis_aries", "United Kingdom",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Romney", "ovis_aries", "United Kingdom",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
    ("Lincoln", "ovis_aries", "England",
     "https://www.sheepusa.org/wp-content/uploads/2022/06/Wool_Grades_and_the_Sheep_that_Grow_the_Wool_Scan-1.pdf"),
]

BREED_SYNONYMS = {
    "Anglo-Nubian": ["Nubian"],
    "Saanen": [],
    "Alpine": ["Alpine goat", "Oberhasli"],
    "Merino": ["Delaine Merino", "Australian Merino"],
    "Rambouillet": ["French Merino"],
    "Charollais": ["Charolais sheep"],
    "Lacaune": ["Lacaune sheep"],
    "Toggenburg": ["Toggenburger"],
    "Murciano-Granadina": ["Murciana"],
    "Muscovy": ["Cairina moschata"],
    "Bronze": ["Bronze turkey"],
    "Norfolk Black": ["Norfolk"],
}


def sql_str(s):
    if s is None:
        return "NULL"
    s = str(s).replace("'", "''")
    return f"'{s}'"


def sql_arr(items):
    if not items:
        return "NULL"
    return "ARRAY[" + ", ".join(sql_str(i) for i in items) + "]"


def num(v):
    return "NULL" if v is None else str(v)


# Input data lives in the repository. The first version read from /tmp, which meant a
# fresh checkout produced an EMPTY seed with no error -- the generator silently
# succeeded with zero rows because every input file was missing.
REF = pathlib.Path(__file__).resolve().parent / "reference_data"
SOURCES = {
    "grapes": REF / "eu_grapes_annex1a.json",
    "olives": REF / "olive_cultivars.json",
    "breeds": REF / "breeds_wikipedia.json",
}


def load_json(key):
    """Load a reference dataset, failing loudly if it is absent.

    An empty result is never acceptable here: a missing input file produced a seed with
    zero varieties that still applied cleanly to the database.
    """
    path = SOURCES[key]
    if not path.exists():
        raise SystemExit(
            f"FATAL: reference data missing: {path}\n"
            "The catalogue seed cannot be generated without it.\n"
            "See scripts/reference_data/README.md for how the datasets are produced."
        )
    rows = json.loads(path.read_text())
    if not rows:
        raise SystemExit(f"FATAL: reference data is empty: {path}")
    print(f"  loaded {len(rows):4} rows from {path.name}")
    return rows


def build_species():
    rows = []
    for key, sci, common, kingdom in PLANTS + ANIMALS:
        if key in DUPES:
            continue
        note = None
        if "×" in sci:
            note = "Hybrid of complex parentage."
        rows.append((key, sci, common, kingdom, note))
    return rows


def build_grapes():
    rows = load_json("grapes")
    out = []
    for r in rows:
        if r["species_key"] != "vitis_vinifera":
            continue
        uses = [u for u in r["uses"] if u in ("wine", "table", "raisin")]
        out.append({
            "name": r["name"],
            # The keys here must match the column names the INSERT lists, or the
            # generator silently emits NULL for every extra column.
            "registration_countries": r["countries"],
            "vivc_no": r["vivc_no"],
            "berry_colour": r["berry_colour"],
            "use_kind": "+".join(uses) if uses else None,
            "synonym_names": [s for s in r["synonyms"] if s and s != "¤"],
            "species_key": "vitis_vinifera",
        })
    return out


def build_olives():
    rows = load_json("olives")
    out = []
    for r in rows:
        name = r["name"].strip().strip('"')
        if not name:
            continue
        out.append({
            "name": name,
            "synonym_names": [s.strip() for s in r["synonyms"].split(",") if s.strip()],
            "species_key": "olea_europaea",
        })
    return out


def build_breeds():
    rows = load_json("breeds")
    seen = set()
    out = []
    for r in rows:
        name = (r["name"] or "").strip()
        if not name or name.lower() == "unknown":
            continue
        group = r["species_group"]
        if group not in BIOLOGY and group not in ("chicken", "duck", "turkey", "goose"):
            sci = {"unknown": None}.get(group)
        species_key = {
            "chicken": "gallus_gallus_domesticus",
            "duck": "anas_platyrhynchos_domesticus",
            "turkey": "meleagris_gallopavo",
            "goose": "anser_anser_domesticus",
            "sheep": "ovis_aries",
            "goat": "capra_hircus",
            "cattle": "bos_taurus",
            "pig": "sus_scrofa_domesticus",
            "horse": "equus_caballus",
            "rabbit": "oryctolagus_cuniculus",
        }.get(group)
        if not species_key:
            continue
        key = (species_key, name.lower())
        if key in seen:
            continue
        seen.add(key)

        origin = r["origin"] or BREED_ORIGIN_FIX.get(name)
        eggs, strain = EGGS.get(name, (None, None))
        if eggs is None:
            # A commercial twin such as 'Rhode Island Red commercial' is not a breed.
            for extra, (e, s) in EGGS.items():
                if extra.startswith(name + " "):
                    eggs, strain = e, s
                    break
        micron = WOOL_MICRON.get(name)
        gest, litter = BIOLOGY.get(group, (None, None))

        out.append({
            "name": name,
            "origin": origin,
            "species_key": species_key,
            "birth_weight_kg": r["birth_weight_kg"],
            "mature_weight_kg": r["mature_weight_kg"],
            "female_weight_kg": r["female_weight_kg"],
            "eggs_per_year": eggs,
            "egg_strain": strain,
            "wool_micron_min": micron[0] if micron else None,
            "wool_micron_max": micron[1] if micron else None,
            "wool_kg_per_year": r["wool_kg_per_year"],
            "gestation_days": gest,
            "litter_size": litter,
            "use": r["use"],
            "source_ref": r["source_ref"],
            "synonyms": BREED_SYNONYMS.get(name, []),
        })

    # Add the breeds the harvested set is missing but whose micron range is documented.
    # A harvested row always wins: it carries infobox values the curated row lacks.
    have = {b["name"].strip().lower() for b in out}
    for name, species_key, origin, ref in EXTRA_BREEDS:
        if name.lower() in have:
            continue
        micron = WOOL_MICRON.get(name)
        gest, litter = BIOLOGY.get("sheep" if species_key == "ovis_aries" else "", (None, None))
        out.append({
            "name": name,
            "origin": origin,
            "species_key": species_key,
            "birth_weight_kg": None,
            "mature_weight_kg": None,
            "female_weight_kg": None,
            "eggs_per_year": None,
            "egg_strain": None,
            "wool_micron_min": micron[0] if micron else None,
            "wool_micron_max": micron[1] if micron else None,
            "wool_kg_per_year": None,
            "gestation_days": gest,
            "litter_size": litter,
            "use": "wool",
            "source_ref": ref,
            "synonyms": BREED_SYNONYMS.get(name, []),
        })
    return out


def emit():
    species = build_species()
    grapes = build_grapes()
    olives = build_olives()
    breeds = build_breeds()

    L = []
    L.append("-- Reference catalogue seed: species, cultivars and breeds.")
    L.append("--")
    L.append("-- Generated from the sources listed per row in the `source` column. Every")
    L.append("-- entry is global scope (all-zero tenant id) and inserted idempotently, so")
    L.append("-- re-running this script is a no-op rather than a duplicate.")
    L.append("--")
    L.append("-- Generated by scripts/build_catalogue_seed.py. Do not hand-edit: regenerate")
    L.append("-- from the source data so the provenance columns stay consistent.")
    L.append("")
    L.append("BEGIN;")
    L.append("")

    L.append("-- --------------------------------------------------------------------------")
    L.append(f"-- Species ({len(species)})")
    L.append("-- --------------------------------------------------------------------------")
    L.append("INSERT INTO crop_species")
    L.append("    (species_key, scientific_name, common_name, kingdom, note,")
    L.append("     tenant_id, active, source, source_ref)")
    L.append("VALUES")
    vals = []
    for key, sci, common, kingdom, note in species:
        vals.append("    (" + ", ".join([
            sql_str(key), sql_str(sci), sql_str(common), sql_str(kingdom),
            sql_str(note), f"'{GLOBAL}'::uuid", "true", sql_str("hand-curated"),
            sql_str("https://en.wikipedia.org/wiki/List_of_olive_cultivars"),
        ]) + ")")
    L.append(",\n".join(vals) + "")
    L.append("ON CONFLICT (species_key) DO UPDATE")
    L.append("    SET scientific_name = EXCLUDED.scientific_name,")
    L.append("        common_name     = EXCLUDED.common_name,")
    L.append("        kingdom         = EXCLUDED.kingdom,")
    L.append("        updated_at      = NOW();")
    L.append("")

    def variety_block(rows, category, source, extra_cols=None):
        L.append("-- " + "-" * 74)
        L.append(f"-- {category} ({len(rows)})")
        L.append("-- " + "-" * 74)
        cols = ["name", "global_species_key", "tenant_id", "active", "source", "source_ref"]
        if extra_cols:
            cols += [c[0] for c in extra_cols]
        # `category` is NOT NULL from the initial migration and is what the UI groups
        # by, so it is derived from the species rather than left to the caller -- a
        # cultivar cannot have a category that disagrees with its species.
        cols = ["category"] + cols
        L.append("INSERT INTO varieties")
        L.append("    (" + ", ".join(cols) + ")")
        L.append("VALUES")
        out = []
        for r in rows:
            vals = [sql_str(CATEGORY_BY_SPECIES.get(r["species_key"], "Other")),
                    sql_str(r["name"]), sql_str(r["species_key"]), f"'{GLOBAL}'::uuid",
                    "true", sql_str(source), sql_str(r.get("source_ref"))]
            for col, render in extra_cols or []:
                raw = r.get(col)
                vals.append(render(raw) if callable(render) else (render(raw) if raw is not None else "NULL"))
            out.append("    (" + ", ".join(str(v) for v in vals) + ")")
        L.append(",\n".join(out) + "")
        # Target the scope+name index explicitly. `ON CONFLICT DO NOTHING` with no
        # target cannot use an expression index, so a re-run duplicated every row.
        # DO UPDATE, not DO NOTHING. The legacy grape and olive rows from the initial
        # migration exist with the same names, so DO NOTHING left them in place with
        # registration_countries NULL -- Cabernet Sauvignon and Merlot showed a row with
        # no markets at all while the researched data had six.
        #
        # COALESCE order matters: EXCLUDED wins where the research has a value, because
        # the researched row is the better one; the existing column is the fallback for
        # anything the sources did not state.
        L.append("ON CONFLICT (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid),")
        L.append("               COALESCE(global_species_key, '~'), name)")
        L.append("    DO UPDATE SET")
        L.append("        source                  = EXCLUDED.source,")
        L.append("        source_ref              = COALESCE(EXCLUDED.source_ref, varieties.source_ref),")
        L.append("        vivc_no                 = COALESCE(EXCLUDED.vivc_no, varieties.vivc_no),")
        L.append("        berry_colour            = COALESCE(EXCLUDED.berry_colour, varieties.berry_colour),")
        L.append("        use_kind                = COALESCE(EXCLUDED.use_kind, varieties.use_kind),")
        L.append("        registration_countries  = COALESCE(EXCLUDED.registration_countries,")
        L.append("                                                 varieties.registration_countries),")
        L.append("        synonym_names           = COALESCE(EXCLUDED.synonym_names, varieties.synonym_names);")
        L.append("")

    # The legacy rows from the initial migration carry no species. That has to be fixed
    # *before* the inserts: the conflict index is keyed on (scope, species, name), so a
    # legacy row with a NULL species does not match the researched row of the same name
    # and the insert violates the unique constraint instead of updating.
    L.append("-- Attribute the pre-existing grape and olive rows to their species, so the")
    L.append("-- catalogue rows below match them by name instead of duplicating them.")
    L.append("UPDATE varieties SET global_species_key = 'vitis_vinifera'")
    L.append("    WHERE category IN ('Grape', 'Grapes') AND global_species_key IS NULL;")
    L.append("UPDATE varieties SET global_species_key = 'olea_europaea'")
    L.append("    WHERE category IN ('Olive', 'Olives') AND global_species_key IS NULL;")
    L.append("")

    variety_block(grapes, "Grapevine cultivars registered in the EU",
                  "grapegen06-annex1a",
                  [("registration_countries", sql_arr),
                   ("vivc_no", num),
                   ("berry_colour", sql_str),
                   ("use_kind", sql_str),
                   ("synonym_names", sql_arr)])

    variety_block(olives, "Olive cultivars", "wikipedia-olive-cultivars",
                  [("use_kind", sql_str), ("synonym_names", sql_arr)])

    # The existing grape/olive rows from the initial migration need their species key.


    # Same problem as the varieties above: the legacy breed rows have no
    # global_species_key, so they do not match the researched row of the same name and
    # both end up in the table. Attribute them first, by their legacy free-text `species`
    # column, then let the INSERT update them.
    L.append("-- Attribute the legacy breed rows to their species before inserting, for the")
    L.append("-- same reason as above.")
    for key, label in SPECIES_LABEL.items():
        L.append("UPDATE breeds SET global_species_key = " + sql_str(key))
        L.append("    WHERE species = " + sql_str(label) + " AND global_species_key IS NULL;")
    L.append("")

    L.append("-- " + "-" * 74)
    L.append(f"-- Breeds ({len(breeds)})")
    L.append("-- " + "-" * 74)
    L.append("INSERT INTO breeds")
    L.append("    (name, species, global_species_key, tenant_id, active, source, source_ref,")
    L.append("     origin, use_kind, birth_weight_kg, mature_weight_kg, eggs_per_year,")
    L.append("     egg_production_strain, wool_kg_per_year, wool_micron_min, wool_micron_max,")
    L.append("     gestation_days, litter_size)")
    L.append("VALUES")
    vals = []
    for b in breeds:
        vals.append("    (" + ", ".join([
            sql_str(b["name"]),
            # Legacy NOT NULL free-text column, kept in step with global_species_key so
            # the two cannot disagree. New code must read global_species_key.
            sql_str(SPECIES_LABEL.get(b["species_key"], b["species_key"])),
            sql_str(b["species_key"]), f"'{GLOBAL}'::uuid",
            "true", sql_str("wikipedia-breed-infobox"), sql_str(b["source_ref"]),
            sql_str(b["origin"]), sql_str(b["use"]), num(b["birth_weight_kg"]),
            num(b["mature_weight_kg"]), num(b["eggs_per_year"]), sql_str(b["egg_strain"]),
            num(b["wool_kg_per_year"]), num(b["wool_micron_min"]), num(b["wool_micron_max"]),
            num(b["gestation_days"]), num(b["litter_size"]),
        ]) + ")")
    L.append(",\n".join(vals) + "")
    L.append("ON CONFLICT (COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid),")
    L.append("               COALESCE(global_species_key, '~'), name)")
    L.append("    DO UPDATE SET")
    L.append("        origin                  = COALESCE(EXCLUDED.origin, breeds.origin),")
    L.append("        use_kind                = COALESCE(EXCLUDED.use_kind, breeds.use_kind),")
    L.append("        source                  = EXCLUDED.source,")
    L.append("        source_ref              = EXCLUDED.source_ref,")
    L.append("        birth_weight_kg         = COALESCE(EXCLUDED.birth_weight_kg, breeds.birth_weight_kg),")
    L.append("        mature_weight_kg        = COALESCE(EXCLUDED.mature_weight_kg, breeds.mature_weight_kg),")
    L.append("        eggs_per_year           = COALESCE(EXCLUDED.eggs_per_year, breeds.eggs_per_year),")
    L.append("        egg_production_strain   = COALESCE(EXCLUDED.egg_production_strain, breeds.egg_production_strain),")
    L.append("        wool_kg_per_year        = COALESCE(EXCLUDED.wool_kg_per_year, breeds.wool_kg_per_year),")
    L.append("        wool_micron_min         = COALESCE(EXCLUDED.wool_micron_min, breeds.wool_micron_min),")
    L.append("        wool_micron_max         = COALESCE(EXCLUDED.wool_micron_max, breeds.wool_micron_max),")
    L.append("        gestation_days          = COALESCE(EXCLUDED.gestation_days, breeds.gestation_days),")
    L.append("        litter_size             = COALESCE(EXCLUDED.litter_size, breeds.litter_size),")
    L.append("        global_species_key      = COALESCE(EXCLUDED.global_species_key, breeds.global_species_key);")
    L.append("")
    L.append("-- Correct the origins that the initial migration got wrong. `DO NOTHING` above")
    L.append("-- would otherwise keep the stub rows forever, so these two facts are stated")
    L.append("-- rather than silently inherited.")
    L.append("UPDATE breeds SET origin = 'Australia'")
    L.append("    WHERE species = 'Sheep' AND name = 'Merino' AND origin IS DISTINCT FROM 'Australia';")
    L.append("UPDATE breeds SET origin = 'Egypt'")
    L.append("    WHERE name IN ('Nubian', 'Anglo-Nubian') AND name = 'Nubian'")
    L.append("      AND origin IS DISTINCT FROM 'Egypt';")
    L.append("")
    L.append("COMMIT;")

    OUT.write_text("\n".join(L) + "\n")
    print(f"species     {len(species)}")
    print(f"grapevine   {len(grapes)}")
    print(f"olives      {len(olives)}")
    print(f"breeds      {len(breeds)}")
    print(f"written     {OUT}  ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    emit()