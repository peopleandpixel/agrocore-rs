-- demo_seed.sql ── Demo data for AgroCore development
-- Run with: docker exec -i agrocore-postgres psql -U agrocore -d agrocore < scripts/demo_seed.sql

-- Create demo tenant
INSERT INTO tenants (name, slug, is_active, created_at, updated_at)
VALUES ('Demo Farm', 'demo', true, NOW(), NOW())
ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name;

-- Create users
INSERT INTO users (tenant_id, firstname, lastname, email, password_hash, roles, is_active, internal_cost_per_hour, external_cost_per_hour, color, language, created_at, updated_at)
SELECT t.id, u.firstname, u.lastname, u.email, u.password_hash, u.roles, u.is_active, u.internal_cost_per_hour, u.external_cost_per_hour, u.color, u.language, NOW(), NOW()
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
CROSS JOIN (
    VALUES
        ('Admin', 'User', 'admin@demo.local', '$argon2id$v=19$m=19456,t=2,p=1$demo123', '["Admin"]'::jsonb, true, 50.00, 80.00, '#3B82F6', 'de'),
        ('Worker', 'Demo', 'worker@demo.local', '$argon2id$v=19$m=19456,t=2,p=1$demo123', '["Worker"]'::jsonb, true, 30.00, 50.00, '#22C55E', 'de')
) AS u(firstname, lastname, email, password_hash, roles, is_active, internal_cost_per_hour, external_cost_per_hour, color, language)
ON CONFLICT (email) DO UPDATE SET
    tenant_id = EXCLUDED.tenant_id,
    roles = EXCLUDED.roles;

-- Create demo sites (3 sites) - each as separate INSERT
-- Disable trigger temporarily to avoid round() issue
SET session_replication_role = 'replica';

INSERT INTO sites (
    tenant_id, business_id, label, code, description, site_type, crop_type, variety,
    area, gross_area, center_lng, center_lat, boundary, center,
    is_active, is_temporary, created_at, updated_at, created_by, updated_by,
    plots, row_config, bbch_stage, planted_date, cleared_date,
    soil_type, slope, slope_facing, altitude, organic, organic_eligible,
    sigpac_data, regepac_id, properties, custom_fields, note1, note2,
    lpis_country, lpis_data
)
SELECT
    t.id, NULL,
    'Weizenfeld Nord'::text, 'WHEAT-N'::text, 'Hauptweizenfeld im Norden'::text,
    '{"type": "arable", "subtype": "wheat"}'::jsonb,
    '{"name": "Winterweizen", "variety": "Bavaria"}'::jsonb, 'Bavaria'::text,
    12.5::numeric, 13.0::numeric, 8.682127::numeric, 50.110922::numeric,
    ST_SetSRID(ST_MakePolygon(ST_GeomFromText('LINESTRING(8.681 50.111, 8.683 50.111, 8.683 50.110, 8.681 50.110, 8.681 50.111)')), 4326),
    ST_SetSRID(ST_MakePoint(8.682127, 50.110922), 4326),
    true, false, NOW(), NOW(),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    '[]'::jsonb, NULL::jsonb,
    '{"stage": "00", "description": "Trockenlegung"}'::jsonb,
    '2024-10-15'::timestamp, NULL::timestamp,
    'Lehmboden'::text, 2.5::numeric, 'S'::text, 120::numeric, true, false,
    NULL::jsonb, NULL::text, '[]'::jsonb, '{}'::jsonb,
    'Hauptfrucht 2024'::text, 'Gute Bodenqualität'::text,
    'DE'::text, NULL::jsonb
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
ON CONFLICT DO NOTHING;

-- Second site
INSERT INTO sites (
    tenant_id, business_id, label, code, description, site_type, crop_type, variety,
    area, gross_area, center_lng, center_lat, boundary, center,
    is_active, is_temporary, created_at, updated_at, created_by, updated_by,
    plots, row_config, bbch_stage, planted_date, cleared_date,
    soil_type, slope, slope_facing, altitude, organic, organic_eligible,
    sigpac_data, regepac_id, properties, custom_fields, note1, note2,
    lpis_country, lpis_data
)
SELECT
    t.id, NULL,
    'Maisfeld Süd'::text, 'MAIZE-S'::text, 'Maisfeld für Silomais'::text,
    '{"type": "arable", "subtype": "maize"}'::jsonb,
    '{"name": "Silomais", "variety": "DKC 3972"}'::jsonb, 'DKC 3972'::text,
    8.3::numeric, 8.5::numeric, 8.678900::numeric, 50.108500::numeric,
    ST_SetSRID(ST_MakePolygon(ST_GeomFromText('LINESTRING(8.678 50.109, 8.680 50.109, 8.680 50.108, 8.678 50.108, 8.678 50.109)')), 4326),
    ST_SetSRID(ST_MakePoint(8.678900, 50.108500), 4326),
    true, false, NOW(), NOW(),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    '[]'::jsonb, NULL::jsonb,
    '{"stage": "00", "description": "Bodenvorbereitung"}'::jsonb,
    '2024-04-20'::timestamp, NULL::timestamp,
    'Sandiger Lehm'::text, 1.8::numeric, 'W'::text, 115::numeric, true, false,
    NULL::jsonb, NULL::text, '[]'::jsonb, '{}'::jsonb,
    'Nachfrucht nach Weizen'::text, 'Gute Entwässerung'::text,
    'DE'::text, NULL::jsonb
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
ON CONFLICT DO NOTHING;

-- Third site
INSERT INTO sites (
    tenant_id, business_id, label, code, description, site_type, crop_type, variety,
    area, gross_area, center_lng, center_lat, boundary, center,
    is_active, is_temporary, created_at, updated_at, created_by, updated_by,
    plots, row_config, bbch_stage, planted_date, cleared_date,
    soil_type, slope, slope_facing, altitude, organic, organic_eligible,
    sigpac_data, regepac_id, properties, custom_fields, note1, note2,
    lpis_country, lpis_data
)
SELECT
    t.id, NULL,
    'Dauergrünland Weide'::text, 'GRASS-P'::text, 'Dauerhaftes Grünland für Rinderhaltung'::text,
    '{"type": "grassland", "subtype": "pasture"}'::jsonb,
    '{"name": "Dauergrünland", "variety": "Mischung"}'::jsonb, 'Deutsches Weidegras'::text,
    5.2::numeric, 5.5::numeric, 8.685000::numeric, 50.112000::numeric,
    ST_SetSRID(ST_MakePolygon(ST_GeomFromText('LINESTRING(8.684 50.112, 8.686 50.112, 8.686 50.111, 8.684 50.111, 8.684 50.112)')), 4326),
    ST_SetSRID(ST_MakePoint(8.685000, 50.112000), 4326),
    true, false, NOW(), NOW(),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    '[]'::jsonb, NULL::jsonb,
    '{"stage": "00", "description": "Dauergrünland"}'::jsonb,
    NULL::timestamp, NULL::timestamp,
    'Torflehm'::text, 0.5::numeric, 'N'::text, 110::numeric, false, true,
    NULL::jsonb, NULL::text, '[]'::jsonb, '{}'::jsonb,
    'Extensive Beweidung'::text, 'NATURA 2000 Gebiet in der Nähe'::text,
    'DE'::text, NULL::jsonb
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
ON CONFLICT DO NOTHING;

SET session_replication_role = 'origin';

-- Create demo equipment
INSERT INTO equipment (
    tenant_id, label, code, equipment_type, in_usage, maintenance_intervals,
    next_maintenance_date, last_maintenance_hours, is_active,
    fuel_capacity_liters, fuel_type, original_cost, salvage_value,
    purchase_date, depreciation_method, useful_life_years,
    created_at, updated_at
)
SELECT
    t.id,
    e.label, e.code, e.equipment_type, e.in_usage, e.maintenance_intervals,
    e.next_maintenance_date::date, e.last_maintenance_hours, e.is_active,
    e.fuel_capacity_liters, e.fuel_type, e.original_cost, e.salvage_value,
    e.purchase_date::date, e.depreciation_method, e.useful_life_years,
    NOW(), NOW()
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
CROSS JOIN (
    VALUES
        ('John Deere 6R 185', 'JD-6R-185',
         '{"category": "tractor", "power_kw": 136, "transmission": "AutoQuad Plus"}'::jsonb,
         false, '{"interval_hours": 500, "last_service_hours": 1200}'::jsonb,
         '2024-12-01', 1350, true,
         250, 'diesel', 145000.00, 15000.00, '2020-03-15',
         'straight_line', 10
        ),
        ('Claas Lexion 7700', 'CLX-7700',
         '{"category": "combine", "cutting_width_m": 9.0, "tank_capacity_l": 12000}'::jsonb,
         false, '{"interval_hours": 250, "last_service_hours": 850}'::jsonb,
         '2024-07-15', 920, true,
         800, 'diesel', 380000.00, 40000.00, '2019-02-20',
         'straight_line', 8
        ),
        ('Amazone UX 5200', 'AMZ-UX5200',
         '{"category": "sprayer", "working_width_m": 36, "tank_capacity_l": 5200}'::jsonb,
         false, '{"interval_hours": 300, "last_service_hours": 400}'::jsonb,
         '2024-06-01', 450, true,
         150, 'diesel', 95000.00, 10000.00, '2021-01-10',
         'straight_line', 7
        )
) AS e(label, code, equipment_type, in_usage, maintenance_intervals,
       next_maintenance_date, last_maintenance_hours, is_active,
       fuel_capacity_liters, fuel_type, original_cost, salvage_value,
       purchase_date, depreciation_method, useful_life_years)
ON CONFLICT DO NOTHING;

-- Create equipment maintenance logs
INSERT INTO equipment_maintenance_log (
    equipment_id, tenant_id, hours, note, performed_at, created_at,
    parts_cost, labor_hours, downtime_hours
)
SELECT
    e.id, t.id, m.hours, m.note, m.performed_at::timestamptz, NOW(),
    m.parts_cost, m.labor_hours, m.downtime_hours
FROM equipment e
JOIN tenants t ON t.slug = 'demo'
CROSS JOIN (
    VALUES
        (100.0, 850.00, 4.5, 8.0, 'Großwartung: Ölwechsel, Filterwechsel, Hydrauliköl', '2024-03-15 09:00:00+01'),
        (200.0, 1200.00, 6.0, 12.0, 'Saisonende-Wartung: Siebe reinigen, Messer schärfen', '2023-11-20 14:30:00+01'),
        (300.0, 350.00, 2.5, 4.0, 'Düsencheck, Druckeinstellung, Schlauchprüfung', '2024-03-10 10:00:00+01')
) AS m(hours, parts_cost, labor_hours, downtime_hours, note, performed_at)
WHERE e.id IN (SELECT id FROM equipment WHERE tenant_id = (SELECT id FROM tenants WHERE slug = 'demo'))
ON CONFLICT DO NOTHING;

-- Create demo workers (linked to users)
INSERT INTO workers (
    tenant_id, user_id, employee_id, firstname, lastname,
    email, phone, role_in_company, hourly_rate, social_security_number,
    bank_account, is_active, created_at, updated_at
)
SELECT
    t.id,
    u.id,
    w.employee_id, w.firstname, w.lastname,
    w.email, w.phone, w.role_in_company, w.hourly_rate, w.social_security_number,
    w.bank_account, w.is_active, NOW(), NOW()
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
JOIN users u ON u.tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')
CROSS JOIN (
    VALUES
        ('EMP-001', 'Hans', 'Müller', 'worker@demo.local', '+49 170 1234567',
         'Maschinist', 32.50, '12 123456 A 123', 'DE89 3704 0044 0532 0130 00', true),
        ('EMP-002', 'Maria', 'Schmidt', 'admin@demo.local', '+49 171 9876543',
         'Betriebsleiterin', 45.00, '12 654321 B 456', 'DE89 3704 0044 0532 0130 01', true)
) AS w(employee_id, firstname, lastname, email, phone, role_in_company, hourly_rate, social_security_number, bank_account, is_active)
ON CONFLICT DO NOTHING;

-- Create worker locations
INSERT INTO worker_locations (
    tenant_id, worker_id, location, timestamp, accuracy_meters, created_at
)
SELECT
    t.id,
    w.id,
    ST_SetSRID(ST_MakePoint(8.682127, 50.110922), 4326),
    NOW() - INTERVAL '2 hours', 5.0, NOW() - INTERVAL '2 hours'
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
JOIN workers w ON w.tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')
WHERE w.employee_id = 'EMP-001'
ON CONFLICT DO NOTHING;

-- Create demo orders (cast timestamps properly)
INSERT INTO orders (
    tenant_id, label, title, description, priority, status, order_type,
    assigned_to, assigned_worker_ids, site_ids, scheduled_start, scheduled_end,
    actual_start, actual_end, deadline_date, planned_date, recurrence,
    last_completed_at, execution_policy, automation_state, articles,
    quantities, results, weather, custom_fields, parent_order_id,
    workflow_config, cost_center_id, customer_id, created_by, updated_by,
    is_active, created_at, updated_at
)
SELECT
    t.id,
    o.label, o.title, o.description, o.priority, o.status, o.order_type,
    (SELECT id FROM users WHERE email = 'worker@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    o.assigned_worker_ids, o.site_ids, o.scheduled_start::timestamptz, o.scheduled_end::timestamptz,
    o.actual_start::timestamptz, o.actual_end::timestamptz, o.deadline_date::date, o.planned_date::date, o.recurrence::jsonb,
    o.last_completed_at::timestamptz, o.execution_policy::jsonb, o.automation_state::jsonb, o.articles::jsonb,
    o.quantities::jsonb, o.results, o.weather::jsonb, o.custom_fields::jsonb, o.parent_order_id::uuid,
        o.workflow_config::jsonb, o.cost_center_id::uuid, NULL::uuid,
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    (SELECT id FROM users WHERE email = 'admin@demo.local' AND tenant_id = (SELECT id FROM tenants WHERE slug = 'demo')),
    o.is_active, NOW(), NOW()
FROM (SELECT id FROM tenants WHERE slug = 'demo') t
CROSS JOIN (
    VALUES
        ('Aussaat Weizen 2024', 'Winterweizen aussäen', 'Aussaat von Winterweizen Sorte Bavaria auf Feld Nord',
         1, '{"state": "planned"}'::jsonb, 'field_work',
         '["gggggggg-gggg-gggg-gggg-gggggggggggg"]'::jsonb,
         '["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"]'::jsonb,
         '2024-10-15 08:00:00+01', '2024-10-16 18:00:00+01',
         NULL, NULL, '2024-10-16', '2024-10-15', NULL,
         NULL, NULL, '[{"item": "Winterweizen Bavaria", "qty": 250, "unit": "kg"}]'::jsonb,
         '[{"item": "Dünger NPK 20-10-10", "qty": 800, "unit": "kg"}]'::jsonb,
         NULL, NULL, '{"field": "Weizenfeld Nord", "variety": "Bavaria"}'::jsonb,
         NULL, NULL, NULL, '22222222-2222-2222-2222-222222222222'::uuid, '22222222-2222-2222-2222-222222222222'::uuid,
                 true
                ),
                ('Gülleausbringung Maisfeld', 'Gülle auf Maisfeld Süd', 'Organische Düngung vor Maisaussaat',
                  2, '{"state": "planned"}'::jsonb, 'field_work',
                  '["gggggggg-gggg-gggg-gggg-gggggggggggg", "hhhhhhhh-hhhh-hhhh-hhhh-hhhhhhhhhhhh"]'::jsonb,
                  '["bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"]'::jsonb,
                  '2024-04-18 06:00:00+01', '2024-04-18 14:00:00+01',
                  NULL, NULL, '2024-04-18', '2024-04-18', NULL,
                  NULL, NULL, '[{"item": "Rindergülle", "qty": 40, "unit": "m³"}]'::jsonb,
                  '[{"item": "Schleppschlauch", "qty": 1, "unit": "Stück"}]'::jsonb,
                  NULL, NULL, '{"field": "Maisfeld Süd", "method": "Schleppschlauch"}'::jsonb,
                  NULL, NULL, NULL, '22222222-2222-2222-2222-222222222222'::uuid, '22222222-2222-2222-2222-222222222222'::uuid,
                  true
                 )
) AS o(label, title, description, priority, status, order_type,
       assigned_worker_ids, site_ids, scheduled_start, scheduled_end,
       actual_start, actual_end, deadline_date, planned_date, recurrence,
       last_completed_at, execution_policy, automation_state, articles,
       quantities, results, weather, custom_fields, parent_order_id,
       workflow_config, cost_center_id, customer_id, is_active)
ON CONFLICT DO NOTHING;