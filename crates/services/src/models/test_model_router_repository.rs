use crate::models::{ModelRouterAlias, ModelRouterRepository, RouterTarget, RoutingStrategyConfig};
use crate::test_utils::{sea_context, setup_env, TEST_TENANT_ID, TEST_USER_ID};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_router(id: &str, alias: &str, now: DateTime<Utc>) -> ModelRouterAlias {
  ModelRouterAlias {
    id: id.to_string(),
    alias: alias.to_string(),
    targets: vec![
      RouterTarget {
        alias: "openai-gpt".to_string(),
        model: "gpt-4o".to_string(),
        enabled: true,
        weight: None,
      },
      RouterTarget {
        alias: "claude".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        enabled: false,
        weight: None,
      },
    ],
    strategy: RoutingStrategyConfig::default(),
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_model_router_crud_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let router = make_router("router-1", "my-stack", ctx.now);

  ctx
    .service
    .create_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, &router)
    .await?;

  // get round-trips targets + strategy from the JSON columns
  let fetched = ctx
    .service
    .get_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, "router-1")
    .await?
    .expect("router should exist");
  assert_eq!(router, fetched);
  assert_eq!(2, fetched.targets.len());
  assert!(fetched.targets[0].enabled);
  assert!(!fetched.targets[1].enabled);

  let listed = ctx
    .service
    .list_model_router_aliases(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, listed.len());

  // rename + drop a target
  let mut updated = router.clone();
  updated.alias = "my-stack-2".to_string();
  updated.targets.truncate(1);
  ctx
    .service
    .update_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, "router-1", &updated)
    .await?;
  let after = ctx
    .service
    .get_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, "router-1")
    .await?
    .expect("router should still exist");
  assert_eq!("my-stack-2", after.alias);
  assert_eq!(1, after.targets.len());

  ctx
    .service
    .delete_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, "router-1")
    .await?;
  let gone = ctx
    .service
    .get_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, "router-1")
    .await?;
  assert_eq!(None, gone);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_model_router_check_alias_exists(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let router = make_router("router-1", "my-stack", ctx.now);
  ctx
    .service
    .create_model_router_alias(TEST_TENANT_ID, TEST_USER_ID, &router)
    .await?;

  assert!(
    ctx
      .service
      .check_router_alias_exists(TEST_TENANT_ID, TEST_USER_ID, "my-stack", None)
      .await?
  );
  // Excluding the owning id makes it not-a-clash (used for self-update).
  assert!(
    !ctx
      .service
      .check_router_alias_exists(
        TEST_TENANT_ID,
        TEST_USER_ID,
        "my-stack",
        Some("router-1".to_string())
      )
      .await?
  );
  assert!(
    !ctx
      .service
      .check_router_alias_exists(TEST_TENANT_ID, TEST_USER_ID, "absent", None)
      .await?
  );

  Ok(())
}
