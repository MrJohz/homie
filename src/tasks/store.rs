// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use chrono::{Duration, NaiveDate};
use sqlx::{types::Json, SqlitePool};

use crate::translations::Language;

use super::{
    time::today,
    types::{Deadline, Routine, Task, TaskId},
};

#[derive(thiserror::Error, Debug)]
pub enum TaskStoreError {
    // 500 type errors (it's probably our fault)
    #[error("underlying data could not be accessed or saved")]
    DbError(#[from] sqlx::Error),

    // 400 type errors (it's probably your fault)
    #[error("unknown task name was used")]
    UnknownTaskName(String),
    #[error("unknown task name was used")]
    UnknownTaskId(TaskId),
    #[error("person does not exist")]
    PersonDoesNotExist(String),
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

        let (task_id,) = sqlx::query_as::<_, (TaskId,)>(include_str!("./insert_new_task.sql"))
            .bind(new_task.routine)
            .bind(new_task.duration)
            .fetch_one(&mut transaction)
            .await?;

        for (lang, name) in new_task.names.drain() {
            sqlx::query(include_str!("./insert_new_task_name.sql"))
                .bind(task_id)
                .bind(lang)
                .bind(name)
                .execute(&mut transaction)
                .await?;
        }

        for person in &new_task.participants {
            let result = sqlx::query(include_str!("./insert_new_task_participant.sql"))
                .bind(task_id)
                .bind(person)
                .execute(&mut transaction)
                .await?;

            if result.rows_affected() == 0 {
                Err(TaskStoreError::PersonDoesNotExist(person.to_owned()))?;
            }
        }

        new_task.participants.reverse();

        let prev_person = next_assignee(&new_task.participants, &new_task.starts_with);
        let started_time = new_task.starts_on - Duration::days(new_task.duration.into());
        sqlx::query(include_str!("./insert_new_task_first_completion.sql"))
            .bind(task_id)
            .bind(started_time)
            .bind(prev_person)
            .execute(&mut transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn tasks(&self, language: &Language) -> Result<Vec<Task>, TaskStoreError> {
        let rows = sqlx::query_as::<
            _,
            (
                TaskId,
                String,
                Routine,
                u16,
                Json<Vec<String>>,
                NaiveDate,
                String,
            ),
        >(include_str!("./select_all_tasks.sql"))
        .bind(language.to_string())
        .fetch_all(&self.conn)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Task {
                id: row.0,
                name: row.1,
                kind: row.2,
                length_days: row.3,
                assigned_to: next_assignee(&row.4, &row.6).into(),
                participants: row.4 .0,
                last_completed: row.5,
                deadline: next_deadline(row.3, row.5),
            })
            .collect())
    }

    pub async fn tasks_for(
        &self,
        person: &str,
        language: &Language,
    ) -> Result<Vec<Task>, TaskStoreError> {
        let person = person.to_lowercase();
        Ok(self
            .tasks(language)
            .await?
            .into_iter()
            .filter(|task| task.assigned_to.to_lowercase() == person)
            .collect())
    }

    pub async fn task(&self, task_id: TaskId, language: &Language) -> Result<Task, TaskStoreError> {
        let row = sqlx::query_as::<
            _,
            (
                TaskId,
                String,
                Routine,
                u16,
                Json<Vec<String>>,
                NaiveDate,
                String,
            ),
        >(include_str!("./select_one_task.sql"))
        .bind(language.to_string())
        .bind(task_id)
        .fetch_optional(&self.conn)
        .await?;

        let row = match row {
            Some(row) => row,
            None => Err(TaskStoreError::UnknownTaskId(task_id))?,
        };

        Ok(Task {
            id: row.0,
            name: row.1,
            kind: row.2,
            length_days: row.3,
            assigned_to: next_assignee(&row.4, &row.6).into(),
            participants: row.4 .0,
            last_completed: row.5,
            deadline: next_deadline(row.3, row.5),
        })
    }

    pub async fn mark_task_done(
        &self,
        task_id: TaskId,
        person: &str,
        date: &NaiveDate,
    ) -> Result<(), TaskStoreError> {
        sqlx::query(include_str!("./insert_completion.sql"))
            .bind(task_id)
            .bind(date)
            .bind(person)
            .execute(&self.conn)
            .await?;
        Ok(())
    }
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
    (last_completed + Duration::days(task_length.into()) - today()).into()
}

pub struct NewTask {
    pub names: HashMap<String, String>,
    pub routine: Routine,
    pub duration: u16,
    pub participants: Vec<String>,
    pub starts_on: NaiveDate,
    pub starts_with: String,
}

#[cfg(test)]
mod tests {
    use crate::{auth::AuthStore, tasks::time};

    use super::*;

    fn names(names: &[(&str, &str)]) -> HashMap<String, String> {
        names
            .iter()
            .map(|(l, r)| (l.to_string(), r.to_string()))
            .collect()
    }

    #[sqlx::test]
    async fn listing_tasks_for_empty_store_gives_no_results(conn: sqlx::SqlitePool) {
        let task_store = TaskStore::new(conn);
        assert_eq!(task_store.tasks(&"en".into()).await.unwrap(), vec![]);
    }

    #[sqlx::test]
    async fn listing_tasks_when_tasks_exist(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        auth_store.create_test_user("claire").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Test Task 1")]),
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
                names: names(&[("en", "Test Task 2")]),
                starts_with: "claire".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 14).unwrap(),
                participants: vec!["arthur".into(), "bob".into(), "claire".into()],
            })
            .await
            .unwrap();

        let tasks = task_store.tasks(&"en".into()).await.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "Test Task 1".to_owned());
        assert_eq!(tasks[1].name, "Test Task 2".to_owned());
    }

    #[sqlx::test]
    async fn returns_tasks_for_a_particular_person(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("claire").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Test Task 1")]),
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
                names: names(&[("en", "Test Task 2")]),
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
                names: names(&[("en", "Test Task 3")]),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["claire".into(), "bob".into(), "arthur".into()],
            })
            .await
            .unwrap();

        let bobs_tasks = task_store.tasks_for("bob", &"en".into()).await.unwrap();
        assert_eq!(bobs_tasks.len(), 2);
        assert_eq!(bobs_tasks[0].name, "Test Task 1");
        assert_eq!(bobs_tasks[1].name, "Test Task 3");

        let claires_tasks = task_store.tasks_for("claire", &"en".into()).await.unwrap();
        assert_eq!(claires_tasks.len(), 1);
        assert_eq!(claires_tasks[0].name, "Test Task 2");

        let arthurs_tasks = task_store.tasks_for("arthur", &"en".into()).await.unwrap();
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
    async fn returns_created_task(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Test Task 1")]),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 10).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Test Task 2")]),
                starts_with: "bob".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 15).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.name, "Test Task 1".to_owned());
        assert_eq!(
            task.last_completed,
            NaiveDate::from_ymd_opt(2020, 1, 3).unwrap()
        );

        let task = task_store.task(2.into(), &"en".into()).await.unwrap();
        assert_eq!(task.name, "Test Task 2".to_owned());
        assert_eq!(
            task.last_completed,
            NaiveDate::from_ymd_opt(2020, 1, 8).unwrap()
        );
    }

    #[sqlx::test]
    async fn completing_interval_task_returns_updated_task(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 12).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        task_store
            .mark_task_done(1.into(), "arthur", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.name, "Task".to_owned());
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(
            task.last_completed,
            NaiveDate::from_ymd_opt(2020, 1, 10).unwrap()
        );
        assert_eq!(task.deadline, Deadline::Upcoming(7));
    }

    #[sqlx::test]
    async fn completing_schedule_task_returns_updated_task(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Schedule,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 12).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        task_store
            .mark_task_done(1.into(), "arthur", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.name, "Task".to_owned());
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(
            task.last_completed,
            NaiveDate::from_ymd_opt(2020, 1, 12).unwrap()
        );
        assert_eq!(task.deadline, Deadline::Upcoming(9));
    }

    #[sqlx::test]
    async fn completing_period_schedule_tasks_rolls_the_date_forward(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 14).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Schedule,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        // complete for period until 1st
        task_store
            .mark_task_done(1.into(), "arthur", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(task.deadline, Deadline::Overdue(6));

        // complete for period 8th - 14th
        task_store
            .mark_task_done(1.into(), "bob", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned()); // assignee updated
        assert_eq!(task.deadline, Deadline::Upcoming(1)); // next period starts on 8th and continues for 7 days
    }

    #[sqlx::test]
    async fn completing_schedule_task_multiple_times(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 14).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Schedule,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 10).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Overdue(4));

        task_store
            .mark_task_done(1.into(), "arthur", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(3));

        task_store
            .mark_task_done(1.into(), "bob", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(10));

        // the same person does the task multiple times in a row
        task_store
            .mark_task_done(1.into(), "bob", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(17));
    }

    #[sqlx::test]
    async fn completing_interval_task_multiple_times(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 14).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        auth_store.create_test_user("bob").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 10).unwrap(),
                participants: vec!["arthur".into(), "bob".into()],
            })
            .await
            .unwrap();

        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Overdue(4));

        task_store
            .mark_task_done(1.into(), "arthur", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "bob".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(7));

        task_store
            .mark_task_done(1.into(), "bob", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(7));

        // the same person does the task multiple times in a row
        task_store
            .mark_task_done(1.into(), "bob", &today())
            .await
            .unwrap();
        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(7));
    }

    #[sqlx::test]
    async fn handles_tasks_with_only_one_participant(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 14).unwrap());
        let task_store = TaskStore::new(conn.clone());
        let auth_store = AuthStore::new(conn);
        auth_store.create_test_user("arthur").await.unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Interval Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Interval,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 10).unwrap(),
                participants: vec!["arthur".into()],
            })
            .await
            .unwrap();
        task_store
            .add_task(NewTask {
                names: names(&[("en", "Schedule Task")]),
                starts_with: "arthur".into(),
                routine: Routine::Schedule,
                duration: 7,
                starts_on: NaiveDate::from_ymd_opt(2020, 1, 10).unwrap(),
                participants: vec!["arthur".into()],
            })
            .await
            .unwrap();

        let task = task_store.task(1.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Overdue(4));

        task_store
            .mark_task_done(2.into(), "arthur", &today())
            .await
            .unwrap();
        let task = task_store.task(2.into(), &"en".into()).await.unwrap();
        assert_eq!(task.assigned_to, "arthur".to_owned());
        assert_eq!(task.deadline, Deadline::Upcoming(3));
    }

    #[sqlx::test]
    async fn returns_error_if_fetched_task_does_not_exist(conn: sqlx::SqlitePool) {
        time::mock::set(NaiveDate::from_ymd_opt(2020, 1, 14).unwrap());
        let task_store = TaskStore::new(conn);
        let result = task_store.task(4.into(), &"en".into()).await.unwrap_err();

        match result {
            TaskStoreError::UnknownTaskId(name) => assert_eq!(name, 4.into()),
            _ => panic!("incorrect error response"),
        }
    }
}
