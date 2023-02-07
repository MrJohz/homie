use std::path::Path;

use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn create_connection() -> SqlitePool {
    sqlx::SqlitePool::connect_with(
        "sqlite://data/homie.db"
            .parse::<SqliteConnectOptions>()
            .unwrap()
            .create_if_missing(true),
    )
    .await
    .unwrap()
}

pub async fn create_connection_in_location(location: impl AsRef<Path>) -> SqlitePool {
    sqlx::SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(location.as_ref().join("homie.db"))
            .create_if_missing(true),
    )
    .await
    .unwrap()
}

pub async fn migrate(conn: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(conn).await
}
