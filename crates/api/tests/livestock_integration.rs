mod common;

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::handlers::configure;
use agrocore_domain::entities::livestock::{Animal, TreatmentRecord};
use agrocore_domain::repositories::{MockAnimalRepository, PaginatedResponse};
use chrono::Utc;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[actix_web::test]
async fn test_list_animals() {
    let mut animal_repo = MockAnimalRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // The current `Animal` shape: `species` and `status` are strings, `tenant_id`
    // is a bare `Uuid`, and `birth_date` is a `NaiveDate`. The fixture this
    // replaces carried `weight_kg`, `last_weight_date`, `group_id`, `treatments`
    // and `grazing_history` — none of which exist on the entity — so this target
    // had not compiled since those were removed.
    let animal = Animal {
        id: Uuid::new_v4(),
        tenant_id,
        livestock_type: "cattle".into(),
        species: "Angus".into(),
        breed: Some("Angus".into()),
        identifier: "COW-001".into(),
        plot_id: None,
        birth_date: chrono::NaiveDate::from_ymd_opt(2021, 3, 14),
        gender: Some("F".into()),
        status: "active".into(),
        current_site_id: None,
        mother_id: None,
        father_id: None,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
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

    let mock_db = crate::common::db_with(|db| db.animal_repo = Some(Arc::new(animal_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Manager"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/livestock/animals")
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
        medication: "Vax-1".into(),
        dosage: Some("5ml".into()),
        veterinarian: Some("Dr. Smith".into()),
        withdrawal_days: Some(0),
        notes: None,
        created_at: Some(Utc::now()),
    };

    animal_repo
        .expect_add_treatment()
        .returning(move |_, _, _| Box::pin(ready(Ok(true))));

    let mock_db = crate::common::db_with(|db| db.animal_repo = Some(Arc::new(animal_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Manager"]);
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/livestock/animals/{}/treatments",
            animal_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&treatment)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
