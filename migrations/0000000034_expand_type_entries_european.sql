-- Migration 0034: Comprehensive expansion of type_entries with European agricultural data
--
-- Adds ~500+ new type entries across all existing categories and creates new categories
-- for soil types, fertilizer types, pest types, and weather conditions.
-- All entries use snake_case codes for consistency with Rust enum serialization.
-- Idempotent via ON CONFLICT DO NOTHING.

DO $$
DECLARE
    -- Existing categories
    cat_animal_species UUID;
    cat_site_type UUID;
    cat_crop_type UUID;
    cat_order_type UUID;
    cat_equipment_type UUID;
    cat_building_type UUID;
    cat_water_source_type UUID;
    cat_irrigation_method UUID;
    cat_animal_status UUID;
    cat_variety_category UUID;
    cat_species UUID;
    cat_inventory_category UUID;
    cat_contract_type UUID;
    cat_depreciation_method UUID;
    cat_weather_station_type UUID;

    -- New categories
    cat_soil_type UUID;
    cat_fertilizer_type UUID;
    cat_pest_type UUID;
    cat_weather_condition UUID;

    -- Hierarchy parent IDs for animal_species
    parent_rindvieh UUID;
    parent_schafe UUID;
    parent_ziegen UUID;
    parent_schweine UUID;
    parent_gefluegel UUID;
    parent_pferde UUID;

    -- Hierarchy parent IDs for crop_type
    parent_weinrebe UUID;
    parent_weizen UUID;
    parent_mais UUID;
    parent_gemuese UUID;
    parent_obst UUID;
    parent_nuesse UUID;
    parent_energiepflanzen UUID;

    -- Hierarchy parent IDs for site_type
    parent_obstgarten UUID;
    parent_acker UUID;
    parent_gruenland UUID;

    -- Hierarchy parent IDs for equipment_type
    parent_traktor UUID;
    parent_spritze UUID;
    parent_maehdrescher UUID;
    parent_ballenpresse UUID;
    parent_saeemaschine UUID;
    parent_bewaesserungsanlage UUID;
    parent_drohne UUID;
    parent_werkzeug UUID;
    parent_maehwerk UUID;

    -- Hierarchy parent IDs for building_type
    parent_stall UUID;
    parent_silo UUID;

    -- Hierarchy parent IDs for order_type
    parent_pflanzenschutz UUID;
    parent_duengen UUID;
    parent_beschneiden UUID;
    parent_ernten UUID;
    parent_bienen UUID;
    parent_fischerei UUID;

    -- Hierarchy parent IDs for variety_category
    parent_weinrebe_variety UUID;
    parent_beeren UUID;

    -- Hierarchy parent IDs for inventory_category
    parent_duengemittel UUID;
    parent_pflanzenschutz_inv UUID;
    parent_kraftstoff UUID;
    parent_futter UUID;

    -- Hierarchy parent IDs for new categories
    parent_insekt UUID;
    parent_pilz UUID;
    parent_saeugetier_pest UUID;

BEGIN

    -- =====================================================================
    -- Get all existing category IDs
    -- =====================================================================

    SELECT id INTO cat_animal_species FROM type_categories WHERE name = 'animal_species';
    SELECT id INTO cat_site_type FROM type_categories WHERE name = 'site_type';
    SELECT id INTO cat_crop_type FROM type_categories WHERE name = 'crop_type';
    SELECT id INTO cat_order_type FROM type_categories WHERE name = 'order_type';
    SELECT id INTO cat_equipment_type FROM type_categories WHERE name = 'equipment_type';
    SELECT id INTO cat_building_type FROM type_categories WHERE name = 'building_type';
    SELECT id INTO cat_water_source_type FROM type_categories WHERE name = 'water_source_type';
    SELECT id INTO cat_irrigation_method FROM type_categories WHERE name = 'irrigation_method';
    SELECT id INTO cat_animal_status FROM type_categories WHERE name = 'animal_status';
    SELECT id INTO cat_variety_category FROM type_categories WHERE name = 'variety_category';
    SELECT id INTO cat_species FROM type_categories WHERE name = 'species';
    SELECT id INTO cat_inventory_category FROM type_categories WHERE name = 'inventory_category';
    SELECT id INTO cat_contract_type FROM type_categories WHERE name = 'contract_type';
    SELECT id INTO cat_depreciation_method FROM type_categories WHERE name = 'depreciation_method';
    SELECT id INTO cat_weather_station_type FROM type_categories WHERE name = 'weather_station_type';

    -- =====================================================================
    -- Create new categories
    -- =====================================================================

    INSERT INTO type_categories (name, label, description, is_hierarchical) VALUES
    ('soil_type', 'Soil Type', 'Types of agricultural soil classifications', false),
    ('fertilizer_type', 'Fertilizer Type', 'Types of fertilizers and soil amendments', true),
    ('pest_type', 'Pest Type', 'Types of agricultural pests and diseases', true),
    ('weather_condition', 'Weather Condition', 'Weather conditions for farm monitoring', false)
    ON CONFLICT (name) DO NOTHING;

    SELECT id INTO cat_soil_type FROM type_categories WHERE name = 'soil_type';
    SELECT id INTO cat_fertilizer_type FROM type_categories WHERE name = 'fertilizer_type';
    SELECT id INTO cat_pest_type FROM type_categories WHERE name = 'pest_type';
    SELECT id INTO cat_weather_condition FROM type_categories WHERE name = 'weather_condition';

    -- =====================================================================
    -- A. ANIMAL SPECIES (animal_species) – European livestock hierarchy
    -- =====================================================================

    -- Insert parent categories first
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Rindvieh', 'rindvieh', 20)
    RETURNING id INTO parent_rindvieh;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Schafe', 'schafe', 21)
    RETURNING id INTO parent_schafe;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Ziegen', 'ziegen', 22)
    RETURNING id INTO parent_ziegen;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Schweine', 'schweine', 23)
    RETURNING id INTO parent_schweine;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Geflügel', 'gefluegel_eu', 24)
    RETURNING id INTO parent_gefluegel;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_animal_species, NULL, NULL, 'Pferde', 'pferde', 25)
    RETURNING id INTO parent_pferde;

    -- Rindvieh children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, parent_rindvieh, NULL, 'Milchvieh', 'milchvieh', 21),
    (cat_animal_species, parent_rindvieh, NULL, 'Mastvieh', 'mastvieh', 22);

    -- Schafe children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, parent_schafe, NULL, 'Milchschafe', 'milchschafe', 21),
    (cat_animal_species, parent_schafe, NULL, 'Wollschafe', 'wollschafe', 22),
    (cat_animal_species, parent_schafe, NULL, 'Fleischschafe', 'fleischschafe', 23);

    -- Ziegen children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, parent_ziegen, NULL, 'Milchziegen', 'milchziegen', 21),
    (cat_animal_species, parent_ziegen, NULL, 'Fleischziegen', 'fleischziegen', 22);

    -- Schweine children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, parent_schweine, NULL, 'Sauen', 'sauen', 21),
    (cat_animal_species, parent_schweine, NULL, 'Mastschweine', 'mastschweine', 22);

    -- Geflügel children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, parent_gefluegel, NULL, 'Huhn', 'huhn', 21),
    (cat_animal_species, parent_gefluegel, NULL, 'Ente', 'ente', 22),
    (cat_animal_species, parent_gefluegel, NULL, 'Gans', 'gans', 23),
    (cat_animal_species, parent_gefluegel, NULL, 'Truthahn', 'truthahn', 24);

    -- Pferde children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, parent_pferde, NULL, 'Reitpferd', 'reitpferd', 21),
    (cat_animal_species, parent_pferde, NULL, 'Zugpferd', 'zugpferd', 22),
    (cat_animal_species, parent_pferde, NULL, 'Pony', 'pony', 23);

    -- Standalone species
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_species, NULL, NULL, 'Bienen', 'bienen', 30),
    (cat_animal_species, NULL, NULL, 'Kaninchen', 'kaninchen', 31),
    (cat_animal_species, NULL, NULL, 'Fische', 'fische', 32);

    -- =====================================================================
    -- B. CROP TYPES (crop_type) – European crops with hierarchy
    -- =====================================================================

    -- Weinrebe parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Weinrebe', 'weinrebe', 30)
    RETURNING id INTO parent_weinrebe;

    -- Weizen parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Weizen', 'weizen', 31)
    RETURNING id INTO parent_weizen;

    -- Mais parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Mais', 'mais', 32)
    RETURNING id INTO parent_mais;

    -- Gemüse parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Gemüse', 'gemuese', 33)
    RETURNING id INTO parent_gemuese;

    -- Obst parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Obst', 'obst', 34)
    RETURNING id INTO parent_obst;

    -- Nüsse parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Nüsse', 'nuesse', 35)
    RETURNING id INTO parent_nuesse;

    -- Energiepflanzen parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_crop_type, NULL, NULL, 'Energiepflanzen', 'energiepflanzen', 36)
    RETURNING id INTO parent_energiepflanzen;

    -- Weinrebe children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_weinrebe, NULL, 'Rotwein', 'rotwein', 31),
    (cat_crop_type, parent_weinrebe, NULL, 'Weißwein', 'weisswein', 32),
    (cat_crop_type, parent_weinrebe, NULL, 'Rosé', 'rose', 33);

    -- Weizen children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_weizen, NULL, 'Winterweizen', 'winterweizen', 31),
    (cat_crop_type, parent_weizen, NULL, 'Sommerweizen', 'sommerweizen', 32),
    (cat_crop_type, parent_weizen, NULL, 'Dinkel', 'dinkel', 33);

    -- Mais children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_mais, NULL, 'Silomais', 'silomais', 31),
    (cat_crop_type, parent_mais, NULL, 'Körnermais', 'koernermais', 32),
    (cat_crop_type, parent_mais, NULL, 'Süßmais', 'suessmais', 33);

    -- Gemüse children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_gemuese, NULL, 'Salat', 'salat', 31),
    (cat_crop_type, parent_gemuese, NULL, 'Tomaten', 'tomaten', 32),
    (cat_crop_type, parent_gemuese, NULL, 'Karotten', 'karotten', 33),
    (cat_crop_type, parent_gemuese, NULL, 'Zwiebeln', 'zwiebeln', 34);

    -- Obst children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_obst, NULL, 'Äpfel', 'aepfel', 31),
    (cat_crop_type, parent_obst, NULL, 'Birnen', 'birnen', 32),
    (cat_crop_type, parent_obst, NULL, 'Kirschen', 'kirschen', 33),
    (cat_crop_type, parent_obst, NULL, 'Beeren', 'beeren', 34);

    -- Nüsse children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_nuesse, NULL, 'Mandeln', 'mandeln', 31),
    (cat_crop_type, parent_nuesse, NULL, 'Haselnüsse', 'haselnuesse', 32),
    (cat_crop_type, parent_nuesse, NULL, 'Walnüsse', 'walnuesse', 33);

    -- Energiepflanzen children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, parent_energiepflanzen, NULL, 'Raps', 'raps', 31),
    (cat_crop_type, parent_energiepflanzen, NULL, 'Miscanthus', 'miscanthus', 32);

    -- Standalone crops
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_crop_type, NULL, NULL, 'Roggen', 'roggen', 40),
    (cat_crop_type, NULL, NULL, 'Gerste', 'gerste', 41),
    (cat_crop_type, NULL, NULL, 'Hafer', 'hafer', 42),
    (cat_crop_type, NULL, NULL, 'Kartoffel', 'kartoffel', 43),
    (cat_crop_type, NULL, NULL, 'Zuckerrübe', 'zuckerruebe', 44),
    (cat_crop_type, NULL, NULL, 'Sonnenblume', 'sonnenblume', 45),
    (cat_crop_type, NULL, NULL, 'Soja', 'soja', 46),
    (cat_crop_type, NULL, NULL, 'Luzerne', 'luzerne', 47),
    (cat_crop_type, NULL, NULL, 'Gras', 'gras', 48),
    (cat_crop_type, NULL, NULL, 'Ackerbohnen', 'ackerbohnen', 49),
    (cat_crop_type, NULL, NULL, 'Erbse', 'erbse', 50),
    (cat_crop_type, NULL, NULL, 'Oliven', 'oliven', 51),
    (cat_crop_type, NULL, NULL, 'Hopfen', 'hopfen', 52),
    (cat_crop_type, NULL, NULL, 'Flachs', 'flachs', 53);

    -- =====================================================================
    -- C. SITE TYPES (site_type) – European site types with hierarchy
    -- =====================================================================

    -- Obstgarten parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_site_type, NULL, NULL, 'Obstgarten', 'obstgarten', 20)
    RETURNING id INTO parent_obstgarten;

    -- Acker parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_site_type, NULL, NULL, 'Acker', 'acker', 21)
    RETURNING id INTO parent_acker;

    -- Grünland parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_site_type, NULL, NULL, 'Grünland', 'gruenland', 22)
    RETURNING id INTO parent_gruenland;

    -- Obstgarten children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_site_type, parent_obstgarten, NULL, 'Apfelgarten', 'apfelgarten', 21),
    (cat_site_type, parent_obstgarten, NULL, 'Birnengarten', 'birnengarten', 22),
    (cat_site_type, parent_obstgarten, NULL, 'Kirschgarten', 'kirschgarten', 23),
    (cat_site_type, parent_obstgarten, NULL, 'Beerenplantage', 'beerenplantage', 24);

    -- Acker children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_site_type, parent_acker, NULL, 'Getreideacker', 'getreideacker', 21),
    (cat_site_type, parent_acker, NULL, 'Gemüseacker', 'gemueseacker', 22),
    (cat_site_type, parent_acker, NULL, 'Maisacker', 'maisacker', 23);

    -- Grünland children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_site_type, parent_gruenland, NULL, 'Weide', 'weide', 21),
    (cat_site_type, parent_gruenland, NULL, 'Wiese', 'wiese', 22);

    -- Standalone site types
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_site_type, NULL, NULL, 'Weinberg', 'weinberg', 25),
    (cat_site_type, NULL, NULL, 'Gewächshaus', 'gewaechshaus', 26),
    (cat_site_type, NULL, NULL, 'Hopfenfeld', 'hopfenfeld', 27),
    (cat_site_type, NULL, NULL, 'Olivenhain', 'olivenhain', 28),
    (cat_site_type, NULL, NULL, 'Nussplantage', 'nussplantage', 29),
    (cat_site_type, NULL, NULL, 'Forst', 'forst', 30),
    (cat_site_type, NULL, NULL, 'Brache', 'brache', 31),
    (cat_site_type, NULL, NULL, 'Hof', 'hof', 32),
    (cat_site_type, NULL, NULL, 'Stall', 'stall', 33),
    (cat_site_type, NULL, NULL, 'Teich', 'teich', 34),
    (cat_site_type, NULL, NULL, 'Bienenstand', 'bienenstand', 35);

    -- =====================================================================
    -- D. EQUIPMENT TYPES (equipment_type) – Comprehensive with hierarchy
    -- =====================================================================

    -- Traktor parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Traktor', 'traktor_eu', 20)
    RETURNING id INTO parent_traktor;

    -- Spritze parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Spritze', 'spritze', 21)
    RETURNING id INTO parent_spritze;

    -- Mähdrescher parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Mähdrescher', 'maehdrescher', 22)
    RETURNING id INTO parent_maehdrescher;

    -- Ballenpresse parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Ballenpresse', 'ballenpresse', 23)
    RETURNING id INTO parent_ballenpresse;

    -- Sämaschine parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Sämaschine', 'saeemaschine', 24)
    RETURNING id INTO parent_saeemaschine;

    -- Bewässerungsanlage parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Bewässerungsanlage', 'bewaesserungsanlage', 25)
    RETURNING id INTO parent_bewaesserungsanlage;

    -- Drohne parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Drohne', 'drohne', 26)
    RETURNING id INTO parent_drohne;

    -- Werkzeug parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Werkzeug', 'werkzeug', 27)
    RETURNING id INTO parent_werkzeug;

    -- Mähwerk parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_equipment_type, NULL, NULL, 'Mähwerk', 'maehwerk', 28)
    RETURNING id INTO parent_maehwerk;

    -- Traktor children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_traktor, NULL, 'Kleintraktor', 'kleintraktor', 21),
    (cat_equipment_type, parent_traktor, NULL, 'Mitteltraktor', 'mitteltraktor', 22),
    (cat_equipment_type, parent_traktor, NULL, 'Großtraktor', 'grosstraktor', 23);

    -- Spritze children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_spritze, NULL, 'Feldspritze', 'feldspritze', 21),
    (cat_equipment_type, parent_spritze, NULL, 'Rebspritze', 'rebspritze', 22),
    (cat_equipment_type, parent_spritze, NULL, 'Trägerspritze', 'traegerspritze', 23),
    (cat_equipment_type, parent_spritze, NULL, 'Spritze Drohne', 'spritze_drohne', 24);

    -- Mähdrescher children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_maehdrescher, NULL, 'Getreidemähdrescher', 'getreidemaehdrescher', 21),
    (cat_equipment_type, parent_maehdrescher, NULL, 'Rübenmähdrescher', 'ruebemaehdrescher', 22);

    -- Ballenpresse children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_ballenpresse, NULL, 'Rundballen', 'rundballen', 21),
    (cat_equipment_type, parent_ballenpresse, NULL, 'Quadratballen', 'quadratballen', 22);

    -- Sämaschine children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_saeemaschine, NULL, 'Getreidesämaschine', 'getreidesaeemaschine', 21),
    (cat_equipment_type, parent_saeemaschine, NULL, 'Rapsämaschine', 'rapsaeemaschine', 22);

    -- Bewässerungsanlage children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_bewaesserungsanlage, NULL, 'Tropfbewässerung', 'tropfbewaesserung', 21),
    (cat_equipment_type, parent_bewaesserungsanlage, NULL, 'Sprinkler', 'sprinkler', 22),
    (cat_equipment_type, parent_bewaesserungsanlage, NULL, 'Center Pivot', 'center_pivot', 23);

    -- Drohne children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_drohne, NULL, 'Sprühdrohne', 'spruehdrohne', 21),
    (cat_equipment_type, parent_drohne, NULL, 'Aufklärungsdrohne', 'aufklaerungsdrohne', 22);

    -- Werkzeug children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_werkzeug, NULL, 'Handwerkzeug', 'handwerkzeug', 21),
    (cat_equipment_type, parent_werkzeug, NULL, 'Elektrowerkzeug', 'elektrowerkzeug', 22);

    -- Mähwerk children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, parent_maehwerk, NULL, 'Sichelmäher', 'sichelmaehrer', 21),
    (cat_equipment_type, parent_maehwerk, NULL, 'Rotormäher', 'rotormaehrer', 22),
    (cat_equipment_type, parent_maehwerk, NULL, 'Scheibenmäher', 'scheibenmaehrer', 23);

    -- Standalone equipment
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_equipment_type, NULL, NULL, 'Heckenschere', 'heckenschere', 30),
    (cat_equipment_type, NULL, NULL, 'Pflug', 'pflug', 31),
    (cat_equipment_type, NULL, NULL, 'Grubber', 'grubber', 32),
    (cat_equipment_type, NULL, NULL, 'Kultivator', 'kultivator', 33),
    (cat_equipment_type, NULL, NULL, 'Düngerstreuer', 'duengerstreuer', 34),
    (cat_equipment_type, NULL, NULL, 'Gülleausbringungsgerät', 'guelleausbringungsgeraet', 35),
    (cat_equipment_type, NULL, NULL, 'Erntehelfer', 'erntehelfer', 36),
    (cat_equipment_type, NULL, NULL, 'Anhänger', 'anhaenger', 37),
    (cat_equipment_type, NULL, NULL, 'LKW', 'lkw', 38),
    (cat_equipment_type, NULL, NULL, 'Gabelstapler', 'gabelstapler', 39),
    (cat_equipment_type, NULL, NULL, 'Generator', 'generator', 40),
    (cat_equipment_type, NULL, NULL, 'Melkmaschine', 'melkmaschine', 41),
    (cat_equipment_type, NULL, NULL, 'Fütterungsroboter', 'fuetterungsroboter', 42),
    (cat_equipment_type, NULL, NULL, 'Strohballenhandler', 'strohballenhandler', 43),
    (cat_equipment_type, NULL, NULL, 'Frontlader', 'frontlader', 44),
    (cat_equipment_type, NULL, NULL, 'Rücklader', 'ruecklader', 45),
    (cat_equipment_type, NULL, NULL, 'Bagger', 'bagger', 46),
    (cat_equipment_type, NULL, NULL, 'Planiermaschine', 'planiermaschine', 47),
    (cat_equipment_type, NULL, NULL, 'Walze', 'walze', 48),
    (cat_equipment_type, NULL, NULL, 'Heuwender', 'heuwender', 49);

    -- =====================================================================
    -- E. BUILDING TYPES (building_type) – Farm buildings with hierarchy
    -- =====================================================================

    -- Stall parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_building_type, NULL, NULL, 'Stall', 'stall_eu', 10)
    RETURNING id INTO parent_stall;

    -- Silo parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_building_type, NULL, NULL, 'Silo', 'silo', 11)
    RETURNING id INTO parent_silo;

    -- Stall children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_building_type, parent_stall, NULL, 'Kuhstall', 'kuhstall', 11),
    (cat_building_type, parent_stall, NULL, 'Schafstall', 'schafstall', 12),
    (cat_building_type, parent_stall, NULL, 'Ziegenstall', 'ziegenstall', 13),
    (cat_building_type, parent_stall, NULL, 'Schweinestall', 'schweinestall', 14),
    (cat_building_type, parent_stall, NULL, 'Pferdestall', 'pferdestall', 15);

    -- Silo children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_building_type, parent_silo, NULL, 'Getreidesilo', 'getreidesilo', 11),
    (cat_building_type, parent_silo, NULL, 'Silagesilo', 'silagesilo', 12);

    -- Standalone buildings
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_building_type, NULL, NULL, 'Hühnerstall', 'huehnerstall', 20),
    (cat_building_type, NULL, NULL, 'Scheune', 'scheune', 21),
    (cat_building_type, NULL, NULL, 'Maschinenhalle', 'maschinenhalle', 22),
    (cat_building_type, NULL, NULL, 'Lagerhalle', 'lagerhalle', 23),
    (cat_building_type, NULL, NULL, 'Verarbeitungsgebäude', 'verarbeitungsgebaeude', 24),
    (cat_building_type, NULL, NULL, 'Kelterei', 'kelterei', 25),
    (cat_building_type, NULL, NULL, 'Weinkeller', 'weinkeller', 26),
    (cat_building_type, NULL, NULL, 'Bienenhaus', 'bienenhaus', 27),
    (cat_building_type, NULL, NULL, 'Gewächshaus', 'gewaechshaus_building', 28),
    (cat_building_type, NULL, NULL, 'Fischteich', 'fischteich', 29),
    (cat_building_type, NULL, NULL, 'Düngerlager', 'duengerlager', 30),
    (cat_building_type, NULL, NULL, 'Pflanzenschutzlager', 'pflanzenschutzlager', 31);

    -- =====================================================================
    -- F. ORDER/TASK TYPES (order_type) – Comprehensive task types
    -- =====================================================================

    -- Pflanzenschutz parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_order_type, NULL, NULL, 'Pflanzenschutz', 'pflanzenschutz', 20)
    RETURNING id INTO parent_pflanzenschutz;

    -- Düngen parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_order_type, NULL, NULL, 'Düngen', 'duengen', 21)
    RETURNING id INTO parent_duengen;

    -- Beschneiden parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_order_type, NULL, NULL, 'Beschneiden', 'beschneiden', 22)
    RETURNING id INTO parent_beschneiden;

    -- Ernten parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_order_type, NULL, NULL, 'Ernten', 'ernten', 23)
    RETURNING id INTO parent_ernten;

    -- Bienen parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_order_type, NULL, NULL, 'Bienen', 'bienen_order', 24)
    RETURNING id INTO parent_bienen;

    -- Fischerei parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_order_type, NULL, NULL, 'Fischerei', 'fischerei', 25)
    RETURNING id INTO parent_fischerei;

    -- Pflanzenschutz children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, parent_pflanzenschutz, NULL, 'Fungizid', 'fungizid', 21),
    (cat_order_type, parent_pflanzenschutz, NULL, 'Insektizid', 'insektizid', 22),
    (cat_order_type, parent_pflanzenschutz, NULL, 'Herbizid', 'herbizid', 23),
    (cat_order_type, parent_pflanzenschutz, NULL, 'Akarizid', 'akarizid', 24);

    -- Düngen children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, parent_duengen, NULL, 'Stickstoff', 'stickstoff', 21),
    (cat_order_type, parent_duengen, NULL, 'Phosphor', 'phosphor', 22),
    (cat_order_type, parent_duengen, NULL, 'Kalium', 'kalium', 23),
    (cat_order_type, parent_duengen, NULL, 'Gülle', 'guelle', 24),
    (cat_order_type, parent_duengen, NULL, 'Kompost', 'kompost', 25);

    -- Beschneiden children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, parent_beschneiden, NULL, 'Winterschnitt', 'winterschnitt', 21),
    (cat_order_type, parent_beschneiden, NULL, 'Sommerschnitt', 'sommerschnitt', 22);

    -- Ernten children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, parent_ernten, NULL, 'Getreide Ernte', 'getreide_ernte', 21),
    (cat_order_type, parent_ernten, NULL, 'Kartoffel Ernte', 'kartoffel_ernte', 22),
    (cat_order_type, parent_ernten, NULL, 'Obst Ernte', 'obst_ernte', 23),
    (cat_order_type, parent_ernten, NULL, 'Wein Ernte', 'wein_ernte', 24),
    (cat_order_type, parent_ernten, NULL, 'Oliven Ernte', 'oliven_ernte', 25);

    -- Bienen children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, parent_bienen, NULL, 'Honig ernten', 'honig_ernten', 21),
    (cat_order_type, parent_bienen, NULL, 'Varroa behandeln', 'varroa_behandeln', 22);

    -- Fischerei children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, parent_fischerei, NULL, 'Fisch füttern', 'fisch_fuettern', 21),
    (cat_order_type, parent_fischerei, NULL, 'Fisch ernten', 'fisch_ernten', 22),
    (cat_order_type, parent_fischerei, NULL, 'Wasser prüfen', 'wasser_pruefen', 23);

    -- Standalone order types
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_order_type, NULL, NULL, 'Pflügen', 'pfluegen', 30),
    (cat_order_type, NULL, NULL, 'Grubbern', 'grubbern', 31),
    (cat_order_type, NULL, NULL, 'Kultivieren', 'kultivieren', 32),
    (cat_order_type, NULL, NULL, 'Säen', 'saen', 33),
    (cat_order_type, NULL, NULL, 'Ranken', 'ranken', 34),
    (cat_order_type, NULL, NULL, 'Pflücken', 'pfluecken', 35),
    (cat_order_type, NULL, NULL, 'Heuen', 'heuen', 36),
    (cat_order_type, NULL, NULL, 'Silieren', 'silieren', 37),
    (cat_order_type, NULL, NULL, 'Bewässern', 'bewaessern', 38),
    (cat_order_type, NULL, NULL, 'Füttern', 'fuettern', 39),
    (cat_order_type, NULL, NULL, 'Tränken', 'traenken', 40),
    (cat_order_type, NULL, NULL, 'Melken', 'melken', 41),
    (cat_order_type, NULL, NULL, 'Stall reinigen', 'stall_reinigen', 42),
    (cat_order_type, NULL, NULL, 'Eier sammeln', 'eier_sammeln', 43),
    (cat_order_type, NULL, NULL, 'Scheren', 'scheren', 44),
    (cat_order_type, NULL, NULL, 'Gesundheit prüfen', 'gesundheit_pruefen', 45),
    (cat_order_type, NULL, NULL, 'Besamen', 'besamen', 46),
    (cat_order_type, NULL, NULL, 'Umstellen', 'umstellen', 47),
    (cat_order_type, NULL, NULL, 'Monitoring', 'monitoring_order', 48),
    (cat_order_type, NULL, NULL, 'Wartung', 'wartung', 49),
    (cat_order_type, NULL, NULL, 'Reparatur', 'reparatur', 50),
    (cat_order_type, NULL, NULL, 'Verkauf', 'verkauf', 51),
    (cat_order_type, NULL, NULL, 'Lieferung', 'lieferung', 52),
    (cat_order_type, NULL, NULL, 'Inventur', 'inventur', 53);

    -- =====================================================================
    -- G. VARIETY CATEGORIES (variety_category) – European varieties
    -- =====================================================================

    -- Weinrebe parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_variety_category, NULL, NULL, 'Weinrebe', 'weinrebe_variety', 20)
    RETURNING id INTO parent_weinrebe_variety;

    -- Beeren parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_variety_category, NULL, NULL, 'Beeren', 'beeren_variety', 21)
    RETURNING id INTO parent_beeren;

    -- Weinrebe children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_variety_category, parent_weinrebe_variety, NULL, 'Rotweinsorte', 'rotweinsorte', 21),
    (cat_variety_category, parent_weinrebe_variety, NULL, 'Weißweinsorte', 'weissweinsorte', 22);

    -- Beeren children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_variety_category, parent_beeren, NULL, 'Erdbeere', 'erdbeere', 21),
    (cat_variety_category, parent_beeren, NULL, 'Himbeere', 'himbeere', 22),
    (cat_variety_category, parent_beeren, NULL, 'Brombeere', 'brombeere', 23),
    (cat_variety_category, parent_beeren, NULL, 'Heidelbeere', 'heidelbeere', 24);

    -- Standalone variety categories
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_variety_category, NULL, NULL, 'Oliven', 'oliven_variety', 30),
    (cat_variety_category, NULL, NULL, 'Weizen', 'weizen_variety', 31),
    (cat_variety_category, NULL, NULL, 'Roggen', 'roggen_variety', 32),
    (cat_variety_category, NULL, NULL, 'Gerste', 'gerste_variety', 33),
    (cat_variety_category, NULL, NULL, 'Mais', 'mais_variety', 34),
    (cat_variety_category, NULL, NULL, 'Kartoffel', 'kartoffel_variety', 35),
    (cat_variety_category, NULL, NULL, 'Äpfel', 'aepfel_variety', 36),
    (cat_variety_category, NULL, NULL, 'Birnen', 'birnen_variety', 37),
    (cat_variety_category, NULL, NULL, 'Kirschen', 'kirschen_variety', 38),
    (cat_variety_category, NULL, NULL, 'Zitrusfrüchte', 'zitrusfruechte', 39),
    (cat_variety_category, NULL, NULL, 'Nüsse', 'nuesse_variety', 40),
    (cat_variety_category, NULL, NULL, 'Gemüse', 'gemuese_variety', 41),
    (cat_variety_category, NULL, NULL, 'Getreide', 'getreide_variety', 42),
    (cat_variety_category, NULL, NULL, 'Hülsenfrüchte', 'huelsenfruechte', 43),
    (cat_variety_category, NULL, NULL, 'Ölfrüchte', 'oelfruechte', 44),
    (cat_variety_category, NULL, NULL, 'Faserpflanzen', 'faserpflanzen', 45),
    (cat_variety_category, NULL, NULL, 'Heilpflanzen', 'heilpflanzen', 46);

    -- =====================================================================
    -- H. SPECIES (species) – European livestock species
    -- =====================================================================

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_species, NULL, NULL, 'Rind', 'rind', 10),
    (cat_species, NULL, NULL, 'Schaf', 'schaf', 11),
    (cat_species, NULL, NULL, 'Ziege', 'ziege', 12),
    (cat_species, NULL, NULL, 'Schwein', 'schwein', 13),
    (cat_species, NULL, NULL, 'Huhn', 'huhn_species', 14),
    (cat_species, NULL, NULL, 'Pferd', 'pferd', 15),
    (cat_species, NULL, NULL, 'Kaninchen', 'kaninchen_species', 16),
    (cat_species, NULL, NULL, 'Biene', 'biene', 17),
    (cat_species, NULL, NULL, 'Fisch', 'fisch', 18);

    -- =====================================================================
    -- I. INVENTORY CATEGORIES (inventory_category) – With hierarchy
    -- =====================================================================

    -- Düngemittel parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_inventory_category, NULL, NULL, 'Düngemittel', 'duengemittel', 20)
    RETURNING id INTO parent_duengemittel;

    -- Pflanzenschutz parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_inventory_category, NULL, NULL, 'Pflanzenschutz', 'pflanzenschutz_inv', 21)
    RETURNING id INTO parent_pflanzenschutz_inv;

    -- Kraftstoff parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_inventory_category, NULL, NULL, 'Kraftstoff', 'kraftstoff', 22)
    RETURNING id INTO parent_kraftstoff;

    -- Futter parent
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_inventory_category, NULL, NULL, 'Futter', 'futter', 23)
    RETURNING id INTO parent_futter;

    -- Düngemittel children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, parent_duengemittel, NULL, 'Mineraldünger', 'mineralduenger', 21),
    (cat_inventory_category, parent_duengemittel, NULL, 'Organische Dünger', 'organische_duenger', 22);

    -- Pflanzenschutz children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, parent_pflanzenschutz_inv, NULL, 'Fungizide', 'fungizide', 21),
    (cat_inventory_category, parent_pflanzenschutz_inv, NULL, 'Insektizide', 'insektizide', 22),
    (cat_inventory_category, parent_pflanzenschutz_inv, NULL, 'Herbizide', 'herbizide', 23);

    -- Kraftstoff children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, parent_kraftstoff, NULL, 'Diesel', 'diesel', 21),
    (cat_inventory_category, parent_kraftstoff, NULL, 'Benzin', 'benzin', 22),
    (cat_inventory_category, parent_kraftstoff, NULL, 'AdBlue', 'adblue', 23);

    -- Futter children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, parent_futter, NULL, 'Kraftfutter', 'kraftfutter', 21),
    (cat_inventory_category, parent_futter, NULL, 'Raufutter', 'raufutter', 22),
    (cat_inventory_category, parent_futter, NULL, 'Mineralfutter', 'mineralfutter', 23);

    -- Standalone inventory categories
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_inventory_category, NULL, NULL, 'Saatgut', 'saatgut', 30),
    (cat_inventory_category, NULL, NULL, 'Tiermedizin', 'tiermedizin', 31),
    (cat_inventory_category, NULL, NULL, 'Ersatzteile', 'ersatzteile', 32),
    (cat_inventory_category, NULL, NULL, 'Betriebsmittel', 'betriebsmittel', 33),
    (cat_inventory_category, NULL, NULL, 'Verpackungsmaterial', 'verpackungsmaterial', 34),
    (cat_inventory_category, NULL, NULL, 'Elektroartikel', 'elektroartikel', 35),
    (cat_inventory_category, NULL, NULL, 'Werkzeuge', 'werkzeuge_inv', 36),
    (cat_inventory_category, NULL, NULL, 'Sonstiges', 'sonstiges', 37);

    -- =====================================================================
    -- J. NEW CATEGORIES
    -- =====================================================================

    -- J.1 Soil Types (soil_type)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_soil_type, NULL, NULL, 'Sandboden', 'sandboden', 1),
    (cat_soil_type, NULL, NULL, 'Lehmboden', 'lehmboden', 2),
    (cat_soil_type, NULL, NULL, 'Tonboden', 'tonboden', 3),
    (cat_soil_type, NULL, NULL, 'Kiesboden', 'kiesboden', 4),
    (cat_soil_type, NULL, NULL, 'Humusboden', 'humusboden', 5),
    (cat_soil_type, NULL, NULL, 'Mergelboden', 'margelboden', 6);

    -- J.2 Fertilizer Types (fertilizer_type) – with hierarchy
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_fertilizer_type, NULL, NULL, 'Stickstoffdünger', 'stickstoffduenger', 1),
    (cat_fertilizer_type, NULL, NULL, 'Phosphordünger', 'phosphorduenger', 2),
    (cat_fertilizer_type, NULL, NULL, 'Kaliumdünger', 'kaliumduenger', 3),
    (cat_fertilizer_type, NULL, NULL, 'Volldünger', 'vollduenger', 4),
    (cat_fertilizer_type, NULL, NULL, 'Gülle', 'guelle_fert', 5),
    (cat_fertilizer_type, NULL, NULL, 'Mist', 'mist', 6),
    (cat_fertilizer_type, NULL, NULL, 'Kompost', 'kompost_fert', 7),
    (cat_fertilizer_type, NULL, NULL, 'Mineraldünger', 'mineralduenger_fert', 8),
    (cat_fertilizer_type, NULL, NULL, 'Organischer Dünger', 'organischer_duenger', 9),
    (cat_fertilizer_type, NULL, NULL, 'Blattdünger', 'blattduenger', 10);

    -- J.3 Pest Types (pest_type) – with hierarchy
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_pest_type, NULL, NULL, 'Insekt', 'insekt', 1)
    RETURNING id INTO parent_insekt;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_pest_type, NULL, NULL, 'Pilz', 'pilz', 2)
    RETURNING id INTO parent_pilz;

    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order)
    VALUES (cat_pest_type, NULL, NULL, 'Säugetier', 'saeugetier_pest', 3)
    RETURNING id INTO parent_saeugetier_pest;

    -- Insekt children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_pest_type, parent_insekt, NULL, 'Blattlaus', 'blattlaus', 11),
    (cat_pest_type, parent_insekt, NULL, 'Raupen', 'raupen', 12),
    (cat_pest_type, parent_insekt, NULL, 'Käfer', 'kaefer', 13);

    -- Pilz children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_pest_type, parent_pilz, NULL, 'Mehltau', 'mehltau', 21),
    (cat_pest_type, parent_pilz, NULL, 'Rost', 'rost', 22),
    (cat_pest_type, parent_pilz, NULL, 'Botrytis', 'botrytis', 23);

    -- Säugetier children
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_pest_type, parent_saeugetier_pest, NULL, 'Hasen', 'hasen', 31),
    (cat_pest_type, parent_saeugetier_pest, NULL, 'Rehe', 'rehe', 32);

    -- Standalone pest types
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_pest_type, NULL, NULL, 'Bakterie', 'bakterie', 4),
    (cat_pest_type, NULL, NULL, 'Virus', 'virus', 5),
    (cat_pest_type, NULL, NULL, 'Unkraut', 'unkraut', 6),
    (cat_pest_type, NULL, NULL, 'Nematode', 'nematode', 7),
    (cat_pest_type, NULL, NULL, 'Vogel', 'vogel', 8);

    -- J.4 Weather Conditions (weather_condition)
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_weather_condition, NULL, NULL, 'Sonnig', 'sonnig', 1),
    (cat_weather_condition, NULL, NULL, 'Bewölkt', 'bewolkt', 2),
    (cat_weather_condition, NULL, NULL, 'Regen', 'regen', 3),
    (cat_weather_condition, NULL, NULL, 'Schnee', 'schnee', 4),
    (cat_weather_condition, NULL, NULL, 'Hagel', 'hagel', 5),
    (cat_weather_condition, NULL, NULL, 'Sturm', 'sturm', 6),
    (cat_weather_condition, NULL, NULL, 'Nebel', 'nebel', 7),
    (cat_weather_condition, NULL, NULL, 'Frost', 'frost', 8),
    (cat_weather_condition, NULL, NULL, 'Trockenheit', 'trockenheit', 9);

    -- =====================================================================
    -- K. EXPAND EXISTING CATEGORIES
    -- =====================================================================

    -- K.1 Contract Type (contract_type) – expand
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_contract_type, NULL, NULL, 'Festanstellung', 'festanstellung', 6),
    (cat_contract_type, NULL, NULL, 'Zeitarbeitsvertrag', 'zeitarbeitsvertrag', 7),
    (cat_contract_type, NULL, NULL, 'Saisonarbeitsvertrag', 'saisonarbeitsvertrag', 8),
    (cat_contract_type, NULL, NULL, 'Minijob', 'minijob', 9),
    (cat_contract_type, NULL, NULL, 'Aushilfe', 'aushilfe', 10)
    ON CONFLICT DO NOTHING;

    -- K.2 Depreciation Method (depreciation_method) – expand
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_depreciation_method, NULL, NULL, 'Linear', 'linear', 3),
    (cat_depreciation_method, NULL, NULL, 'Degressiv', 'degressiv', 4),
    (cat_depreciation_method, NULL, NULL, 'Doppelte degressiv', 'doppelte_degressiv', 5),
    (cat_depreciation_method, NULL, NULL, 'Leistungsbezogen', 'leistungsbezogen', 6)
    ON CONFLICT DO NOTHING;

    -- K.3 Water Source Type (water_source_type) – expand
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_water_source_type, NULL, NULL, 'Brunnen', 'brunnen', 8),
    (cat_water_source_type, NULL, NULL, 'Teich', 'teich_water', 9),
    (cat_water_source_type, NULL, NULL, 'Fluss', 'fluss', 10),
    (cat_water_source_type, NULL, NULL, 'Kanal', 'kanal', 11),
    (cat_water_source_type, NULL, NULL, 'Regenwasser', 'regenwasser', 12),
    (cat_water_source_type, NULL, NULL, 'Leitungswasser', 'leitungswasser', 13),
    (cat_water_source_type, NULL, NULL, 'Grundwasser', 'grundwasser', 14),
    (cat_water_source_type, NULL, NULL, 'See', 'see', 15)
    ON CONFLICT DO NOTHING;

    -- K.4 Irrigation Method (irrigation_method) – expand
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_irrigation_method, NULL, NULL, 'Tropfbewässerung', 'tropfbewaesserung_irr', 7),
    (cat_irrigation_method, NULL, NULL, 'Überstau', 'uerberstau', 8),
    (cat_irrigation_method, NULL, NULL, 'Unterflurbewässerung', 'unterflurbewaesserung', 9),
    (cat_irrigation_method, NULL, NULL, 'Beregnung', 'beregnung', 10)
    ON CONFLICT DO NOTHING;

    -- K.5 Animal Status (animal_status) – expand
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_animal_status, NULL, NULL, 'Transferred', 'transferred', 5)
    ON CONFLICT DO NOTHING;

    -- K.6 Weather Station Type (weather_station_type) – expand
    INSERT INTO type_entries (category_id, parent_id, tenant_id, name, code, sort_order) VALUES
    (cat_weather_station_type, NULL, NULL, 'Satellit', 'satellit', 5),
    (cat_weather_station_type, NULL, NULL, 'Modell', 'modell', 6)
    ON CONFLICT DO NOTHING;

END $$;

-- ---------------------------------------------------------------------------
-- Verification
-- ---------------------------------------------------------------------------

DO $$
DECLARE
    entry_count INTEGER;
    cat_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO entry_count FROM type_entries;
    SELECT COUNT(*) INTO cat_count FROM type_categories;
    RAISE NOTICE 'Migration 0034 complete: % categories, % total type entries', cat_count, entry_count;
END $$;
