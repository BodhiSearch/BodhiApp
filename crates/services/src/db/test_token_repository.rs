use crate::{
  db::{ApiToken, DbError, SqlxError, TokenRepository, TokenStatus},
  test_utils::{test_db_service, TestDbService},
};
use anyhow_trace::anyhow_trace;
use chrono::Utc;
use pretty_assertions::assert_eq;
use rstest::rstest;
use uuid::Uuid;

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_create_api_token(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Create token
  let user_id = Uuid::new_v4().to_string();
  let mut token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: user_id.clone(),
    name: "".to_string(),
    token_prefix: "bodhiapp_test01".to_string(),
    token_hash: "token_hash".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };

  service.create_api_token(&mut token).await?;

  // List tokens
  let (tokens, _) = service.list_api_tokens(&user_id, 1, 10).await?;
  assert_eq!(1, tokens.len());

  assert_eq!(token, tokens[0]);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_api_token(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  // Create initial token
  let mut token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test_user".to_string(),
    name: "Initial Name".to_string(),
    token_prefix: "bodhiapp_test02".to_string(),
    token_hash: "token_hash".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  service.create_api_token(&mut token).await?;

  // Update token
  token.name = "Updated Name".to_string();
  token.status = TokenStatus::Inactive;
  token.updated_at = Utc::now();
  service.update_api_token("test_user", &mut token).await?;
  // Verify update
  let updated = service
    .get_api_token_by_id("test_user", &token.id)
    .await?
    .unwrap();
  assert_eq!(updated.name, "Updated Name");
  assert_eq!(updated.status, TokenStatus::Inactive);
  assert_eq!(updated.id, token.id);
  assert_eq!(updated.user_id, token.user_id);
  assert_eq!(updated.token_prefix, token.token_prefix);
  assert_eq!(updated.created_at, token.created_at);
  assert!(updated.updated_at >= token.updated_at);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_list_api_tokens_user_scoped(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Create tokens for two different users
  let user1_id = "user1";
  let user2_id = "user2";

  // Create token for user1
  let mut token1 = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: user1_id.to_string(),
    name: "User1 Token".to_string(),
    token_prefix: "bodhiapp_test03".to_string(),
    token_hash: "hash1".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  service.create_api_token(&mut token1).await?;

  // Create token for user2
  let mut token2 = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: user2_id.to_string(),
    name: "User2 Token".to_string(),
    token_prefix: "bodhiapp_test04".to_string(),
    token_hash: "hash2".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  service.create_api_token(&mut token2).await?;

  // List tokens for user1
  let (tokens, total) = service.list_api_tokens(user1_id, 1, 10).await?;
  assert_eq!(tokens.len(), 1);
  assert_eq!(total, 1);
  assert_eq!(tokens[0].user_id, user1_id);
  assert_eq!(tokens[0].name, "User1 Token");

  // List tokens for user2
  let (tokens, total) = service.list_api_tokens(user2_id, 1, 10).await?;
  assert_eq!(tokens.len(), 1);
  assert_eq!(total, 1);
  assert_eq!(tokens[0].user_id, user2_id);
  assert_eq!(tokens[0].name, "User2 Token");

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_api_token_user_scoped(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Create a token for user1
  let user1_id = "user1";
  let mut token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: user1_id.to_string(),
    name: "Initial Name".to_string(),
    token_prefix: "bodhiapp_test05".to_string(),
    token_hash: "hash".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  service.create_api_token(&mut token).await?;

  // Try to update token as user2 (should fail)
  let user2_id = "user2";
  token.name = "Updated Name".to_string();
  let result = service.update_api_token(user2_id, &mut token).await;
  assert!(matches!(
    result,
    Err(DbError::SqlxError(SqlxError { source })) if source.to_string() == sqlx::Error::RowNotFound.to_string()
  ));

  // Verify token was not updated
  let unchanged = service
    .get_api_token_by_id(user1_id, &token.id)
    .await?
    .unwrap();
  assert_eq!(unchanged.name, "Initial Name");
  assert_eq!(unchanged.user_id, user1_id);

  // Update token as user1 (should succeed)
  let result = service.update_api_token(user1_id, &mut token).await;
  assert!(result.is_ok());

  // Verify the update succeeded
  let updated = service
    .get_api_token_by_id(user1_id, &token.id)
    .await?
    .unwrap();
  assert_eq!(updated.name, "Updated Name");
  assert_eq!(updated.user_id, user1_id);

  Ok(())
}
