use std::path::PathBuf;

use chrono::{Duration, Local, NaiveDate};
use tokio::fs;

use super::types::{Routine, Task};

#[derive(thiserror::Error, Debug)]
pub enum TaskStoreError {
    // 500 type errors (it's probably our fault)
    #[error("underlying data could not be accessed or saved")]
    FileIo(#[from] tokio::io::Error),
    #[error("underlying data is corrupted")]
    FileInvalidFormat(#[from] toml::de::Error),

    // 400 type errors (it's probably your fault)
    #[error("unknown task name was used")]
    UnknownTaskName(String),
    #[error("person not in task rota")]
    PersonNotInTaskRota(String),
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
struct DurationSpec {
    weeks: u16,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct Completion {
    date: NaiveDate,
    by: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SavedTask {
    name: String,
    kind: Routine,
    participants: Vec<String>,
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
            length_days: task.duration.weeks * 7,
            last_completed: task.last_completed.date,
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

    async fn load(&self) -> Result<Vec<SavedTask>, TaskStoreError> {
        let contents = fs::read_to_string(&self.path).await?;
        let TaskToml { task } = toml::from_str(&contents)?;
        Ok(task)
    }

    async fn store(&self, tasks: Vec<SavedTask>) -> Result<(), TaskStoreError> {
        fs::write(&self.path, toml::to_vec(&TaskToml { task: tasks }).unwrap()).await?;
        Ok(())
    }

    pub async fn tasks(&self) -> Result<Vec<Task>, TaskStoreError> {
        Ok(self
            .load()
            .await?
            .into_iter()
            .map(SavedTask::into)
            .collect())
    }

    pub async fn tasks_for(&self, person: &str) -> Result<Vec<Task>, TaskStoreError> {
        Ok(self
            .load()
            .await?
            .into_iter()
            .filter(|task| task.assigned_to() == person)
            .map(SavedTask::into)
            .collect())
    }

    pub async fn mark_task_as_done_by(
        &self,
        task_name: &str,
        person: Option<&str>,
    ) -> Result<(), TaskStoreError> {
        let mut tasks = self.load().await?;
        if let Some(task) = tasks.iter_mut().find(|task| task.name == task_name) {
            if let Some(person) = person {
                if !task.participants.iter().any(|p| p == person) {
                    Err(TaskStoreError::PersonNotInTaskRota(person.into()))?
                }
            }
            match task.kind {
                Routine::Schedule => {
                    task.last_completed.date += Duration::weeks(task.duration.weeks.into());
                    task.last_completed.by = person.unwrap_or_else(|| task.assigned_to()).into()
                }
                Routine::Interval => {
                    task.last_completed = Completion {
                        date: Local::now().date_naive(),
                        by: person.unwrap_or_else(|| task.assigned_to()).into(),
                    };
                }
            }
        } else {
            Err(TaskStoreError::UnknownTaskName(task_name.into()))?
        }
        self.store(tasks).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use crate::tasks::types::Deadline;

    use super::*;

    fn file_with_tasks(tasks: Vec<SavedTask>) -> NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&toml::to_vec(&TaskToml { task: tasks }).unwrap())
            .unwrap();
        file
    }

    #[tokio::test]
    async fn test_returns_tasks_when_given_file_with_no_tasks() {
        let file = file_with_tasks(vec![]);
        let store = Store::from_file(file.path()).await;
        let tasks = store.tasks().await.unwrap();
        assert_eq!(tasks, vec![]);
    }

    #[tokio::test]
    async fn test_returns_tasks_when_given_file_with_tasks() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Interval,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(2)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        let tasks = store.tasks().await.unwrap();
        assert_eq!(
            tasks,
            vec![Task {
                name: "Test Task".into(),
                kind: Routine::Interval,
                assigned_to: "Bob".into(),
                deadline: Deadline::Upcoming(12),
                length_days: 14,
                last_completed: (Local::now() - Duration::days(2)).date_naive(),
            }]
        );
    }

    #[tokio::test]
    async fn test_returns_tasks_for_a_particular_person() {
        let file = file_with_tasks(vec![
            SavedTask {
                name: "Test Task 1".into(),
                kind: Routine::Interval,
                participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
                last_completed: Completion {
                    date: (Local::now() - Duration::days(2)).date_naive(),
                    by: "Kevin".into(), // Bob is next
                },
                duration: DurationSpec { weeks: 1 },
            },
            SavedTask {
                name: "Test Task 2".into(),
                kind: Routine::Interval,
                participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
                last_completed: Completion {
                    date: (Local::now() - Duration::days(2)).date_naive(),
                    by: "Bob".into(), // Samantha is next
                },
                duration: DurationSpec { weeks: 1 },
            },
            SavedTask {
                name: "Test Task 3".into(),
                kind: Routine::Interval,
                participants: vec!["Bob".into(), "Kevin".into(), "Samantha".into()],
                last_completed: Completion {
                    date: (Local::now() - Duration::days(2)).date_naive(),
                    by: "Bob".into(), // Kevin is next
                },
                duration: DurationSpec { weeks: 1 },
            },
        ]);

        let store = Store::from_file(file.path()).await;
        let (tasks_bob, tasks_kevin, tasks_samantha) = tokio::join!(
            store.tasks_for("Bob"),
            store.tasks_for("Kevin"),
            store.tasks_for("Samantha")
        );

        assert_eq!(
            tasks_bob.unwrap(),
            vec![Task {
                name: "Test Task 1".into(),
                kind: Routine::Interval,
                assigned_to: "Bob".into(),
                deadline: Deadline::Upcoming(5),
                length_days: 7,
                last_completed: (Local::now() - Duration::days(2)).date_naive(),
            }]
        );
        assert_eq!(
            tasks_kevin.unwrap(),
            vec![Task {
                name: "Test Task 3".into(),
                kind: Routine::Interval,
                assigned_to: "Kevin".into(),
                deadline: Deadline::Upcoming(5),
                length_days: 7,
                last_completed: (Local::now() - Duration::days(2)).date_naive(),
            }]
        );
        assert_eq!(
            tasks_samantha.unwrap(),
            vec![Task {
                name: "Test Task 2".into(),
                kind: Routine::Interval,
                assigned_to: "Samantha".into(),
                deadline: Deadline::Upcoming(5),
                length_days: 7,
                last_completed: (Local::now() - Duration::days(2)).date_naive(),
            }]
        );
    }

    #[tokio::test]
    async fn test_correctly_updates_partially_completed_interval_task() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Interval,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(2)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        store.mark_task_as_done_by("Test Task", None).await.unwrap();
        assert_eq!(
            store.tasks().await.unwrap(),
            vec![Task {
                name: "Test Task".into(),
                kind: Routine::Interval,
                assigned_to: "Samantha".into(),
                deadline: Deadline::Upcoming(14),
                length_days: 14,
                last_completed: (Local::now()).date_naive(),
            }]
        );
    }

    #[tokio::test]
    async fn test_correctly_updates_partially_completed_interval_task_with_given_person() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Interval,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(2)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        store
            .mark_task_as_done_by("Test Task", Some("Kevin"))
            .await
            .unwrap();
        assert_eq!(
            store.tasks().await.unwrap(),
            vec![Task {
                name: "Test Task".into(),
                kind: Routine::Interval,
                assigned_to: "Bob".into(),
                deadline: Deadline::Upcoming(14),
                length_days: 14,
                last_completed: (Local::now()).date_naive(),
            }]
        );
    }

    #[tokio::test]
    async fn test_correctly_updates_partially_completed_schedule_task() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Schedule,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(10)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        store.mark_task_as_done_by("Test Task", None).await.unwrap();
        assert_eq!(
            store.tasks().await.unwrap(),
            vec![Task {
                name: "Test Task".into(),
                kind: Routine::Schedule,
                assigned_to: "Samantha".into(),
                deadline: Deadline::Upcoming(18), // = 4 (days remaining of original task) + 14 (length of task)
                length_days: 14,
                last_completed: (Local::now() + Duration::days(4)).date_naive(),
            }]
        );
    }

    #[tokio::test]
    async fn test_correctly_updates_overdue_schedule_task() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Schedule,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(18)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        store.mark_task_as_done_by("Test Task", None).await.unwrap();
        assert_eq!(
            store.tasks().await.unwrap(),
            vec![Task {
                name: "Test Task".into(),
                kind: Routine::Schedule,
                assigned_to: "Samantha".into(),
                deadline: Deadline::Upcoming(10), // = 14 (length of task) - 4 (days remaining of original task)
                length_days: 14,
                last_completed: (Local::now() - Duration::days(4)).date_naive(),
            }]
        );
    }

    #[tokio::test]
    async fn test_returns_error_if_no_task_can_be_found() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Schedule,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(10)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        let error = store
            .mark_task_as_done_by("Unknown Task", None)
            .await
            .unwrap_err();

        match error {
            TaskStoreError::UnknownTaskName(name) => assert_eq!(name, "Unknown Task"),
            _ => panic!("Incorrect error kind, {error:?}"),
        }
    }

    #[tokio::test]
    async fn test_returns_error_if_the_given_user_does_not_belong_to_task() {
        let file = file_with_tasks(vec![SavedTask {
            name: "Test Task".into(),
            kind: Routine::Schedule,
            participants: vec!["Kevin".into(), "Bob".into(), "Samantha".into()],
            last_completed: Completion {
                date: (Local::now() - Duration::days(10)).date_naive(),
                by: "Kevin".into(),
            },
            duration: DurationSpec { weeks: 2 },
        }]);
        let store = Store::from_file(file.path()).await;
        let error = store
            .mark_task_as_done_by("Test Task", Some("Edgar"))
            .await
            .unwrap_err();

        match error {
            TaskStoreError::PersonNotInTaskRota(name) => assert_eq!(name, "Edgar"),
            _ => panic!("Incorrect error kind, {error:?}"),
        }
    }
}
