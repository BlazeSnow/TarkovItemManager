use crate::{
    http::{auth, catalog, progress},
    state::AppState,
};
use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, post, put},
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
pub fn router(state: AppState) -> Router {
    let origin = state
        .config
        .app_origin
        .parse::<HeaderValue>()
        .expect("validated APP_ORIGIN");
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/catalog", get(catalog::get))
        .route("/api/progress/facilities", put(progress::facilities))
        .route("/api/progress/merchants", put(progress::merchants))
        .route("/api/progress/skills", put(progress::skills))
        .fallback_service(
            ServeDir::new("frontend/dist")
                .not_found_service(ServeFile::new("frontend/dist/index.html")),
        )
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET, Method::POST, Method::PUT])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http())
}
