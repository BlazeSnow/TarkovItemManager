pub mod auth;
pub mod catalog;
pub mod config;
pub mod db;
pub mod error;
pub mod http;
pub mod state;

use std::{fs, sync::Arc};

use anyhow::{Context, Result};
use axum::Router;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::{catalog::Catalog, config::Config, state::AppState};

pub const APP_NAME: &str = "Tarkov Item Manager";
pub const REPOSITORY_URL: &str = "https://github.com/BlazeSnow/TarkovItemManager";

pub fn app_version() -> &'static str {
    version_from_env(option_env!("TARKOV_ITEM_MANAGER_VERSION"))
        .unwrap_or(env!("CARGO_PKG_VERSION"))
}

fn version_from_env(raw: Option<&'static str>) -> Option<&'static str> {
    raw.map(|value| value.strip_prefix('v').unwrap_or(value))
}

#[cfg(test)]
mod tests {
    use super::version_from_env;

    #[test]
    fn strips_release_tag_prefix() {
        assert_eq!(
            version_from_env(Some("v2026.8.31-beta.1")),
            Some("2026.8.31-beta.1")
        );
        assert_eq!(version_from_env(Some("dev")), Some("dev"));
        assert_eq!(version_from_env(None), None);
    }
}

pub async fn build_app(config: Config) -> Result<Router> {
    let catalog = match &config.dataset_dir {
        Some(dir) => Catalog::load_external(dir)?,
        None => Catalog::load_embedded()?,
    };
    if config.database_url.starts_with("sqlite:data/") {
        fs::create_dir_all("data").context("无法创建 SQLite 数据目录")?;
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(http::routes::router(AppState {
        pool,
        catalog: Arc::new(catalog),
        config,
    }))
}

pub async fn connect_for_tests(database_url: &str) -> Result<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?)
}
