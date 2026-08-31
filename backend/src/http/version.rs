use crate::{APP_NAME, REPOSITORY_URL, app_version};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct VersionResponse {
    name: &'static str,
    version: &'static str,
    repository: &'static str,
}

pub async fn get() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: APP_NAME,
        version: app_version(),
        repository: REPOSITORY_URL,
    })
}
