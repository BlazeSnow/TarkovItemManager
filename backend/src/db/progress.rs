use crate::error::{ApiResult, internal};
use sqlx::SqlitePool;
use std::collections::HashMap;
pub async fn load_levels(pool: &SqlitePool, user_id: i64) -> ApiResult<HashMap<i64, i64>> {
    Ok(
        sqlx::query_as("SELECT facility_id, level FROM facility_levels WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(internal)?
            .into_iter()
            .collect(),
    )
}
pub async fn replace_levels(
    pool: &SqlitePool,
    user_id: i64,
    levels: HashMap<i64, i64>,
) -> ApiResult<()> {
    replace(pool, user_id, "facility_levels", "facility_id", levels).await
}
pub async fn load_merchant_levels(pool: &SqlitePool, user_id: i64) -> ApiResult<HashMap<i64, i64>> {
    Ok(
        sqlx::query_as("SELECT merchant_id, level FROM merchant_levels WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(internal)?
            .into_iter()
            .collect(),
    )
}
pub async fn replace_merchant_levels(
    pool: &SqlitePool,
    user_id: i64,
    levels: HashMap<i64, i64>,
) -> ApiResult<()> {
    replace(pool, user_id, "merchant_levels", "merchant_id", levels).await
}
pub async fn load_skill_levels(pool: &SqlitePool, user_id: i64) -> ApiResult<HashMap<String, i64>> {
    Ok(
        sqlx::query_as("SELECT skill_name, level FROM skill_levels WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(internal)?
            .into_iter()
            .collect(),
    )
}
pub async fn replace_skill_levels(
    pool: &SqlitePool,
    user_id: i64,
    levels: HashMap<String, i64>,
) -> ApiResult<()> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("DELETE FROM skill_levels WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    for (name, level) in levels {
        sqlx::query("INSERT INTO skill_levels (user_id, skill_name, level) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(name)
            .bind(level)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
    }
    tx.commit().await.map_err(internal)?;
    Ok(())
}
async fn replace(
    pool: &SqlitePool,
    user_id: i64,
    table: &str,
    column: &str,
    levels: HashMap<i64, i64>,
) -> ApiResult<()> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query(&format!("DELETE FROM {table} WHERE user_id = ?"))
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    for (id, level) in levels {
        sqlx::query(&format!(
            "INSERT INTO {table} (user_id, {column}, level) VALUES (?, ?, ?)"
        ))
        .bind(user_id)
        .bind(id)
        .bind(level)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    }
    tx.commit().await.map_err(internal)?;
    Ok(())
}
