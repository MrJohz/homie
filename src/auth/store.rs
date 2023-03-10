// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

use argon2::{password_hash::SaltString, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::{AuthError, Token, UserId};

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
        let stored_hash = sqlx::query_as::<_, (UserId, String)>(
            "SELECT id, hash FROM users WHERE username = ? COLLATE NOCASE",
        )
        .bind(username)
        .fetch_optional(&self.conn)
        .await?;

        let id = match stored_hash {
            Some((id, hash)) => self
                .hasher
                .verify_password(
                    password.as_bytes(),
                    &argon2::PasswordHash::new(&hash)
                        .map_err(|_| AuthError::UserPasswordMismatch)?,
                )
                .map(|_| id)
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

        sqlx::query("INSERT INTO users (username, hash) VALUES (?, ?)")
            .bind(username)
            .bind(&hash)
            .execute(&self.conn)
            .await?;

        Ok(())
    }

    #[cfg(test)]
    pub async fn create_test_user(&self, username: &str) -> Result<(), AuthError> {
        sqlx::query("INSERT INTO users (username, hash) VALUES (?, ?)")
            .bind(username)
            .bind("test_user")
            .execute(&self.conn)
            .await?;

        Ok(())
    }
}
