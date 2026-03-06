use crate::{
  test_utils::{
    sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
  },
  tokens::{TokenEntity, TokenRepository, TokenStatus},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_token(
  id: &str,
  user_id: &str,
  prefix: &str,
  tenant_id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> TokenEntity {
  TokenEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    user_id: user_id.to_string(),
    name: format!("Token {prefix}"),
    token_prefix: prefix.to_string(),
    token_hash: format!("hash_{prefix}"),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_token_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let mut token_a = make_token(
    &ulid::Ulid::new().to_string(),
    TEST_USER_ID,
    "bodhiapp_xa",
    TEST_TENANT_ID,
    ctx.now,
  );
  let mut token_b = make_token(
    &ulid::Ulid::new().to_string(),
    TEST_USER_ID,
    "bodhiapp_xb",
    TEST_TENANT_B_ID,
    ctx.now,
  );

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_a)
    .await?;
  ctx
    .service
    .create_api_token(TEST_TENANT_B_ID, &mut token_b)
    .await?;

  // Listing tokens in tenant A should only return token_a
  let (tokens_a, total_a) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, TEST_USER_ID, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(1, tokens_a.len());
  assert_eq!(TEST_TENANT_ID, tokens_a[0].tenant_id);

  // Listing tokens in tenant B should only return token_b
  let (tokens_b, total_b) = ctx
    .service
    .list_api_tokens(TEST_TENANT_B_ID, TEST_USER_ID, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, tokens_b.len());
  assert_eq!(TEST_TENANT_B_ID, tokens_b[0].tenant_id);

  // Getting token_b by ID under tenant A should return None
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
  let mut token_user_a = make_token(
    &ulid::Ulid::new().to_string(),
    TEST_USER_ID,
    "bodhiapp_ua",
    TEST_TENANT_ID,
    ctx.now,
  );
  let mut token_user_b = make_token(
    &ulid::Ulid::new().to_string(),
    TEST_TENANT_A_USER_B_ID,
    "bodhiapp_ub",
    TEST_TENANT_ID,
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

  // Listing tokens for user A should only return token_user_a
  let (tokens_a, total_a) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, TEST_USER_ID, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(1, tokens_a.len());
  assert_eq!(TEST_USER_ID, tokens_a[0].user_id);

  // Listing tokens for user B should only return token_user_b
  let (tokens_b, total_b) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, tokens_b.len());
  assert_eq!(TEST_TENANT_A_USER_B_ID, tokens_b[0].user_id);

  Ok(())
}
