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
use std::collections::{HashMap, HashSet};
#[derive(Deserialize)]
pub struct FacilityLevelInput {
    #[serde(rename = "facilityId")]
    facility_id: String,
    level: i64,
}
#[derive(Deserialize)]
pub struct CheckedMaterialsInput {
    #[serde(rename = "itemIds")]
    item_ids: Vec<String>,
}
pub async fn levels(
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
            return Err(bad_request(format!("{} 的等级不合法", input.facility_id)));
        }
        values.insert(input.facility_id, input.level);
    }
    db::progress::replace_levels(&state.pool, user_id, values).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn materials(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CheckedMaterialsInput>,
) -> ApiResult<StatusCode> {
    let (user_id, _) = session::user(&state.pool, &state.config, &headers).await?;
    let ids: HashSet<_> = input.item_ids.into_iter().collect();
    if ids.iter().any(|id| !state.catalog.items.contains_key(id)) {
        return Err(bad_request("包含未知物品"));
    }
    db::progress::replace_checked(&state.pool, user_id, ids).await?;
    Ok(StatusCode::NO_CONTENT)
}
