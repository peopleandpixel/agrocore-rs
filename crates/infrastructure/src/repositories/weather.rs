use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::weather::{
    WeatherStation, WeatherData, PhenologyRecord,
    CreateWeatherStationDto, CreateWeatherDataDto, CreatePhenologyRecordDto
};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct WeatherStationRepo { base: MongoRepository<WeatherStation> }
impl WeatherStationRepo {
    pub fn new(c: Collection<WeatherStation>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherStation>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherStation>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateWeatherStationDto) -> Fut<WeatherStation> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let ws = WeatherStation {
                id: Uuid::new_v4(),
                tenant_id: tid,
                label: dto.label,
                station_type: dto.station_type,
                location: dto.location,
                manufacturer: dto.manufacturer,
                model: dto.model,
                serial_number: dto.serial_number,
                api_key_config: dto.api_key_config,
                is_active: true,
                sensor_metadata: None,
                firmware_version: None,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&ws).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(ws)
        })
    }
}

#[derive(Clone)]
pub struct WeatherDataRepo { base: MongoRepository<WeatherData> }
impl WeatherDataRepo {
    pub fn new(c: Collection<WeatherData>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WeatherData>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherData>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! { "tenant_id": tid.to_string() };
            paginate(&c, f, p, Some(doc! { "timestamp": -1 })).await
        })
    }
    pub fn find_by_station(&self, tid: TenantId, station_id: Uuid, p: Pagination) -> RepositoryFuture<PaginatedResponse<WeatherData>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! { "tenant_id": tid.to_string(), "station_id": station_id.to_string() };
            paginate(&c, f, p, Some(doc! { "timestamp": -1 })).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateWeatherDataDto) -> Fut<WeatherData> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let wd = WeatherData {
                id: Uuid::new_v4(),
                station_id: dto.station_id,
                tenant_id: tid,
                timestamp: dto.timestamp,
                temperature_c: dto.temperature_c,
                humidity_percent: dto.humidity_percent,
                precipitation_mm: dto.precipitation_mm,
                wind_speed_kmh: dto.wind_speed_kmh,
                wind_direction_deg: dto.wind_direction_deg,
                solar_radiation_wm2: dto.solar_radiation_wm2,
                pressure_hpa: dto.pressure_hpa,
                soil_temperature_c: None,
                soil_moisture_percent: None,
                leaf_wetness: None,
            };
            c.insert_one(&wd).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(wd)
        })
    }
}

#[derive(Clone)]
pub struct PhenologyRecordRepo { base: MongoRepository<PhenologyRecord> }
impl PhenologyRecordRepo {
    pub fn new(c: Collection<PhenologyRecord>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PhenologyRecord>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! { "tenant_id": tid.to_string() };
            paginate(&c, f, p, Some(doc! { "observation_date": -1 })).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreatePhenologyRecordDto) -> Fut<PhenologyRecord> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let pr = PhenologyRecord {
                id: Uuid::new_v4(),
                tenant_id: tid,
                site_id: dto.site_id,
                observation_date: dto.observation_date,
                stage: dto.stage,
                forecast_next_stage_date: None,
                notes: dto.notes,
                photo_url: dto.photo_url,
                observer_id: None,
            };
            c.insert_one(&pr).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(pr)
        })
    }
}
