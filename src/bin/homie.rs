#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let conn = homie::db::create_connection().await;

    axum::Server::bind(&"0.0.0.0:3030".parse().unwrap())
        .serve(homie::server(conn))
        .await
        .unwrap();
}
