use std::path::PathBuf;

use chrono::{Duration, Local, NaiveDate};
use tokio::fs;

use super::types::{Routine, Task};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct DurationSpec {
    weeks: u8,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Completion {
    date: NaiveDate,
    by: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SavedTask {
    name: String,
    kind: Routine,
    participants: heapless::Vec<String, 2>,
    last_completed: Completion,
    duration: DurationSpec,
}

impl SavedTask {
    pub fn assigned_to(&self) -> &str {
        let mut participants_iter = self.participants.iter();
        while let Some(person) = participants_iter.next() {
            if person == &self.last_completed.by {
                return participants_iter.next().unwrap_or(&self.participants[0]);
            }
        }

        unreachable!("Invalid Data")
    }
}

impl From<SavedTask> for Task {
    fn from(task: SavedTask) -> Self {
        Self {
            name: task.name.as_str().into(),
            kind: task.kind,
            assigned_to: task.assigned_to().into(),
            deadline: (task.last_completed.date + Duration::weeks(task.duration.weeks as i64)
                - Local::now().date_naive())
            .into(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TaskToml {
    task: Vec<SavedTask>,
}

#[derive(Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub async fn from_file(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    async fn load(&self) -> Vec<SavedTask> {
        let contents = fs::read_to_string(&self.path).await.unwrap();
        let TaskToml { task } = toml::from_str(&contents).unwrap();
        task
    }

    async fn store(&self, tasks: Vec<SavedTask>) {
        fs::write(&self.path, toml::to_vec(&TaskToml { task: tasks }).unwrap())
            .await
            .unwrap();
    }

    pub async fn tasks(&self) -> Vec<Task> {
        self.load().await.into_iter().map(SavedTask::into).collect()
    }

    pub async fn tasks_for(&self, person: &str) -> Vec<Task> {
        self.load()
            .await
            .into_iter()
            .filter(|task| task.assigned_to() == person)
            .map(SavedTask::into)
            .collect()
    }

    pub async fn mark_task_as_done_by(&self, task_name: &str, person: Option<&str>) {
        let mut tasks = self.load().await;
        if let Some(task) = tasks.iter_mut().find(|task| task.name == task_name) {
            task.last_completed = Completion {
                date: Local::now().date_naive(),
                by: person.unwrap_or_else(|| task.assigned_to()).into(),
            }
        }
        self.store(tasks).await;
    }
}
