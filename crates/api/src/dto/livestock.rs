//! Livestock DTOs

use agrocore_domain::entities::livestock::{
    Animal, AnimalSpecies, AnimalStatus, CreateAnimalDto as DomainCreateAnimalDto,
    CreateGrazingRecordDto as DomainCreateGrazingRecordDto,
    CreateTreatmentRecordDto as DomainCreateTreatmentRecordDto,
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
            tenant_id: a.tenant_id.into(),
            species: a.species,
            breed: a.breed,
            identifier: a.identifier,
            birth_date: a.birth_date.map(|d| d.to_rfc3339()),
            gender: a.gender,
            status: a.status,
            current_site_id: a.current_site_id,
            mother_id: a.mother_id,
            father_id: a.father_id,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
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
    pub current_site_id: Option<Uuid>,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
}

impl From<CreateAnimalDto> for DomainCreateAnimalDto {
    fn from(dto: CreateAnimalDto) -> Self {
        Self {
            species: dto.species,
            breed: dto.breed,
            identifier: dto.identifier,
            birth_date: dto.birth_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            gender: dto.gender,
            current_site_id: dto.current_site_id,
            mother_id: dto.mother_id,
            father_id: dto.father_id,
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
}

impl From<UpdateAnimalDto> for DomainUpdateAnimalDto {
    fn from(dto: UpdateAnimalDto) -> Self {
        Self {
            species: dto.species,
            breed: dto.breed,
            identifier: dto.identifier,
            birth_date: dto.birth_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            gender: dto.gender,
            status: dto.status,
            current_site_id: dto.current_site_id,
            group_id: dto.group_id,
            weight_kg: dto.weight_kg,
            mother_id: dto.mother_id,
            father_id: dto.father_id,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TreatmentRecordDto {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub date: String,
    pub treatment_type: String,
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<u32>,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<agrocore_domain::entities::livestock::TreatmentRecord> for TreatmentRecordDto {
    fn from(t: agrocore_domain::entities::livestock::TreatmentRecord) -> Self {
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
            created_at: t.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateTreatmentRecordDto {
    #[validate(length(min = 1))]
    pub treatment_type: String,
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<u32>,
    pub notes: Option<String>,
}

impl From<CreateTreatmentRecordDto> for DomainCreateTreatmentRecordDto {
    fn from(dto: CreateTreatmentRecordDto) -> Self {
        Self {
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
    pub site_id: Uuid,
    pub start_date: String,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<agrocore_domain::entities::livestock::GrazingRecord> for GrazingRecordDto {
    fn from(g: agrocore_domain::entities::livestock::GrazingRecord) -> Self {
        Self {
            id: g.id,
            animal_id: g.animal_id,
            site_id: g.site_id,
            start_date: g.start_date.to_rfc3339(),
            end_date: g.end_date.map(|d| d.to_rfc3339()),
            notes: g.notes,
            created_at: g.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateGrazingRecordDto {
    pub site_id: Uuid,
    pub start_date: String,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

impl From<CreateGrazingRecordDto> for DomainCreateGrazingRecordDto {
    fn from(dto: CreateGrazingRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            start_date: chrono::DateTime::parse_from_rfc3339(&dto.start_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            end_date: dto.end_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            notes: dto.notes,
        }
    }
}
