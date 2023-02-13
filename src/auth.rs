// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Debug;

use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

mod store;
mod types;

pub use store::AuthStore;
pub use types::{AuthError, Token};

#[derive(Debug, serde::Deserialize)]
struct LoginArgs {
    username: String,
    password: String,
}

async fn login(
    State(auth): State<AuthStore>,
    args: Json<LoginArgs>,
) -> Result<Json<Token>, AuthError> {
    let token = auth.login(&args.username, &args.password).await?;
    Ok(Json(token))
}

pub fn routes(auth_state: AuthStore) -> Router {
    Router::new()
        .route("/login", post(login))
        .with_state(auth_state)
}

async fn evaluate_token<B: Debug>(auth: &AuthStore, request: &Request<B>) -> Result<(), AuthError> {
    let token = request
        .headers()
        .get("token")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = token.parse()?;
    auth.validate_token(&token).await?;
    Ok(())
}

pub async fn login_middleware<B: Debug>(
    State(auth): State<AuthStore>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    if let Err(error) = evaluate_token(&auth, &request).await {
        return error.into_response();
    }

    next.run(request).await
}
