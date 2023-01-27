use std::str::FromStr;

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Encode, sqlx::Decode)]
pub struct Token(uuid::Uuid);

impl sqlx::Type<sqlx::Sqlite> for Token {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <uuid::Uuid as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl Token {
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl FromStr for Token {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    // 500 type errors (it's probably our fault)
    #[error("underlying data could not be accessed or saved")]
    DbError(#[from] sqlx::Error),

    // 400 type errors (it's probably your fault)
    #[error("user/password mismatch")]
    UserPasswordMismatch,
    #[error("unknown token")]
    UnknownToken(#[from] Option<uuid::Error>),
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
            AuthError::UserPasswordMismatch
            | AuthError::UnknownToken(_)
            | AuthError::MissingToken => {
                tracing::warn!({ details = self.to_string() }, "Authentication failure");
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}
