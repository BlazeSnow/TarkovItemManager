use crate::{auth::session, catalog::progress, db, error::ApiResult, state::AppState};
use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;

#[derive(Serialize)]
pub struct CatalogResponse {
    facilities: Vec<FacilityResponse>,
    materials: Vec<MaterialResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacilityResponse {
    id: String,
    name: String,
    max_level: i64,
    current_level: i64,
    prerequisites: Vec<PrerequisiteResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrerequisiteResponse {
    upgrade_level: i64,
    facility_id: String,
    facility_name: String,
    level: i64,
    satisfied: bool,
}

#[derive(Serialize)]
pub struct MaterialResponse {
    id: String,
    name: String,
    quantity: i64,
    checked: bool,
}

pub async fn get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<CatalogResponse>> {
    let (user_id, _) = session::user(&state.pool, &state.config, &headers).await?;
    let levels = db::progress::load_levels(&state.pool, user_id).await?;
    let checked = db::progress::load_checked(&state.pool, user_id).await?;
    let result = progress::calculate(&state.catalog, &levels, &checked);

    Ok(Json(CatalogResponse {
        facilities: result
            .facilities
            .into_iter()
            .map(|facility| FacilityResponse {
                id: facility.id,
                name: facility.name,
                max_level: facility.max_level,
                current_level: facility.current_level,
                prerequisites: facility
                    .prerequisites
                    .into_iter()
                    .map(|prerequisite| PrerequisiteResponse {
                        upgrade_level: prerequisite.upgrade_level,
                        facility_id: prerequisite.facility_id,
                        facility_name: prerequisite.facility_name,
                        level: prerequisite.level,
                        satisfied: prerequisite.satisfied,
                    })
                    .collect(),
            })
            .collect(),
        materials: result
            .materials
            .into_iter()
            .map(|material| MaterialResponse {
                id: material.id,
                name: material.name,
                quantity: material.quantity,
                checked: material.checked,
            })
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        path::Path,
    };

    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn response_preserves_current_level_json_contract() {
        let catalog = Catalog::load(Path::new("../dataset")).unwrap();
        let result = progress::calculate(
            &catalog,
            &HashMap::from([(String::from("generator"), 1)]),
            &HashSet::new(),
        );
        let response = CatalogResponse {
            facilities: result
                .facilities
                .into_iter()
                .map(|facility| FacilityResponse {
                    id: facility.id,
                    name: facility.name,
                    max_level: facility.max_level,
                    current_level: facility.current_level,
                    prerequisites: facility
                        .prerequisites
                        .into_iter()
                        .map(|prerequisite| PrerequisiteResponse {
                            upgrade_level: prerequisite.upgrade_level,
                            facility_id: prerequisite.facility_id,
                            facility_name: prerequisite.facility_name,
                            level: prerequisite.level,
                            satisfied: prerequisite.satisfied,
                        })
                        .collect(),
                })
                .collect(),
            materials: Vec::new(),
        };
        let value = serde_json::to_value(response).unwrap();
        let generator = value["facilities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|facility| facility["id"] == "generator")
            .unwrap();

        assert_eq!(generator["currentLevel"], 1);
        assert_eq!(generator["prerequisites"][0]["upgradeLevel"], 2);
        assert_eq!(generator["prerequisites"][0]["facilityId"], "generator");
    }
}
