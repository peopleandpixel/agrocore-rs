//! PostgreSQL Integration Tests using testcontainers
//!
//! These tests spin up a real PostgreSQL instance with PostGIS
//! and test the actual repository implementations.

mod common;
use agrocore_domain::TenantId;
use agrocore_domain::entities::worker_task_status::{
    CreateWorkerTaskStatusDto, WorkerTaskStatusType,
};
use agrocore_domain::entities::workforce::ContractType;
use agrocore_domain::entities::workforce::{CreateWorkerDto, CreateWorkerLocationDto};
use agrocore_domain::repositories::Pagination;
use common::PostgresTestFixture;
use uuid::Uuid;

#[tokio::test]
#[ignore]
async fn test_worker_repository_crud() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");
    let tid = TenantId(fixture.create_test_tenant().await);
    let repo = fixture.database.worker_repo();

    let dto = CreateWorkerDto {
        user_id: Uuid::new_v4(),
        contract_type: ContractType::Permanent,
        language: Some("de".into()),
        skills: None,
        emergency_contact: None,
        nationality: None,
    };

    let worker = repo
        .create(tid, dto.clone(), Uuid::new_v4())
        .await
        .expect("Failed to create worker");
    assert_eq!(worker.language.as_deref(), Some("de"));

    let found = repo
        .find_by_id(tid, worker.id)
        .await
        .expect("Failed to find worker");
    assert!(found.is_some());

    let all = repo
        .find_all(tid, Pagination::default())
        .await
        .expect("Failed to find all workers");
    assert_eq!(all.total, 1);
}

#[tokio::test]
#[ignore]
async fn test_worker_location_repository() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");
    let tid = TenantId(fixture.create_test_tenant().await);
    let repo = fixture.database.worker_location_repo();

    let worker_id = Uuid::new_v4();
    let dto = CreateWorkerLocationDto {
        worker_id,
        lat: 52.5200,
        lng: 13.4050,
        current_task_id: None,
        timestamp: chrono::Utc::now(),
    };

    let loc = repo
        .create(tid, dto)
        .await
        .expect("Failed to report location");
    assert_eq!(loc.lat, 52.5200);

    let latest = repo
        .find_latest_by_worker(tid, worker_id)
        .await
        .expect("Failed to find latest");
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().lat, 52.5200);
}

#[tokio::test]
#[ignore]
async fn test_worker_task_status_repository() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");
    let tid = TenantId(fixture.create_test_tenant().await);
    let repo = fixture.database.worker_task_status_repo();

    let task_id = Uuid::new_v4();
    let worker_id = Uuid::new_v4();

    let dto = CreateWorkerTaskStatusDto {
        task_id,
        worker_id,
        tenant_id: tid,
    };

    let status = repo
        .create(tid, dto)
        .await
        .expect("Failed to create status");
    assert_eq!(status.status, WorkerTaskStatusType::New);

    repo.update_status(tid, task_id, worker_id, WorkerTaskStatusType::Started)
        .await
        .expect("Failed to update status");

    let found = repo
        .find_by_task_and_worker(tid, task_id, worker_id)
        .await
        .expect("Failed to find");
    assert!(found.is_some());
    assert_eq!(found.unwrap().status, WorkerTaskStatusType::Started);
}
