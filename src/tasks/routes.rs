use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use sqlx::SqlitePool;

use super::{store::TaskStoreError, time::today, types::Task, TaskStore};

impl IntoResponse for TaskStoreError {
    fn into_response(self) -> axum::response::Response {
        match self {
            TaskStoreError::DbError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            TaskStoreError::UnknownTaskName(_) | TaskStoreError::PersonDoesNotExist(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}

async fn list_all_tasks(State(store): State<TaskStore>) -> Result<Json<Vec<Task>>, TaskStoreError> {
    store.tasks().await.map(Json)
}

async fn tasks_for_person(
    Path(person): Path<String>,
    State(store): State<TaskStore>,
) -> Result<Json<Vec<Task>>, TaskStoreError> {
    store.tasks_for(&person).await.map(Json)
}

#[derive(Debug, serde::Deserialize)]
struct MarkTaskDoneQuery {
    by: String,
    on: Option<NaiveDate>,
}

async fn mark_task_done(
    Path(task): Path<String>,
    Query(query): Query<MarkTaskDoneQuery>,
    State(store): State<TaskStore>,
) -> Result<Json<Task>, TaskStoreError> {
    let task = store
        .mark_task_done(&task, &query.by, &query.on.unwrap_or_else(today))
        .await?;
    Ok(Json(task))
}

pub fn routes(conn: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_all_tasks))
        .route("/people/:person", get(tasks_for_person))
        .route("/actions/mark_task_done/:task", post(mark_task_done))
        .with_state(TaskStore::new(conn))
}
