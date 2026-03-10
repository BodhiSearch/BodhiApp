use crate::tenants::TenantRepository;
use crate::test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID};
use crate::{AppStatus, Tenant};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_test_tenant(id: &str, client_id: &str, now: chrono::DateTime<chrono::Utc>) -> Tenant {
  Tenant {
    id: id.to_string(),
    client_id: client_id.to_string(),
    client_secret: "test-secret".to_string(),
    name: format!("Tenant {id}"),
    description: None,
    status: AppStatus::Ready,
    created_by: Some(TEST_USER_ID.to_string()),
    created_at: now,
    updated_at: now,
  }
}

// ============================================================================
// Cross-Tenant tenants_users Isolation
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_tenant_user_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let tenant_a = make_test_tenant(TEST_TENANT_ID, "client-a", ctx.now);
  let tenant_b = make_test_tenant(TEST_TENANT_B_ID, "client-b", ctx.now);
  ctx.service.create_tenant_test(&tenant_a).await?;
  ctx.service.create_tenant_test(&tenant_b).await?;

  // Create membership in tenant A
  ctx
    .service
    .upsert_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  // Create membership in tenant B
  ctx
    .service
    .upsert_tenant_user(TEST_TENANT_B_ID, TEST_USER_ID)
    .await?;

  // list_user_tenants returns both (cross-tenant read by design)
  let tenants = ctx.service.list_user_tenants(TEST_USER_ID).await?;
  assert_eq!(2, tenants.len());

  // Delete membership in tenant A
  ctx
    .service
    .delete_tenant_user(TEST_TENANT_ID, TEST_USER_ID)
    .await?;

  // Only tenant B membership remains
  let tenants = ctx.service.list_user_tenants(TEST_USER_ID).await?;
  assert_eq!(1, tenants.len());
  assert_eq!(TEST_TENANT_B_ID, tenants[0].id);

  // has_tenant_memberships still true (tenant B membership)
  let has = ctx.service.has_tenant_memberships(TEST_USER_ID).await?;
  assert_eq!(true, has);

  Ok(())
}
