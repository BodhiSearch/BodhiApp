use super::{DefaultTokenService, TokenService};
use crate::db::ApiToken;
use crate::test_utils::{test_db_service, TestDbService};
use anyhow_trace::anyhow_trace;
use objs::TokenStatus;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_and_get_api_token(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  let db_service = Arc::new(db_service);
  let token_service = DefaultTokenService::new(db_service.clone());

  let mut token = ApiToken {
    id: "tok_test1".to_string(),
    user_id: "user1".to_string(),
    name: "test-token".to_string(),
    token_hash: "hash123".to_string(),
    token_prefix: "bt_abc".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };

  token_service.create_api_token(&mut token).await?;

  let retrieved = token_service
    .get_api_token_by_id("user1", "tok_test1")
    .await?;
  assert!(retrieved.is_some());
  let retrieved = retrieved.unwrap();
  assert_eq!("test-token", retrieved.name);
  assert_eq!("bt_abc", retrieved.token_prefix);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_api_tokens(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  let db_service = Arc::new(db_service);
  let token_service = DefaultTokenService::new(db_service.clone());

  for i in 0..3 {
    let mut token = ApiToken {
      id: format!("tok_{i}"),
      user_id: "user1".to_string(),
      name: format!("token-{i}"),
      token_hash: format!("hash_{i}"),
      token_prefix: format!("bt_{i}"),
      scopes: "user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    token_service.create_api_token(&mut token).await?;
  }

  let (tokens, total) = token_service.list_api_tokens("user1", 1, 10).await?;
  assert_eq!(3, total);
  assert_eq!(3, tokens.len());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_api_token_by_prefix(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  let db_service = Arc::new(db_service);
  let token_service = DefaultTokenService::new(db_service.clone());

  let mut token = ApiToken {
    id: "tok_prefix".to_string(),
    user_id: "user1".to_string(),
    name: "prefix-token".to_string(),
    token_hash: "hash_prefix".to_string(),
    token_prefix: "bt_unique".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  token_service.create_api_token(&mut token).await?;

  let result = token_service.get_api_token_by_prefix("bt_unique").await?;
  assert!(result.is_some());
  assert_eq!("prefix-token", result.unwrap().name);

  let result = token_service.get_api_token_by_prefix("bt_nonexist").await?;
  assert!(result.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_api_token(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  let db_service = Arc::new(db_service);
  let token_service = DefaultTokenService::new(db_service.clone());

  let mut token = ApiToken {
    id: "tok_update".to_string(),
    user_id: "user1".to_string(),
    name: "original-name".to_string(),
    token_hash: "hash_update".to_string(),
    token_prefix: "bt_upd".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  token_service.create_api_token(&mut token).await?;

  token.name = "updated-name".to_string();
  token.status = TokenStatus::Inactive;
  token_service.update_api_token("user1", &mut token).await?;

  let updated = token_service
    .get_api_token_by_id("user1", "tok_update")
    .await?
    .unwrap();
  assert_eq!("updated-name", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);

  Ok(())
}
