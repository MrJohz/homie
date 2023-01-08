use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
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

#[derive(Debug, serde::Deserialize)]
struct MarkTaskDoneQuery {
    by: Option<String>,
}

async fn mark_task_done(
    Path(task): Path<String>,
    Query(query): Query<MarkTaskDoneQuery>,
    State(store): State<Store>,
) -> Json<Vec<Task>> {
    match query.by {
        Some(q) => store.mark_task_as_done_by(&task, Some(q.as_str())).await,
        None => store.mark_task_as_done_by(&task, None).await,
    }
    Json(store.tasks().await)
}

pub async fn routes() -> Router {
    Router::new()
        .route("/", get(list_all_tasks))
        .route("/people/:person", get(tasks_for_person))
        .route("/actions/mark_task_done/:task", post(mark_task_done))
        .with_state(super::Store::from_file("data/tasks.toml").await)
}
