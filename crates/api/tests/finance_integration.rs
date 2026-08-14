use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::finance::{
    CostCenter, CostCenterType, FinancialRecord, FinancialRecordType, PACApplication, PACStatus,
};
use agrocore_domain::repositories::{
    MockCostCenterRepo, MockFinancialRecordRepo, MockPACApplicationRepo, PaginatedResponse,
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
async fn test_list_pac_applications() {
    let mut pac_repo = MockPACApplicationRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let app_record = PACApplication {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        year: 2026,
        application_number: "PAC-2026-001".into(),
        status: PACStatus::Draft,
        total_eligible_area: 50.5,
        submitted_at: None,
        eco_schemes: vec![],
        documents_urls: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    pac_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![app_record.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.pac_application_repo = Some(Arc::new(pac_repo));

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
        .uri("/api/v1/finance/pac-applications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_cost_centers() {
    let mut cc_repo = MockCostCenterRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let cc = CostCenter {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        label: "Site A".into(),
        code: "SITE-A".into(),
        cost_center_type: CostCenterType::Site,
        reference_id: Some(Uuid::new_v4()),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    cc_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![cc.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.cost_center_repo = Some(Arc::new(cc_repo));

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
        .uri("/api/v1/finance/cost-centers")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_financial_records() {
    let mut fr_repo = MockFinancialRecordRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let record = FinancialRecord {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        cost_center_id: Uuid::new_v4(),
        date: Utc::now(),
        amount: 1250.0,
        currency: "EUR".into(),
        record_type: FinancialRecordType::Expense,
        category: "Fertilizer".into(),
        description: "Purchase of NPK".into(),
        reference_id: None,
        created_at: Utc::now(),
    };

    fr_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![record.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.financial_record_repo = Some(Arc::new(fr_repo));

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
        .uri("/api/v1/finance/financial-records")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
