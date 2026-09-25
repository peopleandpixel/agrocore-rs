// API-specific enums with frontend-compatible serialization
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Site type with flat string serialization for API compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiSiteType {
    Vineyard,
    Field,
    CorkOakMontado,
    HolmOakMontado,
    OliveGrove,
    Orchard,
    AlmondOrchard,
    CitrusGrove,
    Pasture,
    Greenhouse,
    Other(String),
}

/// Crop type with flat string serialization for API compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiCropType {
    Grape,
    Olive,
    Apple,
    Citrus,
    Vegetable(String),
    Nut,
    CorkOak,
    Almond,
    Hazelnut,
    Chestnut,
    Berry,
    Tropical,
    Poultry,
    Livestock,
    Fallow,
    Forest,
    Pasture,
    Grain(String),
    Other(String),
    Unknown,
}

impl From<agrocore_domain::entities::SiteType> for ApiSiteType {
    fn from(st: agrocore_domain::entities::SiteType) -> Self {
        match st {
            agrocore_domain::entities::SiteType::Vineyard => ApiSiteType::Vineyard,
            agrocore_domain::entities::SiteType::Field => ApiSiteType::Field,
            agrocore_domain::entities::SiteType::CorkOakMontado => ApiSiteType::CorkOakMontado,
            agrocore_domain::entities::SiteType::HolmOakMontado => ApiSiteType::HolmOakMontado,
            agrocore_domain::entities::SiteType::OliveGrove => ApiSiteType::OliveGrove,
            agrocore_domain::entities::SiteType::Orchard => ApiSiteType::Orchard,
            agrocore_domain::entities::SiteType::AlmondOrchard => ApiSiteType::AlmondOrchard,
            agrocore_domain::entities::SiteType::CitrusGrove => ApiSiteType::CitrusGrove,
            agrocore_domain::entities::SiteType::Pasture => ApiSiteType::Pasture,
            agrocore_domain::entities::SiteType::Greenhouse => ApiSiteType::Greenhouse,
            agrocore_domain::entities::SiteType::Other(s) => ApiSiteType::Other(s),
        }
    }
}

impl From<ApiSiteType> for agrocore_domain::entities::SiteType {
    fn from(st: ApiSiteType) -> Self {
        match st {
            ApiSiteType::Vineyard => agrocore_domain::entities::SiteType::Vineyard,
            ApiSiteType::Field => agrocore_domain::entities::SiteType::Field,
            ApiSiteType::CorkOakMontado => agrocore_domain::entities::SiteType::CorkOakMontado,
            ApiSiteType::HolmOakMontado => agrocore_domain::entities::SiteType::HolmOakMontado,
            ApiSiteType::OliveGrove => agrocore_domain::entities::SiteType::OliveGrove,
            ApiSiteType::Orchard => agrocore_domain::entities::SiteType::Orchard,
            ApiSiteType::AlmondOrchard => agrocore_domain::entities::SiteType::AlmondOrchard,
            ApiSiteType::CitrusGrove => agrocore_domain::entities::SiteType::CitrusGrove,
            ApiSiteType::Pasture => agrocore_domain::entities::SiteType::Pasture,
            ApiSiteType::Greenhouse => agrocore_domain::entities::SiteType::Greenhouse,
            ApiSiteType::Other(s) => agrocore_domain::entities::SiteType::Other(s),
        }
    }
}

impl From<agrocore_domain::entities::CropType> for ApiCropType {
    fn from(ct: agrocore_domain::entities::CropType) -> Self {
        match ct {
            agrocore_domain::entities::CropType::Grape => ApiCropType::Grape,
            agrocore_domain::entities::CropType::Olive => ApiCropType::Olive,
            agrocore_domain::entities::CropType::Apple => ApiCropType::Apple,
            agrocore_domain::entities::CropType::Citrus => ApiCropType::Citrus,
            agrocore_domain::entities::CropType::Vegetable(s) => ApiCropType::Vegetable(s),
            agrocore_domain::entities::CropType::Nut => ApiCropType::Nut,
            agrocore_domain::entities::CropType::CorkOak => ApiCropType::CorkOak,
            agrocore_domain::entities::CropType::Almond => ApiCropType::Almond,
            agrocore_domain::entities::CropType::Hazelnut => ApiCropType::Hazelnut,
            agrocore_domain::entities::CropType::Chestnut => ApiCropType::Chestnut,
            agrocore_domain::entities::CropType::Berry => ApiCropType::Berry,
            agrocore_domain::entities::CropType::Tropical => ApiCropType::Tropical,
            agrocore_domain::entities::CropType::Poultry => ApiCropType::Poultry,
            agrocore_domain::entities::CropType::Livestock => ApiCropType::Livestock,
            agrocore_domain::entities::CropType::Fallow => ApiCropType::Fallow,
            agrocore_domain::entities::CropType::Forest => ApiCropType::Forest,
            agrocore_domain::entities::CropType::Pasture => ApiCropType::Pasture,
            agrocore_domain::entities::CropType::Grain(s) => ApiCropType::Grain(s),
            agrocore_domain::entities::CropType::Other(s) => ApiCropType::Other(s),
            agrocore_domain::entities::CropType::Unknown => ApiCropType::Unknown,
        }
    }
}

impl From<ApiCropType> for agrocore_domain::entities::CropType {
    fn from(ct: ApiCropType) -> Self {
        match ct {
            ApiCropType::Grape => agrocore_domain::entities::CropType::Grape,
            ApiCropType::Olive => agrocore_domain::entities::CropType::Olive,
            ApiCropType::Apple => agrocore_domain::entities::CropType::Apple,
            ApiCropType::Citrus => agrocore_domain::entities::CropType::Citrus,
            ApiCropType::Vegetable(s) => agrocore_domain::entities::CropType::Vegetable(s),
            ApiCropType::Nut => agrocore_domain::entities::CropType::Nut,
            ApiCropType::CorkOak => agrocore_domain::entities::CropType::CorkOak,
            ApiCropType::Almond => agrocore_domain::entities::CropType::Almond,
            ApiCropType::Hazelnut => agrocore_domain::entities::CropType::Hazelnut,
            ApiCropType::Chestnut => agrocore_domain::entities::CropType::Chestnut,
            ApiCropType::Berry => agrocore_domain::entities::CropType::Berry,
            ApiCropType::Tropical => agrocore_domain::entities::CropType::Tropical,
            ApiCropType::Poultry => agrocore_domain::entities::CropType::Poultry,
            ApiCropType::Livestock => agrocore_domain::entities::CropType::Livestock,
            ApiCropType::Fallow => agrocore_domain::entities::CropType::Fallow,
            ApiCropType::Forest => agrocore_domain::entities::CropType::Forest,
            ApiCropType::Pasture => agrocore_domain::entities::CropType::Pasture,
            ApiCropType::Grain(s) => agrocore_domain::entities::CropType::Grain(s),
            ApiCropType::Other(s) => agrocore_domain::entities::CropType::Other(s),
            ApiCropType::Unknown => agrocore_domain::entities::CropType::Unknown,
        }
    }
}
