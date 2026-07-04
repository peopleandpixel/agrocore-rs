mod repositories;

use agrocore_domain::repositories::SiteRepository;
use std::sync::Arc;

pub use repositories::AnimalRepo;
pub use repositories::EquipmentRepo;
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
pub use repositories::{WorkLogRepo, WorkerLocationRepo, WorkerRepo};

use mongodb::bson::doc;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};

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

        // Orders: tenant_id + status + deadline_date
        let order_collection = self.collection::<agrocore_domain::entities::order::Order>("orders");
        order_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "status": 1, "deadline_date": 1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&order_collection).await?;

        // Task Data: tenant_id + worker_id + started_at
        let task_collection =
            self.collection::<agrocore_domain::entities::task::TaskData>("task_data");
        task_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "worker_id": 1, "started_at": -1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&task_collection).await?;

        // Sites: 2dsphere index for geo-queries
        site_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "center": "2dsphere" })
                    .build(),
            )
            .await?;
        create_active_updated_index(&site_collection).await?;

        // Compliance: tenant_id + site_id
        let checklist_collection = self
            .collection::<agrocore_domain::entities::compliance::ComplianceChecklist>(
                "compliance_checklists",
            );
        checklist_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "site_id": 1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&checklist_collection).await?;

        // Fertilizer: tenant_id + application_date
        let fertilizer_collection = self
            .collection::<agrocore_domain::entities::compliance::FertilizerRecord>(
                "fertilizer_records",
            );
        fertilizer_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "application_date": -1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&fertilizer_collection).await?;

        let plant_protection_collection =
            self.collection::<agrocore_domain::entities::plant_protection::PlantProtectionRecord>(
                "plant_protection_records",
            );
        create_active_updated_index(&plant_protection_collection).await?;

        let applicator_license_collection =
            self.collection::<agrocore_domain::entities::plant_protection::ApplicatorLicense>(
                "applicator_licenses",
            );
        create_active_updated_index(&applicator_license_collection).await?;

        let olive_grove_collection =
            self.collection::<agrocore_domain::entities::olive::OliveGrove>("olive_groves");
        create_active_updated_index(&olive_grove_collection).await?;

        let olive_oil_record_collection = self
            .collection::<agrocore_domain::entities::olive::OliveOilRecord>("olive_oil_records");
        create_active_updated_index(&olive_oil_record_collection).await?;

        let vineyard_collection =
            self.collection::<agrocore_domain::entities::vineyard::Vineyard>("vineyards");
        create_active_updated_index(&vineyard_collection).await?;

        let kelter_delivery_collection = self
            .collection::<agrocore_domain::entities::vineyard::KelterDelivery>("kelter_deliveries");
        create_active_updated_index(&kelter_delivery_collection).await?;

        let water_source_collection =
            self.collection::<agrocore_domain::entities::water::WaterSource>("water_sources");
        create_active_updated_index(&water_source_collection).await?;

        let water_usage_collection =
            self.collection::<agrocore_domain::entities::water::WaterUsage>("water_usages");
        create_active_updated_index(&water_usage_collection).await?;

        let water_quota_collection =
            self.collection::<agrocore_domain::entities::water::WaterQuota>("water_quotas");
        create_active_updated_index(&water_quota_collection).await?;

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

        let work_log_collection =
            self.collection::<agrocore_domain::entities::workforce::WorkLog>("work_logs");
        work_log_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "worker_id": 1, "date": -1 })
                    .build(),
            )
            .await?;
        create_active_updated_index(&work_log_collection).await?;

        let worker_location_collection = self
            .collection::<agrocore_domain::entities::workforce::WorkerLocation>("worker_locations");
        create_active_updated_index(&worker_location_collection).await?;

        let weather_station_collection = self
            .collection::<agrocore_domain::entities::weather::WeatherStation>("weather_stations");
        create_active_updated_index(&weather_station_collection).await?;

        let weather_data_collection =
            self.collection::<agrocore_domain::entities::weather::WeatherData>("weather_data");
        create_active_updated_index(&weather_data_collection).await?;

        let phenology_record_collection = self
            .collection::<agrocore_domain::entities::weather::PhenologyRecord>("phenology_records");
        create_active_updated_index(&phenology_record_collection).await?;

        let pac_application_collection = self
            .collection::<agrocore_domain::entities::finance::PACApplication>("pac_applications");
        create_active_updated_index(&pac_application_collection).await?;

        let cost_center_collection =
            self.collection::<agrocore_domain::entities::finance::CostCenter>("cost_centers");
        create_active_updated_index(&cost_center_collection).await?;

        let financial_record_collection = self
            .collection::<agrocore_domain::entities::finance::FinancialRecord>("financial_records");
        create_active_updated_index(&financial_record_collection).await?;

        let equipment_collection =
            self.collection::<agrocore_domain::entities::equipment::Equipment>("equipment");
        create_active_updated_index(&equipment_collection).await?;

        let cold_chain_log_collection =
            self.collection::<agrocore_domain::entities::harvest::ColdChainLog>("cold_chain_logs");
        create_active_updated_index(&cold_chain_log_collection).await?;

        let harvest_season_collection =
            self.collection::<agrocore_domain::entities::harvest::HarvestSeason>("harvest_seasons");
        create_active_updated_index(&harvest_season_collection).await?;

        let harvest_lot_collection =
            self.collection::<agrocore_domain::entities::harvest::HarvestLot>("harvest_lots");
        create_active_updated_index(&harvest_lot_collection).await?;

        let harvest_delivery_collection = self
            .collection::<agrocore_domain::entities::harvest::HarvestDelivery>(
                "harvest_deliveries",
            );
        create_active_updated_index(&harvest_delivery_collection).await?;

        let animal_collection =
            self.collection::<agrocore_domain::entities::livestock::Animal>("animals");
        create_active_updated_index(&animal_collection).await?;

        let spatial_collection =
            self.collection::<agrocore_domain::entities::spatial::SpatialObject>("spatial_objects");
        spatial_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "site_id": 1, "object_type": 1 })
                    .build(),
            )
            .await?;
        spatial_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "tenant_id": 1, "parent_id": 1 })
                    .build(),
            )
            .await?;
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
