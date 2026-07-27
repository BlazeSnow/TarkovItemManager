use crate::{
    config::Config,
    error::{ApiResult, internal, unauthorized},
};
use anyhow::Result;
use axum::{
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use rand::TryRngCore;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
const NAME: &str = "tim_session";
const DAYS: i64 = 30;
pub fn token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|v| {
            let (n, v) = v.trim().split_once('=')?;
            (n == NAME).then(|| v.to_owned())
        })
}
pub fn hash(secret: &str, token: &str) -> String {
    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    h.update(token.as_bytes());
    format!("{:x}", h.finalize())
}
pub async fn user(
    pool: &SqlitePool,
    config: &Config,
    headers: &HeaderMap,
) -> ApiResult<(i64, String)> {
    let Some(token) = token(headers) else {
        return Err(unauthorized());
    };
    sqlx::query_as("SELECT users.id, users.username FROM sessions JOIN users ON users.id = sessions.user_id WHERE sessions.token_hash = ? AND sessions.expires_at > CURRENT_TIMESTAMP").bind(hash(&config.session_secret,&token)).fetch_optional(pool).await.map_err(internal)?.ok_or_else(unauthorized)
}
pub async fn create(pool: &SqlitePool, config: &Config, user_id: i64) -> Result<String> {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut bytes)?;
    let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(hash(&config.session_secret, &token))
        .bind(user_id)
        .bind((Utc::now() + Duration::days(DAYS)).to_rfc3339())
        .execute(pool)
        .await?;
    Ok(token)
}
pub fn set_cookie(config: &Config, token: &str) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={};{}",
        NAME,
        token,
        DAYS * 86400,
        if config.secure_cookies {
            " Secure;"
        } else {
            ""
        }
    )
}
pub fn clear_cookie() -> String {
    format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0", NAME)
}
pub async fn delete(
    pool: &SqlitePool,
    config: &Config,
    headers: &HeaderMap,
) -> ApiResult<Response> {
    if let Some(token) = token(headers) {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(hash(&config.session_secret, &token))
            .execute(pool)
            .await
            .map_err(internal)?;
    }
    Ok((
        axum::http::StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, clear_cookie())],
    )
        .into_response())
}
