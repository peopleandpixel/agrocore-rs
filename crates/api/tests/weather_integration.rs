use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::BbchStage;
use agrocore_domain::entities::weather::{
    PhenologyRecord, WeatherData, WeatherStation, WeatherStationType,
};
use agrocore_domain::repositories::{
    MockPhenologyRecordRepo, MockWeatherDataRepo, MockWeatherStationRepo, PaginatedResponse,
};
use agrocore_infrastructure::{Database, MockDatabase};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    exp: usize,
    jti: String,
}

fn signed_token(sub: &str, tenant_id: &str, roles: Vec<&str>) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: sub.to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            exp: usize::MAX / 2,
            jti: uuid::Uuid::new_v4().to_string(),
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

#[actix_web::test]
async fn test_list_weather_stations() {
    let mut station_repo = MockWeatherStationRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let station = WeatherStation {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        label: "Station 1".into(),
        station_type: WeatherStationType::Iot,
        location: None,
        manufacturer: None,
        model: None,
        serial_number: None,
        api_key_config: None,
        is_active: true,
        sensor_metadata: None,
        firmware_version: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    station_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![station.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.weather_station_repo = Some(Arc::new(station_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(
        &user_id.to_string(),
        &tenant_id.to_string(),
        vec!["Manager"],
    );
    let req = test::TestRequest::get()
        .uri("/api/v1/weather/stations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_weather_data() {
    let mut data_repo = MockWeatherDataRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let data = WeatherData {
        id: Uuid::new_v4(),
        station_id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        timestamp: Utc::now(),
        temperature_c: Some(25.5),
        humidity_percent: Some(60.0),
        precipitation_mm: Some(0.0),
        wind_speed_kmh: Some(10.0),
        wind_direction_deg: Some(180),
        solar_radiation_wm2: None,
        pressure_hpa: None,
        soil_temperature_c: None,
        soil_moisture_percent: None,
        leaf_wetness: None,
        created_at: Utc::now(),
    };

    data_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![data.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.weather_data_repo = Some(Arc::new(data_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(
        &user_id.to_string(),
        &tenant_id.to_string(),
        vec!["Manager"],
    );
    let req = test::TestRequest::get()
        .uri("/api/v1/weather/data")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_phenology() {
    let mut pheno_repo = MockPhenologyRecordRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let record = PhenologyRecord {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        observation_date: Utc::now(),
        stage: BbchStage::FloweringBegins,
        forecast_next_stage_date: None,
        notes: None,
        photo_url: None,
        observer_id: None,
        created_at: Utc::now(),
    };

    pheno_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![record.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.phenology_record_repo = Some(Arc::new(pheno_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(
        &user_id.to_string(),
        &tenant_id.to_string(),
        vec!["Manager"],
    );
    let req = test::TestRequest::get()
        .uri("/api/v1/weather/phenology")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
