//! Database Setup Integration Tests
//!
//! These tests verify the database setup works correctly with a real PostgreSQL instance

use agrocore_domain::TenantId;
use agrocore_domain::entities::site::CreateSiteDto;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_shared::Pagination;
use chrono::Utc;
use uuid::Uuid;

mod common;
use common::PostgresTestFixture;

/// Test that database migrations apply cleanly and tables exist
#[tokio::test]
#[ignore]
async fn test_database_migrations_applied() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");

    // Verify key tables exist
    let tables = vec![
        "tenants",
        "sites",
        "users",
        "orders",
        "workers",
        "equipment",
        "customers",
        "weather_data",
        "weather_stations",
        "animals",
        "grazing_records",
        "treatment_records",
        "harvest_seasons",
        "harvest_lots",
        "harvest_deliveries",
        "varieties",
        "breeds",
        "spatial_objects",
        "spatial_properties",
    ];

    for table in tables {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)",
        )
        .bind(table)
        .fetch_one(&fixture.pool)
        .await
        .expect("Failed to check table existence");

        assert!(exists, "Table {} should exist after migrations", table);
    }
}

/// Test that PostGIS extension is enabled
#[tokio::test]
#[ignore]
async fn test_postgis_extension_enabled() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");

    let has_postgis: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis')")
            .fetch_one(&fixture.pool)
            .await
            .expect("Failed to check PostGIS extension");

    assert!(has_postgis, "PostGIS extension should be enabled");
}

/// Test that uuid-ossp extension is enabled
#[tokio::test]
#[ignore]
async fn test_uuid_ossp_extension_enabled() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");

    let has_uuid_ossp: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'uuid-ossp')",
    )
    .fetch_one(&fixture.pool)
    .await
    .expect("Failed to check uuid-ossp extension");

    assert!(has_uuid_ossp, "uuid-ossp extension should be enabled");
}

/// Test tenant creation and isolation
#[tokio::test]
#[ignore]
async fn test_tenant_creation_and_isolation() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");

    // Create two tenants
    let tenant1_id = fixture.create_test_tenant().await;
    let tenant2_id = fixture.create_test_tenant().await;

    assert_ne!(tenant1_id, tenant2_id);

    // Verify tenants are isolated
    let count1: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sites WHERE tenant_id = $1")
        .bind(tenant1_id)
        .fetch_one(&fixture.pool)
        .await
        .expect("Failed to count tenant1 sites");

    let count2: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sites WHERE tenant_id = $1")
        .bind(tenant2_id)
        .fetch_one(&fixture.pool)
        .await
        .expect("Failed to count tenant2 sites");

    assert_eq!(count1, 0);
    assert_eq!(count2, 0);
}

/// Test site CRUD operations with database
#[tokio::test]
#[ignore]
async fn test_site_crud_operations() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");

    let tenant_id = TenantId(fixture.create_test_tenant().await);
    let repo = fixture.database.site_repo();

    // Create site
    let create_dto = CreateSiteDto {
        label: "Test Vineyard".to_string(),
        site_type: SiteType::Vineyard,
        crop_type: CropType::Grape,
        variety: Some("Cabernet Sauvignon".to_string()),
        area: 10.5,
        gross_area: Some(12.0),
        plots: None,
        row_config: None,
        bbch_stage: None,
        planted_date: Some(Utc::now()),
        cleared_date: None,
        soil_type: Some("Clay".to_string()),
        slope: Some(5.0),
        slope_facing: Some("South".to_string()),
        altitude: Some(150.0),
        organic: Some(false),
        center: None,
        boundary: None,
        properties: None,
    };

    let site = repo
        .create(tenant_id, create_dto, Uuid::new_v4())
        .await
        .expect("Failed to create site");

    assert_eq!(site.label, "Test Vineyard");
    assert_eq!(site.area, 10.5);

    // Find by ID
    let found = repo
        .find_by_id(tenant_id, site.id)
        .await
        .expect("Failed to find site");
    assert!(found.is_some());
    assert_eq!(found.unwrap().label, "Test Vineyard");

    // Find all
    let all = repo
        .find_all(tenant_id, Pagination::default())
        .await
        .expect("Failed to find all sites");
    assert_eq!(all.total, 1);
    assert_eq!(all.data.len(), 1);

    // Delete
    let deleted = repo
        .delete(tenant_id, site.id)
        .await
        .expect("Failed to delete site");
    assert!(deleted);

    // Verify deleted
    let not_found = repo
        .find_by_id(tenant_id, site.id)
        .await
        .expect("Failed to find deleted site");
    assert!(not_found.is_none());
}

/// Test that trigger functions work (updated_at auto-updates)
#[tokio::test]
#[ignore]
async fn test_updated_at_trigger() {
    let fixture = PostgresTestFixture::new()
        .await
        .expect("Failed to create fixture");

    let tenant_id = TenantId(fixture.create_test_tenant().await);
    let repo = fixture.database.site_repo();

    let create_dto = CreateSiteDto {
        label: "Trigger Test".to_string(),
        site_type: SiteType::Field,
        crop_type: CropType::Other("Wheat".to_string()),
        variety: None,
        area: 5.0,
        gross_area: None,
        plots: None,
        row_config: None,
        bbch_stage: None,
        planted_date: None,
        cleared_date: None,
        soil_type: None,
        slope: None,
        slope_facing: None,
        altitude: None,
        organic: None,
        center: None,
        boundary: None,
        properties: None,
    };

    let site = repo
        .create(tenant_id, create_dto, Uuid::new_v4())
        .await
        .expect("Failed to create site");

    let created_at = site.created_at;
    let updated_at = site.updated_at;

    // Small delay to ensure timestamp difference
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Update site
    let update_dto = agrocore_domain::entities::site::UpdateSiteDto {
        label: Some("Updated Trigger Test".to_string()),
        site_type: None,
        crop_type: None,
        variety: None,
        area: None,
        gross_area: None,
        plots: None,
        row_config: None,
        bbch_stage: None,
        planted_date: None,
        cleared_date: None,
        soil_type: None,
        slope: None,
        slope_facing: None,
        altitude: None,
        organic: None,
        organic_eligible: None,
        center: None,
        sigpac_data: None,
        regepac_id: None,
        lpis_country: None,
        lpis_data: None,
        properties: None,
        custom_fields: None,
        note1: None,
        note2: None,
        is_active: None,
        is_temporary: None,
        boundary: None,
    };

    let updated = repo
        .update(tenant_id, site.id, update_dto, Uuid::new_v4())
        .await
        .expect("Failed to update site")
        .expect("Site not found");

    // updated_at should be newer
    assert!(updated.updated_at > created_at);
    assert!(updated.updated_at > updated_at);
}
