use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tarkov_item_manager::{REPOSITORY_URL, build_app, config::Config};
use tower::ServiceExt;

struct TestApp {
    app: Router,
    _database_dir: tempfile::TempDir,
}

async fn test_app() -> Result<TestApp> {
    let dir = tempfile::tempdir()?;
    let database_url = format!(
        "sqlite:{}/test.db?mode=rwc",
        dir.path().to_string_lossy().replace('\\', "/")
    );
    let config = Config {
        database_url,
        dataset_dir: None,
        app_origin: "http://localhost:5173".to_string(),
        session_secret: "integration-test-secret-0123456789".to_string(),
        secure_cookies: false,
        desktop_app: false,
    };
    let app = build_app(config).await?;
    Ok(TestApp {
        app,
        _database_dir: dir,
    })
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>)> {
    let builder = Request::builder().method(method).uri(uri);
    let mut request = match &body {
        Some(value) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(value.to_string()))?,
        None => builder.body(Body::empty())?,
    };
    if let Some(value) = cookie {
        request.headers_mut().insert(header::COOKIE, value.parse()?);
    }
    let response = app.clone().oneshot(request).await?;
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await?.to_bytes();
    Ok((status, headers, bytes.to_vec()))
}

fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap_or(Value::Null)
}

fn session_cookie(headers: &HeaderMap) -> String {
    let value = headers
        .get(header::SET_COOKIE)
        .expect("login response sets session cookie")
        .to_str()
        .expect("session cookie is ASCII");
    value
        .split(';')
        .next()
        .expect("cookie has a name part")
        .to_string()
}

async fn register(
    app: &Router,
    username: &str,
    password: &str,
) -> Result<(StatusCode, String, Vec<u8>)> {
    let (status, headers, body) = send(
        app,
        "POST",
        "/api/auth/register",
        Some(json!({ "username": username, "password": password })),
        None,
    )
    .await?;
    let cookie = if status == StatusCode::OK {
        session_cookie(&headers)
    } else {
        String::new()
    };
    Ok((status, cookie, body))
}

async fn login(
    app: &Router,
    username: &str,
    password: &str,
) -> Result<(StatusCode, String, Vec<u8>)> {
    let (status, headers, body) = send(
        app,
        "POST",
        "/api/auth/login",
        Some(json!({ "username": username, "password": password })),
        None,
    )
    .await?;
    let cookie = if status == StatusCode::OK {
        session_cookie(&headers)
    } else {
        String::new()
    };
    Ok((status, cookie, body))
}

#[tokio::test]
async fn health_and_version_respond() -> Result<()> {
    let app = test_app().await?;
    let (status, _, body) = send(&app.app, "GET", "/api/health", None, None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"ok");

    let (status, _, body) = send(&app.app, "GET", "/api/version", None, None).await?;
    assert_eq!(status, StatusCode::OK);
    let payload = json(&body);
    assert_eq!(payload["name"], "Tarkov Item Manager");
    assert_eq!(payload["repository"], REPOSITORY_URL);
    assert!(payload["version"].is_string());
    Ok(())
}

#[tokio::test]
async fn register_me_and_logout_roundtrip() -> Result<()> {
    let app = test_app().await?;
    let (status, cookie, _) = register(&app.app, "alice", "password123").await?;
    assert_eq!(status, StatusCode::OK);
    assert!(cookie.starts_with("tim_session="));

    let (status, _, body) = send(&app.app, "GET", "/api/auth/me", None, Some(&cookie)).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json(&body)["username"], "alice");

    let (status, _, body) = register(&app.app, "alice", "password123").await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json(&body)["error"], "用户名已存在");

    let (status, _, body) = register(&app.app, "bob", "short").await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        json(&body)["error"],
        "用户名需为 3-32 个字符，密码需为 8-128 个字符"
    );

    let (status, _, body) = send(&app.app, "GET", "/api/auth/me", None, None).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json(&body)["error"], "请先登录");

    let (status, _, _) = send(&app.app, "POST", "/api/auth/logout", None, Some(&cookie)).await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = send(&app.app, "GET", "/api/auth/me", None, Some(&cookie)).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn login_rejects_wrong_password() -> Result<()> {
    let app = test_app().await?;
    register(&app.app, "carol", "password123").await?;

    let (status, _, body) = login(&app.app, "carol", "wrong-password").await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json(&body)["error"], "用户名或密码错误");

    let (status, _, body) = login(&app.app, "dave", "password123").await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json(&body)["error"], "用户名或密码错误");

    let (status, _, _) = login(&app.app, "carol", "password123").await?;
    assert_eq!(status, StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn change_password_requires_current_and_updates() -> Result<()> {
    let app = test_app().await?;
    let (_, cookie, _) = register(&app.app, "erin", "old-password").await?;

    let (status, _, body) = send(
        &app.app,
        "PUT",
        "/api/auth/password",
        Some(json!({ "currentPassword": "wrong-current", "newPassword": "new-password" })),
        Some(&cookie),
    )
    .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json(&body)["error"], "当前密码错误");

    let (status, _, body) = send(
        &app.app,
        "PUT",
        "/api/auth/password",
        Some(json!({ "currentPassword": "old-password", "newPassword": "short" })),
        Some(&cookie),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json(&body)["error"], "新密码需为 8-128 个字符");

    let (status, _, _) = send(
        &app.app,
        "PUT",
        "/api/auth/password",
        Some(json!({ "currentPassword": "old-password", "newPassword": "new-password" })),
        Some(&cookie),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _, _) = login(&app.app, "erin", "old-password").await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = login(&app.app, "erin", "new-password").await?;
    assert_eq!(status, StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn change_password_revokes_other_sessions() -> Result<()> {
    let app = test_app().await?;
    let (_, current, _) = register(&app.app, "frank", "old-password").await?;
    let (_, other, _) = login(&app.app, "frank", "old-password").await?;
    assert_ne!(current, other);

    let (status, _, _) = send(
        &app.app,
        "PUT",
        "/api/auth/password",
        Some(json!({ "currentPassword": "old-password", "newPassword": "new-password" })),
        Some(&current),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _, _) = send(&app.app, "GET", "/api/auth/me", None, Some(&current)).await?;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = send(&app.app, "GET", "/api/auth/me", None, Some(&other)).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn catalog_and_progress_roundtrip() -> Result<()> {
    let app = test_app().await?;

    let (status, _, _) = send(&app.app, "GET", "/api/catalog", None, None).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (_, cookie, _) = register(&app.app, "gina", "password123").await?;
    let (status, _, body) = send(&app.app, "GET", "/api/catalog", None, Some(&cookie)).await?;
    assert_eq!(status, StatusCode::OK);
    let catalog = json(&body);
    assert_eq!(catalog["gameMode"], "PVE");
    let facility = &catalog["facilities"][0];
    assert!(
        !catalog["facilities"]
            .as_array()
            .expect("facilities")
            .is_empty()
    );
    let facility_id = facility["id"].as_i64().context("facility id")?;
    let max_level = facility["maxLevel"]
        .as_i64()
        .context("facility max level")?;

    let (status, _, _) = send(
        &app.app,
        "PUT",
        "/api/progress/facilities",
        Some(json!([{ "facilityId": facility_id, "level": max_level }])),
        Some(&cookie),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _, body) = send(&app.app, "GET", "/api/catalog", None, Some(&cookie)).await?;
    assert_eq!(status, StatusCode::OK);
    let catalog = json(&body);
    let updated = catalog["facilities"]
        .as_array()
        .expect("facilities")
        .iter()
        .find(|f| f["id"] == facility_id)
        .expect("facility still present");
    assert_eq!(updated["currentLevel"], max_level);

    let (status, _, body) = send(
        &app.app,
        "PUT",
        "/api/progress/facilities",
        Some(json!([{ "facilityId": facility_id, "level": max_level + 1 }])),
        Some(&cookie),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        json(&body)["error"],
        format!("设施 {facility_id} 的等级不合法")
    );
    Ok(())
}
