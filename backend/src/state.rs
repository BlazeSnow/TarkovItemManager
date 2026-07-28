use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::{catalog::Catalog, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub catalog: Arc<Catalog>,
    pub config: Config,
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}
