use crate::{
    auth::session,
    catalog::progress,
    db,
    error::{ApiResult, bad_request},
    state::AppState,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use std::collections::HashMap;
#[derive(Deserialize)]
pub struct FacilityLevelInput {
    #[serde(rename = "facilityId")]
    facility_id: i64,
    level: i64,
}
#[derive(Deserialize)]
pub struct MerchantLevelInput {
    #[serde(rename = "merchantId")]
    merchant_id: i64,
    level: i64,
}
#[derive(Deserialize)]
pub struct SkillLevelInput {
    name: String,
    level: i64,
}
pub async fn facilities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(inputs): Json<Vec<FacilityLevelInput>>,
) -> ApiResult<StatusCode> {
    let (user_id, _) = session::user(&state.pool, &state.config, &headers).await?;
    let max = progress::maximum_levels(&state.catalog.upgrades);
    let mut values = HashMap::new();
    for input in inputs {
        let Some(limit) = max.get(&input.facility_id) else {
            return Err(bad_request("包含未知设施"));
        };
        if input.level < 0 || input.level > *limit {
            return Err(bad_request(format!(
                "设施 {} 的等级不合法",
                input.facility_id
            )));
        }
        values.insert(input.facility_id, input.level);
    }
    db::progress::replace_levels(&state.pool, user_id, values).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn merchants(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(inputs): Json<Vec<MerchantLevelInput>>,
) -> ApiResult<StatusCode> {
    let (user_id, _) = session::user(&state.pool, &state.config, &headers).await?;
    let mut values = HashMap::new();
    for input in inputs {
        if input.level < 0 || !state.catalog.merchants.contains_key(&input.merchant_id) {
            return Err(bad_request("包含未知商人或非法等级"));
        }
        values.insert(input.merchant_id, input.level);
    }
    db::progress::replace_merchant_levels(&state.pool, user_id, values).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn skills(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(inputs): Json<Vec<SkillLevelInput>>,
) -> ApiResult<StatusCode> {
    let (user_id, _) = session::user(&state.pool, &state.config, &headers).await?;
    let supported = state.catalog.skill_names();
    let mut values = HashMap::new();
    for input in inputs {
        if input.level < 0 || input.name.trim().is_empty() || !supported.contains(&input.name) {
            return Err(bad_request("包含未知技能或非法等级"));
        }
        values.insert(input.name, input.level);
    }
    db::progress::replace_skill_levels(&state.pool, user_id, values).await?;
    Ok(StatusCode::NO_CONTENT)
}
