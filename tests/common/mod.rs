use std::net::SocketAddr;

use reqwest::Client;
use sqlx::SqlitePool;
use tempfile::{tempdir, TempDir};
use tokio::task::JoinHandle;

use homie::{auth, db, server};

#[allow(dead_code)]
pub async fn harness() -> TestHarness {
    let file_handle = tempdir().unwrap();
    let conn = db::create_connection_in_location(file_handle.path()).await;
    db::migrate(&conn).await.unwrap();
    let server = axum::Server::bind(&"127.0.0.1:0".parse().unwrap()).serve(server(conn.clone()));
    let addr = server.local_addr();

    TestHarness {
        conn,
        addr,
        token: None,
        client: Client::new(),
        _file_handle: file_handle,
        _server_handle: tokio::spawn(async {
            server.await.unwrap();
        }),
    }
}

#[allow(dead_code)]
pub async fn harness_with_token() -> TestHarness {
    let mut harness = harness().await;
    let auth = harness.auth_store();

    auth.create_user("__test_user", "").await.unwrap();
    let token = auth.login("__test_user", "").await.unwrap();
    harness.token = Some(token);
    harness
}

pub struct TestHarness {
    conn: SqlitePool,
    addr: SocketAddr,
    token: Option<auth::Token>,
    client: Client,
    _file_handle: TempDir,
    _server_handle: JoinHandle<()>,
}

impl TestHarness {
    pub fn auth_store(&self) -> auth::AuthStore {
        auth::AuthStore::new(self.conn.clone())
    }
    pub fn request(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
    ) -> reqwest::RequestBuilder {
        let builder = self
            .client
            .request(method, format!("http://{}{}", self.addr, path.as_ref()));
        match &self.token {
            Some(token) => builder.header("token", token),
            None => builder,
        }
    }
}
