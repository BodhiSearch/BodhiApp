use crate::{
  db::{
    encryption::encrypt_api_key, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow,
    McpRepository, McpRow, McpServerRow,
  },
  test_utils::{test_db_service, TestDbService},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;

fn test_mcp_server_row(now: i64) -> McpServerRow {
  McpServerRow {
    id: "server-1".to_string(),
    url: "https://mcp.example.com/mcp".to_string(),
    name: "Test MCP Server".to_string(),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn test_mcp_row_public(now: i64) -> McpRow {
  McpRow {
    id: "mcp-pub-1".to_string(),
    created_by: "user-1".to_string(),
    mcp_server_id: "server-1".to_string(),
    name: "Public MCP".to_string(),
    slug: "public-mcp".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "public".to_string(),
    auth_uuid: None,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_get_decrypted_auth_header_roundtrip(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();

  let (encrypted, salt, nonce) =
    encrypt_api_key(&service.encryption_key, "Bearer sk-secret-token-123")?;
  let auth_header_row = McpAuthHeaderRow {
    id: "auth-header-1".to_string(),
    header_key: "Authorization".to_string(),
    encrypted_header_value: encrypted,
    header_value_salt: salt,
    header_value_nonce: nonce,
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  };
  service.create_mcp_auth_header(&auth_header_row).await?;

  let result = service.get_decrypted_auth_header("auth-header-1").await?;
  // Also verify the user-scoped get works
  let scoped = service
    .get_mcp_auth_header("user-1", "auth-header-1")
    .await?;
  assert!(scoped.is_some());
  assert_eq!(
    Some((
      "Authorization".to_string(),
      "Bearer sk-secret-token-123".to_string()
    )),
    result
  );
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_get_decrypted_auth_header_returns_none_for_missing(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let result = service.get_decrypted_auth_header("nonexistent").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_mcp_crud_with_auth_uuid(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();
  service.create_mcp_server(&test_mcp_server_row(now)).await?;

  let (encrypted, salt, nonce) = encrypt_api_key(&service.encryption_key, "Bearer secret")?;
  let auth_header_row = McpAuthHeaderRow {
    id: "auth-header-2".to_string(),
    header_key: "X-Api-Key".to_string(),
    encrypted_header_value: encrypted,
    header_value_salt: salt,
    header_value_nonce: nonce,
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  };
  service.create_mcp_auth_header(&auth_header_row).await?;

  let mcp_row = McpRow {
    id: "mcp-auth-1".to_string(),
    created_by: "user-1".to_string(),
    mcp_server_id: "server-1".to_string(),
    name: "Header MCP".to_string(),
    slug: "header-mcp".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "header".to_string(),
    auth_uuid: Some("auth-header-2".to_string()),
    created_at: now,
    updated_at: now,
  };
  service.create_mcp(&mcp_row).await?;

  let mcps = service.list_mcps_with_server("user-1").await?;
  assert_eq!(1, mcps.len());
  assert_eq!("header", mcps[0].auth_type);
  assert_eq!(Some("auth-header-2".to_string()), mcps[0].auth_uuid);

  let fetched = service.get_mcp("user-1", "mcp-auth-1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("header", fetched.auth_type);
  assert_eq!(Some("auth-header-2".to_string()), fetched.auth_uuid);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_list_mcps_with_server_public(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();
  service.create_mcp_server(&test_mcp_server_row(now)).await?;
  service.create_mcp(&test_mcp_row_public(now)).await?;

  let mcps = service.list_mcps_with_server("user-1").await?;
  assert_eq!(1, mcps.len());
  assert_eq!("public", mcps[0].auth_type);
  assert_eq!(None, mcps[0].auth_uuid);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_mcp_auth_header_crud(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();

  let (encrypted, salt, nonce) = encrypt_api_key(&service.encryption_key, "my-secret")?;
  let row = McpAuthHeaderRow {
    id: "ah-crud-1".to_string(),
    header_key: "X-Key".to_string(),
    encrypted_header_value: encrypted,
    header_value_salt: salt,
    header_value_nonce: nonce,
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  };
  let created = service.create_mcp_auth_header(&row).await?;
  assert_eq!("ah-crud-1", created.id);
  assert_eq!("X-Key", created.header_key);

  let fetched = service.get_mcp_auth_header("user-1", "ah-crud-1").await?;
  assert!(fetched.is_some());
  assert_eq!("X-Key", fetched.unwrap().header_key);

  let (encrypted2, salt2, nonce2) = encrypt_api_key(&service.encryption_key, "new-secret")?;
  let updated_row = McpAuthHeaderRow {
    id: "ah-crud-1".to_string(),
    header_key: "X-New-Key".to_string(),
    encrypted_header_value: encrypted2,
    header_value_salt: salt2,
    header_value_nonce: nonce2,
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now + 1,
  };
  let updated = service.update_mcp_auth_header(&updated_row).await?;
  assert_eq!("X-New-Key", updated.header_key);

  service
    .delete_mcp_auth_header("user-1", "ah-crud-1")
    .await?;
  let gone = service.get_mcp_auth_header("user-1", "ah-crud-1").await?;
  assert!(gone.is_none());

  Ok(())
}

// ============================================================================
// OAuth Config Tests
// ============================================================================

fn test_oauth_config_row(encryption_key: &[u8], now: i64) -> McpOAuthConfigRow {
  let (encrypted, salt, nonce) =
    encrypt_api_key(encryption_key, "my-client-secret-123").expect("encryption failed");
  McpOAuthConfigRow {
    id: uuid::Uuid::new_v4().to_string(),
    mcp_server_id: "server-1".to_string(),
    client_id: "my-client-id".to_string(),
    encrypted_client_secret: encrypted,
    client_secret_salt: salt,
    client_secret_nonce: nonce,
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    scopes: Some("openid profile".to_string()),
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn test_oauth_token_row(encryption_key: &[u8], config_id: &str, now: i64) -> McpOAuthTokenRow {
  let (enc_access, salt_access, nonce_access) =
    encrypt_api_key(encryption_key, "access-token-abc").expect("encryption failed");
  let (enc_refresh, salt_refresh, nonce_refresh) =
    encrypt_api_key(encryption_key, "refresh-token-xyz").expect("encryption failed");
  McpOAuthTokenRow {
    id: uuid::Uuid::new_v4().to_string(),
    mcp_oauth_config_id: config_id.to_string(),
    encrypted_access_token: enc_access,
    access_token_salt: salt_access,
    access_token_nonce: nonce_access,
    encrypted_refresh_token: Some(enc_refresh),
    refresh_token_salt: Some(salt_refresh),
    refresh_token_nonce: Some(nonce_refresh),
    scopes_granted: Some("openid profile".to_string()),
    expires_at: Some(now + 3600),
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_create_mcp_oauth_config_and_read(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;
  let row = test_oauth_config_row(&db_service.encryption_key, now);
  let created = db_service.create_mcp_oauth_config(&row).await?;
  assert_eq!(row.id, created.id);
  assert_eq!("my-client-id", created.client_id);
  assert_eq!("server-1", created.mcp_server_id);

  let fetched = db_service.get_mcp_oauth_config(&row.id).await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(row.id, fetched.id);
  assert_eq!("my-client-id", fetched.client_id);
  assert_eq!("server-1", fetched.mcp_server_id);
  assert_eq!(
    "https://auth.example.com/authorize",
    fetched.authorization_endpoint
  );
  assert_eq!("https://auth.example.com/token", fetched.token_endpoint);
  assert_eq!(Some("openid profile".to_string()), fetched.scopes);
  assert_eq!("user-1", fetched.created_by);
  assert_eq!(now, fetched.created_at);
  assert_eq!(now, fetched.updated_at);

  assert_ne!("my-client-secret-123", fetched.encrypted_client_secret);
  assert!(!fetched.client_secret_salt.is_empty());
  assert!(!fetched.client_secret_nonce.is_empty());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_list_mcp_oauth_configs_by_server(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;

  let config1 = test_oauth_config_row(&db_service.encryption_key, now);
  db_service.create_mcp_oauth_config(&config1).await?;

  let (encrypted2, salt2, nonce2) =
    encrypt_api_key(&db_service.encryption_key, "secret-2").expect("encryption failed");
  let config2 = McpOAuthConfigRow {
    id: uuid::Uuid::new_v4().to_string(),
    mcp_server_id: "server-1".to_string(),
    client_id: "other-client".to_string(),
    encrypted_client_secret: encrypted2,
    client_secret_salt: salt2,
    client_secret_nonce: nonce2,
    authorization_endpoint: "https://auth2.example.com/authorize".to_string(),
    token_endpoint: "https://auth2.example.com/token".to_string(),
    scopes: None,
    created_by: "user-1".to_string(),
    created_at: now + 1,
    updated_at: now + 1,
  };
  db_service.create_mcp_oauth_config(&config2).await?;

  let results = db_service
    .list_mcp_oauth_configs_by_server("server-1", "user-1")
    .await?;
  assert_eq!(2, results.len());
  assert_eq!(config2.id, results[0].id);
  assert_eq!(config1.id, results[1].id);

  let results_other_user = db_service
    .list_mcp_oauth_configs_by_server("server-1", "user-999")
    .await?;
  assert_eq!(0, results_other_user.len());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_delete_mcp_oauth_config(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;
  let row = test_oauth_config_row(&db_service.encryption_key, now);
  db_service.create_mcp_oauth_config(&row).await?;

  let fetched = db_service.get_mcp_oauth_config(&row.id).await?;
  assert!(fetched.is_some());

  db_service.delete_mcp_oauth_config(&row.id).await?;

  let gone = db_service.get_mcp_oauth_config(&row.id).await?;
  assert_eq!(None, gone);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_get_decrypted_client_secret(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;
  let row = test_oauth_config_row(&db_service.encryption_key, now);
  db_service.create_mcp_oauth_config(&row).await?;

  let result = db_service.get_decrypted_client_secret(&row.id).await?;
  assert_eq!(
    Some((
      "my-client-id".to_string(),
      "my-client-secret-123".to_string()
    )),
    result
  );

  let missing = db_service
    .get_decrypted_client_secret("nonexistent")
    .await?;
  assert_eq!(None, missing);

  Ok(())
}

// ============================================================================
// OAuth Token Tests
// ============================================================================

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_create_mcp_oauth_token_and_read(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;
  let config = test_oauth_config_row(&db_service.encryption_key, now);
  db_service.create_mcp_oauth_config(&config).await?;

  let token = test_oauth_token_row(&db_service.encryption_key, &config.id, now);
  let created = db_service.create_mcp_oauth_token(&token).await?;
  assert_eq!(token.id, created.id);
  assert_eq!(config.id, created.mcp_oauth_config_id);

  let fetched = db_service
    .get_mcp_oauth_token("user-1", &token.id)
    .await?
    .expect("token should exist");
  assert_eq!(token.id, fetched.id);
  assert_eq!(config.id, fetched.mcp_oauth_config_id);
  assert_eq!(Some("openid profile".to_string()), fetched.scopes_granted);
  assert_eq!(Some(now + 3600), fetched.expires_at);
  assert_eq!("user-1", fetched.created_by);
  assert_eq!(now, fetched.created_at);

  assert_ne!("access-token-abc", fetched.encrypted_access_token);
  assert!(fetched.encrypted_refresh_token.is_some());

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_get_latest_oauth_token_by_config(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;
  let config = test_oauth_config_row(&db_service.encryption_key, now);
  db_service.create_mcp_oauth_config(&config).await?;

  let older_token = McpOAuthTokenRow {
    created_at: now,
    updated_at: now,
    ..test_oauth_token_row(&db_service.encryption_key, &config.id, now)
  };
  db_service.create_mcp_oauth_token(&older_token).await?;

  let newer_token = McpOAuthTokenRow {
    created_at: now + 100,
    updated_at: now + 100,
    ..test_oauth_token_row(&db_service.encryption_key, &config.id, now + 100)
  };
  db_service.create_mcp_oauth_token(&newer_token).await?;

  let latest = db_service
    .get_latest_oauth_token_by_config(&config.id)
    .await?
    .expect("latest token should exist");
  assert_eq!(newer_token.id, latest.id);
  assert_eq!(now + 100, latest.created_at);

  let missing = db_service
    .get_latest_oauth_token_by_config("nonexistent-config")
    .await?;
  assert_eq!(None, missing);

  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_init_service_delete_oauth_tokens_by_config(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now().timestamp();
  db_service
    .create_mcp_server(&test_mcp_server_row(now))
    .await?;
  let config = test_oauth_config_row(&db_service.encryption_key, now);
  db_service.create_mcp_oauth_config(&config).await?;

  let token1 = test_oauth_token_row(&db_service.encryption_key, &config.id, now);
  let token2 = McpOAuthTokenRow {
    created_at: now + 50,
    updated_at: now + 50,
    ..test_oauth_token_row(&db_service.encryption_key, &config.id, now + 50)
  };
  db_service.create_mcp_oauth_token(&token1).await?;
  db_service.create_mcp_oauth_token(&token2).await?;

  db_service.delete_oauth_tokens_by_config(&config.id).await?;

  let gone1 = db_service.get_mcp_oauth_token("user-1", &token1.id).await?;
  let gone2 = db_service.get_mcp_oauth_token("user-1", &token2.id).await?;
  assert_eq!(None, gone1);
  assert_eq!(None, gone2);

  Ok(())
}
