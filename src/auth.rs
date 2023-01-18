use std::{fmt::Debug, str::FromStr};

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use rand_core::OsRng;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Token(String);

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    // 500 type errors (it's probably our fault)
    #[error("underlying data could not be accessed or saved")]
    DbError(#[from] sqlx::Error),

    // 400 type errors (it's probably your fault)
    #[error("user/password mismatch")]
    UserPasswordMismatch,
    #[error("unknown token")]
    UnknownToken,
    #[error("missing token")]
    MissingToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthError::DbError(ref err) => {
                tracing::error!({ details = &err.to_string() }, "DB connection error");
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            AuthError::UserPasswordMismatch | AuthError::UnknownToken | AuthError::MissingToken => {
                tracing::warn!({ details = self.to_string() }, "Authentication failure");
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}

#[derive(Clone)]
pub struct AuthState {
    conn: SqlitePool,
    hasher: Argon2<'static>,
}

impl AuthState {
    pub async fn from_path(path: impl AsRef<str>) -> Self {
        // See https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
        let mut argon2_params = argon2::ParamsBuilder::new();
        argon2_params.m_cost(15360).unwrap();
        argon2_params.t_cost(2).unwrap();
        argon2_params.p_cost(1).unwrap();
        let argon2_params = argon2_params.params().unwrap();

        Self {
            conn: SqlitePool::connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}", path.as_ref()))
                    .unwrap()
                    .create_if_missing(true),
            )
            .await
            .unwrap(),
            hasher: Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                argon2_params,
            ),
        }
    }

    async fn setup(&self, users: Vec<(String, String)>) -> Result<(), AuthError> {
        sqlx::query("CREATE TABLE IF NOT EXISTS users (username string primary key, hash string)")
            .execute(&self.conn)
            .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tokens (token string, username string REFERENCES users (username))",
        )
        .execute(&self.conn)
        .await?;

        for (username, password) in users {
            let (value,) =
                sqlx::query_as::<_, (u8,)>("SELECT COUNT(*) FROM users WHERE username = ?")
                    .bind(&username)
                    .fetch_one(&self.conn)
                    .await?;
            if value > 0 {
                continue;
            }

            sqlx::query("INSERT INTO users (username, hash) VALUES (?, ?)")
                .bind(&username)
                .bind(
                    &self
                        .hasher
                        .hash_password(password.as_bytes(), &SaltString::generate(OsRng))
                        .unwrap()
                        .to_string(),
                )
                .execute(&self.conn)
                .await?;
        }

        Ok(())
    }

    async fn login(&self, username: &str, password: &str) -> Result<Token, AuthError> {
        let stored_hash =
            sqlx::query_as::<_, (String,)>("SELECT hash FROM users WHERE username = ?")
                .bind(username)
                .fetch_optional(&self.conn)
                .await?;

        match stored_hash {
            Some((hash,)) => self
                .hasher
                .verify_password(
                    password.as_bytes(),
                    &PasswordHash::new(&hash).map_err(|_| AuthError::UserPasswordMismatch)?,
                )
                .map_err(|_| AuthError::UserPasswordMismatch)?,
            None => Err(AuthError::UserPasswordMismatch)?,
        };

        let token = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO tokens (username, token) VALUES (?, ?)")
            .bind(username)
            .bind(&token)
            .execute(&self.conn)
            .await?;

        Ok(Token(token))
    }

    async fn validate_token(&self, token: &str) -> Result<(), AuthError> {
        let (found,) = sqlx::query_as::<_, (u8,)>("SELECT COUNT(*) FROM tokens WHERE token = ?")
            .bind(token)
            .fetch_one(&self.conn)
            .await?;

        if found > 0 {
            Ok(())
        } else {
            Err(AuthError::UnknownToken)
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct LoginArgs {
    username: String,
    password: String,
}

async fn login(
    State(auth): State<AuthState>,
    args: Json<LoginArgs>,
) -> Result<Json<Token>, AuthError> {
    let token = auth.login(&args.username, &args.password).await?;
    Ok(Json(token))
}

pub async fn routes(auth_state: AuthState) -> Router {
    auth_state
        .setup(vec![("Test User".into(), "testpw123".into())])
        .await
        .unwrap();

    Router::new()
        .route("/login", post(login))
        .with_state(auth_state)
}

async fn evaluate_token<B: Debug>(auth: &AuthState, request: &Request<B>) -> Result<(), AuthError> {
    let token = request
        .headers()
        .get("token")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    auth.validate_token(token).await?;
    Ok(())
}

pub async fn login_middleware<B: Debug>(
    State(auth): State<AuthState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    if let Err(error) = evaluate_token(&auth, &request).await {
        return error.into_response();
    }

    let response = next.run(request).await;
    response
}
