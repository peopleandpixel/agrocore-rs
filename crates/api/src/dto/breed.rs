//! Breed DTOs

use agrocore_domain::entities::breed::{
    Breed, CreateBreedDto as DomainCreateBreedDto, Species, UpdateBreedDto as DomainUpdateBreedDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedBreedResponse {
    pub data: Vec<BreedDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BreedDto {
    pub id: Uuid,
    pub species: Species,
    pub name: String,
    pub origin: Option<String>,
}

impl From<Breed> for BreedDto {
    fn from(b: Breed) -> Self {
        Self {
            id: b.id,
            species: Species::from_str(&b.species),
            name: b.name,
            origin: b.origin,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateBreedDto {
    pub species: Species,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub origin: Option<String>,
}

impl From<CreateBreedDto> for DomainCreateBreedDto {
    fn from(dto: CreateBreedDto) -> Self {
        Self {
            species: dto.species,
            name: dto.name,
            origin: dto.origin,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateBreedDto {
    pub species: Option<Species>,
    pub name: Option<String>,
    pub origin: Option<String>,
}

impl From<UpdateBreedDto> for DomainUpdateBreedDto {
    fn from(dto: UpdateBreedDto) -> Self {
        Self {
            species: dto.species,
            name: dto.name,
            origin: dto.origin,
        }
    }
}
