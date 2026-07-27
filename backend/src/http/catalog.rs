use crate::{auth::session, catalog::progress, db, error::ApiResult, state::AppState};
use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponse {
    schema_version: u8,
    game_mode: String,
    retrieved_at: String,
    facilities: Vec<FacilityResponse>,
    materials: Vec<MaterialResponse>,
    merchants: Vec<LevelResponse>,
    skills: Vec<SkillResponse>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacilityResponse {
    id: i64,
    name: String,
    max_level: i64,
    current_level: i64,
    upgrades: Vec<UpgradeResponse>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeResponse {
    level: i64,
    available: bool,
    construction_time_seconds: i64,
    requirements: Vec<RequirementResponse>,
    facility_prerequisites: Vec<FacilityGateResponse>,
    merchant_prerequisites: Vec<MerchantGateResponse>,
    skill_prerequisites: Vec<SkillGateResponse>,
    source_requirements_available: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementResponse {
    item_id: i64,
    name: String,
    quantity: i64,
    found_in_raid: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacilityGateResponse {
    facility_id: i64,
    name: String,
    level: i64,
    satisfied: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MerchantGateResponse {
    merchant_id: i64,
    name: String,
    level: i64,
    satisfied: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillGateResponse {
    name: String,
    level: i64,
    satisfied: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialResponse {
    item_id: i64,
    name: String,
    quantity: i64,
    found_in_raid: bool,
}
#[derive(Serialize)]
pub struct LevelResponse {
    id: i64,
    name: String,
    level: i64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillResponse {
    name: String,
    level: i64,
}
pub async fn get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<CatalogResponse>> {
    let (user_id, _) = session::user(&state.pool, &state.config, &headers).await?;
    let levels = db::progress::load_levels(&state.pool, user_id).await?;
    let merchant_levels = db::progress::load_merchant_levels(&state.pool, user_id).await?;
    let skill_levels = db::progress::load_skill_levels(&state.pool, user_id).await?;
    let result = progress::calculate(&state.catalog, &levels, &merchant_levels, &skill_levels);
    Ok(Json(CatalogResponse {
        schema_version: state.catalog.schema_version,
        game_mode: state.catalog.game_mode.clone(),
        retrieved_at: state.catalog.retrieved_at.clone(),
        facilities: result
            .facilities
            .into_iter()
            .map(|f| FacilityResponse {
                id: f.id,
                name: f.name,
                max_level: f.max_level,
                current_level: f.current_level,
                upgrades: f
                    .upgrades
                    .into_iter()
                    .map(|u| UpgradeResponse {
                        level: u.level,
                        available: u.available,
                        construction_time_seconds: u.construction_time_seconds,
                        requirements: u
                            .requirements
                            .into_iter()
                            .map(|r| RequirementResponse {
                                item_id: r.item_id,
                                name: r.name,
                                quantity: r.quantity,
                                found_in_raid: r.found_in_raid,
                            })
                            .collect(),
                        facility_prerequisites: u
                            .facility_prerequisites
                            .into_iter()
                            .map(|p| FacilityGateResponse {
                                facility_id: p.facility_id,
                                name: p.name,
                                level: p.level,
                                satisfied: p.satisfied,
                            })
                            .collect(),
                        merchant_prerequisites: u
                            .merchant_prerequisites
                            .into_iter()
                            .map(|p| MerchantGateResponse {
                                merchant_id: p.merchant_id,
                                name: p.name,
                                level: p.level,
                                satisfied: p.satisfied,
                            })
                            .collect(),
                        skill_prerequisites: u
                            .skill_prerequisites
                            .into_iter()
                            .map(|p| SkillGateResponse {
                                name: p.name,
                                level: p.level,
                                satisfied: p.satisfied,
                            })
                            .collect(),
                        source_requirements_available: u.source_requirements_available,
                    })
                    .collect(),
            })
            .collect(),
        materials: result
            .materials
            .into_iter()
            .map(|m| MaterialResponse {
                item_id: m.item_id,
                name: m.name,
                quantity: m.quantity,
                found_in_raid: m.found_in_raid,
            })
            .collect(),
        merchants: state
            .catalog
            .merchants
            .iter()
            .map(|(&id, name)| LevelResponse {
                id,
                name: name.clone(),
                level: merchant_levels.get(&id).copied().unwrap_or(0),
            })
            .collect(),
        skills: state
            .catalog
            .skill_names()
            .into_iter()
            .map(|name| SkillResponse {
                level: skill_levels.get(&name).copied().unwrap_or(0),
                name,
            })
            .collect(),
    }))
}
