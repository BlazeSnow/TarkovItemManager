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
