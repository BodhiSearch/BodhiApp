use crate::models::{ModelRouterAlias, ModelRouterRepository, RouterTarget, RoutingStrategyConfig};
use crate::test_utils::{
  sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_router(id: &str, alias: &str, now: DateTime<Utc>) -> ModelRouterAlias {
  ModelRouterAlias {
    id: id.to_string(),
    alias: alias.to_string(),
    targets: vec![RouterTarget {
      alias: "openai-gpt".to_string(),
      model: "gpt-4o".to_string(),
      enabled: true,
      weight: None,
    }],
    strategy: RoutingStrategyConfig::default(),
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_model_router_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let router_a = make_router("router-a1", "stack-a", ctx.now);
  ctx
    .service
    .create_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, &router_a)
    .await?;

  let router_b = make_router("router-b1", "stack-b", ctx.now);
  ctx
    .service
    .create_model_router_alias(TEST_TENANT_B_ID, TEST_USER_ID, &router_b)
    .await?;

  let aliases_a = ctx
    .service
    .list_model_router_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, aliases_a.len());
  assert_eq!("router-a1", aliases_a[0].id);

  let cross = ctx
    .service
    .get_model_router_alias(TEST_TENANT_B_ID, TEST_USER_ID, "router-a1")
    .await?;
  assert_eq!(None, cross);

  // alias name "stack-a" is free in tenant B (isolation)
  let cross_name = ctx
    .service
    .check_router_alias_exists(TEST_TENANT_B_ID, TEST_USER_ID, "stack-a", None)
    .await?;
  assert_eq!(false, cross_name);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_intra_tenant_user_model_router_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  let router_a = make_router("router-u1", "stack-u1", ctx.now);
  ctx
    .service
    .create_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, &router_a)
    .await?;

  let router_b = make_router("router-u2", "stack-u2", ctx.now);
  ctx
    .service
    .create_model_router_alias(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, &router_b)
    .await?;

  let aliases_u1 = ctx
    .service
    .list_model_router_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, aliases_u1.len());
  assert_eq!("router-u1", aliases_u1[0].id);

  let cross = ctx
    .service
    .get_model_router_alias(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, "router-u1")
    .await?;
  assert_eq!(None, cross);

  Ok(())
}
