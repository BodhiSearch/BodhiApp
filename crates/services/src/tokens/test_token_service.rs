use super::{DefaultTokenService, TokenService};
use crate::test_utils::{test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID};
use crate::tokens::TokenEntity;
use crate::TokenScope;
use crate::TokenStatus;
use crate::{CreateTokenRequest, UpdateTokenRequest};
use anyhow_trace::anyhow_trace;
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
  let time_service = Arc::new(FrozenTimeService::default());
  let token_service = DefaultTokenService::new(db_service.clone(), time_service);

  let mut token = TokenEntity {
    id: "tok_test1".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "user1".to_string(),
    name: "test-token".to_string(),
    token_hash: "hash123".to_string(),
    token_prefix: "bt_abc".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };

  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let retrieved = token_service
    .get_api_token_by_id(TEST_TENANT_ID, "user1", "tok_test1")
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
  let time_service = Arc::new(FrozenTimeService::default());
  let token_service = DefaultTokenService::new(db_service.clone(), time_service);

  for i in 0..3 {
    let mut token = TokenEntity {
      id: format!("tok_{i}"),
      tenant_id: TEST_TENANT_ID.to_string(),
      user_id: "user1".to_string(),
      name: format!("token-{i}"),
      token_hash: format!("hash_{i}"),
      token_prefix: format!("bt_{i}"),
      scopes: "user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    token_service
      .create_api_token(TEST_TENANT_ID, &mut token)
      .await?;
  }

  let response = token_service
    .list_api_tokens(TEST_TENANT_ID, "user1", 1, 10)
    .await?;
  assert_eq!(3, response.total);
  assert_eq!(3, response.data.len());

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
  let time_service = Arc::new(FrozenTimeService::default());
  let token_service = DefaultTokenService::new(db_service.clone(), time_service);

  let mut token = TokenEntity {
    id: "tok_prefix".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "user1".to_string(),
    name: "prefix-token".to_string(),
    token_hash: "hash_prefix".to_string(),
    token_prefix: "bt_unique".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

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
  let time_service = Arc::new(FrozenTimeService::default());
  let token_service = DefaultTokenService::new(db_service.clone(), time_service);

  let mut token = TokenEntity {
    id: "tok_update".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "user1".to_string(),
    name: "original-name".to_string(),
    token_hash: "hash_update".to_string(),
    token_prefix: "bt_upd".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  token.name = "updated-name".to_string();
  token.status = TokenStatus::Inactive;
  token_service
    .update_api_token(TEST_TENANT_ID, "user1", &mut token)
    .await?;

  let updated = token_service
    .get_api_token_by_id(TEST_TENANT_ID, "user1", "tok_update")
    .await?
    .unwrap();
  assert_eq!("updated-name", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_generates_valid_token(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);
  let time_service = Arc::new(FrozenTimeService::default());
  let token_service = DefaultTokenService::new(db_service.clone(), time_service);

  let form = CreateTokenRequest {
    name: Some("my-token".to_string()),
    scope: TokenScope::User,
  };
  let token_created = token_service
    .create_token(TEST_TENANT_ID, "user1", form)
    .await?;

  let raw_token = &token_created.token;

  // Raw token starts with bodhiapp_ prefix
  assert!(raw_token.starts_with("bodhiapp_"));

  // Token has .<client_id> suffix (new multi-tenant format)
  assert!(
    raw_token.contains('.'),
    "Token should contain '.' separator"
  );

  // Token prefix is "bodhiapp_" + first 8 chars of random part (not including .<client_id>)
  let random_part = raw_token
    .strip_prefix("bodhiapp_")
    .unwrap()
    .split('.')
    .next()
    .unwrap();
  let token_prefix = format!("bodhiapp_{}", &random_part[..8]);

  // Can retrieve by prefix
  let retrieved = token_service.get_api_token_by_prefix(&token_prefix).await?;
  assert!(retrieved.is_some());
  let entity = retrieved.unwrap();

  // Token is persisted and retrievable
  assert_eq!("my-token", entity.name);
  assert_eq!("user1", entity.user_id);
  assert_eq!("scope_token_user", entity.scopes);
  assert_eq!(TokenStatus::Active, entity.status);
  assert_eq!(token_prefix, entity.token_prefix);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_token_with_form(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();
  let db_service = Arc::new(db_service);
  let time_service = Arc::new(FrozenTimeService::default());
  let token_service = DefaultTokenService::new(db_service.clone(), time_service);

  let mut token = TokenEntity {
    id: "tok_form_update".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "user1".to_string(),
    name: "original-name".to_string(),
    token_hash: "hash_form".to_string(),
    token_prefix: "bt_form".to_string(),
    scopes: "user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  token_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let form = UpdateTokenRequest {
    name: "updated-via-form".to_string(),
    status: TokenStatus::Inactive,
  };
  let updated = token_service
    .update_token(TEST_TENANT_ID, "user1", "tok_form_update", form)
    .await?;

  assert_eq!("updated-via-form", updated.name);
  assert_eq!(TokenStatus::Inactive, updated.status);

  Ok(())
}
