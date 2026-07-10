// Integration Tests - Compliance Mock

use agrocore_domain::entities::compliance::{
    ApplicatorLicense, ComplianceChecklist, ComplianceItem, ComplianceStatus,
};
use agrocore_shared::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct MockComplianceRepo {
    items: Arc<Mutex<Vec<ComplianceItem>>>,
}

impl MockComplianceRepo {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_item(&self, item: ComplianceItem) -> Result<()> {
        self.items.lock().unwrap().push(item);
        Ok(())
    }

    pub fn list_by_site(&self, site_id: Uuid) -> Vec<ComplianceItem> {
        self.items
            .lock()
            .unwrap()
            .iter()
            .filter(|i| i.site_id == site_id)
            .cloned()
            .collect()
    }
}

#[test]
fn test_compliance_gap_checklist() {
    let repo = MockComplianceRepo::new();
    let tenant_id = Uuid::new_v4();
    let site_id = Uuid::new_v4();

    let item = ComplianceItem {
        id: Uuid::new_v4(),
        tenant_id,
        checklist_id: Uuid::new_v4(),
        site_id,
        label: "GAP-001".into(),
        status: ComplianceStatus::Pending,
        evidence_url: None,
        completed_at: None,
        notes: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let _ = repo.add_item(item.clone());
    let items = repo.list_by_site(site_id);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].label, "GAP-001");
}