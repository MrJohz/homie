use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use super::{types::Task, Store};

async fn list_all_tasks(State(store): State<Store>) -> Json<Vec<Task>> {
    Json(store.tasks().await)
}

async fn tasks_for_person(
    Path(person): Path<String>,
    State(store): State<Store>,
) -> Json<Vec<Task>> {
    Json(store.tasks_for(&person).await)
}

pub async fn routes() -> Router {
    Router::new()
        .route("/", get(list_all_tasks))
        .route("/for/:person", get(tasks_for_person))
        .with_state(super::Store::from_file("data/tasks.toml").await)
}
