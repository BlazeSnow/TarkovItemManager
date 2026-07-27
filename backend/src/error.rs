use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct ApiError {
    error: String,
}

pub type ApiResult<T> = Result<T, ApiProblem>;
pub struct ApiProblem(pub StatusCode, pub String);

impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        (self.0, Json(ApiError { error: self.1 })).into_response()
    }
}

pub fn bad_request(message: impl Into<String>) -> ApiProblem {
    ApiProblem(StatusCode::BAD_REQUEST, message.into())
}
pub fn unauthorized() -> ApiProblem {
    ApiProblem(StatusCode::UNAUTHORIZED, "请先登录".into())
}
pub fn internal(error: impl std::fmt::Display) -> ApiProblem {
    tracing::error!("{error}");
    ApiProblem(StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误".into())
}
