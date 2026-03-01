use crate::{
  test_utils::{sea_context, setup_env},
  tokens::{ApiToken, TokenRepository, TokenStatus},
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_token(id: &str, user_id: &str, prefix: &str, now: DateTime<Utc>) -> ApiToken {
  ApiToken {
    id: id.to_string(),
    user_id: user_id.to_string(),
    name: "".to_string(),
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
async fn test_create_and_list_api_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let user_id = ulid::Ulid::new().to_string();
  let mut token = make_token(
    &ulid::Ulid::new().to_string(),
    &user_id,
    "bodhiapp_t01",
    ctx.now,
  );

  ctx.service.create_api_token(&mut token).await?;

  let (tokens, total) = ctx.service.list_api_tokens(&user_id, 1, 10).await?;
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
  let user_id = ulid::Ulid::new().to_string();
  let token_id = ulid::Ulid::new().to_string();
  let mut token = make_token(&token_id, &user_id, "bodhiapp_t02", ctx.now);

  ctx.service.create_api_token(&mut token).await?;

  let fetched = ctx.service.get_api_token_by_id(&user_id, &token_id).await?;
  assert!(fetched.is_some());
  assert_eq!(token, fetched.unwrap());

  let not_found = ctx
    .service
    .get_api_token_by_id(&user_id, "nonexistent")
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
  let user_id = ulid::Ulid::new().to_string();
  let mut token = make_token(
    &ulid::Ulid::new().to_string(),
    &user_id,
    "bodhiapp_t03",
    ctx.now,
  );

  ctx.service.create_api_token(&mut token).await?;

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
  let user_id = ulid::Ulid::new().to_string();
  let token_id = ulid::Ulid::new().to_string();
  let mut token = make_token(&token_id, &user_id, "bodhiapp_t04", ctx.now);

  ctx.service.create_api_token(&mut token).await?;

  token.name = "Updated Name".to_string();
  token.status = TokenStatus::Inactive;
  ctx.service.update_api_token(&user_id, &mut token).await?;

  let updated = ctx
    .service
    .get_api_token_by_id(&user_id, &token_id)
    .await?
    .expect("Token should exist");

  assert_eq!("Updated Name", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);
  assert_eq!(token_id, updated.id);
  assert_eq!(user_id, updated.user_id);

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
  let user1_id = ulid::Ulid::new().to_string();
  let user2_id = ulid::Ulid::new().to_string();

  let mut token1 = make_token(
    &ulid::Ulid::new().to_string(),
    &user1_id,
    "bodhiapp_t05",
    ctx.now,
  );
  token1.name = "User1 Token".to_string();
  ctx.service.create_api_token(&mut token1).await?;

  let mut token2 = make_token(
    &ulid::Ulid::new().to_string(),
    &user2_id,
    "bodhiapp_t06",
    ctx.now,
  );
  token2.name = "User2 Token".to_string();
  ctx.service.create_api_token(&mut token2).await?;

  let (tokens, total) = ctx.service.list_api_tokens(&user1_id, 1, 10).await?;
  assert_eq!(1, total);
  assert_eq!(1, tokens.len());
  assert_eq!(user1_id, tokens[0].user_id);
  assert_eq!("User1 Token", tokens[0].name);

  let (tokens, total) = ctx.service.list_api_tokens(&user2_id, 1, 10).await?;
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
  let user1_id = ulid::Ulid::new().to_string();
  let user2_id = ulid::Ulid::new().to_string();

  let mut token = make_token(
    &ulid::Ulid::new().to_string(),
    &user1_id,
    "bodhiapp_t07",
    ctx.now,
  );
  token.name = "Initial Name".to_string();
  ctx.service.create_api_token(&mut token).await?;

  token.name = "Updated Name".to_string();
  let result = ctx.service.update_api_token(&user2_id, &mut token).await;
  assert!(result.is_err());

  let unchanged = ctx
    .service
    .get_api_token_by_id(&user1_id, &token.id)
    .await?
    .unwrap();
  assert_eq!("Initial Name", unchanged.name);

  let result = ctx.service.update_api_token(&user1_id, &mut token).await;
  assert!(result.is_ok());

  let updated = ctx
    .service
    .get_api_token_by_id(&user1_id, &token.id)
    .await?
    .unwrap();
  assert_eq!("Updated Name", updated.name);

  Ok(())
}
