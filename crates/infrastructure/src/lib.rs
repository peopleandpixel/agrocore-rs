mod defaults;
mod repositories;

use agrocore_domain::repositories::SiteRepository;
use std::sync::Arc;

// Re-export auth_utils functions at crate root for API access
pub use defaults::{default_bind_addr, default_mongodb_uri, default_nats_url};
pub use repositories::AnimalRepo;
pub use repositories::EquipmentRepo;
pub use repositories::auth_utils::{generate_jwt, hash_password, verify_password};
pub use repositories::{
    ApplicatorLicenseRepo, AuditLogRepo, ComplianceChecklistRepo, FertilizerRecordRepo,
    PlantProtectionRecordRepo,
};
pub use repositories::{ColdChainLogRepo, HarvestDeliveryRepo, HarvestLotRepo, HarvestSeasonRepo};
pub use repositories::{CostCenterRepo, FinancialRecordRepo, PACApplicationRepo};
pub use repositories::{KelterDeliveryRepo, VineyardRepo};
pub use repositories::{OliveGroveRepo, OliveOilRecordRepo};
pub use repositories::{
    OrderRepo, SiteRepo, SpatialObjectRepo, TaskDataRepo, TenantRepo, UserRepo,
};
pub use repositories::{PhenologyRecordRepo, WeatherDataRepo, WeatherStationRepo};
pub use repositories::{WaterQuotaRepo, WaterSourceRepo, WaterUsageRepo};
pub use repositories::{WorkLogRepo, WorkerLocationRepo, WorkerRepo, WorkerTaskStatusRepo};

use mongodb::bson::doc;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};
use std::env;

/// Standard-MongoDB-Pool-Größe für Agrar-Anwendung (weniger als 100 gleichzeitige Requests)
const DEFAULT_MAX_POOL_SIZE: u32 = 20;
const DEFAULT_MIN_POOL_SIZE: u32 = 5;

#[derive(Clone)]
pub struct Database {
    client: Client,
    db_name: String,
}

impl Database {
    pub async fn connect(uri: &str, db_name: &str) -> anyhow::Result<Self> {
        let mut opts = ClientOptions::parse(uri).await?;

        opts.max_pool_size = Some(
            env::var("MONGODB_MAX_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MAX_POOL_SIZE),
        );
        opts.min_pool_size = Some(
            env::var("MONGODB_MIN_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MIN_POOL_SIZE),
        );
        opts.max_idle_time = Some(std::time::Duration::from_secs(300));
        opts.connect_timeout = Some(std::time::Duration::from_secs(10));
        opts.server_selection_timeout = Some(std::time::Duration::from_secs(5));
        let client = Client::with_options(opts)?;
        let db = Self {
            client,
            db_name: db_name.to_string(),
        };
        db.create_indexes().await?;
        Ok(db)
    }

    async fn create_indexes(&self) -> anyhow::Result<()> {
        async fn create_active_updated_index<T: Send + Sync>(
            collection: &Collection<T>,
        ) -> anyhow::Result<()> {
            collection
                .create_index(
                    IndexModel::builder()
                        .keys(doc! { "tenant_id": 1, "is_active": 1, "updated_at": -1 })
                        .build(),
                )
                .await?;
            Ok(())
        }

        // Users: unique email
        let user_collection = self.collection::<agrocore_domain::entities::user::User>("users");
        user_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "email": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Sites: tenant_id + label
        let site_collection = self.collection::<agrocore_domain::entities::site::Site>("sites");
        site_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "label": 1 })
                    .build(),
            )
            .await?;
        site_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "center": "2dsphere" })
                    .build(),
            )
            .await?;
        create_active_updated_index(&site_collection).await?;

        // Orders
        let order_collection = self.collection::<agrocore_domain::entities::order::Order>("orders");
        create_active_updated_index(&order_collection).await?;

        // Task Data
        let task_collection =
            self.collection::<agrocore_domain::entities::task::TaskData>("task_data");
        create_active_updated_index(&task_collection).await?;

        // Worker Task Status
        let worker_task_status_collection =
            self.collection::<agrocore_domain::entities::worker_task_status::WorkerTaskStatus>(
                "worker_task_status",
            );
        worker_task_status_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "task_id": 1, "worker_id": 1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&worker_task_status_collection).await?;

        // Compliance
        let checklist_collection = self
            .collection::<agrocore_domain::entities::compliance::ComplianceChecklist>(
                "compliance_checklists",
            );
        create_active_updated_index(&checklist_collection).await?;

        // Worker
        let worker_collection =
            self.collection::<agrocore_domain::entities::workforce::Worker>("workers");
        worker_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "user_id": 1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&worker_collection).await?;

        // Spatial
        let spatial_collection =
            self.collection::<agrocore_domain::entities::spatial::SpatialObject>("spatial_objects");
        create_active_updated_index(&spatial_collection).await?;

        Ok(())
    }

    pub fn db(&self) -> mongodb::Database {
        self.client.database(&self.db_name)
    }

    pub fn collection<T: Send + Sync>(&self, name: &str) -> Collection<T> {
        self.db().collection(name)
    }

    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        Arc::new(SiteRepo::new(self.collection("sites")))
    }

    pub fn order_repo(&self) -> OrderRepo {
        OrderRepo::new(self.collection("orders"))
    }

    pub fn user_repo(&self) -> UserRepo {
        UserRepo::new(self.collection("users"))
    }

    pub fn tenant_repo(&self) -> TenantRepo {
        TenantRepo::new(self.collection("tenants"))
    }

    pub fn task_data_repo(&self) -> TaskDataRepo {
        TaskDataRepo::new(self.collection("task_data"))
    }

    pub fn audit_log_repo(&self) -> AuditLogRepo {
        AuditLogRepo::new(self.collection("audit_logs"))
    }

    pub fn compliance_checklist_repo(&self) -> ComplianceChecklistRepo {
        ComplianceChecklistRepo::new(self.collection("compliance_checklists"))
    }

    pub fn fertilizer_record_repo(&self) -> FertilizerRecordRepo {
        FertilizerRecordRepo::new(self.collection("fertilizer_records"))
    }

    pub fn plant_protection_record_repo(&self) -> PlantProtectionRecordRepo {
        PlantProtectionRecordRepo::new(self.collection("plant_protection_records"))
    }

    pub fn applicator_license_repo(&self) -> ApplicatorLicenseRepo {
        ApplicatorLicenseRepo::new(self.collection("applicator_licenses"))
    }

    pub fn olive_grove_repo(&self) -> OliveGroveRepo {
        OliveGroveRepo::new(self.collection("olive_groves"))
    }

    pub fn olive_oil_record_repo(&self) -> OliveOilRecordRepo {
        OliveOilRecordRepo::new(self.collection("olive_oil_records"))
    }

    pub fn vineyard_repo(&self) -> VineyardRepo {
        VineyardRepo::new(self.collection("vineyards"))
    }

    pub fn kelter_delivery_repo(&self) -> KelterDeliveryRepo {
        KelterDeliveryRepo::new(self.collection("kelter_deliveries"))
    }

    pub fn water_source_repo(&self) -> WaterSourceRepo {
        WaterSourceRepo::new(self.collection("water_sources"))
    }

    pub fn water_usage_repo(&self) -> WaterUsageRepo {
        WaterUsageRepo::new(self.collection("water_usages"))
    }

    pub fn water_quota_repo(&self) -> WaterQuotaRepo {
        WaterQuotaRepo::new(self.collection("water_quotas"))
    }

    pub fn worker_repo(&self) -> WorkerRepo {
        WorkerRepo::new(self.collection("workers"))
    }

    pub fn work_log_repo(&self) -> WorkLogRepo {
        WorkLogRepo::new(self.collection("work_logs"))
    }

    pub fn worker_location_repo(&self) -> WorkerLocationRepo {
        WorkerLocationRepo::new(self.collection("worker_locations"))
    }

    pub fn worker_task_status_repo(&self) -> WorkerTaskStatusRepo {
        WorkerTaskStatusRepo::new(self.collection("worker_task_status"))
    }

    pub fn weather_station_repo(&self) -> WeatherStationRepo {
        WeatherStationRepo::new(self.collection("weather_stations"))
    }

    pub fn weather_data_repo(&self) -> WeatherDataRepo {
        WeatherDataRepo::new(self.collection("weather_data"))
    }

    pub fn phenology_record_repo(&self) -> PhenologyRecordRepo {
        PhenologyRecordRepo::new(self.collection("phenology_records"))
    }

    pub fn pac_application_repo(&self) -> PACApplicationRepo {
        PACApplicationRepo::new(self.collection("pac_applications"))
    }

    pub fn cost_center_repo(&self) -> CostCenterRepo {
        CostCenterRepo::new(self.collection("cost_centers"))
    }

    pub fn financial_record_repo(&self) -> FinancialRecordRepo {
        FinancialRecordRepo::new(self.collection("financial_records"))
    }

    pub fn equipment_repo(&self) -> EquipmentRepo {
        EquipmentRepo::new(self.collection("equipment"))
    }

    pub fn cold_chain_log_repo(&self) -> ColdChainLogRepo {
        ColdChainLogRepo::new(self.collection("cold_chain_logs"))
    }

    pub fn harvest_season_repo(&self) -> HarvestSeasonRepo {
        HarvestSeasonRepo::new(self.collection("harvest_seasons"))
    }

    pub fn harvest_lot_repo(&self) -> HarvestLotRepo {
        HarvestLotRepo::new(self.collection("harvest_lots"))
    }

    pub fn harvest_delivery_repo(&self) -> HarvestDeliveryRepo {
        HarvestDeliveryRepo::new(self.collection("harvest_deliveries"))
    }

    pub fn animal_repo(&self) -> AnimalRepo {
        AnimalRepo::new(self.collection("animals"))
    }

    pub fn spatial_object_repo(&self) -> SpatialObjectRepo {
        SpatialObjectRepo::new(self.collection("spatial_objects"))
    }
}
