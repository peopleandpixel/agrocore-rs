-- Demo Seed Data for AgroCore
-- This file creates a complete demo setup with:
-- 1 Tenant (demo) + 1 Admin User
-- 3 Sites (Weizen, Mais, Weide)
-- 3 Equipment (Traktor, Mähdrescher, Spritze)
-- 2 Workers (Hans, Maria)
-- 2 Orders (Aussaat, Gülle)
-- 3 Inventory Items + Lagerort
-- 3 Tiere (Bella, Lotte, Bruno)




















-- ============================================================
-- 1. TENANT & USERS
-- ============================================================

-- Tenant
INSERT INTO tenants (id, name, slug, config, is_active, created_at, updated_at)
VALUES ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Demo Farm', 'demo', '{}', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, updated_at = NOW();

-- Admin User. Password: demo1234-agrocore (must match DEMO_ADMIN_PASSWORD in
-- scripts/dev.sh and DEMO_DEFAULT_PASSWORD in crates/api/src/handlers/demo.rs).
INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, language, color, is_active, roles, created_at, updated_at)
VALUES ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Demo', 'Admin', 'admin@demo.local', '$argon2id$v=19$m=19456,t=2,p=1$D64+rL2hziARrbXCSZtVoQ$CniAVPik4wd4H5hmYu3r6XxOSQveEV2CQsFN044W/D4', 'de', '#3B82F6', true, '["Admin"]'::jsonb, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, password_hash = EXCLUDED.password_hash, updated_at = NOW();

-- Worker Users
INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, language, color, is_active, roles, created_at, updated_at)
VALUES ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Hans', 'Müller', 'hans@demo.local', '$argon2id$v=19$m=19456,t=2,p=1$D64+rL2hziARrbXCSZtVoQ$CniAVPik4wd4H5hmYu3r6XxOSQveEV2CQsFN044W/D4', 'de', '#10B981', true, '["Worker"]'::jsonb, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, password_hash = EXCLUDED.password_hash, updated_at = NOW();

INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, language, color, is_active, roles, created_at, updated_at)
VALUES ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Maria', 'Schmidt', 'maria@demo.local', '$argon2id$v=19$m=19456,t=2,p=1$D64+rL2hziARrbXCSZtVoQ$CniAVPik4wd4H5hmYu3r6XxOSQveEV2CQsFN044W/D4', 'de', '#F59E0B', true, '["Worker"]'::jsonb, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, password_hash = EXCLUDED.password_hash, updated_at = NOW();

-- Worker records
INSERT INTO workers (id, tenant_id, user_id, employee_id, firstname, lastname, email, phone, role_in_company, hourly_rate, social_security_number, bank_account, is_active, created_at, updated_at)
VALUES ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'EMP-001', 'Hans', 'Müller', 'hans@demo.local', '+49 170 1234567', 'operator', 18.50, 'DE123456789', 'DE89370400440532013000', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO workers (id, tenant_id, user_id, employee_id, firstname, lastname, email, phone, role_in_company, hourly_rate, social_security_number, bank_account, is_active, created_at, updated_at)
VALUES ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 'EMP-002', 'Maria', 'Schmidt', 'maria@demo.local', '+49 170 7654321', 'operator', 16.00, 'DE987654321', 'DE89370400440532013001', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 2. SITES
-- ============================================================

-- Site 1: Weizenfeld (Field)
INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety, area, gross_area, planted_date, soil_type, slope, slope_facing, altitude, organic, is_active, is_temporary, created_at, updated_at, created_by, updated_by)
VALUES ('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', NULL, 'Nordfeld - Weizen', '"field"', '{"vegetable":"Wheat"}', 'Winterweizen', 15.5, 17.0, '2024-10-15'::date, 'Lehmboden', 2.5, 'Süd', 180.0, false, true, false, NOW(), NOW(), 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label, updated_at = NOW();

-- Site 2: Maisfeld (Field)
INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety, area, gross_area, planted_date, soil_type, slope, slope_facing, altitude, organic, is_active, is_temporary, created_at, updated_at, created_by, updated_by)
VALUES ('ffffffff-ffff-ffff-ffff-ffffffffffff', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', NULL, 'Südhang - Mais', '"field"', '{"grain":"Corn"}', 'Silomais DKC 3505', 12.0, 13.5, '2024-05-01'::date, 'Sandiger Lehm', 5.0, 'Südwest', 165.0, false, true, false, NOW(), NOW(), 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label, updated_at = NOW();

-- Site 3: Weide (Pasture)
INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety, area, gross_area, planted_date, soil_type, slope, slope_facing, altitude, organic, is_active, is_temporary, created_at, updated_at, created_by, updated_by)
VALUES ('11111111-1111-1111-1111-111111111111', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', NULL, 'Westweide - Rinder', '"pasture"', '{"other":"Grassland"}', 'Dauergrünland', 25.0, 27.0, '2020-03-01'::date, 'Toniger Lehm', 3.0, 'West', 170.0, true, true, false, NOW(), NOW(), 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label, updated_at = NOW();

-- ============================================================
-- 3. EQUIPMENT
-- ============================================================

-- Equipment 1: Traktor (Fendt 724 Vario)
INSERT INTO equipment (id, tenant_id, label, code, equipment_type, in_usage, next_maintenance_date, last_maintenance_hours, is_active, fuel_type, fuel_capacity_liters, original_cost, salvage_value, purchase_date, depreciation_method, useful_life_years, created_at, updated_at)
VALUES ('22222222-2222-2222-2222-222222222222', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Traktor Fendt 724', 'FEN724-2023-001', '"Tractor"', false, '2025-05-15'::date, 450, true, 'diesel', 300, 185000.00, 35000.00, '2023-03-15'::date, 'straight_line', 10, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Equipment 2: Mähdrescher (Claas Lexion 7600)
INSERT INTO equipment (id, tenant_id, label, code, equipment_type, in_usage, next_maintenance_date, last_maintenance_hours, is_active, fuel_type, fuel_capacity_liters, original_cost, salvage_value, purchase_date, depreciation_method, useful_life_years, created_at, updated_at)
VALUES ('33333333-3333-3333-3333-333333333333', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Mähdrescher Claas 7600', 'CLA7600-2022-003', '"Harvester"', false, '2025-06-01'::date, 890, true, 'diesel', 500, 380000.00, 90000.00, '2022-07-20'::date, 'straight_line', 12, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Equipment 3: Spritze (Amazone UX 5201)
INSERT INTO equipment (id, tenant_id, label, code, equipment_type, in_usage, next_maintenance_date, last_maintenance_hours, is_active, fuel_type, fuel_capacity_liters, original_cost, salvage_value, purchase_date, depreciation_method, useful_life_years, created_at, updated_at)
VALUES ('44444444-4444-4444-4444-444444444444', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Spritze Amazone UX 5201', 'AMA5201-2023-007', '"Sprayer"', true, '2025-03-01'::date, 120, true, 'diesel', 150, 65000.00, 15000.00, '2023-02-10'::date, 'straight_line', 8, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 4. ORDERS
-- ============================================================
INSERT INTO orders (id, tenant_id, label, title, description, priority, status, order_type, assigned_worker_ids, site_ids, scheduled_start, scheduled_end, deadline_date, planned_date, execution_policy, is_active, created_at, updated_at, created_by)
VALUES ('55555555-5555-5555-5555-555555555555', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Aussaat Winterweizen Nordfeld', 'Aussaat Winterweizen Nordfeld', 'Aussaat von Winterweizen auf dem Nordfeld (15.5 ha). Saatstärke: 350 Koerner/m2. Reihe: 12.5 cm.', 1, '"planned"', '"soil_work"', jsonb_build_array('cccccccc-cccc-cccc-cccc-cccccccccccc'::uuid), jsonb_build_array('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee'::uuid), '2024-10-20'::timestamptz, NULL::timestamptz, '2024-10-25'::date, '2024-10-20'::date, '{"mode": "manual"}'::jsonb, true, NOW(), NOW(), 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO orders (id, tenant_id, label, title, description, priority, status, order_type, assigned_worker_ids, site_ids, scheduled_start, scheduled_end, deadline_date, planned_date, execution_policy, is_active, created_at, updated_at, created_by)
VALUES ('66666666-6666-6666-6666-666666666666', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Guelleausbringung Westweide', 'Guelleausbringung Westweide', 'Ausbringung von Rinderguelle auf der Westweide (25 ha). 30 m3/ha. Bodennah mit Schleppschlauch.', 2, '"in_progress"', '"fertilization"', jsonb_build_array('dddddddd-dddd-dddd-dddd-dddddddddddd'::uuid), jsonb_build_array('11111111-1111-1111-1111-111111111111'::uuid), '2024-11-10'::timestamptz, NULL::timestamptz, '2024-11-15'::date, '2024-11-10'::date, '{"mode": "manual"}'::jsonb, true, NOW(), NOW(), 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 5. INVENTORY LOCATION & ITEMS
-- ============================================================
INSERT INTO inventory_locations (id, tenant_id, name, code, description, is_active, created_at, updated_at)
VALUES ('77777777-7777-7777-7777-777777777777', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Haupthalle', 'HALLE-01', 'Zentrale Lagerhalle für Betriebsmittel', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO inventory_items (id, tenant_id, name, sku, description, category, unit, minimum_stock, inventory_method, is_active, created_at, updated_at)
VALUES ('88888888-8888-8888-8888-888888888888', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Winterweizensaatgut DKC 3505', 'SEED-WW-3505', 'Winterweizensaatgut DKC 3505', '"seed"', '"kg"', 1000.0, '"FIFO"', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO inventory_items (id, tenant_id, name, sku, description, category, unit, minimum_stock, inventory_method, is_active, created_at, updated_at)
VALUES ('99999999-9999-9999-9999-999999999999', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'NPK Dünger 20-10-10', 'FERT-NPK-201010', 'NPK Dünger 20-10-10', '"fertilizer"', '"kg"', 500.0, '"FIFO"', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO inventory_items (id, tenant_id, name, sku, description, category, unit, minimum_stock, inventory_method, is_active, created_at, updated_at)
VALUES ('aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Herbizid Mais - MaisTer', 'CHEM-HERB-MAISTER', 'Herbizid Mais - MaisTer', '"herbicide"', '"l"', 50.0, '"FIFO"', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 6. ANIMALS (Rinder)
-- ============================================================
INSERT INTO animals (id, tenant_id, tag_number, species, breed, birth_date, gender, current_site_id, mother_id, father_id, is_active, created_at, updated_at)
VALUES ('bbbbbbbb-cccc-dddd-eeee-ffffffffffff', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'DE0912345678', 'Cattle', 'Fleckvieh', '2022-03-15'::date, 'female', '11111111-1111-1111-1111-111111111111', NULL, NULL, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO animals (id, tenant_id, tag_number, species, breed, birth_date, gender, current_site_id, mother_id, father_id, is_active, created_at, updated_at)
VALUES ('cccccccc-dddd-eeee-ffff-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'DE0912345679', 'Cattle', 'Fleckvieh', '2021-07-22'::date, 'female', '11111111-1111-1111-1111-111111111111', NULL, NULL, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO animals (id, tenant_id, tag_number, species, breed, birth_date, gender, current_site_id, mother_id, father_id, is_active, created_at, updated_at)
VALUES ('dddddddd-eeee-ffff-aaaa-bbbbbbbbbbbb', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'DE0912345680', 'Cattle', 'Fleckvieh', '2023-01-10'::date, 'male', '11111111-1111-1111-1111-111111111111', 'bbbbbbbb-cccc-dddd-eeee-ffffffffffff', NULL, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 7. LIVESTOCK SUMMARY (Counts per plot)
-- ============================================================
-- The `livestock` aggregate table does not exist in the current schema;
-- per-animal data lives in `animals` and `grazing_records`.

-- ============================================================
-- 8. GRAZING RECORDS (for the animals on pasture)
-- ============================================================
INSERT INTO grazing_records (id, tenant_id, animal_id, site_id, start_date, end_date, notes, created_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'bbbbbbbb-cccc-dddd-eeee-ffffffffffff', '11111111-1111-1111-1111-111111111111', NOW() - interval '30 days', NULL, 'Bella auf Westweide aufgetrieben', NOW() - interval '30 days')
ON CONFLICT DO NOTHING;

INSERT INTO grazing_records (id, tenant_id, animal_id, site_id, start_date, end_date, notes, created_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'cccccccc-dddd-eeee-ffff-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', NOW() - interval '30 days', NULL, 'Lotte auf Westweide aufgetrieben', NOW() - interval '30 days')
ON CONFLICT DO NOTHING;

INSERT INTO grazing_records (id, tenant_id, animal_id, site_id, start_date, end_date, notes, created_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'dddddddd-eeee-ffff-aaaa-bbbbbbbbbbbb', '11111111-1111-1111-1111-111111111111', NOW() - interval '30 days', NULL, 'Bruno auf Westweide aufgetrieben', NOW() - interval '30 days')
ON CONFLICT DO NOTHING;

-- ============================================================
-- 9. COST CENTERS (for financial tracking)
-- ============================================================

INSERT INTO cost_centers (id, tenant_id, code, label, description, parent_id, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'CC001', 'Pflanzenbau', 'Kosten für Ackerbau und Pflanzenbau', NULL, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO cost_centers (id, tenant_id, code, label, description, parent_id, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'CC002', 'Tierhaltung', 'Kosten für Rinderhaltung und Weide', NULL, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO cost_centers (id, tenant_id, code, label, description, parent_id, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'CC003', 'Maschinen', 'Kosten für Traktor, Mähdrescher, Spritze', NULL, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============================================================
-- 10. CUSTOMERS (for orders/sales)
-- ============================================================

INSERT INTO customers (id, tenant_id, name, company, email, phone, address, customer_number, vat_rate, payment_terms, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Raiffeisen Waren GmbH', 'Raiffeisen Waren GmbH', 'einkauf@raiffeisen.de', '+49 89 12345678', 'Musterstraße 10, 80333 München', 'CUST-001', 19.00, '30 Tage', true, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO customers (id, tenant_id, name, company, email, phone, address, customer_number, vat_rate, payment_terms, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'BayWa AG', 'BayWa AG', 'procurement@baywa.de', '+49 89 87654321', 'Arabellastraße 4, 81925 München', 'CUST-002', 19.00, '30 Tage', true, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============================================================
-- 11. FINANCIAL RECORDS (sample)
-- ============================================================
INSERT INTO financial_records (id, tenant_id, cost_center_id, site_id, record_type, amount_eur, currency, date, description, external_id, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', (SELECT id FROM cost_centers WHERE code = 'CC001' AND tenant_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'), NULL, 'expense', -4250.00, 'EUR', '2024-10-01'::timestamptz, 'Saatgut Winterweizen, 5000 kg DKC 3505', NULL, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO financial_records (id, tenant_id, cost_center_id, site_id, record_type, amount_eur, currency, date, description, external_id, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', (SELECT id FROM cost_centers WHERE code = 'CC003' AND tenant_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'), NULL, 'expense', -1250.00, 'EUR', '2024-11-15'::timestamptz, 'Grosswartung Fendt 724 nach 450 Bh', NULL, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO financial_records (id, tenant_id, cost_center_id, site_id, record_type, amount_eur, currency, date, description, external_id, created_at, updated_at)
VALUES (gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', (SELECT id FROM cost_centers WHERE code = 'CC002' AND tenant_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'), NULL, 'expense', -180.00, 'EUR', '2024-11-05'::timestamptz, 'Tierarzt Bella', NULL, NOW(), NOW())
ON CONFLICT DO NOTHING;
