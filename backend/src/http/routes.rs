use crate::{
    http::{auth, catalog, progress, version},
    state::AppState,
};
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use include_dir::{Dir, include_dir};
use std::path::{Component, Path as FilePath};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

static EMBEDDED_FRONTEND: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

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
        .route("/api/auth/password", put(auth::change_password))
        .route("/api/version", get(version::get))
        .route("/api/catalog", get(catalog::get))
        .route("/api/progress/facilities", put(progress::facilities))
        .route("/api/progress/merchants", put(progress::merchants))
        .route("/api/progress/skills", put(progress::skills))
        .fallback(get(frontend))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET, Method::POST, Method::PUT])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http())
}

async fn frontend(uri: Uri) -> Response {
    let relative = uri.path().trim_start_matches('/');
    if let Some(file) = frontend_file(relative) {
        return response(file.contents(), relative);
    }
    if relative.is_empty() || !relative.contains('.') {
        let index = EMBEDDED_FRONTEND
            .get_file("index.html")
            .expect("embedded frontend contains index.html");
        return response(index.contents(), "index.html");
    }
    StatusCode::NOT_FOUND.into_response()
}

fn frontend_file(path: &str) -> Option<&'static include_dir::File<'static>> {
    let path = FilePath::new(path);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return None;
    }
    EMBEDDED_FRONTEND.get_file(path)
}

fn response(content: &'static [u8], path: &str) -> Response {
    let mime: HeaderValue = if path.ends_with(".html") {
        "text/html; charset=utf-8".parse().expect("valid MIME type")
    } else {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
            .parse()
            .expect("valid MIME type")
    };
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(content))
        .expect("valid embedded frontend response")
}

#[cfg(test)]
mod tests {
    use super::{frontend_file, response};
    use axum::http::header;

    #[test]
    fn serves_embedded_frontend_assets() {
        let index = frontend_file("index.html").expect("index should be embedded");
        let response = response(index.contents(), "index.html");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn falls_back_to_embedded_index_for_spa_routes() {
        assert!(frontend_file("login").is_none());
        assert!(frontend_file("index.html").is_some());
    }

    #[test]
    fn rejects_frontend_path_traversal() {
        assert!(frontend_file("../index.html").is_none());
    }
}
