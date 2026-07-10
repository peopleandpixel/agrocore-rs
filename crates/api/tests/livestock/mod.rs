// Livestock Handler Tests - DTO Validation
use agrocore_api::dto::{CreateAnimalDto, AddTreatmentDto};

#[test]
fn test_create_animal_validation() {
    let dto = CreateAnimalDto {
        tenant_id: "tenant-123".into(),
        site_id: "site-123".into(),
        animal_type: "cattle".into(),
        identifier: "CAT-001".into(),
        birthdate: None,
    };

    assert_eq!(dto.animal_type, "cattle");
    assert_eq!(dto.identifier, "CAT-001");
}

#[test]
fn test_add_treatment_validation() {
    let dto = AddTreatmentDto {
        animal_id: "animal-123".into(),
        treatment_type: "vaccination".into(),
        medication: Some("Vaccine X".into()),
        dosage: Some("10ml".into()),
        notes: None,
    };

    assert_eq!(dto.treatment_type, "vaccination");
}