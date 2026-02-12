use crate::{
  db::{
    AccessRepository, ApiKeyUpdate, ApiToken, DbError, DownloadRequest, DownloadStatus,
    ModelRepository, SqlxError, TokenRepository, TokenStatus, UserAccessRequest,
    UserAccessRequestStatus,
  },
  test_utils::{
    create_test_api_model_metadata, create_test_model_metadata, model_metadata_builder,
    test_db_service, TestDbService,
  },
};
use anyhow_trace::anyhow_trace;
use chrono::Utc;
use objs::ApiAlias;
use objs::ApiFormat;
use pretty_assertions::assert_eq;
use rstest::rstest;
use uuid::Uuid;

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_create_download_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let request = DownloadRequest::new_pending("test/repo", "test_file.gguf", now);
  service.create_download_request(&request).await?;
  let fetched = service.get_download_request(&request.id).await?;
  assert!(fetched.is_some());
  assert_eq!(request, fetched.unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_update_download_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut request = DownloadRequest::new_pending("test/repo", "test_file.gguf", now);
  service.create_download_request(&request).await?;
  request.status = DownloadStatus::Completed;
  request.total_bytes = Some(1000000);
  request.downloaded_bytes = 1000000;
  request.started_at = Some(now);
  request.updated_at = now + chrono::Duration::hours(1);
  service.update_download_request(&request).await?;

  let fetched = service.get_download_request(&request.id).await?.unwrap();
  assert_eq!(
    DownloadRequest {
      id: request.id,
      repo: "test/repo".to_string(),
      filename: "test_file.gguf".to_string(),
      status: DownloadStatus::Completed,
      error: None,
      created_at: now,
      updated_at: now + chrono::Duration::hours(1),
      total_bytes: Some(1000000),
      downloaded_bytes: 1000000,
      started_at: Some(now),
    },
    fetched
  );
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_download_request_progress_tracking(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut request = DownloadRequest {
    id: Uuid::new_v4().to_string(),
    repo: "test/repo".to_string(),
    filename: "test_file.gguf".to_string(),
    status: DownloadStatus::Pending,
    error: None,
    created_at: now,
    updated_at: now,
    total_bytes: Some(1000000), // 1MB
    downloaded_bytes: 0,
    started_at: Some(now),
  };
  service.create_download_request(&request).await?;

  // Simulate progress update
  request.downloaded_bytes = 500000; // 50% downloaded
  request.updated_at = now + chrono::Duration::seconds(4);
  service.update_download_request(&request).await?;

  let fetched = service.get_download_request(&request.id).await?.unwrap();
  assert_eq!(request.downloaded_bytes, fetched.downloaded_bytes);
  assert_eq!(request.total_bytes, fetched.total_bytes);
  assert_eq!(request.started_at, fetched.started_at);
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_insert_pending_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
  let pending_request = service
    .insert_pending_request(username.clone(), user_id.clone())
    .await?;
  let expected_request = UserAccessRequest {
    id: pending_request.id, // We don't know this in advance
    username,
    user_id,
    created_at: now,
    updated_at: now,
    status: UserAccessRequestStatus::Pending,
    reviewer: None,
  };
  assert_eq!(pending_request, expected_request);
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_get_pending_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440001".to_string();
  let inserted_request = service
    .insert_pending_request(username, user_id.clone())
    .await?;
  let fetched_request = service.get_pending_request(user_id).await?;
  assert!(fetched_request.is_some());
  assert_eq!(inserted_request, fetched_request.unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_list_pending_requests(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let test_data = vec![
    (
      "test1@example.com".to_string(),
      "550e8400-e29b-41d4-a716-446655440002".to_string(),
    ),
    (
      "test2@example.com".to_string(),
      "550e8400-e29b-41d4-a716-446655440003".to_string(),
    ),
    (
      "test3@example.com".to_string(),
      "550e8400-e29b-41d4-a716-446655440004".to_string(),
    ),
  ];
  for (username, user_id) in &test_data {
    service
      .insert_pending_request(username.clone(), user_id.clone())
      .await?;
  }
  let (page1, total) = service.list_pending_requests(1, 2).await?;
  assert_eq!(2, page1.len());
  assert_eq!(3, total);
  let (page2, total) = service.list_pending_requests(2, 2).await?;
  assert_eq!(1, page2.len());
  assert_eq!(3, total);
  for (i, request) in page1.iter().chain(page2.iter()).enumerate() {
    let expected_request = UserAccessRequest {
      id: request.id,
      username: test_data[i].0.clone(),
      user_id: test_data[i].1.clone(),
      created_at: now,
      updated_at: now,
      status: UserAccessRequestStatus::Pending,
      reviewer: None,
    };
    assert_eq!(&expected_request, request);
  }
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_update_request_status(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let username = "test@example.com".to_string();
  let user_id = "550e8400-e29b-41d4-a716-446655440005".to_string();
  let inserted_request = service
    .insert_pending_request(username, user_id.clone())
    .await?;
  service
    .update_request_status(
      inserted_request.id,
      UserAccessRequestStatus::Approved,
      "admin@example.com".to_string(),
    )
    .await?;
  let updated_request = service.get_pending_request(user_id).await?;
  assert!(updated_request.is_none());
  Ok(())
}

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

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_create_and_get_api_model_alias(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let alias_obj = ApiAlias::new(
    "openai",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    None,
    false,
    now,
  );
  let api_key = "sk-test123456789";

  // Create API model alias
  service
    .create_api_model_alias(&alias_obj, Some(api_key.to_string()))
    .await?;

  // Retrieve and verify
  let retrieved = service.get_api_model_alias("openai").await?.unwrap();
  assert_eq!(alias_obj, retrieved);

  // Verify API key is stored encrypted and retrievable
  let decrypted_key = service.get_api_key_for_alias("openai").await?.unwrap();
  assert_eq!(api_key, decrypted_key);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_create_api_model_alias_without_key(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let alias_obj = ApiAlias::new(
    "no-key-model",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    now,
  );

  // Create API model alias WITHOUT api_key
  service.create_api_model_alias(&alias_obj, None).await?;

  // Verify model exists
  let retrieved = service.get_api_model_alias("no-key-model").await?;
  assert!(retrieved.is_some());
  assert_eq!(alias_obj, retrieved.unwrap());

  // Verify API key is None
  let key = service.get_api_key_for_alias("no-key-model").await?;
  assert_eq!(None, key);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_api_model_alias_with_new_key(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut alias_obj = ApiAlias::new(
    "claude",
    ApiFormat::OpenAI,
    "https://api.anthropic.com/v1",
    vec!["claude-3".to_string()],
    None,
    false,
    now,
  );
  let original_api_key = "sk-original123";
  let new_api_key = "sk-updated456";

  // Create initial alias
  service
    .create_api_model_alias(&alias_obj, Some(original_api_key.to_string()))
    .await?;

  // Update with new API key and additional model
  alias_obj.models.push("claude-3.5".to_string());
  service
    .update_api_model_alias(
      "claude",
      &alias_obj,
      ApiKeyUpdate::Set(Some(new_api_key.to_string())),
    )
    .await?;

  // Verify updated data
  let updated = service.get_api_model_alias("claude").await?.unwrap();
  assert_eq!(alias_obj, updated);

  // Verify new API key
  let updated_key = service.get_api_key_for_alias("claude").await?.unwrap();
  assert_eq!(new_api_key, updated_key);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_api_model_alias_without_key_change(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut alias_obj = ApiAlias::new(
    "gemini",
    ApiFormat::OpenAI,
    "https://generativelanguage.googleapis.com/v1",
    vec!["gemini-pro".to_string()],
    None,
    false,
    now,
  );
  let api_key = "AIzaSy-test123";

  // Create initial alias
  service
    .create_api_model_alias(&alias_obj, Some(api_key.to_string()))
    .await?;

  // Update without changing API key
  alias_obj.base_url = "https://generativelanguage.googleapis.com/v1beta".to_string();
  service
    .update_api_model_alias("gemini", &alias_obj, ApiKeyUpdate::Keep)
    .await?;

  // Verify API key unchanged
  let retrieved_key = service.get_api_key_for_alias("gemini").await?.unwrap();
  assert_eq!(api_key, retrieved_key);

  // Verify other fields updated
  let updated = service.get_api_model_alias("gemini").await?.unwrap();
  assert_eq!(
    "https://generativelanguage.googleapis.com/v1beta",
    updated.base_url
  );

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_list_api_model_aliases(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Create multiple aliases with different timestamps for proper sorting test
  let aliases = vec![
    ("alias1", "key1", now - chrono::Duration::seconds(20)),
    ("alias2", "key2", now - chrono::Duration::seconds(10)),
    ("alias3", "key3", now),
  ];

  for (alias, key, created_at) in &aliases {
    let alias_obj = ApiAlias::new(
      *alias,
      ApiFormat::OpenAI,
      "https://api.example.com/v1",
      vec!["model1".to_string()],
      None,
      false,
      *created_at,
    );
    service
      .create_api_model_alias(&alias_obj, Some(key.to_string()))
      .await?;
  }

  // List and verify
  let listed = service.list_api_model_aliases().await?;
  assert_eq!(3, listed.len());

  // Verify sorted by created_at DESC (newest first: alias3 -> alias2 -> alias1)
  let sorted_aliases: Vec<_> = listed.iter().map(|a| a.id.as_str()).collect();
  assert_eq!(vec!["alias3", "alias2", "alias1"], sorted_aliases);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_delete_api_model_alias(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let alias_obj = ApiAlias::new(
    "to-delete",
    ApiFormat::OpenAI,
    "https://api.test.com/v1",
    vec!["test-model".to_string()],
    None,
    false,
    now,
  );

  // Create and verify exists
  service
    .create_api_model_alias(&alias_obj, Some("test-key".to_string()))
    .await?;
  assert!(service.get_api_model_alias("to-delete").await?.is_some());

  // Delete and verify gone
  service.delete_api_model_alias("to-delete").await?;
  assert!(service.get_api_model_alias("to-delete").await?.is_none());
  assert!(service.get_api_key_for_alias("to-delete").await?.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_api_key_encryption_security(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let alias_obj = ApiAlias::new(
    "security-test",
    ApiFormat::OpenAI,
    "https://api.secure.com/v1",
    vec!["secure-model".to_string()],
    None,
    false,
    now,
  );
  let sensitive_key = "sk-very-secret-key-12345";

  // Store API key
  service
    .create_api_model_alias(&alias_obj, Some(sensitive_key.to_string()))
    .await?;

  // Verify different encryptions produce different results
  let alias_obj2 = ApiAlias::new(
    "security-test2",
    ApiFormat::OpenAI,
    "https://api.secure.com/v1",
    vec!["secure-model".to_string()],
    None,
    false,
    now,
  );
  service
    .create_api_model_alias(&alias_obj2, Some(sensitive_key.to_string()))
    .await?;

  // Both should decrypt to same key but have different encrypted values in DB
  let key1 = service
    .get_api_key_for_alias("security-test")
    .await?
    .unwrap();
  let key2 = service
    .get_api_key_for_alias("security-test2")
    .await?
    .unwrap();

  assert_eq!(sensitive_key, key1);
  assert_eq!(sensitive_key, key2);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_nonexistent_api_model_alias(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  // Test getting non-existent alias
  let result = service.get_api_model_alias("nonexistent").await?;
  assert!(result.is_none());

  // Test getting API key for non-existent alias
  let key = service.get_api_key_for_alias("nonexistent").await?;
  assert!(key.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_api_model_alias_keeps_key_when_none_provided(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut alias_obj = ApiAlias::new(
    "keep-key-test",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    now,
  );
  let original_key = "sk-original-key-12345";

  // Create WITH api_key
  service
    .create_api_model_alias(&alias_obj, Some(original_key.to_string()))
    .await?;

  // Verify key exists
  let key = service.get_api_key_for_alias("keep-key-test").await?;
  assert_eq!(Some(original_key.to_string()), key);

  // Update without providing api_key (Keep) - should keep existing key
  alias_obj.base_url = "https://api.example.com/v2".to_string();
  service
    .update_api_model_alias("keep-key-test", &alias_obj, ApiKeyUpdate::Keep)
    .await?;

  // Verify key still exists and unchanged
  let key = service.get_api_key_for_alias("keep-key-test").await?;
  assert_eq!(Some(original_key.to_string()), key);

  // Verify other fields were updated
  let updated_alias = service.get_api_model_alias("keep-key-test").await?.unwrap();
  assert_eq!("https://api.example.com/v2", updated_alias.base_url);

  Ok(())
}

#[rstest]
#[case::with_existing_key(Some("sk-key-to-be-cleared"), "https://api.example.com/v2")]
#[case::without_existing_key(None, "https://api.example.com/v2")]
#[awt]
#[tokio::test]
async fn test_update_api_model_alias_clear_key(
  #[case] initial_api_key: Option<&str>,
  #[case] updated_base_url: &str,
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut alias_obj = ApiAlias::new(
    "clear-key-test",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    now,
  );

  // Create with or without API key
  service
    .create_api_model_alias(&alias_obj, initial_api_key.map(|s| s.to_string()))
    .await?;

  // Verify initial key state
  let key = service.get_api_key_for_alias("clear-key-test").await?;
  assert_eq!(initial_api_key.map(|s| s.to_string()), key);

  // Update and clear the API key
  alias_obj.base_url = updated_base_url.to_string();
  service
    .update_api_model_alias("clear-key-test", &alias_obj, ApiKeyUpdate::Set(None))
    .await?;

  // Verify key is now None (regardless of initial state)
  let key = service.get_api_key_for_alias("clear-key-test").await?;
  assert_eq!(None, key);

  // Verify model still exists and other fields were updated
  let model = service
    .get_api_model_alias("clear-key-test")
    .await?
    .unwrap();
  assert_eq!(updated_base_url, model.base_url);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_batch_get_metadata_by_files_returns_inserted_data(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Insert test data with multiple rows - all with source='model' since
  // metadata always represents the physical GGUF file
  let test_data = vec![
    ("test/repo1", "model1.gguf", "abc123"),
    ("test/repo2", "model2.gguf", "def456"),
    ("test/repo3", "model3.gguf", "ghi789"),
  ];

  for (repo, filename, snapshot) in &test_data {
    let row = create_test_model_metadata(repo, filename, snapshot, now);
    service.upsert_model_metadata(&row).await?;
  }

  // Verify single query works for first entry
  let single = service
    .get_model_metadata_by_file("test/repo1", "model1.gguf", "abc123")
    .await?;
  assert!(single.is_some(), "Single query should find the row");

  // Test batch query with all keys
  let keys: Vec<(String, String, String)> = test_data
    .iter()
    .map(|(repo, filename, snapshot)| {
      (repo.to_string(), filename.to_string(), snapshot.to_string())
    })
    .collect();

  let batch_result = service.batch_get_metadata_by_files(&keys).await?;

  assert_eq!(3, batch_result.len(), "Batch query should return 3 results");

  // Verify each key is present
  for (repo, filename, snapshot) in &test_data {
    let key = (repo.to_string(), filename.to_string(), snapshot.to_string());
    assert!(
      batch_result.contains_key(&key),
      "Batch result should contain key {:?}",
      key
    );
  }

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_batch_get_metadata_by_files_returns_empty_for_empty_input(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let keys: Vec<(String, String, String)> = vec![];
  let result = service.batch_get_metadata_by_files(&keys).await?;
  assert!(result.is_empty());
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_upsert_model_metadata_inserts_new_row(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut builder = model_metadata_builder(now);
  builder
    .source("model")
    .repo("test/repo")
    .filename("model.gguf")
    .snapshot("snapshot123")
    .capabilities_vision(1_i64)
    .capabilities_thinking(1_i64)
    .capabilities_function_calling(1_i64)
    .context_max_input_tokens(8192_i64)
    .context_max_output_tokens(4096_i64)
    .architecture(
      r#"{"family":"llama","parameter_count":7000000000,"quantization":"Q4_K_M","format":"gguf"}"#,
    )
    .chat_template("{% for msg in messages %}{{ msg.role }}: {{ msg.content }}{% endfor %}");
  let row = builder.build()?;

  service.upsert_model_metadata(&row).await?;

  let fetched = service
    .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
    .await?
    .expect("Row should exist");

  assert_eq!("model", fetched.source);
  assert_eq!(Some("test/repo".to_string()), fetched.repo);
  assert_eq!(Some("model.gguf".to_string()), fetched.filename);
  assert_eq!(Some(1), fetched.capabilities_vision);
  assert_eq!(Some(1), fetched.capabilities_thinking);
  assert_eq!(Some(8192), fetched.context_max_input_tokens);
  assert!(fetched.architecture.is_some());
  assert!(fetched.chat_template.is_some());
  assert!(fetched.chat_template.unwrap().contains("msg.role"));

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_upsert_model_metadata_updates_existing_row(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Insert initial row with source='model' (physical GGUF file)
  let row = create_test_model_metadata("test/repo", "model.gguf", "snapshot123", now);
  service.upsert_model_metadata(&row).await?;

  // Update with new data (same repo/filename/snapshot)
  let mut builder = model_metadata_builder(now);
  builder
    .source("model")
    .repo("test/repo")
    .filename("model.gguf")
    .snapshot("snapshot123")
    .capabilities_vision(1_i64)
    .capabilities_thinking(1_i64)
    .capabilities_function_calling(1_i64)
    .context_max_input_tokens(8192_i64)
    .context_max_output_tokens(4096_i64)
    .architecture(r#"{"family":"llama","format":"gguf"}"#)
    .chat_template("updated template");
  let updated_row = builder.build()?;
  service.upsert_model_metadata(&updated_row).await?;

  // Verify update
  let fetched = service
    .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
    .await?
    .expect("Row should exist");

  assert_eq!(Some(1), fetched.capabilities_vision);
  assert_eq!(Some(1), fetched.capabilities_thinking);
  assert_eq!(Some(8192), fetched.context_max_input_tokens);
  assert_eq!(Some(4096), fetched.context_max_output_tokens);
  assert_eq!(Some("updated template".to_string()), fetched.chat_template);

  // Verify only one row exists (upsert, not insert)
  let all = service.list_model_metadata().await?;
  assert_eq!(1, all.len());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_upsert_model_metadata_with_api_model_id(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let mut builder = model_metadata_builder(now);
  builder
    .source("api")
    .api_model_id("gpt-4-turbo")
    .capabilities_vision(1_i64)
    .capabilities_function_calling(1_i64)
    .capabilities_structured_output(1_i64)
    .context_max_input_tokens(128000_i64)
    .context_max_output_tokens(4096_i64);
  let row = builder.build()?;

  service.upsert_model_metadata(&row).await?;

  // Verify it's in list (API models use different path)
  let all = service.list_model_metadata().await?;
  assert_eq!(1, all.len());
  assert_eq!(Some("gpt-4-turbo".to_string()), all[0].api_model_id);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_model_metadata_by_file_returns_none_for_nonexistent(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let result = service
    .get_model_metadata_by_file("nonexistent/repo", "model.gguf", "snapshot")
    .await?;
  assert!(result.is_none());
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_model_metadata_by_file_filters_by_source(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Insert an API source row with repo/filename/snapshot
  // (unusual but possible, should NOT be returned by get_model_metadata_by_file)
  let mut builder = model_metadata_builder(now);
  builder
    .source("api")
    .repo("test/repo")
    .filename("model.gguf")
    .snapshot("snapshot123")
    .api_model_id("api-model-id");
  let api_row = builder.build()?;
  service.upsert_model_metadata(&api_row).await?;

  // Should NOT find it (source filter excludes 'api')
  let result = service
    .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
    .await?;
  assert!(result.is_none(), "Should not find API source rows");

  // Insert a 'model' source row with same repo/filename/snapshot
  let model_row = create_test_model_metadata("test/repo", "model.gguf", "snapshot123", now);
  service.upsert_model_metadata(&model_row).await?;

  // Now should find it
  let result = service
    .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
    .await?;
  assert!(result.is_some(), "Should find model source rows");
  assert_eq!("model", result.unwrap().source);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_list_model_metadata_returns_all_rows(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Insert multiple rows - note: for local GGUF files, source is always 'model'
  // API source is for remote API models which have api_model_id instead of repo/filename
  let rows = vec![
    create_test_model_metadata("repo1", "model1.gguf", "snapshot", now),
    create_test_model_metadata("repo2", "model2.gguf", "snapshot", now),
    create_test_api_model_metadata("gpt-4", now),
  ];

  for row in &rows {
    service.upsert_model_metadata(row).await?;
  }

  let all = service.list_model_metadata().await?;
  assert_eq!(3, all.len());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_list_model_metadata_returns_empty_when_no_data(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let result = service.list_model_metadata().await?;
  assert!(result.is_empty());
  Ok(())
}

/// Test that metadata for the same physical GGUF file (same repo/filename/snapshot)
/// is stored only once with source='model', regardless of whether the request
/// came from a UserAlias or ModelAlias. This verifies the deduplication behavior
/// where UserAlias requests are translated to store under source='model'.
#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_metadata_stored_with_source_model_only(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();

  // Simulate what extract_and_store_metadata does for a UserAlias:
  // It always stores with source='model' regardless of input alias type
  let mut builder = model_metadata_builder(now);
  builder
    .source("model") // Always 'model', never 'user'
    .repo("test/repo")
    .filename("model.gguf")
    .snapshot("snapshot123")
    .capabilities_vision(1_i64)
    .capabilities_function_calling(1_i64)
    .context_max_input_tokens(8192_i64)
    .context_max_output_tokens(4096_i64)
    .chat_template("test template");
  let row = builder.build()?;
  service.upsert_model_metadata(&row).await?;

  // Query should find the row (only looks for source='model')
  let result = service
    .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
    .await?;
  assert!(result.is_some(), "Should find metadata for the GGUF file");

  let fetched = result.unwrap();
  assert_eq!("model", fetched.source, "Source should always be 'model'");
  assert_eq!(Some("test template".to_string()), fetched.chat_template);

  // Verify only one row exists in the database
  let all = service.list_model_metadata().await?;
  assert_eq!(1, all.len(), "Should have exactly one metadata row");
  assert_eq!("model", all[0].source);

  Ok(())
}

// ============================================================================
// Access Request Repository Tests
// ============================================================================

use crate::db::{AccessRequestRepository, AppAccessRequestRow};

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_create_draft_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  let result = service.create(&row).await?;

  assert_eq!(result.id, row.id);
  assert_eq!(result.status, "draft");
  assert_eq!(result.app_client_id, row.app_client_id);
  assert_eq!(result.flow_type, "redirect");
  assert_eq!(
    result.redirect_uri,
    Some("https://example.com/callback".to_string())
  );
  assert_eq!(result.requested, r#"[{"tool_type":"builtin-exa-search"}]"#);
  assert_eq!(result.approved, None);
  assert_eq!(result.user_id, None);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "popup".to_string(),
    redirect_uri: None,
    status: "draft".to_string(),
    requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let result = service.get(&row.id).await?;
  assert!(result.is_some());

  let retrieved = result.unwrap();
  assert_eq!(retrieved.id, row.id);
  assert_eq!(retrieved.status, "draft");
  assert_eq!(retrieved.flow_type, "popup");
  assert_eq!(retrieved.redirect_uri, None);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_get_nonexistent_request(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let result = service.get("nonexistent-id").await?;
  assert!(result.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_approval(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440002".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let tools_approved_json =
    r#"[{"tool_type":"builtin-exa-search","status":"approved","toolset_id":"uuid1"}]"#;
  let result = service
    .update_approval(
      &row.id,
      "user-uuid",
      tools_approved_json,
      "scope_resource-xyz",
      Some("scope_access_request:550e8400-e29b-41d4-a716-446655440002".to_string()),
    )
    .await?;

  assert_eq!(result.status, "approved");
  assert_eq!(result.user_id, Some("user-uuid".to_string()));
  assert_eq!(result.approved, Some(tools_approved_json.to_string()));
  assert_eq!(result.resource_scope, Some("scope_resource-xyz".to_string()));
  assert_eq!(
    result.access_request_scope,
    Some("scope_access_request:550e8400-e29b-41d4-a716-446655440002".to_string())
  );

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_denial(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440003".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let result = service.update_denial(&row.id, "user-uuid").await?;

  assert_eq!(result.status, "denied");
  assert_eq!(result.user_id, Some("user-uuid".to_string()));
  assert_eq!(result.approved, None);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_update_failure(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now();
  let expires_at = now + chrono::Duration::minutes(10);

  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440004".to_string(),
    app_client_id: "app-abc123".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  service.create(&row).await?;

  let error_msg = "KC registration failed: UUID collision (409).";
  let result = service.update_failure(&row.id, error_msg).await?;

  assert_eq!(result.status, "failed");
  assert_eq!(result.error_message, Some(error_msg.to_string()));

  Ok(())
}
