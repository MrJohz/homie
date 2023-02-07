mod common;
use chrono::{Duration, Local};
use homie::tasks::Task;
use reqwest::Method;

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
            name: "Task 1".into(),
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
}
