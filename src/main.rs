use auth::AuthState;
use axum::{middleware, Router};

mod auth;
mod tasks;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let auth_state = AuthState::from_path("data/auth.db").await;

    // build our application with a single route
    let app = Router::new()
        .nest(
            "/tasks",
            tasks::routes()
                .await
                .route_layer(middleware::from_fn_with_state(
                    auth_state.clone(),
                    auth::login_middleware,
                )),
        )
        .nest("/auth", auth::routes(auth_state.clone()).await)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
