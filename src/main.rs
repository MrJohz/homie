use std::{ffi::OsStr, path::Path};

use auth::AuthState;
use axum::{http::header, middleware, routing::get, Router};
use include_dir::{include_dir, Dir};

mod auth;
mod tasks;

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");
static MIME_JAVASCRIPT: &'_ str = "text/javascript";
static MIME_CSS: &'_ str = "text/css";
static MIME_HTML: &'_ str = "text/html";
static MIME_WOFF2: &'_ str = "font/woff2";
static MIME_OTHER: &'_ str = "application/octet-stream";

fn apply_routes(mut router: Router, dir: &'static Dir) -> Router {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(dir) => router = apply_routes(router, dir),
            include_dir::DirEntry::File(file) => {
                let mimetype = if file.path().extension() == Some(OsStr::new("js")) {
                    MIME_JAVASCRIPT.to_string()
                } else if file.path().extension() == Some(OsStr::new("css")) {
                    MIME_CSS.to_string()
                } else if file.path().extension() == Some(OsStr::new("woff2")) {
                    MIME_WOFF2.to_string()
                } else if file.path().extension() == Some(OsStr::new("html")) {
                    MIME_HTML.to_string()
                } else {
                    MIME_OTHER.to_string()
                };
                router = router.route(
                    &format!("/{}", entry.path().to_str().unwrap()),
                    get(move || async { ([(header::CONTENT_TYPE, mimetype)], file.contents()) }),
                );
                if file.path() == Path::new("index.html") {
                    router = router.route(
                        "/",
                        get(|| async { ([(header::CONTENT_TYPE, MIME_HTML)], file.contents()) }),
                    )
                }
            }
        }
    }
    router
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let auth_state = AuthState::from_path("data/auth.db").await;

    let app = Router::new();
    let app = apply_routes(app, &ASSETS);
    let app = app
        .nest(
            "/api/tasks",
            tasks::routes()
                .await
                .route_layer(middleware::from_fn_with_state(
                    auth_state.clone(),
                    auth::login_middleware,
                )),
        )
        .nest("/api/auth", auth::routes(auth_state.clone()).await)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
