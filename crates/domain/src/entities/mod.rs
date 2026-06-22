pub mod order;
pub mod site;
pub mod task;
pub mod tenant;
pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SiteType {
    #[serde(rename = "vineyard")]
    Vineyard,
    #[serde(rename = "olive_grove")]
    OliveGrove,
    #[serde(rename = "orchard")]
    Orchard,
    #[serde(rename = "field")]
    Field,
    #[serde(rename = "greenhouse")]
    Greenhouse,
    #[serde(rename = "other")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CropType {
    #[serde(rename = "grape")]
    Grape,
    #[serde(rename = "olive")]
    Olive,
    #[serde(rename = "apple")]
    Apple,
    #[serde(rename = "citrus")]
    Citrus,
    #[serde(rename = "vegetable")]
    Vegetable(String),
    #[serde(rename = "grain")]
    Grain(String),
    #[serde(rename = "other")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BbchStage {
    #[serde(rename = "0")] Dormancy,
    #[serde(rename = "1")] BudSwelling,
    #[serde(rename = "5")] WoolStage,
    #[serde(rename = "9")] BudBreak,
    #[serde(rename = "11")] FirstLeaf,
    #[serde(rename = "15")] LeafDevelopment,
    #[serde(rename = "19")] LeafFall,
    #[serde(rename = "53")] InflorescenceVisible,
    #[serde(rename = "57")] InflorescenceFullyDeveloped,
    #[serde(rename = "61")] FloweringBegins,
    #[serde(rename = "69")] FloweringEnds,
    #[serde(rename = "71")] FruitSet,
    #[serde(rename = "75")] PeaSizeBerries,
    #[serde(rename = "77")] BerriesTouching,
    #[serde(rename = "81")] VeraisonBegins,
    #[serde(rename = "89")] BerriesRipe,
    #[serde(rename = "91")] AfterHarvest,
    #[serde(rename = "97")] WinterDormancy,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    #[serde(rename = "draft")] Draft,
    #[serde(rename = "planned")] Planned,
    #[serde(rename = "in_progress")] InProgress,
    #[serde(rename = "completed")] Completed,
    #[serde(rename = "cancelled")] Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    #[serde(rename = "plant_protection")] PlantProtection,
    #[serde(rename = "fertilization")] Fertilization,
    #[serde(rename = "pruning")] Pruning,
    #[serde(rename = "harvest")] Harvest,
    #[serde(rename = "soil_work")] SoilWork,
    #[serde(rename = "irrigation")] Irrigation,
    #[serde(rename = "monitoring")] Monitoring,
    #[serde(rename = "other")] Other(String),
}
