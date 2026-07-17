use crate::{
  test_utils::{
    sea_context, setup_env, token_entity, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID,
    TEST_TENANT_ID, TEST_USER_ID,
  },
  tokens::TokenRepository,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_token_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let mut token_a = token_entity(&crate::new_ulid(), TEST_USER_ID, "sk-bodhiapp_xa", ctx.now);
  let mut token_b = token_entity(&crate::new_ulid(), TEST_USER_ID, "sk-bodhiapp_xb", ctx.now);
  token_b.tenant_id = TEST_TENANT_B_ID.to_string();

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_a)
    .await?;
  ctx
    .service
    .create_api_token(TEST_TENANT_B_ID, &mut token_b)
    .await?;

  let (tokens_a, total_a) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, TEST_USER_ID, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(1, tokens_a.len());
  assert_eq!(TEST_TENANT_ID, tokens_a[0].tenant_id);

  let (tokens_b, total_b) = ctx
    .service
    .list_api_tokens(TEST_TENANT_B_ID, TEST_USER_ID, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, tokens_b.len());
  assert_eq!(TEST_TENANT_B_ID, tokens_b[0].tenant_id);

  let cross = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, TEST_USER_ID, &token_b.id)
    .await?;
  assert_eq!(None, cross);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_token_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let mut token_user_a = token_entity(&crate::new_ulid(), TEST_USER_ID, "sk-bodhiapp_ua", ctx.now);
  let mut token_user_b = token_entity(
    &crate::new_ulid(),
    TEST_TENANT_A_USER_B_ID,
    "sk-bodhiapp_ub",
    ctx.now,
  );

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_user_a)
    .await?;
  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_user_b)
    .await?;

  let (tokens_a, total_a) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, TEST_USER_ID, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(1, tokens_a.len());
  assert_eq!(TEST_USER_ID, tokens_a[0].user_id);

  let (tokens_b, total_b) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, tokens_b.len());
  assert_eq!(TEST_TENANT_A_USER_B_ID, tokens_b[0].user_id);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_token_delete_blocked(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let mut token_b = token_entity(&crate::new_ulid(), TEST_USER_ID, "sk-bodhiapp_db", ctx.now);
  token_b.tenant_id = TEST_TENANT_B_ID.to_string();
  ctx
    .service
    .create_api_token(TEST_TENANT_B_ID, &mut token_b)
    .await?;

  // Tenant A cannot delete tenant B's token.
  let err = ctx
    .service
    .delete_api_token(TEST_TENANT_ID, TEST_USER_ID, &token_b.id)
    .await
    .unwrap_err();
  assert!(matches!(err, crate::db::DbError::ItemNotFound { .. }));

  // The token is still present in tenant B.
  let still_there = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_B_ID, TEST_USER_ID, &token_b.id)
    .await?;
  assert!(still_there.is_some());

  Ok(())
}
