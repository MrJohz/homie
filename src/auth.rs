use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, routing::post, Json, Router};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Token(String);

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    // 500 type errors (it's probably our fault)
    #[error("underlying data could not be accessed or saved")]
    FileIo(#[from] sqlx::Error),

    // 400 type errors (it's probably your fault)
    #[error("user/password mismatch")]
    UserPasswordMismatch,
}

#[derive(Clone)]
struct AuthState {
    conn: SqlitePool,
    hasher: Argon2<'static>,
}

impl AuthState {
    async fn from_path(path: impl AsRef<str>) -> Self {
        // See https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
        let mut argon2_params = argon2::ParamsBuilder::new();
        argon2_params.m_cost(15360).unwrap();
        argon2_params.t_cost(2).unwrap();
        argon2_params.p_cost(1).unwrap();
        let argon2_params = argon2_params.params().unwrap();

        Self {
            conn: SqlitePool::connect(&format!("sqlite://{}", path.as_ref()))
                .await
                .unwrap(),
            hasher: Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                argon2_params,
            ),
        }
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

        let token = Uuid::new_v4();
        let x = sqlx::query_as::<_, (String,)>(
            "INSERT INTO tokens (username, token, expiry) VALUES (?, ?, ?)",
        )
        .bind(username)
        .bind(token.into_bytes().as_slice())
        .bind("todo")
        .fetch_one(&self.conn)
        .await?;

        dbg!(x);

        Ok(Token("".into()))
    }
}

#[derive(Debug, serde::Deserialize)]
struct LoginArgs {
    username: String,
    password: String,
}

async fn login(State(auth): State<AuthState>, args: Json<LoginArgs>) -> Json<Token> {
    let token = auth.login(&args.username, &args.password).await;
    dbg!(token);
    Json(Token("".into()))
}

async fn refresh_token(token: Json<Token>) -> Json<Token> {
    Json(Token("".into()))
}

pub async fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .with_state(AuthState::from_path("data/auth.db").await)
}
