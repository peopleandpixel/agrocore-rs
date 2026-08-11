use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::livestock::{Animal, AnimalSpecies, AnimalStatus, TreatmentRecord};
use agrocore_domain::repositories::{MockAnimalRepository, PaginatedResponse};
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
}

fn signed_token(sub: &str, tenant_id: &str, roles: Vec<&str>) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: sub.to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            exp: usize::MAX / 2,
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

#[actix_web::test]
async fn test_list_animals() {
    let mut animal_repo = MockAnimalRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let animal = Animal {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        species: AnimalSpecies::Cattle,
        breed: Some("Angus".into()),
        identifier: "COW-001".into(),
        birth_date: Some(Utc::now()),
        gender: Some("F".into()),
        status: AnimalStatus::Active,
        current_site_id: None,
        group_id: None,
        weight_kg: Some(500.0),
        last_weight_date: Some(Utc::now()),
        treatments: vec![],
        grazing_history: vec![],
        mother_id: None,
        father_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    animal_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![animal.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.animal_repo = Some(Arc::new(animal_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
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
        .uri("/api/v1/animals")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_add_treatment() {
    let mut animal_repo = MockAnimalRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let animal_id = Uuid::new_v4();

    let treatment = TreatmentRecord {
        id: Uuid::new_v4(),
        animal_id,
        date: Utc::now(),
        treatment_type: "Vaccination".into(),
        medication: Some("Vax-1".into()),
        dosage: Some("5ml".into()),
        veterinarian: Some("Dr. Smith".into()),
        withdrawal_days: Some(0),
        notes: None,
        created_at: Utc::now(),
    };

    animal_repo
        .expect_add_treatment()
        .returning(move |_, _, _| Box::pin(ready(Ok(true))));

    let mut mock_db = MockDatabase::default();
    mock_db.animal_repo = Some(Arc::new(animal_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
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
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/animals/{}/treatments", animal_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&treatment)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
