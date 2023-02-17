// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

use axum::{middleware, routing::IntoMakeService, Router};
use sqlx::SqlitePool;

pub mod auth;
pub mod db;
pub mod static_files;
pub mod tasks;
mod translations;

pub fn server(conn: SqlitePool) -> IntoMakeService<Router> {
    let auth = auth::AuthStore::new(conn.clone());

    let app = Router::new();
    let app = app
        .merge(static_files::routes())
        .nest(
            "/api/tasks",
            tasks::routes(conn).route_layer(middleware::from_fn_with_state(
                auth.clone(),
                auth::login_middleware,
            )),
        )
        .nest("/api/auth", auth::routes(auth))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    app.into_make_service()
}
