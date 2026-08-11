use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::compliance::{
    AuditAction, AuditLog, ChecklistType, ComplianceChecklist, ComplianceStatus,
};
use agrocore_domain::entities::fertilizer::FertilizerRecord;
use agrocore_domain::entities::plant_protection::{
    ApplicatorLicense, LicenseType, PlantProtectionRecord,
};
use agrocore_domain::repositories::{
    MockAuditLogRepo, MockComplianceChecklistRepo, MockFertilizerRecordRepo,
    MockPlantProtectionRecordRepo, PaginatedResponse,
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
async fn test_list_compliance_checklists() {
    let mut checklist_repo = MockComplianceChecklistRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let checklist = ComplianceChecklist {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        checklist_type: ChecklistType::GAP,
        status: ComplianceStatus::Pending,
        items: vec![],
        due_date: None,
        completed_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    checklist_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![checklist.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.compliance_checklist_repo = Some(Arc::new(checklist_repo));

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
        .uri("/api/v1/compliance/checklists")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_audit_logs() {
    let mut audit_repo = MockAuditLogRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let log = AuditLog {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        user_id,
        action: AuditAction::Created,
        entity_type: "Test".into(),
        entity_id: Uuid::new_v4(),
        old_value: None,
        new_value: None,
        ip_address: None,
        created_at: Utc::now(),
    };

    audit_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![log.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.audit_log_repo = Some(Arc::new(audit_repo));

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

    let token = signed_token(&user_id.to_string(), &tenant_id.to_string(), vec!["Admin"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/compliance/audit-logs")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_applicator_license_crud() {
    let mut plant_repo = MockPlantProtectionRecordRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let license = ApplicatorLicense {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        user_id,
        license_type: LicenseType::Professional,
        license_number: "LIC-123".into(),
        issued_by: "Test Auth".into(),
        valid_from: Utc::now(),
        valid_until: Utc::now(),
        is_active: true,
        created_at: Utc::now(),
    };

    plant_repo
        .expect_find_all_applicator_licenses()
        .returning(move |_| Box::pin(ready(Ok(vec![license.clone()]))));

    let mut mock_db = MockDatabase::default();
    mock_db.plant_protection_record_repo = Some(Arc::new(plant_repo));

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
        .uri("/api/v1/compliance/applicator-licenses")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_plant_protection_record_list() {
    let mut plant_repo = MockPlantProtectionRecordRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let record = PlantProtectionRecord {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        order_id: None,
        product_name: "Test Pesticide".into(),
        active_substance: "Substance X".into(),
        dosage_per_ha: 2.5,
        total_quantity: 10.0,
        area_ha: 4.0,
        application_date: Utc::now(),
        pre_harvest_days: 14,
        re_entry_days: 2,
        weather_conditions: Some("Sunny".into()),
        applicator_license: Some("LIC-123".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    plant_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![record.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.plant_protection_record_repo = Some(Arc::new(plant_repo));

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
        .uri("/api/v1/compliance/plant-protection")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_fertilizer_record_list() {
    let mut fert_repo = MockFertilizerRecordRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let record = FertilizerRecord {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        order_id: None,
        product_name: "Test Fertilizer".into(),
        nutrient_n: 10.0,
        nutrient_p: 5.0,
        nutrient_k: 5.0,
        quantity_kg: 100.0,
        area_ha: 1.0,
        application_date: Utc::now(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    fert_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![record.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.fertilizer_record_repo = Some(Arc::new(fert_repo));

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
        .uri("/api/v1/compliance/fertilizer")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
