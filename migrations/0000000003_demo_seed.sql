-- Demo Seed Data for AgroCore
-- This file creates a complete demo setup with:
-- 1 Tenant (demo) + 1 Admin User
-- 3 Sites (Weizen, Mais, Weide)
-- 3 Equipment (Traktor, Mähdrescher, Spritze)
-- 2 Workers (Hans, Maria)
-- 2 Orders (Aussaat, Gülle)
-- 3 Inventory Items + Lagerort
-- 3 Tiere (Bella, Lotte, Bruno)

-- Note: Uses the same UUIDs as the API demo seed endpoint for consistency

\set tenant_id 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
\set admin_user_id 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
\set worker1_id 'cccccccc-cccc-cccc-cccc-cccccccccccc'
\set worker2_id 'dddddddd-dddd-dddd-dddd-dddddddddddd'
\set site1_id 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee'
\set site2_id 'ffffffff-ffff-ffff-ffff-ffffffffffff'
\set site3_id '11111111-1111-1111-1111-111111111111'
\set equip1_id '22222222-2222-2222-2222-222222222222'
\set equip2_id '33333333-3333-3333-3333-333333333333'
\set equip3_id '44444444-4444-4444-4444-444444444444'
\set order1_id '55555555-5555-5555-5555-555555555555'
\set order2_id '66666666-6666-6666-6666-666666666666'
\set inv_loc_id '77777777-7777-7777-7777-777777777777'
\set inv1_id '88888888-8888-8888-8888-888888888888'
\set inv2_id '99999999-9999-9999-9999-999999999999'
\set inv3_id 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee'
\set animal1_id 'bbbbbbbb-cccc-dddd-eeee-ffffffffffff'
\set animal2_id 'cccccccc-dddd-eeee-ffff-gggggggggggg'
\set animal3_id 'dddddddd-eeee-ffff-gggg-hhhhhhhhhhhh'

-- ============================================================
-- 1. TENANT & USERS
-- ============================================================

-- Tenant
INSERT INTO tenants (id, name, slug, config, is_active, created_at, updated_at)
VALUES (:'tenant_id', 'Demo Farm', 'demo', '{}', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, updated_at = NOW();

-- Admin User
INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, language, color, is_active, roles, created_at, updated_at)
VALUES (:'admin_user_id', :'tenant_id', 'Demo', 'Admin', 'admin@demo.local', '$2a$10$XQx5YjZ5YjZ5YjZ5YjZ5YOXQx5YjZ5YjZ5YjZ5YjZ5YjZ5YjZ5YjZ', 'de', '#3B82F6', true, ARRAY['ADMIN'], NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, updated_at = NOW();

-- Worker Users
INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, language, color, is_active, roles, created_at, updated_at)
VALUES (:'worker1_id', :'tenant_id', 'Hans', 'Müller', 'hans@demo.local', '$2a$10$XQx5YjZ5YjZ5YjZ5YjZ5YOXQx5YjZ5YjZ5YjZ5YjZ5YjZ5YjZ5YjZ', 'de', '#10B981', true, ARRAY['WORKER'], NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, updated_at = NOW();

INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, language, color, is_active, roles, created_at, updated_at)
VALUES (:'worker2_id', :'tenant_id', 'Maria', 'Schmidt', 'maria@demo.local', '$2a$10$XQx5YjZ5YjZ5YjZ5YjZ5YOXQx5YjZ5YjZ5YjZ5YjZ5YjZ5YjZ5YjZ', 'de', '#F59E0B', true, ARRAY['WORKER'], NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, updated_at = NOW();

-- Worker records
INSERT INTO workers (id, tenant_id, user_id, employee_id, firstname, lastname, email, phone, role_in_company, hourly_rate, social_security_number, bank_account, is_active, created_at, updated_at)
VALUES (:'worker1_id', :'tenant_id', :'worker1_id', 'EMP-001', 'Hans', 'Müller', 'hans@demo.local', '+49 170 1234567', 'operator', 18.50, 'DE123456789', 'DE89370400440532013000', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

INSERT INTO workers (id, tenant_id, user_id, employee_id, firstname, lastname, email, phone, role_in_company, hourly_rate, social_security_number, bank_account, is_active, created_at, updated_at)
VALUES (:'worker2_id', :'tenant_id', :'worker2_id', 'EMP-002', 'Maria', 'Schmidt', 'maria@demo.local', '+49 170 7654321', 'operator', 16.00, 'DE987654321', 'DE89370400440532013001', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 2. SITES
-- ============================================================

-- Site 1: Weizenfeld (Field)
INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety, area, gross_area, planted_date, soil_type, slope, slope_facing, altitude, organic, is_active, is_temporary, created_at, updated_at, created_by, updated_by)
VALUES (:'site1_id', :'tenant_id', NULL, 'Nordfeld - Weizen', 'field', '{"Vegetable": "Wheat"}', 'Winterweizen', 15.5, 17.0, '2024-10-15'::date, 'Lehmboden', 2.5, 'Süd', 180.0, false, true, false, NOW(), NOW(), :'admin_user_id', :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label, updated_at = NOW();

-- Site 2: Maisfeld (Field)
INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety, area, gross_area, planted_date, soil_type, slope, slope_facing, altitude, organic, is_active, is_temporary, created_at, updated_at, created_by, updated_by)
VALUES (:'site2_id', :'tenant_id', NULL, 'Südhang - Mais', 'field', '{"Vegetable": "Corn"}', 'Silomais DKC 3505', 12.0, 13.5, '2024-05-01'::date, 'Sandiger Lehm', 5.0, 'Südwest', 165.0, false, true, false, NOW(), NOW(), :'admin_user_id', :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label, updated_at = NOW();

-- Site 3: Weide (Pasture)
INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety, area, gross_area, planted_date, soil_type, slope, slope_facing, altitude, organic, is_active, is_temporary, created_at, updated_at, created_by, updated_by)
VALUES (:'site3_id', :'tenant_id', NULL, 'Westweide - Rinder', 'pasture', '{"Other": "Grassland"}', 'Dauergrünland', 25.0, 27.0, '2020-03-01'::date, 'Toniger Lehm', 3.0, 'West', 170.0, true, true, false, NOW(), NOW(), :'admin_user_id', :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label, updated_at = NOW();

-- ============================================================
-- 3. EQUIPMENT
-- ============================================================

-- Equipment 1: Traktor (Fendt 724 Vario)
INSERT INTO equipment (id, tenant_id, label, equipment_type, manufacturer, model, serial_number, purchase_date, purchase_price, current_value, operating_hours, last_maintenance, next_maintenance, fuel_type, fuel_consumption_per_hour, status, location_site_id, created_at, updated_at, created_by)
VALUES (:'equip1_id', :'tenant_id', 'Traktor Fendt 724', 'tractor', 'Fendt', '724 Vario', 'FEN724-2023-001', '2023-03-15', 185000.00, 165000.00, 450, '2024-11-15'::date, '2025-05-15'::date, 'diesel', 12.5, 'available', :'site1_id', NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Equipment 2: Mähdrescher (Claas Lexion 7600)
INSERT INTO equipment (id, tenant_id, label, equipment_type, manufacturer, model, serial_number, purchase_date, purchase_price, current_value, operating_hours, last_maintenance, next_maintenance, fuel_type, fuel_consumption_per_hour, status, location_site_id, created_at, updated_at, created_by)
VALUES (:'equip2_id', :'tenant_id', 'Mähdrescher Claas 7600', 'harvester', 'Claas', 'Lexion 7600', 'CLA7600-2022-003', '2022-07-20', 380000.00, 320000.00, 890, '2024-10-01'::date, '2025-06-01'::date, 'diesel', 28.0, 'available', :'site2_id', NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Equipment 3: Spritze (Amazone UX 5201)
INSERT INTO equipment (id, tenant_id, label, equipment_type, manufacturer, model, serial_number, purchase_date, purchase_price, current_value, operating_hours, last_maintenance, next_maintenance, fuel_type, fuel_consumption_per_hour, status, location_site_id, created_at, updated_at, created_by)
VALUES (:'equip3_id', :'tenant_id', 'Spritze Amazone UX 5201', 'sprayer', 'Amazone', 'UX 5201', 'AMA5201-2023-007', '2023-02-10', 65000.00, 58000.00, 120, '2024-12-01'::date, '2025-03-01'::date, 'diesel', 4.5, 'maintenance', :'site1_id', NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 4. ORDERS
-- ============================================================

-- Order 1: Aussaat Weizen (assigned to Hans)
INSERT INTO orders (id, tenant_id, site_id, label, description, order_type, status, priority, planned_date, deadline_date, started_at, completed_at, assigned_worker_ids, equipment_ids, execution_policy, created_at, updated_at, created_by)
VALUES (:'order1_id', :'tenant_id', :'site1_id', 'Aussaat Winterweizen Nordfeld', 'Aussaat von Winterweizen auf dem Nordfeld (15.5 ha). Saatstärke: 350 Körner/m². Reihe: 12.5 cm.', 'seeding', 'planned', 'high', '2024-10-20'::date, '2024-10-25'::date, NULL, NULL, ARRAY[:'worker1_id'], ARRAY[:'equip1_id', :'equip3_id'], '{"mode": "Manual"}', NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Order 2: Gülle ausbringen (assigned to Maria)
INSERT INTO orders (id, tenant_id, site_id, label, description, order_type, status, priority, planned_date, deadline_date, started_at, completed_at, assigned_worker_ids, equipment_ids, execution_policy, created_at, updated_at, created_by)
VALUES (:'order2_id', :'tenant_id', :'site3_id', 'Gülleausbringung Westweide', 'Ausbringung von Rindergülle auf der Westweide (25 ha). 30 m³/ha. Bodennah mit Schleppschlauch.', 'fertilizing', 'in_progress', 'medium', '2024-11-10'::date, '2024-11-15'::date, NOW(), NULL, ARRAY[:'worker2_id'], ARRAY[:'equip1_id'], '{"mode": "Manual"}', NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 5. INVENTORY LOCATION & ITEMS
-- ============================================================

-- Lagerort: Haupthalle
INSERT INTO inventory_locations (id, tenant_id, label, description, location_type, address, gps_coordinates, is_active, created_at, updated_at)
VALUES (:'inv_loc_id', :'tenant_id', 'Haupthalle', 'Zentrale Lagerhalle für Betriebsmittel', 'warehouse', 'Hofstraße 1, 12345 Demodorf', '{"lng": 10.123, "lat": 51.456}', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Inventory Item 1: Winterweizensaatgut
INSERT INTO inventory_items (id, tenant_id, label, sku, category, unit, current_stock, min_stock, max_stock, unit_price, supplier, location_id, batch_number, expiry_date, is_active, created_at, updated_at)
VALUES (:'inv1_id', :'tenant_id', 'Winterweizensaatgut DKC 3505', 'SEED-WW-3505', 'seeds', 'kg', 5000, 1000, 10000, 0.85, 'Raiffeisen', :'inv_loc_id', 'DKC3505-2024-001', '2025-12-31'::date, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Inventory Item 2: Dünger NPK 20-10-10
INSERT INTO inventory_items (id, tenant_id, label, sku, category, unit, current_stock, min_stock, max_stock, unit_price, supplier, location_id, batch_number, expiry_date, is_active, created_at, updated_at)
VALUES (:'inv2_id', :'tenant_id', 'NPK Dünger 20-10-10', 'FERT-NPK-201010', 'fertilizer', 'kg', 3000, 500, 5000, 0.65, 'BayWa', :'inv_loc_id', 'NPK201010-2024-005', '2026-06-30'::date, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Inventory Item 3: Pflanzenschutzmittel (Herbizid)
INSERT INTO inventory_items (id, tenant_id, label, sku, category, unit, current_stock, min_stock, max_stock, unit_price, supplier, location_id, batch_number, expiry_date, is_active, created_at, updated_at)
VALUES (:'inv3_id', :'tenant_id', 'Herbizid Mais - MaisTer', 'CHEM-HERB-MAISTER', 'crop_protection', 'l', 200, 50, 500, 45.00, 'BASF', :'inv_loc_id', 'MAISTER-2024-012', '2025-09-30'::date, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 6. ANIMALS (Rinder)
-- ============================================================

-- Animal 1: Bella (Kuh)
INSERT INTO animals (id, tenant_id, identifier, plot_id, livestock_type, species, breed, birth_date, gender, status, current_site_id, weight_kg, mother_id, father_id, created_at, updated_at, created_by)
VALUES (:'animal1_id', :'tenant_id', 'DE0912345678', NULL, 'cattle', 'Cattle', 'Fleckvieh', '2022-03-15'::date, 'female', 'Active', :'site3_id', 580.0, NULL, NULL, NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Animal 2: Lotte (Kuh)
INSERT INTO animals (id, tenant_id, identifier, plot_id, livestock_type, species, breed, birth_date, gender, status, current_site_id, weight_kg, mother_id, father_id, created_at, updated_at, created_by)
VALUES (:'animal2_id', :'tenant_id', 'DE0912345679', NULL, 'cattle', 'Cattle', 'Fleckvieh', '2021-07-22'::date, 'female', 'Active', :'site3_id', 620.0, NULL, NULL, NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- Animal 3: Bruno (Stier)
INSERT INTO animals (id, tenant_id, identifier, plot_id, livestock_type, species, breed, birth_date, gender, status, current_site_id, weight_kg, mother_id, father_id, created_at, updated_at, created_by)
VALUES (:'animal3_id', :'tenant_id', 'DE0912345680', NULL, 'cattle', 'Cattle', 'Fleckvieh', '2023-01-10'::date, 'male', 'Active', :'site3_id', 320.0, :'animal1_id', NULL, NOW(), NOW(), :'admin_user_id')
ON CONFLICT (id) DO UPDATE SET updated_at = NOW();

-- ============================================================
-- 7. LIVESTOCK SUMMARY (Counts per plot)
-- ============================================================

INSERT INTO livestock (id, plot_id, herd_id, livestock_type, count, label, created_at)
VALUES (gen_random_uuid(), NULL, 'Rinderherde Westweide', 'cattle', 3, 'Fleckvieh Herde Westweide', NOW())
ON CONFLICT DO NOTHING;

-- ============================================================
-- 8. GRAZING RECORDS (for the animals on pasture)
-- ============================================================

INSERT INTO grazing_records (id, animal_id, plot_id, site_id, start_time, start_date, end_time, end_date, notes, created_at)
VALUES (gen_random_uuid(), :'animal1_id', NULL, :'site3_id', NOW() - interval '30 days', CURRENT_DATE - 30, NULL, NULL, 'Bella auf Westweide aufgetrieben', NOW() - interval '30 days')
ON CONFLICT DO NOTHING;

INSERT INTO grazing_records (id, animal_id, plot_id, site_id, start_time, start_date, end_time, end_date, notes, created_at)
VALUES (gen_random_uuid(), :'animal2_id', NULL, :'site3_id', NOW() - interval '30 days', CURRENT_DATE - 30, NULL, NULL, 'Lotte auf Westweide aufgetrieben', NOW() - interval '30 days')
ON CONFLICT DO NOTHING;

INSERT INTO grazing_records (id, animal_id, plot_id, site_id, start_time, start_date, end_time, end_date, notes, created_at)
VALUES (gen_random_uuid(), :'animal3_id', NULL, :'site3_id', NOW() - interval '10 days', CURRENT_DATE - 10, NULL, NULL, 'Bruno auf Westweide aufgetrieben', NOW() - interval '10 days')
ON CONFLICT DO NOTHING;

-- ============================================================
-- 9. COST CENTERS (for financial tracking)
-- ============================================================

INSERT INTO cost_centers (id, tenant_id, code, label, description, parent_id, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', 'CC001', 'Pflanzenbau', 'Kosten für Ackerbau und Pflanzenbau', NULL, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO cost_centers (id, tenant_id, code, label, description, parent_id, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', 'CC002', 'Tierhaltung', 'Kosten für Rinderhaltung und Weide', NULL, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO cost_centers (id, tenant_id, code, label, description, parent_id, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', 'CC003', 'Maschinen', 'Kosten für Traktor, Mähdrescher, Spritze', NULL, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============================================================
-- 10. CUSTOMERS (for orders/sales)
-- ============================================================

INSERT INTO customers (id, tenant_id, label, customer_type, email, phone, address, vat_id, payment_terms, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', 'Raiffeisen Waren GmbH', 'company', 'einkauf@raiffeisen.de', '+49 89 12345678', 'Musterstraße 10, 80333 München', 'DE123456789', 30, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO customers (id, tenant_id, label, customer_type, email, phone, address, vat_id, payment_terms, is_active, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', 'BayWa AG', 'company', 'procurement@baywa.de', '+49 89 87654321', 'Arabellastraße 4, 81925 München', 'DE987654321', 30, true, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============================================================
-- 11. FINANCIAL RECORDS (sample)
-- ============================================================

INSERT INTO financial_records (id, tenant_id, cost_center_id, record_type, label, amount, currency, record_date, description, reference_type, reference_id, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', (SELECT id FROM cost_centers WHERE code = 'CC001' AND tenant_id = :'tenant_id'), 'expense', 'Saatchgut Winterweizen', -4250.00, 'EUR', '2024-10-01'::date, '5000 kg Saatgut DKC 3505 à 0.85 €/kg', 'inventory', :'inv1_id', NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO financial_records (id, tenant_id, cost_center_id, record_type, label, amount, currency, record_date, description, reference_type, reference_id, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', (SELECT id FROM cost_centers WHERE code = 'CC003' AND tenant_id = :'tenant_id'), 'expense', 'Wartung Traktor', -1250.00, 'EUR', '2024-11-15'::date, 'Großwartung Fendt 724 nach 450 Bh', 'equipment', :'equip1_id', NOW(), NOW())
ON CONFLICT DO NOTHING;

INSERT INTO financial_records (id, tenant_id, cost_center_id, record_type, label, amount, currency, record_date, description, reference_type, reference_id, created_at, updated_at)
VALUES (gen_random_uuid(), :'tenant_id', (SELECT id FROM cost_centers WHERE code = 'CC002' AND tenant_id = :'tenant_id'), 'expense', 'Tierarzt Bella', -180.00, 'EUR', '2024-11-05'::date, 'Routineuntersuchung + Impfung', 'animal', :'animal1_id', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============================================================
-- COMPLETION MESSAGE
-- ============================================================
\echo '✅ Demo seed data inserted successfully!'
\echo 'Tenant: demo'
\echo 'Admin: admin@demo.local / demo123'
\echo 'Workers: hans@demo.local, maria@demo.local'
\echo 'Sites: 3 (Weizen, Mais, Weide)'
\echo 'Equipment: 3 (Traktor, Mähdrescher, Spritze)'
\echo 'Orders: 2 (Aussaat, Gülle)'
\echo 'Inventory: 3 items in 1 location'
\echo 'Animals: 3 (Bella, Lotte, Bruno)'