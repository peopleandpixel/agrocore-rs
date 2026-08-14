use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::worker_task_status::{WorkerTaskStatus, WorkerTaskStatusType};
use agrocore_domain::entities::workforce::{
    ContractType, CreateWorkerDto, ReportLocationDto, Worker, WorkerLocation,
};
use agrocore_domain::repositories::{
    MockOrderRepository, MockSpatialObjectRepository, MockWorkerLocationRepo, MockWorkerRepo,
    MockWorkerTaskStatusRepository, PaginatedResponse,
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
async fn test_list_workers_with_pagination_and_tenant_filtering() {
    let mut worker_repo = MockWorkerRepo::new();

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let worker = Worker {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        user_id,
        contract_type: ContractType::Permanent,
        language: Some("en".into()),
        skills: vec![],
        certifications: vec![],
        emergency_contact: None,
        nationality: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    worker_repo
        .expect_find_all_visible()
        .withf(move |tid, _p, uid, _roles| tid.0 == tenant_id && *uid == user_id)
        .returning(move |_, p, _, _| {
            Box::pin(ready(Ok(PaginatedResponse {
                data: vec![worker.clone()],
                total: 1,
                page: p.page.unwrap_or(0),
                per_page: p.per_page.unwrap_or(20),
                total_pages: 1,
            })))
        });

    let mut mock_db = MockDatabase::default();
    mock_db.worker_repo = Some(Arc::new(worker_repo));

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

    let token = signed_token(&user_id.to_string(), &tenant_id.to_string(), vec!["Admin"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/workforce/workers")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: agrocore_api::dto::PaginatedResponseDto<Worker> = test::read_body_json(resp).await;
    assert_eq!(body.total, 1);
    assert_eq!(body.data[0].tenant_id.0, tenant_id);
}

#[actix_web::test]
async fn test_create_worker_authorization() {
    let mut worker_repo = MockWorkerRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let dto = CreateWorkerDto {
        user_id: Uuid::new_v4(),
        contract_type: ContractType::Permanent,
        language: Some("de".into()),
        skills: None,
        emergency_contact: None,
        nationality: None,
    };

    let worker = Worker {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        user_id: dto.user_id,
        contract_type: dto.contract_type.clone(),
        language: dto.language.clone(),
        skills: vec![],
        certifications: vec![],
        emergency_contact: None,
        nationality: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    worker_repo
        .expect_create()
        .returning(move |_, _, _| Box::pin(ready(Ok(worker.clone()))));

    let mut mock_db = MockDatabase::default();
    mock_db.worker_repo = Some(Arc::new(worker_repo));

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

    // Admin can create
    let token = signed_token(&user_id.to_string(), &tenant_id.to_string(), vec!["Admin"]);
    let req = test::TestRequest::post()
        .uri("/api/v1/workforce/workers")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[actix_web::test]
async fn test_report_location() {
    let mut location_repo = MockWorkerLocationRepo::new();
    let mut spatial_repo = MockSpatialObjectRepository::new();
    let mut order_repo = MockOrderRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let dto = ReportLocationDto {
        lat: 48.1351,
        lng: 11.5820,
        current_task_id: None,
    };

    location_repo
        .expect_find_latest_by_worker()
        .returning(|_, _| Box::pin(ready(Ok(None))));

    location_repo.expect_create().returning(move |tid, d| {
        Box::pin(ready(Ok(WorkerLocation {
            id: Uuid::new_v4(),
            tenant_id: tid,
            worker_id: d.worker_id,
            lat: d.lat,
            lng: d.lng,
            timestamp: d.timestamp,
        })))
    });

    spatial_repo
        .expect_find_containing_point()
        .returning(|_, _, _| Box::pin(ready(Ok(vec![]))));

    order_repo
        .expect_find_assigned_to_worker()
        .returning(|_, _| Box::pin(ready(Ok(vec![]))));

    let mut mock_db = MockDatabase::default();
    mock_db.worker_location_repo = Some(Arc::new(location_repo));
    mock_db.spatial_object_repo = Some(Arc::new(spatial_repo));
    mock_db.order_repo = Some(Arc::new(order_repo));

    // Mock worker repo to find current worker
    let mut worker_repo = MockWorkerRepo::new();
    worker_repo.expect_find_by_user_id().returning(|_, _| {
        Box::pin(ready(Ok(Some(Worker {
            id: Uuid::new_v4(),
            tenant_id: TenantId(Uuid::new_v4()),
            user_id: Uuid::new_v4(),
            contract_type: ContractType::Permanent,
            language: None,
            skills: vec![],
            certifications: vec![],
            emergency_contact: None,
            nationality: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))))
    });
    mock_db.worker_repo = Some(Arc::new(worker_repo));

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

    let token = signed_token(&user_id.to_string(), &tenant_id.to_string(), vec!["Worker"]);
    let req = test::TestRequest::post()
        .uri("/api/v1/workforce/locations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[actix_web::test]
async fn test_worker_task_status_lifecycle() {
    let mut status_repo = MockWorkerTaskStatusRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let worker_id = Uuid::new_v4();

    status_repo.expect_create().returning(move |tid, d| {
        Box::pin(ready(Ok(WorkerTaskStatus {
            task_id: d.task_id,
            worker_id: d.worker_id,
            tenant_id: tid,
            status: WorkerTaskStatusType::New,
            started_at: None,
            paused_at: None,
            resumed_at: None,
            stopped_at: None,
            done_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })))
    });

    status_repo
        .expect_update_status()
        .returning(move |tid, tid_task, wid, s| {
            Box::pin(ready(Ok(Some(WorkerTaskStatus {
                task_id: tid_task,
                worker_id: wid,
                tenant_id: tid,
                status: s,
                started_at: Some(Utc::now()),
                paused_at: None,
                resumed_at: None,
                stopped_at: None,
                done_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))))
        });

    let mut mock_db = MockDatabase::default();
    mock_db.worker_task_status_repo = Some(Arc::new(status_repo));

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

    // POST create status
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/workforce/tasks/{}/status", task_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&serde_json::json!({
            "task_id": task_id,
            "worker_id": worker_id,
            "tenant_id": tenant_id,
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // PUT update status
    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/workforce/tasks/{}/status/{}",
            task_id, worker_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&serde_json::json!({
            "status": "started"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
