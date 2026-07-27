use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use chrono::{Duration, Utc};
use rand::TryRngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

const SESSION_COOKIE: &str = "tim_session";
const SESSION_DAYS: i64 = 30;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    catalog: Arc<Catalog>,
    config: Config,
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

#[derive(Clone)]
struct Config {
    database_url: String,
    dataset_dir: PathBuf,
    app_origin: String,
    session_secret: String,
    secure_cookies: bool,
}

impl Config {
    fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/tarkov-item-manager.db?mode=rwc".to_string());
        if !database_url.starts_with("sqlite:") {
            bail!(
                "DATABASE_URL 目前只支持 SQLite；PostgreSQL/MySQL 连接字符串已预留，尚未实现迁移和方言支持"
            );
        }
        let dataset_dir = env::var("DATASET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("../dataset"));
        let session_secret = env::var("SESSION_SECRET")
            .unwrap_or_else(|_| "development-only-change-this-secret".to_string());
        if session_secret.len() < 16 {
            bail!("SESSION_SECRET 至少需要 16 个字符");
        }
        Ok(Self {
            database_url,
            dataset_dir,
            app_origin: env::var("APP_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            session_secret,
            secure_cookies: env::var("SECURE_COOKIES")
                .map(|value| value == "true")
                .unwrap_or(false),
        })
    }
}

#[derive(Debug, Deserialize)]
struct NamedFile {
    version: u8,
    items: Option<Vec<Named>>,
    facilities: Option<Vec<Named>>,
}
#[derive(Debug, Deserialize, Clone)]
struct Named {
    id: String,
    name: String,
}
#[derive(Debug, Deserialize)]
struct HideoutFile {
    version: u8,
    upgrades: Vec<Upgrade>,
}
#[derive(Debug, Deserialize, Clone)]
struct Upgrade {
    #[serde(rename = "facilityId")]
    facility_id: String,
    level: i64,
    requirements: Vec<Requirement>,
    prerequisites: Vec<Prerequisite>,
}
#[derive(Debug, Deserialize, Clone)]
struct Requirement {
    #[serde(rename = "itemId")]
    item_id: String,
    quantity: i64,
}
#[derive(Debug, Deserialize, Clone)]
struct Prerequisite {
    #[serde(rename = "facilityId")]
    facility_id: String,
    level: i64,
}

#[derive(Debug, Clone)]
struct Catalog {
    items: BTreeMap<String, String>,
    facilities: BTreeMap<String, String>,
    upgrades: Vec<Upgrade>,
}

impl Catalog {
    fn load(dir: &Path) -> Result<Self> {
        let items = read_named(dir.join("items.json"), "items")?;
        let items_cn = read_named(dir.join("items.cn.json"), "items.cn")?;
        let facilities = read_facilities(dir.join("facilities.json"), "facilities")?;
        let facilities_cn = read_facilities(dir.join("facilities.cn.json"), "facilities.cn")?;
        let hideout: HideoutFile = read_json(dir.join("hideout.json"), "hideout")?;
        if hideout.version != 1 {
            bail!("hideout.json: 不支持的数据集版本 {}", hideout.version);
        }
        ensure_same_ids(&items, &items_cn, "items", "items.cn")?;
        ensure_same_ids(&facilities, &facilities_cn, "facilities", "facilities.cn")?;
        let item_names = indexed_names(items_cn, "items.cn")?;
        let facility_names = indexed_names(facilities_cn, "facilities.cn")?;
        let mut seen = HashSet::new();
        for upgrade in &hideout.upgrades {
            if upgrade.level < 1 {
                bail!("{} 等级必须大于 0", upgrade.facility_id);
            }
            if !facility_names.contains_key(&upgrade.facility_id) {
                bail!("升级引用了未知设施: {}", upgrade.facility_id);
            }
            if !seen.insert((upgrade.facility_id.clone(), upgrade.level)) {
                bail!("设施 {} 的等级 {} 重复", upgrade.facility_id, upgrade.level);
            }
            for requirement in &upgrade.requirements {
                if requirement.quantity < 1 || !item_names.contains_key(&requirement.item_id) {
                    bail!(
                        "设施 {} 的材料引用无效: {}",
                        upgrade.facility_id,
                        requirement.item_id
                    );
                }
            }
            for prerequisite in &upgrade.prerequisites {
                if prerequisite.level < 1 || !seen_key_exists(&hideout.upgrades, prerequisite) {
                    bail!(
                        "设施 {} 的前置条件无效: {} Lv.{}",
                        upgrade.facility_id,
                        prerequisite.facility_id,
                        prerequisite.level
                    );
                }
            }
        }
        validate_cycles(&hideout.upgrades)?;
        Ok(Self {
            items: item_names,
            facilities: facility_names,
            upgrades: hideout.upgrades,
        })
    }
}

fn read_json<T: for<'a> Deserialize<'a>>(path: PathBuf, label: &str) -> Result<T> {
    let content =
        fs::read_to_string(&path).with_context(|| format!("无法读取 {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("{} 不是有效 JSON", label))
}
fn read_named(path: PathBuf, label: &str) -> Result<Vec<Named>> {
    let file: NamedFile = read_json(path, label)?;
    if file.version != 1 {
        bail!("{} 数据集版本不受支持", label);
    }
    file.items
        .ok_or_else(|| anyhow::anyhow!("{} 缺少 items", label))
}
fn read_facilities(path: PathBuf, label: &str) -> Result<Vec<Named>> {
    let file: NamedFile = read_json(path, label)?;
    if file.version != 1 {
        bail!("{} 数据集版本不受支持", label);
    }
    file.facilities
        .ok_or_else(|| anyhow::anyhow!("{} 缺少 facilities", label))
}
fn indexed_names(records: Vec<Named>, label: &str) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for record in records {
        if record.id.trim().is_empty()
            || record.name.trim().is_empty()
            || result.insert(record.id.clone(), record.name).is_some()
        {
            bail!("{} 包含空或重复 ID", label);
        }
    }
    Ok(result)
}
fn ensure_same_ids(
    left: &[Named],
    right: &[Named],
    left_label: &str,
    right_label: &str,
) -> Result<()> {
    let left_ids: HashSet<_> = left.iter().map(|record| &record.id).collect();
    let right_ids: HashSet<_> = right.iter().map(|record| &record.id).collect();
    if left_ids != right_ids {
        bail!("{} 与 {} 的 ID 集合不一致", left_label, right_label);
    }
    Ok(())
}
fn seen_key_exists(upgrades: &[Upgrade], prerequisite: &Prerequisite) -> bool {
    upgrades.iter().any(|upgrade| {
        upgrade.facility_id == prerequisite.facility_id && upgrade.level == prerequisite.level
    })
}
fn validate_cycles(upgrades: &[Upgrade]) -> Result<()> {
    fn visit(
        key: &(String, i64),
        upgrades: &[Upgrade],
        active: &mut HashSet<(String, i64)>,
        done: &mut HashSet<(String, i64)>,
    ) -> Result<()> {
        if done.contains(key) {
            return Ok(());
        }
        if !active.insert(key.clone()) {
            bail!("设施前置条件存在循环依赖: {} Lv.{}", key.0, key.1);
        }
        let upgrade = upgrades
            .iter()
            .find(|item| item.facility_id == key.0 && item.level == key.1)
            .expect("validated prerequisite");
        for prerequisite in &upgrade.prerequisites {
            visit(
                &(prerequisite.facility_id.clone(), prerequisite.level),
                upgrades,
                active,
                done,
            )?;
        }
        active.remove(key);
        done.insert(key.clone());
        Ok(())
    }
    let mut active = HashSet::new();
    let mut done = HashSet::new();
    for upgrade in upgrades {
        visit(
            &(upgrade.facility_id.clone(), upgrade.level),
            upgrades,
            &mut active,
            &mut done,
        )?;
    }
    Ok(())
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}
type ApiResult<T> = Result<T, ApiProblem>;
struct ApiProblem(StatusCode, String);
impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        (self.0, Json(ApiError { error: self.1 })).into_response()
    }
}
fn bad_request(message: impl Into<String>) -> ApiProblem {
    ApiProblem(StatusCode::BAD_REQUEST, message.into())
}
fn unauthorized() -> ApiProblem {
    ApiProblem(StatusCode::UNAUTHORIZED, "请先登录".into())
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}
#[derive(Serialize)]
struct UserResponse {
    id: i64,
    username: String,
}
#[derive(Deserialize)]
struct FacilityTargetInput {
    #[serde(rename = "facilityId")]
    facility_id: String,
    level: i64,
}
#[derive(Deserialize)]
struct CheckedMaterialsInput {
    #[serde(rename = "itemIds")]
    item_ids: Vec<String>,
}
#[derive(Serialize)]
struct CatalogResponse {
    facilities: Vec<FacilityResponse>,
    materials: Vec<MaterialResponse>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FacilityResponse {
    id: String,
    name: String,
    max_level: i64,
    selected_level: i64,
    prerequisites: Vec<PrerequisiteResponse>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrerequisiteResponse {
    facility_id: String,
    facility_name: String,
    level: i64,
    satisfied: bool,
}
#[derive(Serialize)]
struct MaterialResponse {
    id: String,
    name: String,
    quantity: i64,
    checked: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let config = Config::from_env()?;
    let catalog = Catalog::load(&config.dataset_dir)?;
    if let Some(path) = config
        .database_url
        .strip_prefix("sqlite:")
        .and_then(|url| url.strip_prefix("data/"))
    {
        fs::create_dir_all("data").context("无法创建 SQLite 数据目录")?;
        let _ = path;
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let state = AppState {
        pool,
        catalog: Arc::new(catalog),
        config: config.clone(),
    };
    let cors = CorsLayer::new()
        .allow_origin(config.app_origin.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers([header::CONTENT_TYPE]);
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/catalog", get(get_catalog))
        .route("/api/progress/facilities", put(save_facility_targets))
        .route("/api/progress/materials", put(save_checked_materials))
        .fallback_service(
            ServeDir::new("frontend/dist")
                .not_found_service(ServeFile::new("frontend/dist/index.html")),
        )
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    let address: SocketAddr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()?;
    tracing::info!(%address, "服务已启动");
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
async fn register(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<Response> {
    let (username, password) = validated_credentials(input)?;
    let hash = hash_password(&password).map_err(internal)?;
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&username)
        .bind(hash)
        .execute(&state.pool)
        .await;
    let user_id = match result {
        Ok(result) => result.last_insert_rowid(),
        Err(error) if error.to_string().contains("UNIQUE") => {
            return Err(bad_request("用户名已存在"));
        }
        Err(error) => return Err(internal(error)),
    };
    session_response(&state, user_id, username).await
}
async fn login(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<Response> {
    let (username, password) = validated_credentials(input)?;
    let user = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;
    let Some((id, username, password_hash)) = user else {
        return Err(ApiProblem(
            StatusCode::UNAUTHORIZED,
            "用户名或密码错误".into(),
        ));
    };
    verify_password(&password, &password_hash)
        .map_err(|_| ApiProblem(StatusCode::UNAUTHORIZED, "用户名或密码错误".into()))?;
    session_response(&state, id, username).await
}
async fn logout(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Response> {
    if let Some(token) = session_token(&headers) {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(token_hash(&state.config, &token))
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }
    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, clear_cookie())],
    )
        .into_response())
}
async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<UserResponse>> {
    let (id, username) = authenticated_user(&state, &headers).await?;
    Ok(Json(UserResponse { id, username }))
}
async fn get_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<CatalogResponse>> {
    let (user_id, _) = authenticated_user(&state, &headers).await?;
    let targets: HashMap<String, i64> =
        sqlx::query_as("SELECT facility_id, level FROM facility_targets WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?
            .into_iter()
            .collect();
    let checked: HashSet<String> =
        sqlx::query_scalar("SELECT item_id FROM checked_materials WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?
            .into_iter()
            .collect();
    Ok(Json(build_catalog(&state.catalog, &targets, &checked)))
}
async fn save_facility_targets(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(inputs): Json<Vec<FacilityTargetInput>>,
) -> ApiResult<StatusCode> {
    let (user_id, _) = authenticated_user(&state, &headers).await?;
    let max_levels = maximum_levels(&state.catalog.upgrades);
    let mut values = HashMap::new();
    for input in inputs {
        let Some(max) = max_levels.get(&input.facility_id) else {
            return Err(bad_request("包含未知设施"));
        };
        if input.level < 0 || input.level > *max {
            return Err(bad_request(format!("{} 的等级不合法", input.facility_id)));
        }
        values.insert(input.facility_id, input.level);
    }
    let mut transaction = state.pool.begin().await.map_err(internal)?;
    sqlx::query("DELETE FROM facility_targets WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal)?;
    for (facility_id, level) in values {
        sqlx::query("INSERT INTO facility_targets (user_id, facility_id, level) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(facility_id)
            .bind(level)
            .execute(&mut *transaction)
            .await
            .map_err(internal)?;
    }
    transaction.commit().await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn save_checked_materials(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CheckedMaterialsInput>,
) -> ApiResult<StatusCode> {
    let (user_id, _) = authenticated_user(&state, &headers).await?;
    let ids: HashSet<_> = input.item_ids.into_iter().collect();
    if ids.iter().any(|id| !state.catalog.items.contains_key(id)) {
        return Err(bad_request("包含未知物品"));
    }
    let mut transaction = state.pool.begin().await.map_err(internal)?;
    sqlx::query("DELETE FROM checked_materials WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal)?;
    for id in ids {
        sqlx::query("INSERT INTO checked_materials (user_id, item_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(internal)?;
    }
    transaction.commit().await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

fn build_catalog(
    catalog: &Catalog,
    targets: &HashMap<String, i64>,
    checked: &HashSet<String>,
) -> CatalogResponse {
    let max_levels = maximum_levels(&catalog.upgrades);
    let facilities = catalog
        .facilities
        .iter()
        .map(|(id, name)| {
            let selected_level = *targets.get(id).unwrap_or(&0);
            let prerequisites = catalog
                .upgrades
                .iter()
                .filter(|upgrade| upgrade.facility_id == *id && upgrade.level == selected_level)
                .flat_map(|upgrade| &upgrade.prerequisites)
                .map(|prerequisite| PrerequisiteResponse {
                    facility_id: prerequisite.facility_id.clone(),
                    facility_name: catalog.facilities[&prerequisite.facility_id].clone(),
                    level: prerequisite.level,
                    satisfied: targets.get(&prerequisite.facility_id).copied().unwrap_or(0)
                        >= prerequisite.level,
                })
                .collect();
            FacilityResponse {
                id: id.clone(),
                name: name.clone(),
                max_level: max_levels[id],
                selected_level,
                prerequisites,
            }
        })
        .collect();
    let mut quantities = BTreeMap::<String, i64>::new();
    for upgrade in &catalog.upgrades {
        if upgrade.level <= targets.get(&upgrade.facility_id).copied().unwrap_or(0) {
            for requirement in &upgrade.requirements {
                *quantities.entry(requirement.item_id.clone()).or_default() += requirement.quantity;
            }
        }
    }
    let materials = quantities
        .into_iter()
        .map(|(id, quantity)| MaterialResponse {
            name: catalog.items[&id].clone(),
            checked: checked.contains(&id),
            id,
            quantity,
        })
        .collect();
    CatalogResponse {
        facilities,
        materials,
    }
}
fn maximum_levels(upgrades: &[Upgrade]) -> HashMap<String, i64> {
    upgrades.iter().fold(HashMap::new(), |mut all, upgrade| {
        all.entry(upgrade.facility_id.clone())
            .and_modify(|level| *level = (*level).max(upgrade.level))
            .or_insert(upgrade.level);
        all
    })
}
async fn authenticated_user(state: &AppState, headers: &HeaderMap) -> ApiResult<(i64, String)> {
    let Some(token) = session_token(headers) else {
        return Err(unauthorized());
    };
    sqlx::query_as("SELECT users.id, users.username FROM sessions JOIN users ON users.id = sessions.user_id WHERE sessions.token_hash = ? AND sessions.expires_at > CURRENT_TIMESTAMP").bind(token_hash(&state.config, &token)).fetch_optional(&state.pool).await.map_err(internal)?.ok_or_else(unauthorized)
}
async fn session_response(state: &AppState, user_id: i64, username: String) -> ApiResult<Response> {
    let token = secure_token().map_err(internal)?;
    let expires_at = Utc::now() + Duration::days(SESSION_DAYS);
    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(token_hash(&state.config, &token))
        .bind(user_id)
        .bind(expires_at.to_rfc3339())
        .execute(&state.pool)
        .await
        .map_err(internal)?;
    let cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={};{}",
        SESSION_COOKIE,
        token,
        SESSION_DAYS * 86400,
        if state.config.secure_cookies {
            " Secure;"
        } else {
            ""
        }
    );
    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(UserResponse {
            id: user_id,
            username,
        }),
    )
        .into_response())
}
fn validated_credentials(input: Credentials) -> ApiResult<(String, String)> {
    let username = input.username.trim().to_owned();
    if !(3..=32).contains(&username.len()) || !(8..=128).contains(&input.password.len()) {
        return Err(bad_request("用户名需为 3-32 个字符，密码需为 8-128 个字符"));
    }
    Ok((username, input.password))
}
fn hash_password(password: &str) -> Result<String> {
    let mut salt_bytes = [0u8; 16];
    rand::rngs::OsRng
        .try_fill_bytes(&mut salt_bytes)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let salt =
        SaltString::encode_b64(&salt_bytes).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}
fn verify_password(password: &str, hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(hash).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}
fn secure_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut bytes)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}
fn token_hash(config: &Config, token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(config.session_secret.as_bytes());
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
fn session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|entry| {
            let (name, value) = entry.trim().split_once('=')?;
            (name == SESSION_COOKIE).then(|| value.to_owned())
        })
}
fn clear_cookie() -> String {
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        SESSION_COOKIE
    )
}
fn internal(error: impl std::fmt::Display) -> ApiProblem {
    tracing::error!("{error}");
    ApiProblem(StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregates_selected_upgrades() {
        let catalog = Catalog::load(Path::new("../dataset")).unwrap();
        let response = build_catalog(
            &catalog,
            &HashMap::from([(String::from("generator"), 2)]),
            &HashSet::new(),
        );
        assert_eq!(
            response
                .materials
                .iter()
                .find(|item| item.id == "screw-nut")
                .unwrap()
                .quantity,
            12
        );
    }
    #[test]
    fn password_hash_round_trip() {
        let hash = hash_password("a-secure-password").unwrap();
        assert!(verify_password("a-secure-password", &hash).is_ok());
        assert!(verify_password("wrong-password", &hash).is_err());
    }
}
