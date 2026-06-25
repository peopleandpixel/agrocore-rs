mod repositories;

use std::sync::Arc;
use agrocore_domain::repositories::SiteRepository;

pub use repositories::{SiteRepo, OrderRepo, UserRepo, TaskDataRepo};
pub use repositories::{AuditLogRepo, ComplianceChecklistRepo, FertilizerRecordRepo, PlantProtectionRecordRepo, ApplicatorLicenseRepo};
pub use repositories::{OliveGroveRepo, OliveOilRecordRepo};
pub use repositories::{VineyardRepo, KelterDeliveryRepo};
pub use repositories::{WaterSourceRepo, WaterUsageRepo, WaterQuotaRepo};
pub use repositories::{WeatherStationRepo, WeatherDataRepo, PhenologyRecordRepo};
pub use repositories::{WorkerRepo, WorkLogRepo};
pub use repositories::{PACApplicationRepo, CostCenterRepo, FinancialRecordRepo};
pub use repositories::{HarvestSeasonRepo, HarvestLotRepo, HarvestDeliveryRepo, ColdChainLogRepo};

use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};
use mongodb::bson::doc;

#[derive(Clone)]
pub struct Database {
    client: Client,
    db_name: String,
}

impl Database {
    pub async fn connect(uri: &str, db_name: &str) -> anyhow::Result<Self> {
        let mut opts = ClientOptions::parse(uri).await?;
        opts.max_pool_size = Some(100);
        opts.min_pool_size = Some(10);
        opts.max_idle_time = Some(std::time::Duration::from_secs(300));
        opts.connect_timeout = Some(std::time::Duration::from_secs(10));
        opts.server_selection_timeout = Some(std::time::Duration::from_secs(5));
        let client = Client::with_options(opts)?;
        let db = Self { client, db_name: db_name.to_string() };
        db.create_indexes().await?;
        Ok(db)
    }

    async fn create_indexes(&self) -> anyhow::Result<()> {
        // Users: unique email
        let user_collection = self.collection::<agrocore_domain::entities::user::User>("users");
        user_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ).await?;

        // Sites: tenant_id + label
        let site_collection = self.collection::<agrocore_domain::entities::site::Site>("sites");
        site_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "label": 1 })
                .build(),
        ).await?;

        // Orders: tenant_id + status + deadline_date
        let order_collection = self.collection::<agrocore_domain::entities::order::Order>("orders");
        order_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "status": 1, "deadline_date": 1 })
                .build(),
        ).await?;

        // Task Data: tenant_id + worker_id + started_at
        let task_collection = self.collection::<agrocore_domain::entities::task::TaskData>("task_data");
        task_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "worker_id": 1, "started_at": -1 })
                .build(),
        ).await?;

        // Sites: 2dsphere index for geo-queries
        site_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "center": "2dsphere" })
                .build(),
        ).await?;

        // Compliance: tenant_id + site_id
        let checklist_collection = self.collection::<agrocore_domain::entities::compliance::ComplianceChecklist>("compliance_checklists");
        checklist_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "site_id": 1 })
                .build(),
        ).await?;

        // Fertilizer: tenant_id + application_date
        let fertilizer_collection = self.collection::<agrocore_domain::entities::compliance::FertilizerRecord>("fertilizer_records");
        fertilizer_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "application_date": -1 })
                .build(),
        ).await?;

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
}
