use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use super::{store::TaskStoreError, types::Task, Store};

impl IntoResponse for TaskStoreError {
    fn into_response(self) -> axum::response::Response {
        match self {
            TaskStoreError::FileIo(_) | TaskStoreError::FileInvalidFormat(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            TaskStoreError::UnknownTaskName(_) | TaskStoreError::PersonNotInTaskRota(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}

#[axum_macros::debug_handler]
async fn list_all_tasks(State(store): State<Store>) -> Result<Json<Vec<Task>>, TaskStoreError> {
    store.tasks().await.map(Json)
}

async fn tasks_for_person(
    Path(person): Path<String>,
    State(store): State<Store>,
) -> Result<Json<Vec<Task>>, TaskStoreError> {
    store.tasks_for(&person).await.map(Json)
}

#[derive(Debug, serde::Deserialize)]
struct MarkTaskDoneQuery {
    by: Option<String>,
}

async fn mark_task_done(
    Path(task): Path<String>,
    Query(query): Query<MarkTaskDoneQuery>,
    State(store): State<Store>,
) -> Result<Json<Task>, TaskStoreError> {
    let task = match query.by {
        Some(q) => store.mark_task_as_done_by(&task, Some(q.as_str())).await?,
        None => store.mark_task_as_done_by(&task, None).await?,
    };
    Ok(Json(task))
}

pub async fn routes() -> Router {
    Router::new()
        .route("/", get(list_all_tasks))
        .route("/people/:person", get(tasks_for_person))
        .route("/actions/mark_task_done/:task", post(mark_task_done))
        .with_state(super::Store::from_file("data/tasks.toml").await)
}
