use std::{ffi::OsStr, path::Path};

use axum::{http::header, routing::get, Router};
use include_dir::{include_dir, Dir};

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");
static MIME_JAVASCRIPT: &'_ str = "text/javascript";
static MIME_CSS: &'_ str = "text/css";
static MIME_HTML: &'_ str = "text/html";
static MIME_WOFF2: &'_ str = "font/woff2";
static MIME_OTHER: &'_ str = "application/octet-stream";

static CACHE_FOREVER: &'_ str = "public, max-age=31536000, s-maxage=31536000, immutable";
static CACHE_NONE: &'_ str = "no-cache";

pub fn routes() -> Router {
    apply_routes(Router::new(), &ASSETS)
}

fn apply_routes(mut router: Router, dir: &'static Dir) -> Router {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(dir) => router = apply_routes(router, dir),
            include_dir::DirEntry::File(file) => {
                let headers = if file.path().extension() == Some(OsStr::new("js")) {
                    [
                        (header::CONTENT_TYPE, MIME_JAVASCRIPT),
                        (header::CACHE_CONTROL, CACHE_FOREVER),
                    ]
                } else if file.path().extension() == Some(OsStr::new("css")) {
                    [
                        (header::CONTENT_TYPE, MIME_CSS),
                        (header::CACHE_CONTROL, CACHE_FOREVER),
                    ]
                } else if file.path().extension() == Some(OsStr::new("woff2")) {
                    [
                        (header::CONTENT_TYPE, MIME_WOFF2),
                        (header::CACHE_CONTROL, CACHE_FOREVER),
                    ]
                } else if file.path().extension() == Some(OsStr::new("html")) {
                    [
                        (header::CONTENT_TYPE, MIME_HTML),
                        (header::CACHE_CONTROL, CACHE_NONE),
                    ]
                } else {
                    [
                        (header::CONTENT_TYPE, MIME_OTHER),
                        (header::CACHE_CONTROL, CACHE_NONE),
                    ]
                };
                router = router.route(
                    &format!("/{}", entry.path().to_str().unwrap()),
                    get(move || async { (headers, file.contents()) }),
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
