// Integration Tests - Olive Mock

use agrocore_domain::entities::olive::{CreateOliveGroveDto, OliveGrove, OilGrade};
use agrocore_domain::entities::tenant::TenantId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct MockOliveRepo {
    groves: Arc<Mutex<HashMap<Uuid, OliveGrove>>>,
}

impl MockOliveRepo {
    pub fn new() -> Self {
        Self {
            groves: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create(&self, tenant_id: TenantId, dto: CreateOliveGroveDto) -> OliveGrove {
        let grove = OliveGrove {
            id: Uuid::new_v4(),
            tenant_id,
            site_id: dto.site_id,
            variety: dto.variety,
            tree_count: dto.tree_count,
            planting_year: dto.planting_year,
            organic_certified: dto.organic_certified,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let id = grove.id;
        self.groves.lock().unwrap().insert(id, grove.clone());
        grove
    }
}

#[test]
fn test_olive_grove_crud() {
    let repo = MockOliveRepo::new();
    let tenant_id: TenantId = Uuid::nil();
    let site_id = Uuid::new_v4();

    let dto = CreateOliveGroveDto {
        site_id,
        variety: Some("Galega".into()),
        tree_count: Some(150),
        planting_year: Some(2020),
        organic_certified: true,
    };

    let grove = repo.create(tenant_id, dto);
    assert_eq!(grove.variety, Some("Galega".into()));
    assert_eq!(grove.tree_count, Some(150));
    assert!(grove.organic_certified);
}