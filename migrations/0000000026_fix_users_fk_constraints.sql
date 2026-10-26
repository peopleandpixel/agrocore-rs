-- Migration 26: Fix foreign key constraints on users table
--
-- Several tables reference users(id) without an ON DELETE clause, which defaults
-- to RESTRICT. This prevents deleting users who have related records.
-- Nullable columns get ON DELETE SET NULL; NOT NULL columns get ON DELETE CASCADE.

-- audit_logs.user_id (nullable)
ALTER TABLE audit_logs
    DROP CONSTRAINT audit_logs_user_id_fkey,
    ADD CONSTRAINT audit_logs_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;

-- orders.assigned_to (nullable)
ALTER TABLE orders
    DROP CONSTRAINT orders_assigned_to_fkey,
    ADD CONSTRAINT orders_assigned_to_fkey
        FOREIGN KEY (assigned_to) REFERENCES users(id) ON DELETE SET NULL;

-- plant_protection_records.applicator_id (nullable)
ALTER TABLE plant_protection_records
    DROP CONSTRAINT plant_protection_records_applicator_id_fkey,
    ADD CONSTRAINT plant_protection_records_applicator_id_fkey
        FOREIGN KEY (applicator_id) REFERENCES users(id) ON DELETE SET NULL;

-- applicator_licenses.user_id (NOT NULL – cascade)
ALTER TABLE applicator_licenses
    DROP CONSTRAINT applicator_licenses_user_id_fkey,
    ADD CONSTRAINT applicator_licenses_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- worker_task_statuses.worker_id (NOT NULL – cascade)
ALTER TABLE worker_task_statuses
    DROP CONSTRAINT worker_task_statuses_worker_id_fkey,
    ADD CONSTRAINT worker_task_statuses_worker_id_fkey
        FOREIGN KEY (worker_id) REFERENCES users(id) ON DELETE CASCADE;

-- inventory_transactions.created_by (nullable)
ALTER TABLE inventory_transactions
    DROP CONSTRAINT inventory_transactions_created_by_fkey,
    ADD CONSTRAINT inventory_transactions_created_by_fkey
        FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- phenology_records.observer_id (nullable)
ALTER TABLE phenology_records
    DROP CONSTRAINT phenology_records_observer_id_fkey,
    ADD CONSTRAINT phenology_records_observer_id_fkey
        FOREIGN KEY (observer_id) REFERENCES users(id) ON DELETE SET NULL;
