// Integration Tests - Mock Repository für Site & Compliance
// Nutzt In-Memory Implementierung statt echter MongoDB

use agrocore_api::dto::{CreateSiteDto, SiteDto, UpdateSiteDto};
use agrocore_domain::entities::site::{Site, SiteType};
use agrocore_shared::{Pagination, PaginatedResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Mock Repository
#[derive(Default, Clone)]
pub struct MockSiteRepo {
    sites: Arc<Mutex<HashMap<Uuid, Site>>>,
}

impl MockSiteRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, tenant_id: Uuid, mut site_dto: CreateSiteDto) -> Option<Site> {
        let site = Site {
            id: Uuid::new_v4(),
            tenant_id,
            label: site_dto.label.clone(),
            site_type: SiteType::Arable,
            crop_type: site_dto.crop_type.clone(),
            variety: site_dto.variety.clone(),
            area: site_dto.area,
            geometry: site_dto.geometry.clone(),
            doc_area: None,
            vintage: None,
            harvest_date: None,
            last_task_at: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: None,
            updated_by: None,
        };
        let id = site.id;
        self.sites.lock().unwrap().insert(id, site.clone());
        Some(site)
    }
}

// Test: Site Creation mit Mock
#[test]
fn test_site_create_integration() {
    let repo = MockSiteRepo::new();
    let tenant_id = Uuid::new_v4();
    
    let dto = CreateSiteDto {
        tenant_id,
        label: "Test Feld".into(),
        site_type: SiteType::Arable,
        crop_type: "Weizen".into(),
        variety: None,
        area: 10.5,
        geometry: None,
    };
    
    let result = repo.create(tenant_id, dto);
    assert!(result.is_some());
    let site = result.unwrap();
    assert_eq!(site.label, "Test Feld");
    assert_eq!(site.area, 10.5);
}