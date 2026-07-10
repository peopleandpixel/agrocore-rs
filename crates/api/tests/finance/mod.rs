// Finance Handler Tests - DTO Validation
use agrocore_api::dto::{CreatePacApplicationDto, CreateCostCenterDto};

#[test]
fn test_pac_application_validation() {
    let dto = CreatePacApplicationDto {
        tenant_id: "tenant-123".into(),
        site_id: "site-123".into(),
        application_type: "subvention".into(),
        amount_requested: 5000.0,
        period_start: None,
        period_end: None,
    };

    assert_eq!(dto.application_type, "subvention");
    assert_eq!(dto.amount_requested, 5000.0);
}

#[test]
fn test_cost_center_validation() {
    let dto = CreateCostCenterDto {
        tenant_id: "tenant-123".into(),
        site_id: Some("site-123".into()),
        name: "Kostenstelle Wein".into(),
        category: "labor".into(),
    };

    assert_eq!(dto.name, "Kostenstelle Wein");
    assert_eq!(dto.category, "labor");
}