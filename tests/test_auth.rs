// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

use reqwest::Method;
mod common;

#[tokio::test]
async fn login_succeeds() {
    let server = common::harness().await;

    server
        .auth_store()
        .create_user("hello", "password")
        .await
        .unwrap();

    let token = server
        .request(Method::POST, "/api/auth/login")
        .json(&serde_json::json!({"username": "hello", "password": "password"}))
        .send()
        .await
        .unwrap()
        .json::<String>()
        .await
        .unwrap();

    assert!(token.parse::<uuid::Uuid>().is_ok());
}
