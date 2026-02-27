use crate::{
  db::{AccessRepository, UserAccessRequestStatus},
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_insert_and_get_pending_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440000".to_string();

  let inserted = ctx
    .service
    .insert_pending_request(username.clone(), user_id.clone())
    .await?;

  assert!(!inserted.id.is_empty());
  assert_eq!(username, inserted.username);
  assert_eq!(user_id, inserted.user_id);
  assert_eq!(UserAccessRequestStatus::Pending, inserted.status);
  assert!(inserted.reviewer.is_none());
  assert_eq!(ctx.now, inserted.created_at);
  assert_eq!(ctx.now, inserted.updated_at);

  let fetched = ctx.service.get_pending_request(user_id).await?;
  assert!(fetched.is_some());
  assert_eq!(inserted, fetched.unwrap());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_get_request_by_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let inserted = ctx
    .service
    .insert_pending_request("test@example.com".to_string(), "user-001".to_string())
    .await?;

  let fetched = ctx.service.get_request_by_id(&inserted.id).await?;
  assert!(fetched.is_some());
  assert_eq!(inserted, fetched.unwrap());

  let not_found = ctx.service.get_request_by_id("nonexistent-id").await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_list_pending_requests(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let test_data = vec![
    ("test1@example.com", "user-001"),
    ("test2@example.com", "user-002"),
    ("test3@example.com", "user-003"),
  ];

  for (username, user_id) in &test_data {
    ctx
      .service
      .insert_pending_request(username.to_string(), user_id.to_string())
      .await?;
  }

  let (page1, total) = ctx.service.list_pending_requests(1, 2).await?;
  assert_eq!(3, total);
  assert_eq!(2, page1.len());

  let (page2, total) = ctx.service.list_pending_requests(2, 2).await?;
  assert_eq!(3, total);
  assert_eq!(1, page2.len());

  for (i, request) in page1.iter().chain(page2.iter()).enumerate() {
    assert_eq!(test_data[i].0, request.username);
    assert_eq!(test_data[i].1, request.user_id);
    assert_eq!(UserAccessRequestStatus::Pending, request.status);
  }

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_list_all_requests(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let inserted = ctx
    .service
    .insert_pending_request("test1@example.com".to_string(), "user-001".to_string())
    .await?;
  ctx
    .service
    .insert_pending_request("test2@example.com".to_string(), "user-002".to_string())
    .await?;

  ctx
    .service
    .update_request_status(
      &inserted.id,
      UserAccessRequestStatus::Approved,
      "admin@example.com".to_string(),
    )
    .await?;

  let (pending, pending_total) = ctx.service.list_pending_requests(1, 10).await?;
  assert_eq!(1, pending_total);
  assert_eq!(1, pending.len());

  let (all, all_total) = ctx.service.list_all_requests(1, 10).await?;
  assert_eq!(2, all_total);
  assert_eq!(2, all.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_sea_update_request_status(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let inserted = ctx
    .service
    .insert_pending_request("test@example.com".to_string(), "user-001".to_string())
    .await?;

  ctx
    .service
    .update_request_status(
      &inserted.id,
      UserAccessRequestStatus::Approved,
      "admin@example.com".to_string(),
    )
    .await?;

  let updated = ctx
    .service
    .get_request_by_id(&inserted.id)
    .await?
    .expect("Request should exist");

  assert_eq!(UserAccessRequestStatus::Approved, updated.status);
  assert_eq!(Some("admin@example.com".to_string()), updated.reviewer);
  assert_eq!(ctx.now, updated.updated_at);

  let pending = ctx
    .service
    .get_pending_request("user-001".to_string())
    .await?;
  assert!(pending.is_none());

  Ok(())
}
