use std::{iter::Peekable, path::PathBuf};

use chrono::{Duration, Local, NaiveDate};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tokio::fs;

use super::{
    time::today,
    types::{Deadline, Routine, Task},
};

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

        let mut reversed_people = new_task.participants.clone();
        reversed_people.reverse();
        let prev_person = next_assignee(&reversed_people, &new_task.starts_with);

        let started_time = new_task.starts_on - Duration::days(new_task.duration.into());

        sqlx::query(
            "INSERT INTO tasks
                (id, task_name, kind, duration, first_done, start_assignee)
            VALUES
                (?, ?, ?, ?, ?, ?)",
        )
        .bind(&task_id)
        .bind(new_task.name)
        .bind(new_task.routine)
        .bind(new_task.duration)
        .bind(started_time)
        .bind(prev_person)
        .execute(&mut transaction)
        .await?;

        for person in &new_task.participants {
            sqlx::query(
                "INSERT INTO task_participant_link
                    (task_id, user_id)
                VALUES
                    (?, ?)",
            )
            .bind(&task_id)
            .bind(person)
            .execute(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn tasks(&self) -> Result<Vec<Task>, TaskStoreError> {
        let rows = sqlx::query(include_str!("./select_all_tasks.sql"))
            .fetch_all(&self.conn)
            .await?;

        let mut results = Vec::new();

        let mut rows = rows.into_iter().peekable();
        while let Some(task) = parse_task(&mut rows) {
            results.push(task);
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
        let rows = sqlx::query(include_str!("./select_one_task.sql"))
            .bind(task_name.to_lowercase())
            .fetch_all(&self.conn)
            .await?;

        let mut rows = rows.into_iter().peekable();
        parse_task(&mut rows).ok_or_else(|| TaskStoreError::UnknownTaskName(task_name.into()))
    }

    pub async fn mark_task_done(
        &self,
        task_name: &str,
        person: &str,
    ) -> Result<Task, TaskStoreError> {
        sqlx::query(include_str!("./insert_completion.sql"))
            .bind(task_name.to_lowercase())
            .bind(person.to_lowercase())
            .bind(today())
            .execute(&self.conn)
            .await?;

        self.task(task_name).await
    }
}

fn next_assignee<'a>(participants: &'a [String], last_completed_by: &str) -> &'a str {
    let mut participants_iter = participants.iter();
    while let Some(person) = participants_iter.next() {
        if person == last_completed_by {
            return participants_iter.next().unwrap_or(&participants[0]);
        }
    }

    dbg!(participants, last_completed_by);

    unreachable!("Invalid Data")
}

fn next_deadline(
    routine: Routine,
    first_deadline: NaiveDate,
    task_length: u16,
    last_completed: NaiveDate,
) -> Deadline {
    match routine {
        Routine::Interval => (last_completed + Duration::days(task_length.into()) - today()).into(),
        Routine::Schedule => {
            let last_completion_deadline = first_deadline
                + Duration::days(
                    (last_completed - first_deadline).num_days() / (task_length as i64)
                        * (task_length as i64)
                        + (task_length as i64),
                );
            ((last_completion_deadline + Duration::days(task_length as i64)) - today()).into()
        }
    }
}

fn parse_task(rows: &mut Peekable<impl Iterator<Item = SqliteRow>>) -> Option<Task> {
    let mut task: Option<Task> = None;
    while let Some(row) = rows.next() {
        match task {
            None => {
                task = Some(Task {
                    name: row.get("name"),
                    kind: row.get("kind"),
                    length_days: row.get("duration"),
                    last_completed: row.get("done_at"),
                    participants: vec![row.get("participant")],
                    assigned_to: String::new(),
                    deadline: Deadline::Upcoming(0),
                });
            }
            Some(ref mut task_ref) => {
                task_ref.participants.push(row.get("participant"));
            }
        }

        if rows.peek().is_none()
            || rows.peek().unwrap().get::<&str, _>("name") != row.get::<&str, _>("name")
        {
            let mut task = task.take().unwrap();
            task.assigned_to = next_assignee(&task.participants, row.get("done_by")).into();
            task.deadline = next_deadline(
                task.kind,
                row.get("first_done"),
                task.length_days,
                task.last_completed,
            );
            return Some(task);
        }
    }

    None
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
mod tests2 {
    use crate::{auth::AuthStore, tasks::time};

    use super::*;

    #[sqlx::test]
    async fn test_listing_tasks_for_empty_store_gives_no_results(conn: sqlx::SqlitePool) {
        let task_store = TaskStore::new(conn);
        assert_eq!(task_store.tasks().await.unwrap(), vec![]);
    }

    #[sqlx::test]
    async fn test_listing_tasks_when_tasks_exist(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        auth_store.create_test_user("claire").await.unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task".into(),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 10).unwrap(),
                participants: vec!["arthur".into(), "bob".into(), "claire".into()],
            })
            .await
            .unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task 2".into(),
                starts_with: "claire".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 14).unwrap(),
                participants: vec!["arthur".into(), "bob".into(), "claire".into()],
            })
            .await
            .unwrap();
        assert_eq!(
            task_store.tasks().await.unwrap(),
            vec![
                Task {
                    name: "Test Task".into(),
                    assigned_to: "bob".into(),
                    kind: Routine::Interval,
                    deadline: Deadline::Upcoming(0),
                    length_days: 7,
                    last_completed: NaiveDate::from_ymd_opt(2020, 1, 3).unwrap(),
                    participants: vec!["arthur".into(), "bob".into(), "claire".into()]
                },
                Task {
                    name: "Test Task 2".into(),
                    assigned_to: "claire".into(),
                    kind: Routine::Interval,
                    deadline: Deadline::Upcoming(4),
                    length_days: 7,
                    last_completed: NaiveDate::from_ymd_opt(2020, 1, 7).unwrap(),
                    participants: vec!["arthur".into(), "bob".into(), "claire".into()]
                }
            ]
        );
    }

    #[sqlx::test]
    async fn test_returns_tasks_for_a_particular_person(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        auth_store.create_test_user("claire").await.unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task 1".into(),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["arthur".into(), "bob".into(), "claire".into()],
            })
            .await
            .unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task 2".into(),
                starts_with: "claire".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["claire".into(), "bob".into(), "arthur".into()],
            })
            .await
            .unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task 3".into(),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["claire".into(), "bob".into(), "arthur".into()],
            })
            .await
            .unwrap();

        let bobs_tasks = task_store.tasks_for("bob").await.unwrap();
        assert_eq!(bobs_tasks.len(), 2);
        assert_eq!(bobs_tasks[0].name, "Test Task 1");
        assert_eq!(bobs_tasks[1].name, "Test Task 3");

        let claires_tasks = task_store.tasks_for("claire").await.unwrap();
        assert_eq!(claires_tasks.len(), 1);
        assert_eq!(claires_tasks[0].name, "Test Task 2");

        let arthurs_tasks = task_store.tasks_for("arthur").await.unwrap();
        assert_eq!(arthurs_tasks.len(), 0);

        assert_eq!(
            bobs_tasks[0].participants,
            vec!["arthur".to_owned(), "bob".to_owned(), "claire".to_owned()]
        );
        assert_eq!(
            bobs_tasks[1].participants,
            vec!["claire".to_owned(), "bob".to_owned(), "arthur".to_owned()]
        );
    }

    #[sqlx::test]
    async fn test_returns_created_task(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task 1".into(),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();
        task_store
            .add_task(NewTask {
                name: "Test Task 2".into(),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        let task = task_store.task("Test Task 2").await.unwrap();
        assert_eq!(task.name, "Test Task 2".to_owned());
    }

    #[sqlx::test]
    async fn test_completing_interval_task_returns_updated_task(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                name: "Task".into(),
                starts_with: "arthur".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 12).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        let task = task_store.mark_task_done("Task", "arthur").await.unwrap();
        assert_eq!(task.name, "Task".to_owned());
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(
            task.last_completed,
            NaiveDate::from_ymd_opt(2020, 1, 10).unwrap()
        );
        assert_eq!(task.deadline, Deadline::Upcoming(7));
    }

    #[sqlx::test]
    async fn test_completing_schedule_task_returns_updated_task(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                name: "Task".into(),
                starts_with: "arthur".into(),
                routine: Routine::Schedule,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 12).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        let task = task_store.mark_task_done("Task", "arthur").await.unwrap();
        assert_eq!(task.name, "Task".to_owned());
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(
            task.last_completed,
            NaiveDate::from_ymd_opt(2020, 1, 10).unwrap()
        );
        assert_eq!(task.deadline, Deadline::Upcoming(9));
    }

    #[sqlx::test]
    async fn test_completing_previous_period_schedule_task_has_no_effect(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 14).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                name: "Task".into(),
                starts_with: "arthur".into(),
                routine: Routine::Schedule,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        // complete on the 14th, schedule finishes on the 15th
        let task = task_store.mark_task_done("Task", "arthur").await.unwrap();
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(8)); // 1w + 1 remaining day
        let task = task_store.mark_task_done("Task", "bob").await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned()); // assignee updated
        assert_eq!(task.deadline, Deadline::Upcoming(8)); // deadline remains the same
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
