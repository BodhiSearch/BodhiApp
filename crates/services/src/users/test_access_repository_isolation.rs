use crate::{
  test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID},
  users::AccessRepository,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_user_access_request_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = "user-isolation-test";

  ctx
    .service
    .insert_pending_request(
      TEST_TENANT_ID,
      "user-a@test.com".to_string(),
      user_id.to_string(),
    )
    .await?;

  ctx
    .service
    .insert_pending_request(
      TEST_TENANT_B_ID,
      "user-a@test.com".to_string(),
      user_id.to_string(),
    )
    .await?;

  // Tenant A sees only its request
  let (requests_a, total_a) = ctx
    .service
    .list_pending_requests(TEST_TENANT_ID, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(TEST_TENANT_ID, requests_a[0].tenant_id);

  // Tenant B sees only its request
  let (requests_b, total_b) = ctx
    .service
    .list_pending_requests(TEST_TENANT_B_ID, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(TEST_TENANT_B_ID, requests_b[0].tenant_id);

  // get_pending_request scoped by tenant
  let found_a = ctx
    .service
    .get_pending_request(TEST_TENANT_ID, user_id.to_string())
    .await?;
  assert!(found_a.is_some());
  assert_eq!(TEST_TENANT_ID, found_a.unwrap().tenant_id);

  let found_b = ctx
    .service
    .get_pending_request(TEST_TENANT_B_ID, user_id.to_string())
    .await?;
  assert!(found_b.is_some());
  assert_eq!(TEST_TENANT_B_ID, found_b.unwrap().tenant_id);

  Ok(())
}
