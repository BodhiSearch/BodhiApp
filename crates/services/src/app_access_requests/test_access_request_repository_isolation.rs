use crate::{
  app_access_requests::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus},
  new_ulid,
  test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID},
  FlowType,
};
use anyhow_trace::anyhow_trace;
use chrono::Duration;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_request(id: &str, tenant_id: &str, now: chrono::DateTime<chrono::Utc>) -> AppAccessRequest {
  AppAccessRequest {
    id: id.to_string(),
    tenant_id: Some(tenant_id.to_string()),
    app_client_id: "test-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: FlowType::Popup,
    redirect_uri: None,
    status: AppAccessRequestStatus::Draft,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: None,
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: now + Duration::hours(1),
    created_at: now,
    updated_at: now,
  }
}

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

  // Tenant A can only see its own request
  let found_a = ctx.service.get(TEST_TENANT_ID, &id_a).await?;
  assert!(found_a.is_some());
  assert_eq!(Some(TEST_TENANT_ID.to_string()), found_a.unwrap().tenant_id);

  // Tenant A cannot see Tenant B's request
  let cross = ctx.service.get(TEST_TENANT_ID, &id_b).await?;
  assert!(cross.is_none());

  // Tenant B can only see its own request
  let found_b = ctx.service.get(TEST_TENANT_B_ID, &id_b).await?;
  assert!(found_b.is_some());
  assert_eq!(
    Some(TEST_TENANT_B_ID.to_string()),
    found_b.unwrap().tenant_id
  );

  // Tenant B cannot see Tenant A's request
  let cross_b = ctx.service.get(TEST_TENANT_B_ID, &id_a).await?;
  assert!(cross_b.is_none());

  Ok(())
}
