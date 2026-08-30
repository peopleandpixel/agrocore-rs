//! Livestock DTOs (Individual Animal)

use agrocore_domain::entities::livestock::{
    Animal, AnimalSpecies, AnimalStatus, CreateAnimalDto as DomainCreateAnimalDto,
    CreateGrazingRecordDto as DomainCreateGrazingRecordDto,
    CreateTreatmentRecordDto as DomainCreateTreatmentRecordDto, GrazingRecord, TreatmentRecord,
    UpdateAnimalDto as DomainUpdateAnimalDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedAnimalResponse {
    pub data: Vec<AnimalDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnimalDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub species: AnimalSpecies,
    pub breed: Option<String>,
    pub identifier: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub status: AnimalStatus,
    pub current_site_id: Option<Uuid>,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Animal> for AnimalDto {
    fn from(a: Animal) -> Self {
        Self {
            id: a.id,
            tenant_id: a.tenant_id,
            species: AnimalSpecies::from_str(&a.species),
            breed: a.breed,
            identifier: a.identifier,
            birth_date: a.birth_date.map(|d| d.to_string()),
            gender: a.gender,
            status: AnimalStatus::from_str(&a.status),
            current_site_id: a.current_site_id,
            mother_id: a.mother_id,
            father_id: a.father_id,
            created_at: a.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            updated_at: a.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateAnimalDto {
    pub species: AnimalSpecies,
    pub breed: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub identifier: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub livestock_type: String,
    pub status: String,
    pub current_site_id: Option<Uuid>,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub plot_id: Option<Uuid>,
}

impl From<CreateAnimalDto> for DomainCreateAnimalDto {
    fn from(dto: CreateAnimalDto) -> Self {
        Self {
            identifier: dto.identifier,
            plot_id: dto.plot_id,
            livestock_type: dto.livestock_type,
            species: dto.species.as_str().to_string(),
            breed: dto.breed,
            birth_date: dto
                .birth_date
                .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            gender: dto.gender,
            status: dto.status,
            current_site_id: dto.current_site_id,
            mother_id: dto.mother_id,
            father_id: dto.father_id,
            tenant_id: dto.tenant_id,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateAnimalDto {
    pub species: Option<AnimalSpecies>,
    pub breed: Option<String>,
    pub identifier: Option<String>,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub status: Option<AnimalStatus>,
    pub current_site_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub weight_kg: Option<f64>,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub livestock_type: Option<String>,
    pub plot_id: Option<Uuid>,
}

impl From<UpdateAnimalDto> for DomainUpdateAnimalDto {
    fn from(dto: UpdateAnimalDto) -> Self {
        Self {
            identifier: dto.identifier,
            plot_id: dto.plot_id,
            livestock_type: dto.livestock_type,
            species: dto.species.map(|s| s.as_str().to_string()),
            breed: dto.breed,
            birth_date: dto
                .birth_date
                .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            gender: dto.gender,
            status: dto.status.map(|s| s.as_str().to_string()),
            current_site_id: dto.current_site_id,
            group_id: dto.group_id,
            weight_kg: dto.weight_kg,
            mother_id: dto.mother_id,
            father_id: dto.father_id,
            tenant_id: dto.tenant_id,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TreatmentRecordDto {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub date: String,
    pub treatment_type: String,
    pub medication: String,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<TreatmentRecord> for TreatmentRecordDto {
    fn from(t: TreatmentRecord) -> Self {
        Self {
            id: t.id,
            animal_id: t.animal_id,
            date: t.date.to_rfc3339(),
            treatment_type: t.treatment_type,
            medication: t.medication,
            dosage: t.dosage,
            veterinarian: t.veterinarian,
            withdrawal_days: t.withdrawal_days,
            notes: t.notes,
            created_at: t.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateTreatmentRecordDto {
    pub animal_id: Uuid,
    #[validate(length(min = 1))]
    pub treatment_type: String,
    pub medication: String,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub notes: Option<String>,
}

impl From<CreateTreatmentRecordDto> for DomainCreateTreatmentRecordDto {
    fn from(dto: CreateTreatmentRecordDto) -> Self {
        Self {
            animal_id: dto.animal_id,
            treatment_type: dto.treatment_type,
            medication: dto.medication,
            dosage: dto.dosage,
            veterinarian: dto.veterinarian,
            withdrawal_days: dto.withdrawal_days,
            notes: dto.notes,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GrazingRecordDto {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub plot_id: Uuid,
    pub site_id: Option<Uuid>,
    pub start_time: String,
    pub start_date: Option<String>,
    pub end_time: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<GrazingRecord> for GrazingRecordDto {
    fn from(g: GrazingRecord) -> Self {
        Self {
            id: g.id,
            animal_id: g.animal_id,
            plot_id: g.plot_id,
            site_id: g.site_id,
            start_time: g.start_time.to_rfc3339(),
            start_date: g.start_date.map(|d| d.to_string()),
            end_time: g.end_time.map(|d| d.to_rfc3339()),
            end_date: g.end_date.map(|d| d.to_string()),
            notes: g.notes,
            created_at: g.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateGrazingRecordDto {
    pub plot_id: Uuid,
    pub site_id: Option<Uuid>,
    pub animal_id: Option<Uuid>,
    pub start_time: String,
    pub start_date: Option<String>,
}

impl From<CreateGrazingRecordDto> for DomainCreateGrazingRecordDto {
    fn from(dto: CreateGrazingRecordDto) -> Self {
        Self {
            plot_id: dto.plot_id,
            site_id: dto.site_id,
            animal_id: dto.animal_id,
            start_time: chrono::DateTime::parse_from_rfc3339(&dto.start_time)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            start_date: dto
                .start_date
                .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        }
    }
}
