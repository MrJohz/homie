use argon2::{password_hash::SaltString, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::{AuthError, Token};

#[derive(Clone)]
pub struct AuthStore {
    conn: SqlitePool,
    hasher: argon2::Argon2<'static>,
}

impl AuthStore {
    pub fn new(conn: SqlitePool) -> Self {
        // See https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
        let mut argon2_params = argon2::ParamsBuilder::new();
        argon2_params.m_cost(15360).unwrap();
        argon2_params.t_cost(2).unwrap();
        argon2_params.p_cost(1).unwrap();
        let argon2_params = argon2_params.params().unwrap();

        Self {
            conn,
            hasher: argon2::Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                argon2_params,
            ),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Token, AuthError> {
        let id = username.to_lowercase();
        let stored_hash = sqlx::query_as::<_, (String,)>("SELECT hash FROM users WHERE id = ?")
            .bind(&id)
            .fetch_optional(&self.conn)
            .await?;

        match stored_hash {
            Some((hash,)) => self
                .hasher
                .verify_password(
                    password.as_bytes(),
                    &argon2::PasswordHash::new(&hash)
                        .map_err(|_| AuthError::UserPasswordMismatch)?,
                )
                .map_err(|_| AuthError::UserPasswordMismatch)?,
            None => Err(AuthError::UserPasswordMismatch)?,
        };

        let token = Token::from_uuid(Uuid::new_v4());
        sqlx::query("INSERT INTO tokens (id, token) VALUES (?, ?)")
            .bind(&id)
            .bind(&token)
            .execute(&self.conn)
            .await?;

        Ok(token)
    }

    pub async fn validate_token(&self, token: &Token) -> Result<(), AuthError> {
        let (found,) = sqlx::query_as::<_, (u8,)>("SELECT COUNT(*) FROM tokens WHERE token = ?")
            .bind(token)
            .fetch_one(&self.conn)
            .await?;

        if found > 0 {
            Ok(())
        } else {
            Err(AuthError::UnknownToken(None))
        }
    }

    pub async fn create_user(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let hash = self
            .hasher
            .hash_password(password.as_bytes(), &SaltString::generate(OsRng))
            .unwrap()
            .to_string();

        sqlx::query("INSERT INTO users (id, username, hash) VALUES (?, ?, ?)")
            .bind(username.to_lowercase())
            .bind(username)
            .bind(&hash)
            .execute(&self.conn)
            .await?;

        Ok(())
    }
}
