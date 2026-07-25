-- Migration: Row-Level Security (RLS) Policies for Harvest, Water, Olive, Vineyard, Compliance, Finance tables
-- This adds RLS policies for all tables created after the core tables

-- ============================================================
-- HARVEST TABLES
-- ============================================================

ALTER TABLE harvest_seasons ENABLE ROW LEVEL SECURITY;
ALTER TABLE harvest_lots ENABLE ROW LEVEL SECURITY;
ALTER TABLE harvest_deliveries ENABLE ROW LEVEL SECURITY;
ALTER TABLE cold_chain_logs ENABLE ROW LEVEL SECURITY;

CREATE POLICY harvest_seasons_select ON harvest_seasons
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_seasons_insert ON harvest_seasons
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_seasons_update ON harvest_seasons
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_seasons_delete ON harvest_seasons
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_lots_select ON harvest_lots
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_lots_insert ON harvest_lots
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_lots_update ON harvest_lots
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_lots_delete ON harvest_lots
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY harvest_deliveries_select ON harvest_deliveries
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY harvest_deliveries_insert ON harvest_deliveries
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY harvest_deliveries_update ON harvest_deliveries
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY harvest_deliveries_delete ON harvest_deliveries
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY cold_chain_logs_select ON cold_chain_logs
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY cold_chain_logs_insert ON cold_chain_logs
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY cold_chain_logs_update ON cold_chain_logs
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY cold_chain_logs_delete ON cold_chain_logs
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        lot_id IN (SELECT id FROM harvest_lots WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- WATER TABLES
-- ============================================================

ALTER TABLE water_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE water_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE water_quotas ENABLE ROW LEVEL SECURITY;

CREATE POLICY water_sources_select ON water_sources
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_sources_insert ON water_sources
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_sources_update ON water_sources
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_sources_delete ON water_sources
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_usage_select ON water_usage
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        source_id IN (SELECT id FROM water_sources WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY water_usage_insert ON water_usage
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

CREATE POLICY water_usage_update ON water_usage
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        source_id IN (SELECT id FROM water_sources WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

CREATE POLICY water_usage_delete ON water_usage
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        source_id IN (SELECT id FROM water_sources WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY water_quotas_select ON water_quotas
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        source_id IN (SELECT id FROM water_sources WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY water_quotas_insert ON water_quotas
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_quotas_update ON water_quotas
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY water_quotas_delete ON water_quotas
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- ============================================================
-- OLIVE TABLES
-- ============================================================

ALTER TABLE olive_groves ENABLE ROW LEVEL SECURITY;
ALTER TABLE olive_oil_records ENABLE ROW LEVEL SECURITY;

CREATE POLICY olive_groves_select ON olive_groves
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY olive_groves_insert ON olive_groves
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY olive_groves_update ON olive_groves
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY olive_groves_delete ON olive_groves
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY olive_oil_records_select ON olive_oil_records
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY olive_oil_records_insert ON olive_oil_records
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY olive_oil_records_update ON olive_oil_records
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY olive_oil_records_delete ON olive_oil_records
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        grove_id IN (SELECT id FROM olive_groves WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- VINEYARD TABLES
-- ============================================================

ALTER TABLE vineyards ENABLE ROW LEVEL SECURITY;
ALTER TABLE kelter_deliveries ENABLE ROW LEVEL SECURITY;

CREATE POLICY vineyards_select ON vineyards
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY vineyards_insert ON vineyards
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY vineyards_update ON vineyards
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY vineyards_delete ON vineyards
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY kelter_deliveries_select ON kelter_deliveries
    FOR SELECT USING (
        is_superadmin() OR
        vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY kelter_deliveries_insert ON kelter_deliveries
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY kelter_deliveries_update ON kelter_deliveries
    FOR UPDATE USING (
        is_superadmin() OR
        vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY kelter_deliveries_delete ON kelter_deliveries
    FOR DELETE USING (
        is_superadmin() OR
        vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- COMPLIANCE TABLES
-- ============================================================

ALTER TABLE plant_protection_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE applicator_licenses ENABLE ROW LEVEL SECURITY;
ALTER TABLE fertilizer_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE compliance_checklists ENABLE ROW LEVEL SECURITY;
ALTER TABLE compliance_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

CREATE POLICY plant_protection_select ON plant_protection_records
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY plant_protection_insert ON plant_protection_records
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY plant_protection_update ON plant_protection_records
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY plant_protection_delete ON plant_protection_records
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY applicator_licenses_select ON applicator_licenses
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY applicator_licenses_insert ON applicator_licenses
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY applicator_licenses_update ON applicator_licenses
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY applicator_licenses_delete ON applicator_licenses
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY fertilizer_records_select ON fertilizer_records
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY fertilizer_records_insert ON fertilizer_records
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY fertilizer_records_update ON fertilizer_records
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY fertilizer_records_delete ON fertilizer_records
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_checklists_select ON compliance_checklists
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_checklists_insert ON compliance_checklists
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_checklists_update ON compliance_checklists
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_checklists_delete ON compliance_checklists
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY compliance_items_select ON compliance_items
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        checklist_id IN (SELECT id FROM compliance_checklists WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY compliance_items_insert ON compliance_items
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        checklist_id IN (SELECT id FROM compliance_checklists WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY compliance_items_update ON compliance_items
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        checklist_id IN (SELECT id FROM compliance_checklists WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        checklist_id IN (SELECT id FROM compliance_checklists WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY compliance_items_delete ON compliance_items
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        checklist_id IN (SELECT id FROM compliance_checklists WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY audit_logs_select ON audit_logs
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY audit_logs_insert ON audit_logs
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

-- Audit logs are immutable - no update/delete policies

-- ============================================================
-- FINANCE TABLES
-- ============================================================

ALTER TABLE pac_applications ENABLE ROW LEVEL SECURITY;
ALTER TABLE cost_centers ENABLE ROW LEVEL SECURITY;
ALTER TABLE financial_records ENABLE ROW LEVEL SECURITY;

CREATE POLICY pac_applications_select ON pac_applications
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY pac_applications_insert ON pac_applications
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY pac_applications_update ON pac_applications
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY pac_applications_delete ON pac_applications
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY cost_centers_select ON cost_centers
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY cost_centers_insert ON cost_centers
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY cost_centers_update ON cost_centers
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY cost_centers_delete ON cost_centers
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY financial_records_select ON financial_records
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        cost_center_id IN (SELECT id FROM cost_centers WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY financial_records_insert ON financial_records
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

CREATE POLICY financial_records_update ON financial_records
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        cost_center_id IN (SELECT id FROM cost_centers WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

CREATE POLICY financial_records_delete ON financial_records
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        cost_center_id IN (SELECT id FROM cost_centers WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- LIVESTOCK TABLES
-- ============================================================

ALTER TABLE grazing_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE treatment_records ENABLE ROW LEVEL SECURITY;

CREATE POLICY grazing_records_select ON grazing_records
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY grazing_records_insert ON grazing_records
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY grazing_records_update ON grazing_records
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY grazing_records_delete ON grazing_records
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY treatment_records_select ON treatment_records
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY treatment_records_insert ON treatment_records
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY treatment_records_update ON treatment_records
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY treatment_records_delete ON treatment_records
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        animal_id IN (SELECT id FROM animals WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- PHENOLOGY TABLES
-- ============================================================

ALTER TABLE phenology_records ENABLE ROW LEVEL SECURITY;

CREATE POLICY phenology_records_select ON phenology_records
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY phenology_records_insert ON phenology_records
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY phenology_records_update ON phenology_records
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY phenology_records_delete ON phenology_records
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- JUNCTION TABLES
-- ============================================================

ALTER TABLE user_sites ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_sites_select ON user_sites
    FOR SELECT USING (
        is_superadmin() OR
        user_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY user_sites_insert ON user_sites
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        (user_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()) AND
         site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id()))
    );

CREATE POLICY user_sites_delete ON user_sites
    FOR DELETE USING (
        is_superadmin() OR
        user_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- WORKFORCE TABLES
-- ============================================================

ALTER TABLE worker_locations ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE worker_task_statuses ENABLE ROW LEVEL SECURITY;

CREATE POLICY worker_locations_select ON worker_locations
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY worker_locations_insert ON worker_locations
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY worker_locations_update ON worker_locations
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY worker_locations_delete ON worker_locations
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY work_logs_select ON work_logs
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY work_logs_insert ON work_logs
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY work_logs_update ON work_logs
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY work_logs_delete ON work_logs
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY worker_task_statuses_select ON worker_task_statuses
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        task_id IN (SELECT id FROM tasks WHERE tenant_id = get_current_tenant_id()) OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY worker_task_statuses_insert ON worker_task_statuses
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        (task_id IN (SELECT id FROM tasks WHERE tenant_id = get_current_tenant_id()) AND
         worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()))
    );

CREATE POLICY worker_task_statuses_update ON worker_task_statuses
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        task_id IN (SELECT id FROM tasks WHERE tenant_id = get_current_tenant_id()) OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        (task_id IN (SELECT id FROM tasks WHERE tenant_id = get_current_tenant_id()) AND
         worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()))
    );

CREATE POLICY worker_task_statuses_delete ON worker_task_statuses
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        task_id IN (SELECT id FROM tasks WHERE tenant_id = get_current_tenant_id()) OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id())
    );

-- ============================================================
-- COMMENTS
-- ============================================================

COMMENT ON POLICY harvest_seasons_select ON harvest_seasons IS 'Tenant isolation for harvest seasons';
COMMENT ON POLICY harvest_lots_select ON harvest_lots IS 'Tenant isolation for harvest lots';
COMMENT ON POLICY harvest_deliveries_select ON harvest_deliveries IS 'Tenant isolation via lot ownership';
COMMENT ON POLICY cold_chain_logs_select ON cold_chain_logs IS 'Tenant isolation via lot ownership';
COMMENT ON POLICY water_sources_select ON water_sources IS 'Tenant isolation for water sources';
COMMENT ON POLICY water_usage_select ON water_usage IS 'Tenant isolation via source/site ownership';
COMMENT ON POLICY water_quotas_select ON water_quotas IS 'Tenant isolation via source/site ownership';
COMMENT ON POLICY olive_groves_select ON olive_groves IS 'Tenant isolation for olive groves';
COMMENT ON POLICY olive_oil_records_select ON olive_oil_records IS 'Tenant isolation via grove ownership';
COMMENT ON POLICY vineyards_select ON vineyards IS 'Tenant isolation for vineyards';
COMMENT ON POLICY kelter_deliveries_select ON kelter_deliveries IS 'Tenant isolation via vineyard ownership';
COMMENT ON POLICY plant_protection_select ON plant_protection_records IS 'Tenant isolation for plant protection';
COMMENT ON POLICY applicator_licenses_select ON applicator_licenses IS 'Tenant isolation for applicator licenses';
COMMENT ON POLICY fertilizer_records_select ON fertilizer_records IS 'Tenant isolation for fertilizer records';
COMMENT ON POLICY compliance_checklists_select ON compliance_checklists IS 'Tenant isolation for compliance checklists';
COMMENT ON POLICY compliance_items_select ON compliance_items IS 'Tenant isolation via checklist ownership';
COMMENT ON POLICY audit_logs_select ON audit_logs IS 'Tenant isolation for audit logs';
COMMENT ON POLICY pac_applications_select ON pac_applications IS 'Tenant isolation for PAC applications';
COMMENT ON POLICY cost_centers_select ON cost_centers IS 'Tenant isolation for cost centers';
COMMENT ON POLICY financial_records_select ON financial_records IS 'Tenant isolation via cost center ownership';
COMMENT ON POLICY grazing_records_select ON grazing_records IS 'Tenant isolation via animal ownership';
COMMENT ON POLICY treatment_records_select ON treatment_records IS 'Tenant isolation via animal ownership';
COMMENT ON POLICY phenology_records_select ON phenology_records IS 'Tenant isolation via site ownership';
COMMENT ON POLICY user_sites_select ON user_sites IS 'Tenant isolation via user/site ownership';
COMMENT ON POLICY worker_locations_select ON worker_locations IS 'Tenant isolation via worker ownership';
COMMENT ON POLICY work_logs_select ON work_logs IS 'Tenant isolation via worker ownership';
COMMENT ON POLICY worker_task_statuses_select ON worker_task_statuses IS 'Tenant isolation via task/worker ownership';