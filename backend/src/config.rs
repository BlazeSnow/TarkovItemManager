use std::{env, path::PathBuf};

use anyhow::{Result, bail};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub dataset_dir: PathBuf,
    pub app_origin: String,
    pub session_secret: String,
    pub secure_cookies: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/tarkov-item-manager.db?mode=rwc".to_string());
        if !database_url.starts_with("sqlite:") {
            bail!(
                "DATABASE_URL 目前只支持 SQLite；PostgreSQL/MySQL 连接字符串已预留，尚未实现迁移和方言支持"
            );
        }
        let session_secret = env::var("SESSION_SECRET")
            .unwrap_or_else(|_| "development-only-change-this-secret".to_string());
        if session_secret.len() < 16 {
            bail!("SESSION_SECRET 至少需要 16 个字符");
        }
        Ok(Self {
            database_url,
            dataset_dir: env::var("DATASET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("../dataset")),
            app_origin: env::var("APP_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            session_secret,
            secure_cookies: env::var("SECURE_COOKIES")
                .map(|value| value == "true")
                .unwrap_or(false),
        })
    }
}
