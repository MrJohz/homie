use std::path::PathBuf;

use chrono::{Datelike, Duration, Local, NaiveDate};
use sqlx::SqlitePool;
use tokio::fs;

use super::types::{Deadline, Routine, Task};

#[derive(thiserror::Error, Debug)]
pub enum TaskStoreError {
    // 500 type errors (it's probably our fault)
    #[error("underlying data could not be accessed or saved")]
    FileIo(#[from] tokio::io::Error),
    #[error("underlying data is corrupted")]
    FileInvalidFormat(#[from] toml::de::Error),
    #[error("underlying data could not be accessed or saved")]
    DbError(#[from] sqlx::Error),

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
            participants: task
                .participants
                .into_iter()
                .map(|p| p.as_str().into())
                .collect(),
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

    async fn store(&self, tasks: &[SavedTask]) -> Result<(), TaskStoreError> {
        fs::write(
            &self.path,
            toml::to_string(&TaskToml { task: tasks.into() })
                .unwrap()
                .as_bytes(),
        )
        .await?;
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
    ) -> Result<Task, TaskStoreError> {
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
            let ret = task.clone().into();
            self.store(tasks.as_slice()).await?;
            Ok(ret)
        } else {
            Err(TaskStoreError::UnknownTaskName(task_name.into()))?
        }
    }
}

#[derive(Clone)]
pub struct TaskStore {
    conn: SqlitePool,
}

impl TaskStore {
    pub fn new(conn: SqlitePool) -> Self {
        Self { conn }
    }

    pub async fn add_task(&self, mut new_task: NewTask) -> Result<(), TaskStoreError> {
        let mut transaction = self.conn.begin().await?;

        let task_id = new_task.name.to_lowercase();

        new_task.starts_with = new_task.starts_with.to_lowercase();
        for p in new_task.participants.iter_mut() {
            *p = p.to_lowercase()
        }

        sqlx::query("INSERT INTO tasks (id, task_name, kind, duration) VALUES (?, ?, ?, ?)")
            .bind(&task_id)
            .bind(new_task.name)
            .bind(match new_task.routine {
                Routine::Schedule => "SCHEDULE",
                Routine::Interval => "INTERVAL",
            })
            .bind(new_task.duration)
            .execute(&mut transaction)
            .await?;

        for person in &new_task.participants {
            sqlx::query("INSERT INTO task_user_link (task_id, user_id) VALUES (?, ?)")
                .bind(&task_id)
                .bind(person)
                .execute(&mut transaction)
                .await?;
        }

        let pseudo_completion_date = new_task.starts_on - Duration::days(new_task.duration.into());
        let pseudo_completed_by = new_task
            .participants
            .iter()
            .position(|p| p == &new_task.starts_with);
        let pseudo_completed_by = match pseudo_completed_by {
            None => return Err(TaskStoreError::PersonNotInTaskRota(new_task.starts_with)),
            Some(0) => new_task.participants.len() - 1,
            Some(n) => n - 1,
        };
        let pseudo_completed_by = &new_task.participants[pseudo_completed_by];

        sqlx::query("INSERT INTO completions (task_id, completed_by, completed_year, completed_month, completed_day) VALUES (?, ?, ?, ?, ?)")
            .bind(&task_id)
            .bind(pseudo_completed_by.to_lowercase())
            .bind(pseudo_completion_date.year())
            .bind(pseudo_completion_date.month())
            .bind(pseudo_completion_date.day())
            .execute(&mut transaction).await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn tasks(&self) -> Result<Vec<Task>, TaskStoreError> {
        let rows = sqlx::query_as::<_, (String, String, Routine, u16, String, i32, u32, u32)>("SELECT (task_name, task.id, kind, duration, completed_by, username, completed_month, completed_day) FROM tasks INNER JOIN completions ON tasks.id = completions.task_id INNER JOIN users ON users.id = completions.completed_by").fetch_all(&self.conn).await?;
        let mut results = Vec::with_capacity(rows.len());
        for (name, id, kind, duration, completed_by, year, month, day) in rows {
            let participants = sqlx::query_as::<_, (String,)>("SELECT username FROM tasks INNER JOIN task_user_link ON tasks.id = task_user_link.task_id INNER JOIN users ON user_id = users.id WHERE tasks.id = ?")
            .bind(id).fetch_all(&self.conn).await?;
            let participants = participants.into_iter().map(|p| p.0).collect::<Vec<_>>();
            let last_completed = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            results.push(Task {
                name: name.as_str().into(),
                kind,
                length_days: duration,
                participants: participants.iter().map(|p| p.as_str().into()).collect(),
                last_completed,
                assigned_to: next_assignee(participants.as_slice(), &completed_by).into(),
                deadline: next_deadline(duration, last_completed),
            })
        }
        Ok(results)
    }

    pub async fn tasks_for(&self, person: &str) -> Result<Vec<Task>, TaskStoreError> {
        Ok(self
            .tasks()
            .await?
            .into_iter()
            .filter(|task| task.assigned_to == person)
            .collect())
    }

    pub async fn task(&self, task_name: &str) -> Result<Task, TaskStoreError> {
        let results = sqlx::query_as::<_, (String,Routine,u16,i32,u32,u32,String,String)>("
        SELECT
            task_name, kind, duration, completed_year, completed_month, completed_day, task_user.username, complete_user.username
        FROM
            tasks
            INNER JOIN completions ON tasks.id = completions.task_id
            INNER JOIN task_user_link ON tasks.id = task_user_link.task_id
            INNER JOIN users task_user on task_user_link.user_id = task_user.id
            INNER JOIN users complete_user on completions.completed_by = complete_user.id
        WHERE
            tasks.id = ?")
            .bind(task_name.to_lowercase())
            .fetch_all(&self.conn)
            .await?;

        if results.is_empty() {
            return Err(TaskStoreError::UnknownTaskName(task_name.into()));
        }

        let last_completed =
            NaiveDate::from_ymd_opt(results[0].3, results[0].4, results[0].5).unwrap();

        let mut task = Task {
            name: results[0].0.as_str().into(),
            kind: results[0].1,
            length_days: results[0].2,
            last_completed,
            deadline: next_deadline(results[0].2, last_completed),
            participants: Default::default(),
            assigned_to: Default::default(),
        };

        let last_completed = results[0].7.clone();
        let participants = results.into_iter().map(|each| each.6).collect::<Vec<_>>();
        task.assigned_to = next_assignee(participants.as_slice(), &last_completed).into();
        task.participants = participants;

        Ok(task)
    }

    // pub async fn mark_task_as_done_by(
    //     &self,
    //     task_name: &str,
    //     person: &str,
    // ) -> Result<Task, TaskStoreError> {
    //     let task_id = task_name.to_lowercase();
    //     let person_id = person.to_lowercase();
    //     let task = self.task(&task_id).await?;
    //     sqlx::query("INSERT INTO completions (task_id, completed_by, completed_year, completed_month, completed_day VALUES (?, ?, ?, ?, ?)")
    //         .bind(&task_id)
    //         .bind(&person_id)
    //     let task = sqlx::query_as::<_, (i32,)>("SELECT
    //     task_name, kind, duration, completed_year, completed_month, completed_day, task_user.username, complete_user.username
    //   FROM
    //     tasks
    //     INNER JOIN completions ON tasks.id = completions.task_id
    //     INNER JOIN task_user_link ON tasks.id = task_user_link.task_id
    //     INNER JOIN users task_user on task_user_link.user_id = task_user.id
    //     INNER JOIN users complete_user on completions.completed_by = complete_user.id
    //   WHERE
    //     tasks.id = ?")
    //     .bind(task_name.to_lowercase())
    //         .fetch_one(&self.conn)
    //         .await?;

    //     Err(TaskStoreError::UnknownTaskName("Keving".into()))
    // }
}

fn next_assignee<'a>(participants: &'a [String], last_completed_by: &str) -> &'a str {
    let mut participants_iter = participants.iter();
    while let Some(person) = participants_iter.next() {
        if person == last_completed_by {
            return participants_iter.next().unwrap_or(&participants[0]);
        }
    }

    unreachable!("Invalid Data")
}

fn next_deadline(task_length: u16, last_completed: NaiveDate) -> Deadline {
    (last_completed + Duration::days(task_length.into()) - Local::now().date_naive()).into()
}

pub struct NewTask {
    pub name: String,
    pub routine: Routine,
    pub duration: u16,
    pub participants: Vec<String>,
    pub starts_on: NaiveDate,
    pub starts_with: String,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use crate::tasks::types::Deadline;

    use super::*;

    fn file_with_tasks(tasks: Vec<SavedTask>) -> NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(
            toml::to_string(&TaskToml { task: tasks })
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        file
    }

    fn vec(v: Vec<impl Into<String>>) -> Vec<String> {
        v.into_iter().map(|i| i.into()).collect()
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
                participants: vec(vec!["Bob", "Kevin", "Samantha"]),
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
                participants: vec(vec!["Kevin", "Bob", "Samantha"]),
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
