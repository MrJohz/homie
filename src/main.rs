use axum::Router;

mod auth;
mod tasks;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // build our application with a single route
    let app = Router::new()
        .nest("/tasks", tasks::routes().await)
        .nest("/auth", auth::routes().await)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
