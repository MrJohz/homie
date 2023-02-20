// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

mod common;
use std::collections::HashMap;

use chrono::{Duration, Local};
use homie::tasks::{Deadline, Task};
use proptest::{prelude::*, test_runner::TestRunner};
use reqwest::Method;

fn names(names: &[(&str, &str)]) -> HashMap<String, String> {
    names
        .iter()
        .map(|(l, r)| (l.to_string(), r.to_string()))
        .collect()
}

#[tokio::test]
async fn fetches_empty_list_of_tasks() {
    let server = common::harness_with_token().await;

    let tasks = server
        .request(Method::GET, "/api/tasks")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks, vec![]);
}

#[tokio::test]
async fn fetches_tasks_that_exist() {
    let server = common::harness_with_token().await;
    server.auth_store().create_user("Kevin", "").await.unwrap();
    server.auth_store().create_user("Bob", "").await.unwrap();
    server
        .task_store()
        .add_task(homie::tasks::NewTask {
            names: names(&[("en", "Task 1")]),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned(), "Bob".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let tasks = server
        .request(Method::GET, "/api/tasks")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Task 1");
    assert_eq!(tasks[0].assigned_to, "Kevin");
    assert_eq!(tasks[0].deadline, Deadline::Overdue(10));
}

#[tokio::test]
async fn fetches_tasks_for_one_person() {
    let server = common::harness_with_token().await;
    server.auth_store().create_user("Kevin", "").await.unwrap();
    server.auth_store().create_user("Bob", "").await.unwrap();
    server
        .task_store()
        .add_task(homie::tasks::NewTask {
            names: names(&[("en", "Task 1")]),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned(), "Bob".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let tasks = server
        .request(Method::GET, "/api/tasks/people/Kevin")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 1);

    let tasks = server
        .request(Method::GET, "/api/tasks/people/Bob")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 0);

    let tasks = server
        .request(Method::GET, "/api/tasks/people/unknown")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn fetches_tasks_for_people_case_insensitive() {
    let server = common::harness_with_token().await;
    server.auth_store().create_user("Kevin", "").await.unwrap();
    server
        .task_store()
        .add_task(homie::tasks::NewTask {
            names: names(&[("en", "Task 1")]),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let tasks = server
        .request(Method::GET, "/api/tasks/people/Kevin")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 1);

    let tasks = server
        .request(Method::GET, "/api/tasks/people/kevin")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 1);

    let tasks = server
        .request(Method::GET, "/api/tasks/people/KEVIN")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(tasks.len(), 1);
}

#[tokio::test]
async fn updates_tasks_for_current_day() {
    let server = common::harness_with_token().await;
    server.auth_store().create_user("Kevin", "").await.unwrap();
    server.auth_store().create_user("Bob", "").await.unwrap();
    server
        .task_store()
        .add_task(homie::tasks::NewTask {
            names: names(&[("en", "Task 1")]),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned(), "Bob".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let updated = server
        .request(Method::POST, "/api/tasks/actions/mark_task_done/1?by=Bob")
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();

    assert_eq!(updated.deadline, Deadline::Upcoming(7));
    assert_eq!(updated.last_completed, Local::now().date_naive());
}

#[tokio::test]
async fn task_update_can_set_date_explicitly() {
    let server = common::harness_with_token().await;
    server.auth_store().create_user("Kevin", "").await.unwrap();
    server.auth_store().create_user("Bob", "").await.unwrap();
    server
        .task_store()
        .add_task(homie::tasks::NewTask {
            names: names(&[("en", "Task 1")]),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned(), "Bob".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let before_yesterday = (Local::now() - Duration::days(2)).date_naive();
    let after_tomorrow = (Local::now() + Duration::days(2)).date_naive();
    let today = Local::now().date_naive();

    let updated = server
        .request(
            Method::POST,
            format!("/api/tasks/actions/mark_task_done/1?by=Bob&on={before_yesterday}"),
        )
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();

    assert_eq!(updated.deadline, Deadline::Upcoming(5));
    assert_eq!(updated.last_completed, before_yesterday);

    let updated = server
        .request(
            Method::POST,
            format!("/api/tasks/actions/mark_task_done/1?by=Bob&on={today}"),
        )
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();

    assert_eq!(updated.deadline, Deadline::Upcoming(7));
    assert_eq!(updated.last_completed, today);

    let updated = server
        .request(
            Method::POST,
            format!("/api/tasks/actions/mark_task_done/1?by=Bob&on={after_tomorrow}"),
        )
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();

    assert_eq!(updated.deadline, Deadline::Upcoming(9));
    assert_eq!(updated.last_completed, after_tomorrow);
}

#[tokio::test]
async fn fetches_correct_translation_when_accept_lang_header_is_set() {
    let server = common::harness_with_token().await;
    server.auth_store().create_user("Kevin", "").await.unwrap();
    server
        .task_store()
        .add_task(homie::tasks::NewTask {
            names: names(&[
                ("en", "ENGLISH_NAME"),
                ("de", "GERMAN_NAME"),
                ("fr", "FRENCH_NAME"),
            ]),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let english = server
        .request(Method::GET, "/api/tasks")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(english[0].name, "ENGLISH_NAME");

    let german = server
        .request(Method::GET, "/api/tasks")
        .header("Accept-Language", "de")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(german[0].name, "GERMAN_NAME");

    let english_2 = server
        .request(Method::GET, "/api/tasks")
        .header("Accept-Language", "en-US,fr;q=0.5")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(english_2[0].name, "ENGLISH_NAME");

    let french = server
        .request(Method::GET, "/api/tasks")
        .header("Accept-Language", "fr;q=0.5")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(french[0].name, "ENGLISH_NAME");

    let english_3 = server
        .request(Method::GET, "/api/tasks")
        .header("Accept-Language", "*;q=0.5")
        .send()
        .await
        .unwrap()
        .json::<Vec<Task>>()
        .await
        .unwrap();

    assert_eq!(english_3[0].name, "ENGLISH_NAME");

    assert_eq!(english[0].id, german[0].id);
}

fn language_header_value() -> impl Strategy<Value = Option<String>> {
    prop::option::weighted(0.8, r#"[a-zA-Z0-9 :;.,\\/"'?!(){}\[\]@<>=+*#$&`|~^%_-]*"#)
}

#[test]
fn does_not_crash_on_arbitrary_values_for_accept_language() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let server = rt.block_on(common::harness_with_token());
    rt.block_on(server.auth_store().create_user("Kevin", ""))
        .unwrap();
    rt.block_on(server.task_store().add_task(homie::tasks::NewTask {
        names: names(&[
            ("en", "ENGLISH_NAME"),
            ("de", "GERMAN_NAME"),
            ("fr", "FRENCH_NAME"),
        ]),
        routine: homie::tasks::Routine::Interval,
        duration: 7,
        participants: vec!["Kevin".to_owned()],
        starts_on: (Local::now() - Duration::days(10)).date_naive(),
        starts_with: "Kevin".to_owned(),
    }))
    .unwrap();

    let mut runner = TestRunner::new(proptest::test_runner::Config {
        source_file: Some(file!()),
        ..proptest::test_runner::Config::default()
    });
    runner
        .run(&language_header_value(), |header| {
            let tasks = server.request(Method::GET, "/api/tasks");
            let tasks = match header {
                Some(header) => tasks.header("Accept-Language", header),
                None => tasks,
            };

            let tasks = rt.block_on(tasks.send()).unwrap();
            let tasks = rt.block_on(tasks.json::<Vec<Task>>()).unwrap();

            assert_ne!(tasks[0].name, "FRENCH_NAME");
            assert!(tasks[0].name == "ENGLISH_NAME" || tasks[0].name == "GERMAN_NAME");
            Ok(())
        })
        .unwrap();
}
