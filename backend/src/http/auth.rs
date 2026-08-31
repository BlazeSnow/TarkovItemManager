use crate::{
    auth::{password, session},
    db::users,
    error::{ApiProblem, ApiResult, bad_request},
    state::AppState,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}
#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
}
fn valid(input: Credentials) -> ApiResult<(String, String)> {
    let username = input.username.trim().to_owned();
    if !(3..=32).contains(&username.len()) || !(8..=128).contains(&input.password.len()) {
        return Err(bad_request("用户名需为 3-32 个字符，密码需为 8-128 个字符"));
    }
    Ok((username, input.password))
}
async fn response(state: &AppState, id: i64, username: String) -> ApiResult<Response> {
    let token = session::create(&state.pool, &state.config, id)
        .await
        .map_err(crate::error::internal)?;
    Ok((
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            session::set_cookie(&state.config, &token),
        )],
        Json(UserResponse { id, username }),
    )
        .into_response())
}
pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<Response> {
    let (username, password) = valid(input)?;
    let hash = password::hash(&password).map_err(crate::error::internal)?;
    let id = users::create(&state.pool, &username, &hash).await?;
    response(&state, id, username).await
}
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<Response> {
    let (username, password) = valid(input)?;
    let Some((id, username, hash)) = users::find(&state.pool, &username).await? else {
        return Err(ApiProblem(
            StatusCode::UNAUTHORIZED,
            "用户名或密码错误".into(),
        ));
    };
    password::verify(&password, &hash)
        .map_err(|_| ApiProblem(StatusCode::UNAUTHORIZED, "用户名或密码错误".into()))?;
    response(&state, id, username).await
}
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Response> {
    session::delete(&state.pool, &state.config, &headers).await
}
pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<UserResponse>> {
    let (id, username) = session::user(&state.pool, &state.config, &headers).await?;
    Ok(Json(UserResponse { id, username }))
}
#[derive(Deserialize)]
pub struct PasswordChange {
    #[serde(rename = "currentPassword")]
    current_password: String,
    #[serde(rename = "newPassword")]
    new_password: String,
}
pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PasswordChange>,
) -> ApiResult<StatusCode> {
    let (user_id, username) = session::user(&state.pool, &state.config, &headers).await?;
    if !(8..=128).contains(&input.new_password.len()) {
        return Err(bad_request("新密码需为 8-128 个字符"));
    }
    let Some((_, _, hash)) = users::find(&state.pool, &username).await? else {
        return Err(ApiProblem(StatusCode::UNAUTHORIZED, "用户不存在".into()));
    };
    password::verify(&input.current_password, &hash)
        .map_err(|_| ApiProblem(StatusCode::UNAUTHORIZED, "当前密码错误".into()))?;
    let new_hash = password::hash(&input.new_password).map_err(crate::error::internal)?;
    users::update_password(&state.pool, user_id, &new_hash).await?;
    session::delete_others(&state.pool, &state.config, user_id, &headers).await?;
    Ok(StatusCode::NO_CONTENT)
}
