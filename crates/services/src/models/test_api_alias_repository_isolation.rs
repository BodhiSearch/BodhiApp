use crate::models::{ApiAlias, ApiAliasRepository, ApiFormat};
use crate::test_utils::{
  sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_alias(id: &str, prefix: &str, now: DateTime<Utc>) -> ApiAlias {
  ApiAlias {
    id: id.to_string(),
    api_format: ApiFormat::OpenAI,
    base_url: format!("https://{}.example.com/v1", id),
    models: Default::default(),
    prefix: Some(prefix.to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: now,
    updated_at: now,
  }
}

// ============================================================================
// Cross-Tenant API Alias Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_api_alias_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create API alias in tenant A for TEST_USER_ID
  let alias_a = make_alias("alias-a1", "prefix-a", ctx.now);
  ctx
    .service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_a, None)
    .await?;

  // Create API alias in tenant B for TEST_USER_ID (same user)
  let alias_b = make_alias("alias-b1", "prefix-b", ctx.now);
  ctx
    .service
    .create_api_model_alias(TEST_TENANT_B_ID, TEST_USER_ID, &alias_b, None)
    .await?;

  // list_api_model_aliases(TENANT_A, TEST_USER_ID) -> only tenant A's alias
  let aliases_a = ctx
    .service
    .list_api_model_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, aliases_a.len());
  assert_eq!("alias-a1", aliases_a[0].id);

  // get_api_model_alias(TENANT_B, TEST_USER_ID, alias_a_id) -> None
  let cross = ctx
    .service
    .get_api_model_alias(TEST_TENANT_B_ID, TEST_USER_ID, "alias-a1")
    .await?;
  assert_eq!(None, cross);

  // check_prefix_exists(TENANT_B, TEST_USER_ID, "prefix-a", None) -> false
  let cross_prefix = ctx
    .service
    .check_prefix_exists(TEST_TENANT_B_ID, TEST_USER_ID, "prefix-a", None)
    .await?;
  assert_eq!(false, cross_prefix);

  Ok(())
}

// ============================================================================
// Intra-Tenant User API Alias Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_intra_tenant_user_api_alias_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create API alias in tenant A for TEST_USER_ID
  let alias_a = make_alias("alias-u1", "prefix-u1", ctx.now);
  ctx
    .service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_a, None)
    .await?;

  // Create API alias in tenant A for TEST_TENANT_A_USER_B_ID (same tenant, different user)
  let alias_b = make_alias("alias-u2", "prefix-u2", ctx.now);
  ctx
    .service
    .create_api_model_alias(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, &alias_b, None)
    .await?;

  // list_api_model_aliases(TENANT_A, TEST_USER_ID) -> only user A's alias
  let aliases_u1 = ctx
    .service
    .list_api_model_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, aliases_u1.len());
  assert_eq!("alias-u1", aliases_u1[0].id);

  // list_api_model_aliases(TENANT_A, TEST_TENANT_A_USER_B_ID) -> only user B's alias
  let aliases_u2 = ctx
    .service
    .list_api_model_aliases(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID)
    .await?;
  assert_eq!(1, aliases_u2.len());
  assert_eq!("alias-u2", aliases_u2[0].id);

  // get_api_model_alias(TENANT_A, TEST_TENANT_A_USER_B_ID, alias_a_id) -> None
  let cross = ctx
    .service
    .get_api_model_alias(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, "alias-u1")
    .await?;
  assert_eq!(None, cross);

  Ok(())
}
