use crate::{
  new_ulid,
  test_utils::{sea_context, setup_env, TEST_TENANT_ID},
  tokens::{TokenEntity, TokenRepository, TokenStatus},
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_token(id: &str, user_id: &str, prefix: &str, now: DateTime<Utc>) -> TokenEntity {
  TokenEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: user_id.to_string(),
    name: "".to_string(),
    token_prefix: prefix.to_string(),
    token_hash: format!("hash_{prefix}"),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    grants: crate::tokens::default_grants_json(),
    last_used_at: None,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_create_and_list_api_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = new_ulid();
  let mut token = make_token(&new_ulid(), &user_id, "bodhiapp_t01", ctx.now);

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let (tokens, total) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, &user_id, 1, 10)
    .await?;
  assert_eq!(1, total);
  assert_eq!(1, tokens.len());
  assert_eq!(token, tokens[0]);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_api_token_by_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = new_ulid();
  let token_id = new_ulid();
  let mut token = make_token(&token_id, &user_id, "bodhiapp_t02", ctx.now);

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let fetched = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, &user_id, &token_id)
    .await?;
  assert!(fetched.is_some());
  assert_eq!(token, fetched.unwrap());

  let not_found = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, &user_id, "nonexistent")
    .await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_get_api_token_by_prefix(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = new_ulid();
  let mut token = make_token(&new_ulid(), &user_id, "bodhiapp_t03", ctx.now);

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let fetched = ctx.service.get_api_token_by_prefix("bodhiapp_t03").await?;
  assert!(fetched.is_some());
  assert_eq!(token, fetched.unwrap());

  let not_found = ctx
    .service
    .get_api_token_by_prefix("nonexistent_prefix")
    .await?;
  assert!(not_found.is_none());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_api_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = new_ulid();
  let token_id = new_ulid();
  let mut token = make_token(&token_id, &user_id, "bodhiapp_t04", ctx.now);

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  token.name = "Updated Name".to_string();
  token.status = TokenStatus::Inactive;
  ctx
    .service
    .update_api_token(TEST_TENANT_ID, &user_id, &mut token)
    .await?;

  let updated = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, &user_id, &token_id)
    .await?
    .expect("Token should exist");

  assert_eq!("Updated Name", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);
  assert_eq!(token_id, updated.id);
  assert_eq!(user_id, updated.user_id);
  // The update ActiveModel uses `..Default::default()`; grants must survive untouched
  // (guards the SeaORM new-column update trap).
  assert_eq!(crate::tokens::default_grants_json(), updated.grants);
  assert_eq!(None, updated.last_used_at);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_list_api_tokens_user_scoped(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user1_id = new_ulid();
  let user2_id = new_ulid();

  let mut token1 = make_token(&new_ulid(), &user1_id, "bodhiapp_t05", ctx.now);
  token1.name = "User1 Token".to_string();
  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token1)
    .await?;

  let mut token2 = make_token(&new_ulid(), &user2_id, "bodhiapp_t06", ctx.now);
  token2.name = "User2 Token".to_string();
  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token2)
    .await?;

  let (tokens, total) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, &user1_id, 1, 10)
    .await?;
  assert_eq!(1, total);
  assert_eq!(1, tokens.len());
  assert_eq!(user1_id, tokens[0].user_id);
  assert_eq!("User1 Token", tokens[0].name);

  let (tokens, total) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, &user2_id, 1, 10)
    .await?;
  assert_eq!(1, total);
  assert_eq!(1, tokens.len());
  assert_eq!(user2_id, tokens[0].user_id);
  assert_eq!("User2 Token", tokens[0].name);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_update_api_token_user_scoped(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user1_id = new_ulid();
  let user2_id = new_ulid();

  let mut token = make_token(&new_ulid(), &user1_id, "bodhiapp_t07", ctx.now);
  token.name = "Initial Name".to_string();
  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  token.name = "Updated Name".to_string();
  let result = ctx
    .service
    .update_api_token(TEST_TENANT_ID, &user2_id, &mut token)
    .await;
  assert!(result.is_err());

  let unchanged = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, &user1_id, &token.id)
    .await?
    .unwrap();
  assert_eq!("Initial Name", unchanged.name);

  let result = ctx
    .service
    .update_api_token(TEST_TENANT_ID, &user1_id, &mut token)
    .await;
  assert!(result.is_ok());

  let updated = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, &user1_id, &token.id)
    .await?
    .unwrap();
  assert_eq!("Updated Name", updated.name);

  Ok(())
}

/// A row inserted without the `grants` column gets the migration's all-access DEFAULT.
/// SQLite-only: a raw insert bypasses the per-tenant txn that PostgreSQL RLS requires.
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_grants_column_default_backfill(_setup_env: ()) -> anyhow::Result<()> {
  use crate::tokens::{api_token_entity, default_grants_json};
  use sea_orm::{ActiveValue::NotSet, EntityTrait, Set};

  let ctx = sea_context("sqlite").await;
  let user_id = new_ulid();
  let token_id = new_ulid();

  // grants + last_used_at intentionally NotSet → omitted from INSERT → DB DEFAULT applies.
  let active = api_token_entity::ActiveModel {
    id: Set(token_id.clone()),
    tenant_id: Set(TEST_TENANT_ID.to_string()),
    user_id: Set(user_id.clone()),
    name: Set("backfill".to_string()),
    token_prefix: Set("bodhiapp_bf".to_string()),
    token_hash: Set("hash_bf".to_string()),
    scopes: Set("scope_token_user".to_string()),
    status: Set(TokenStatus::Active),
    grants: NotSet,
    last_used_at: NotSet,
    created_at: Set(ctx.now),
    updated_at: Set(ctx.now),
  };
  api_token_entity::Entity::insert(active)
    .exec(ctx.service.db())
    .await?;

  let fetched = ctx
    .service
    .get_api_token_by_id(TEST_TENANT_ID, &user_id, &token_id)
    .await?
    .expect("token exists");
  assert_eq!(default_grants_json(), fetched.grants);
  assert_eq!(None, fetched.last_used_at);

  Ok(())
}
