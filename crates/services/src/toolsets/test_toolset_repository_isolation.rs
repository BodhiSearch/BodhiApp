use crate::{
  new_ulid,
  test_utils::{
    sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
  },
  toolsets::{ToolsetEntity, ToolsetRepository},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_toolset(
  id: &str,
  tenant_id: &str,
  user_id: &str,
  slug: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> ToolsetEntity {
  ToolsetEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    user_id: user_id.to_string(),
    toolset_type: "builtin-exa-search".to_string(),
    slug: slug.to_string(),
    description: Some("Test toolset".to_string()),
    enabled: true,
    encrypted_api_key: None,
    salt: None,
    nonce: None,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_toolset_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id_a = new_ulid();
  let id_b = new_ulid();

  let row_a = make_toolset(&id_a, TEST_TENANT_ID, TEST_USER_ID, "toolset-a", ctx.now);
  let row_b = make_toolset(&id_b, TEST_TENANT_B_ID, TEST_USER_ID, "toolset-b", ctx.now);

  ctx.service.create_toolset(TEST_TENANT_ID, &row_a).await?;
  ctx.service.create_toolset(TEST_TENANT_B_ID, &row_b).await?;

  let list_a = ctx
    .service
    .list_toolsets(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_a.len());

  let list_b = ctx
    .service
    .list_toolsets(TEST_TENANT_B_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_b.len());

  let cross = ctx.service.get_toolset(TEST_TENANT_ID, &row_b.id).await?;
  assert_eq!(None, cross);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_toolset_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let id_a = new_ulid();
  let id_b = new_ulid();

  let row_a = make_toolset(
    &id_a,
    TEST_TENANT_ID,
    TEST_USER_ID,
    "toolset-user-a",
    ctx.now,
  );
  let row_b = make_toolset(
    &id_b,
    TEST_TENANT_ID,
    TEST_TENANT_A_USER_B_ID,
    "toolset-user-b",
    ctx.now,
  );

  ctx.service.create_toolset(TEST_TENANT_ID, &row_a).await?;
  ctx.service.create_toolset(TEST_TENANT_ID, &row_b).await?;

  let list_a = ctx
    .service
    .list_toolsets(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, list_a.len());
  assert_eq!("toolset-user-a", list_a[0].slug);

  let list_b = ctx
    .service
    .list_toolsets(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID)
    .await?;
  assert_eq!(1, list_b.len());
  assert_eq!("toolset-user-b", list_b[0].slug);

  Ok(())
}

// ============================================================================
// Cross-Tenant app_toolset_configs Isolation
// ============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_app_toolset_config_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  use crate::toolsets::ToolsetRepository;

  let ctx = sea_context(db_type).await;

  // Set config in tenant A
  ctx
    .service
    .set_app_toolset_enabled(TEST_TENANT_ID, "builtin-exa-search", true, "admin-a")
    .await?;

  // Set config in tenant B
  ctx
    .service
    .set_app_toolset_enabled(TEST_TENANT_B_ID, "builtin-exa-search", false, "admin-b")
    .await?;

  // list_app_toolset_configs per tenant returns only that tenant's configs
  let configs_a = ctx.service.list_app_toolset_configs(TEST_TENANT_ID).await?;
  assert_eq!(1, configs_a.len());
  assert_eq!(true, configs_a[0].enabled);

  let configs_b = ctx
    .service
    .list_app_toolset_configs(TEST_TENANT_B_ID)
    .await?;
  assert_eq!(1, configs_b.len());
  assert_eq!(false, configs_b[0].enabled);

  // get_app_toolset_config cross-tenant returns None
  // (actually returns the tenant's own config — verifying tenant-scoped values)
  let config_a = ctx
    .service
    .get_app_toolset_config(TEST_TENANT_ID, "builtin-exa-search")
    .await?
    .expect("should exist");
  assert_eq!(true, config_a.enabled);
  assert_eq!("admin-a", config_a.updated_by);

  let config_b = ctx
    .service
    .get_app_toolset_config(TEST_TENANT_B_ID, "builtin-exa-search")
    .await?
    .expect("should exist");
  assert_eq!(false, config_b.enabled);
  assert_eq!("admin-b", config_b.updated_by);

  Ok(())
}
