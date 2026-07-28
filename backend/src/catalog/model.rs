use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Upgrade {
    #[serde(rename = "facilityId")]
    pub facility_id: String,
    pub level: i64,
    pub requirements: Vec<Requirement>,
    pub prerequisites: Vec<Prerequisite>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Requirement {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub quantity: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prerequisite {
    #[serde(rename = "facilityId")]
    pub facility_id: String,
    pub level: i64,
}
