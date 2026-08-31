use crate::error::{ApiResult, bad_request, internal};
use sqlx::SqlitePool;
pub async fn create(pool: &SqlitePool, username: &str, password_hash: &str) -> ApiResult<i64> {
    let r = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(username)
        .bind(password_hash)
        .execute(pool)
        .await;
    match r {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) if e.to_string().contains("UNIQUE") => Err(bad_request("用户名已存在")),
        Err(e) => Err(internal(e)),
    }
}
pub async fn find(pool: &SqlitePool, username: &str) -> ApiResult<Option<(i64, String, String)>> {
    sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(internal)
}
pub async fn update_password(pool: &SqlitePool, id: i64, password_hash: &str) -> ApiResult<()> {
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await
        .map_err(internal)?;
    Ok(())
}
