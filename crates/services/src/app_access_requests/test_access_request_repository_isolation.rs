use crate::{
  app_access_requests::{
    test_access_request_builders::{approved_request, make_request},
    AccessRequestRepository, AppAccessRequestStatus,
  },
  new_ulid,
  test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID},
  DbError,
};
use anyhow_trace::anyhow_trace;
use chrono::Duration;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_app_access_request_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let id_a = new_ulid();
  let id_b = new_ulid();
  let row_a = make_request(&id_a, TEST_TENANT_ID, ctx.now);
  let row_b = make_request(&id_b, TEST_TENANT_B_ID, ctx.now);

  ctx.service.create(&row_a).await?;
  ctx.service.create(&row_b).await?;

  let found_a = ctx.service.get(TEST_TENANT_ID, &id_a).await?;
  assert!(found_a.is_some());
  assert_eq!(Some(TEST_TENANT_ID.to_string()), found_a.unwrap().tenant_id);

  let cross = ctx.service.get(TEST_TENANT_ID, &id_b).await?;
  assert!(cross.is_none());

  let found_b = ctx.service.get(TEST_TENANT_B_ID, &id_b).await?;
  assert!(found_b.is_some());
  assert_eq!(
    Some(TEST_TENANT_B_ID.to_string()),
    found_b.unwrap().tenant_id
  );

  let cross_b = ctx.service.get(TEST_TENANT_B_ID, &id_a).await?;
  assert!(cross_b.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_revocation_owner_tenant_status_guards(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id = new_ulid();
  ctx
    .service
    .create(&approved_request(&id, TEST_TENANT_ID, "owner", ctx.now))
    .await?;

  // Another user must not revoke (and the row's existence is not revealed).
  let err = ctx
    .service
    .update_revocation(TEST_TENANT_ID, &id, "intruder")
    .await
    .unwrap_err();
  assert!(matches!(err, DbError::ItemNotFound { .. }));

  // Another tenant must not revoke.
  let err = ctx
    .service
    .update_revocation(TEST_TENANT_B_ID, &id, "owner")
    .await
    .unwrap_err();
  assert!(matches!(err, DbError::ItemNotFound { .. }));

  // Owner + Approved → flips to Revoked.
  let updated = ctx
    .service
    .update_revocation(TEST_TENANT_ID, &id, "owner")
    .await?;
  assert_eq!(AppAccessRequestStatus::Revoked, updated.status);

  // Revoking again (now non-Approved) → status conflict, not a silent success.
  let err = ctx
    .service
    .update_revocation(TEST_TENANT_ID, &id, "owner")
    .await
    .unwrap_err();
  assert!(matches!(err, DbError::AccessRequestNotDraft { .. }));

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_list_approved_for_user_filters_and_orders(
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
  // Excluded: another user, a non-approved (Draft) row, another tenant.
  ctx
    .service
    .create(&approved_request(
      &new_ulid(),
      TEST_TENANT_ID,
      "other-user",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create(&make_request(&new_ulid(), TEST_TENANT_ID, ctx.now))
    .await?;
  ctx
    .service
    .create(&approved_request(
      &new_ulid(),
      TEST_TENANT_B_ID,
      "owner",
      ctx.now,
    ))
    .await?;

  let list = ctx
    .service
    .list_approved_for_user(TEST_TENANT_ID, "owner")
    .await?;
  assert_eq!(2, list.len());
  // Newest-first.
  assert_eq!(newer, list[0].id);
  assert_eq!(older, list[1].id);

  Ok(())
}
