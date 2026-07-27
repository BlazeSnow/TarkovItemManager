use crate::error::{ApiResult, internal};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
pub async fn load_levels(pool: &SqlitePool, user_id: i64) -> ApiResult<HashMap<String, i64>> {
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
    levels: HashMap<String, i64>,
) -> ApiResult<()> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("DELETE FROM facility_levels WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    for (id, level) in levels {
        sqlx::query("INSERT INTO facility_levels (user_id, facility_id, level) VALUES (?, ?, ?)")
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
pub async fn load_checked(pool: &SqlitePool, user_id: i64) -> ApiResult<HashSet<String>> {
    Ok(
        sqlx::query_scalar("SELECT item_id FROM checked_materials WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(internal)?
            .into_iter()
            .collect(),
    )
}
pub async fn replace_checked(
    pool: &SqlitePool,
    user_id: i64,
    ids: HashSet<String>,
) -> ApiResult<()> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("DELETE FROM checked_materials WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    for id in ids {
        sqlx::query("INSERT INTO checked_materials (user_id, item_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
    }
    tx.commit().await.map_err(internal)?;
    Ok(())
}
