mod common;
use reqwest::Method;

#[tokio::test]
async fn fetches_empty_list_of_tasks() {
    let server = common::harness_with_token().await;

    let tasks = server
        .request(Method::GET, "/api/tasks")
        .send()
        .await
        .unwrap()
        .json::<Vec<()>>()
        .await
        .unwrap();

    assert_eq!(tasks, vec![]);
}
