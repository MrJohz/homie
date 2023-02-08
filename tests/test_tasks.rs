mod common;
use chrono::{Duration, Local};
use homie::tasks::{Deadline, Task};
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
            name: "Task 1".into(),
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
            name: "Task 1".into(),
            routine: homie::tasks::Routine::Interval,
            duration: 7,
            participants: vec!["Kevin".to_owned(), "Bob".to_owned()],
            starts_on: (Local::now() - Duration::days(10)).date_naive(),
            starts_with: "Kevin".to_owned(),
        })
        .await
        .unwrap();

    let updated = server
        .request(
            Method::POST,
            "/api/tasks/actions/mark_task_done/Task%201?by=Bob",
        )
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
async fn task_update_is_case_insensitive() {
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

    let updated = server
        .request(
            Method::POST,
            "/api/tasks/actions/mark_task_done/Task%201?by=Bob",
        )
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();

    assert_eq!(updated.deadline, Deadline::Upcoming(7));
    assert_eq!(updated.last_completed, Local::now().date_naive());

    let updated = server
        .request(
            Method::POST,
            "/api/tasks/actions/mark_task_done/TASK%201?by=BOB",
        )
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();

    assert_eq!(updated.deadline, Deadline::Upcoming(7));
    assert_eq!(updated.last_completed, Local::now().date_naive());

    let updated = server
        .request(
            Method::POST,
            "/api/tasks/actions/mark_task_done/taSK%201?by=boB",
        )
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
            name: "Task 1".into(),
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
            format!("/api/tasks/actions/mark_task_done/Task%201?by=Bob&on={before_yesterday}"),
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
            format!("/api/tasks/actions/mark_task_done/Task%201?by=Bob&on={today}"),
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
            format!("/api/tasks/actions/mark_task_done/Task%201?by=Bob&on={after_tomorrow}"),
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
