use axum::{extract::State, routing::get, Json, Router};

use super::{types::Task, Store};

#[axum_macros::debug_handler]
async fn list_all_tasks(State(store): State<Store>) -> Json<Vec<Task>> {
    Json(store.tasks().await.to_vec())
}

pub async fn routes() -> Router {
    Router::new()
        .route("/", get(list_all_tasks))
        .with_state(super::Store::from_file("data/tasks.toml").await)
}
