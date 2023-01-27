#[tokio::main]
async fn main() {
    let conn = homie::db::create_connection().await;
    homie::db::migrate(conn).await.unwrap();
}
