use crate::{
  db::{AccessRepository, UserAccessRequest, UserAccessRequestStatus},
  test_utils::{test_db_service, TestDbService},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_insert_pending_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
  let pending_request = service
    .insert_pending_request(username.clone(), user_id.clone())
    .await?;
  let expected_request = UserAccessRequest {
    id: pending_request.id, // We don't know this in advance
    username,
    user_id,
    created_at: now,
    updated_at: now,
    status: UserAccessRequestStatus::Pending,
    reviewer: None,
  };
  assert_eq!(pending_request, expected_request);
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_get_pending_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440001".to_string();
  let inserted_request = service
    .insert_pending_request(username, user_id.clone())
    .await?;
  let fetched_request = service.get_pending_request(user_id).await?;
  assert!(fetched_request.is_some());
  assert_eq!(inserted_request, fetched_request.unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_list_pending_requests(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let test_data = vec![
    (
      "test1@example.com".to_string(),
      "550e8400-e29b-41d4-a716-446655440002".to_string(),
    ),
    (
      "test2@example.com".to_string(),
      "550e8400-e29b-41d4-a716-446655440003".to_string(),
    ),
    (
      "test3@example.com".to_string(),
      "550e8400-e29b-41d4-a716-446655440004".to_string(),
    ),
  ];
  for (username, user_id) in &test_data {
    service
      .insert_pending_request(username.clone(), user_id.clone())
      .await?;
  }
  let (page1, total) = service.list_pending_requests(1, 2).await?;
  assert_eq!(2, page1.len());
  assert_eq!(3, total);
  let (page2, total) = service.list_pending_requests(2, 2).await?;
  assert_eq!(1, page2.len());
  assert_eq!(3, total);
  for (i, request) in page1.iter().chain(page2.iter()).enumerate() {
    let expected_request = UserAccessRequest {
      id: request.id,
      username: test_data[i].0.clone(),
      user_id: test_data[i].1.clone(),
      created_at: now,
      updated_at: now,
      status: UserAccessRequestStatus::Pending,
      reviewer: None,
    };
    assert_eq!(&expected_request, request);
  }
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_update_request_status(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440005".to_string();
  let inserted_request = service
    .insert_pending_request(username, user_id.clone())
    .await?;
  service
    .update_request_status(
      inserted_request.id,
      UserAccessRequestStatus::Approved,
      "admin@example.com".to_string(),
    )
    .await?;
  let updated_request = service.get_pending_request(user_id).await?;
  assert!(updated_request.is_none());
  Ok(())
}
