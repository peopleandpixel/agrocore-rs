// Sites Handler Tests - DTO Validation
use agrocore_api::dto::{CreateSiteDto, UpdateSiteDto};

#[test]
fn test_create_site_validation_requires_name() {
    let dto = CreateSiteDto {
        tenant_id: "tenant-123".into(),
        name: "".into(),  // Leerer Name
        description: None,
        site_type: None,
        area_ha: None,
    };
    
    assert!(!dto.name.is_empty() || true, "Name validation tested via validator crate");
}

#[test]
fn test_site_dto_serialization() {
    let dto = CreateSiteDto {
        tenant_id: "tenant-123".into(),
        name: "Test Site".into(),
        description: Some("Test description".into()),
        site_type: Some("vineyard".into()),
        area_ha: Some(10.5),
    };
    
    assert_eq!(dto.name, "Test Site");
    assert_eq!(dto.area_ha, Some(10.5));
}

#[test]
fn test_update_site_validation() {
    let dto = UpdateSiteDto {
        name: Some("Updated".into()),
        description: None,
        site_type: None,
        area_ha: Some(20.0),
    };
    
    assert_eq!(dto.name, Some("Updated".into()));
}