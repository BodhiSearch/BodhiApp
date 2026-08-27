use anyhow_trace::anyhow_trace;
use chrono::Duration;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

use crate::{
  app_access_requests::{AccessRequestRepository, AppAccessRequestStatus},
  new_ulid,
  test_utils::{approved_request, make_request, sea_context, setup_env, TEST_TENANT_ID},
};

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_create_and_get_access_request(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  let row = make_request(&id, TEST_TENANT_ID, ctx.now);

  let created = ctx.service.create(&row).await?;
  assert_eq!(row, created);

  let fetched = ctx.service.get(TEST_TENANT_ID, &id).await?;
  assert_eq!(Some(row), fetched);

  let not_found = ctx.service.get(TEST_TENANT_ID, "nonexistent").await?;
  assert!(not_found.is_none());

  // `get` is tenant-strict: a foreign tenant never sees the row.
  let cross_tenant = ctx.service.get("other-tenant", &id).await?;
  assert!(cross_tenant.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_by_access_request_scope(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  let scope = format!("scope_access_request:resource-client.{}", id);
  let mut row = approved_request(&id, TEST_TENANT_ID, "user-1", ctx.now);
  row.access_request_scope = Some(scope.clone());
  ctx.service.create(&row).await?;

  let found = ctx
    .service
    .get_by_access_request_scope(TEST_TENANT_ID, &scope)
    .await?;
  assert_eq!(Some(id), found.map(|r| r.id));

  let wrong_tenant = ctx
    .service
    .get_by_access_request_scope("other-tenant", &scope)
    .await?;
  assert!(wrong_tenant.is_none());

  let not_found = ctx
    .service
    .get_by_access_request_scope(TEST_TENANT_ID, "nonexistent-scope")
    .await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_latest_approved_for_app_user(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let older = new_ulid();
  ctx
    .service
    .create(&approved_request(&older, TEST_TENANT_ID, "owner", ctx.now))
    .await?;
  let newer = new_ulid();
  ctx
    .service
    .create(&approved_request(
      &newer,
      TEST_TENANT_ID,
      "owner",
      ctx.now + Duration::minutes(1),
    ))
    .await?;
  // Excluded despite newer created_at: other app client, other user, non-approved status.
  let mut other_app = approved_request(
    &new_ulid(),
    TEST_TENANT_ID,
    "owner",
    ctx.now + Duration::minutes(2),
  );
  other_app.app_client_id = "other-client".to_string();
  ctx.service.create(&other_app).await?;
  ctx
    .service
    .create(&approved_request(
      &new_ulid(),
      TEST_TENANT_ID,
      "other-user",
      ctx.now + Duration::minutes(2),
    ))
    .await?;
  let mut revoked = approved_request(
    &new_ulid(),
    TEST_TENANT_ID,
    "owner",
    ctx.now + Duration::minutes(2),
  );
  revoked.status = AppAccessRequestStatus::Revoked;
  ctx.service.create(&revoked).await?;

  let latest = ctx
    .service
    .latest_approved_for_app_user(TEST_TENANT_ID, "test-client", "owner")
    .await?;
  assert_eq!(Some(newer), latest.map(|r| r.id));

  let none = ctx
    .service
    .latest_approved_for_app_user(TEST_TENANT_ID, "unknown-client", "owner")
    .await?;
  assert!(none.is_none());

  Ok(())
}
