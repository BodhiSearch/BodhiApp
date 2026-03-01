use crate::db::encryption::encrypt_api_key;
use crate::mcps::{
  McpAuthHeaderRow, McpAuthRepository, McpOAuthConfigRow, McpOAuthTokenRow, McpServerRepository,
  RegistrationType,
};
use crate::test_utils::{sea_context, setup_env};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

use crate::mcps::test_helpers::{make_auth_header_row, make_server, ENCRYPTION_KEY};

fn make_oauth_config_row(
  id: &str,
  server_id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> McpOAuthConfigRow {
  let (encrypted, salt, nonce) =
    encrypt_api_key(ENCRYPTION_KEY, "my-client-secret-123").expect("encryption failed");
  McpOAuthConfigRow {
    id: id.to_string(),
    name: "OAuth".to_string(),
    mcp_server_id: server_id.to_string(),
    registration_type: RegistrationType::PreRegistered,
    client_id: "my-client-id".to_string(),
    encrypted_client_secret: Some(encrypted),
    client_secret_salt: Some(salt),
    client_secret_nonce: Some(nonce),
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    registration_endpoint: None,
    encrypted_registration_access_token: None,
    registration_access_token_salt: None,
    registration_access_token_nonce: None,
    client_id_issued_at: None,
    token_endpoint_auth_method: None,
    scopes: Some("openid profile".to_string()),
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn make_oauth_token_row(
  config_id: &str,
  id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> McpOAuthTokenRow {
  let (enc_access, salt_access, nonce_access) =
    encrypt_api_key(ENCRYPTION_KEY, "access-token-abc").expect("encryption failed");
  let (enc_refresh, salt_refresh, nonce_refresh) =
    encrypt_api_key(ENCRYPTION_KEY, "refresh-token-xyz").expect("encryption failed");
  McpOAuthTokenRow {
    id: id.to_string(),
    mcp_oauth_config_id: config_id.to_string(),
    encrypted_access_token: enc_access,
    access_token_salt: salt_access,
    access_token_nonce: nonce_access,
    encrypted_refresh_token: Some(enc_refresh),
    refresh_token_salt: Some(salt_refresh),
    refresh_token_nonce: Some(nonce_refresh),
    scopes_granted: Some("openid profile".to_string()),
    expires_at: Some(now + chrono::Duration::seconds(3600)),
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  }
}

// ============================================================================
// MCP Auth Header Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp_auth_header(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;

  let row = make_auth_header_row("ah-1", "s1", ctx.now);
  let created = ctx.service.create_mcp_auth_header(&row).await?;
  assert_eq!("ah-1", created.id);
  assert_eq!("Authorization", created.header_key);

  let fetched = ctx.service.get_mcp_auth_header("ah-1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("ah-1", fetched.id);
  assert_eq!("s1", fetched.mcp_server_id);
  assert_eq!(ctx.now, fetched.created_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_auth_header_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_mcp_auth_header("nonexistent").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp_auth_header(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  let row = make_auth_header_row("ah-1", "s1", ctx.now);
  ctx.service.create_mcp_auth_header(&row).await?;

  let (enc2, salt2, nonce2) = encrypt_api_key(ENCRYPTION_KEY, "new-secret")?;
  let updated_at = ctx.now + chrono::Duration::seconds(30);
  let updated_row = McpAuthHeaderRow {
    header_key: "X-Api-Key".to_string(),
    encrypted_header_value: enc2,
    header_value_salt: salt2,
    header_value_nonce: nonce2,
    updated_at,
    ..row
  };
  let result = ctx.service.update_mcp_auth_header(&updated_row).await?;
  assert_eq!("X-Api-Key", result.header_key);
  assert_eq!(updated_at, result.updated_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp_auth_header(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&make_auth_header_row("ah-1", "s1", ctx.now))
    .await?;

  ctx.service.delete_mcp_auth_header("ah-1").await?;
  let gone = ctx.service.get_mcp_auth_header("ah-1").await?;
  assert_eq!(None, gone);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcp_auth_headers_by_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_server(&make_server("s2", "https://other.example.com", ctx.now))
    .await?;

  ctx
    .service
    .create_mcp_auth_header(&make_auth_header_row("ah-1", "s1", ctx.now))
    .await?;
  let later = ctx.now + chrono::Duration::seconds(10);
  ctx
    .service
    .create_mcp_auth_header(&McpAuthHeaderRow {
      id: "ah-2".to_string(),
      name: "Header 2".to_string(),
      created_at: later,
      updated_at: later,
      ..make_auth_header_row("ah-2", "s1", later)
    })
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&make_auth_header_row("ah-3", "s2", ctx.now))
    .await?;

  let results = ctx.service.list_mcp_auth_headers_by_server("s1").await?;
  assert_eq!(2, results.len());
  // Ordered by created_at DESC
  assert_eq!("ah-2", results[0].id);
  assert_eq!("ah-1", results[1].id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_decrypted_auth_header_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&make_auth_header_row("ah-1", "s1", ctx.now))
    .await?;

  let result = ctx.service.get_decrypted_auth_header("ah-1").await?;
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
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_decrypted_auth_header_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_decrypted_auth_header("nonexistent").await?;
  assert_eq!(None, result);
  Ok(())
}

// ============================================================================
// MCP OAuth Config Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp_oauth_config(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;

  let row = make_oauth_config_row("oc-1", "s1", ctx.now);
  let created = ctx.service.create_mcp_oauth_config(&row).await?;
  assert_eq!("oc-1", created.id);
  assert_eq!("my-client-id", created.client_id);

  let fetched = ctx.service.get_mcp_oauth_config("oc-1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("oc-1", fetched.id);
  assert_eq!("s1", fetched.mcp_server_id);
  assert_eq!("my-client-id", fetched.client_id);
  assert_eq!(
    "https://auth.example.com/authorize",
    fetched.authorization_endpoint
  );
  assert_eq!("https://auth.example.com/token", fetched.token_endpoint);
  assert_eq!(Some("openid profile".to_string()), fetched.scopes);
  assert_eq!(ctx.now, fetched.created_at);

  // Encrypted secret should be masked via has_client_secret flag
  assert!(fetched.has_client_secret);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_oauth_config_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_mcp_oauth_config("nonexistent").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcp_oauth_configs_by_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;

  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  let later = ctx.now + chrono::Duration::seconds(10);
  ctx
    .service
    .create_mcp_oauth_config(&McpOAuthConfigRow {
      id: "oc-2".to_string(),
      name: "OAuth 2".to_string(),
      client_id: "other-client".to_string(),
      created_at: later,
      updated_at: later,
      ..make_oauth_config_row("oc-2", "s1", later)
    })
    .await?;

  let results = ctx.service.list_mcp_oauth_configs_by_server("s1").await?;
  assert_eq!(2, results.len());
  // Ordered by created_at DESC
  assert_eq!("oc-2", results[0].id);
  assert_eq!("oc-1", results[1].id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp_oauth_config(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  ctx.service.delete_mcp_oauth_config("oc-1").await?;
  let gone = ctx.service.get_mcp_oauth_config("oc-1").await?;
  assert_eq!(None, gone);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_oauth_config_cascade(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-2", ctx.now))
    .await?;

  ctx.service.delete_oauth_config_cascade("oc-1").await?;

  let config_gone = ctx.service.get_mcp_oauth_config("oc-1").await?;
  assert_eq!(None, config_gone);
  let token1_gone = ctx.service.get_mcp_oauth_token("user-1", "ot-1").await?;
  assert_eq!(None, token1_gone);
  let token2_gone = ctx.service.get_mcp_oauth_token("user-1", "ot-2").await?;
  assert_eq!(None, token2_gone);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_decrypted_client_secret(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  let result = ctx.service.get_decrypted_client_secret("oc-1").await?;
  assert_eq!(
    Some((
      "my-client-id".to_string(),
      "my-client-secret-123".to_string()
    )),
    result
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_decrypted_client_secret_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_decrypted_client_secret("nonexistent").await;
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("db_error-item_not_found", err.code());
  Ok(())
}

// ============================================================================
// MCP OAuth Token Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp_oauth_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  let token = make_oauth_token_row("oc-1", "ot-1", ctx.now);
  let created = ctx.service.create_mcp_oauth_token(&token).await?;
  assert_eq!("ot-1", created.id);
  assert_eq!("oc-1", created.mcp_oauth_config_id);

  let fetched = ctx.service.get_mcp_oauth_token("user-1", "ot-1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("ot-1", fetched.id);
  assert_eq!(Some("openid profile".to_string()), fetched.scopes_granted);
  assert_eq!(
    Some((ctx.now + chrono::Duration::seconds(3600)).timestamp()),
    fetched.expires_at
  );
  assert_eq!(ctx.now, fetched.created_at);

  // Encrypted values should be masked via has_* flags
  assert!(fetched.has_access_token);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_oauth_token_wrong_user(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;

  let result = ctx.service.get_mcp_oauth_token("user-2", "ot-1").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_latest_oauth_token_by_config(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  let older = make_oauth_token_row("oc-1", "ot-1", ctx.now);
  ctx.service.create_mcp_oauth_token(&older).await?;

  let later = ctx.now + chrono::Duration::seconds(100);
  let newer = McpOAuthTokenRow {
    id: "ot-2".to_string(),
    created_at: later,
    updated_at: later,
    ..make_oauth_token_row("oc-1", "ot-2", later)
  };
  ctx.service.create_mcp_oauth_token(&newer).await?;

  let latest = ctx.service.get_latest_oauth_token_by_config("oc-1").await?;
  assert!(latest.is_some());
  assert_eq!("ot-2", latest.unwrap().id);

  let missing = ctx
    .service
    .get_latest_oauth_token_by_config("nonexistent")
    .await?;
  assert_eq!(None, missing);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp_oauth_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  let token = make_oauth_token_row("oc-1", "ot-1", ctx.now);
  ctx.service.create_mcp_oauth_token(&token).await?;

  let (new_enc_at, new_salt_at, new_nonce_at) =
    encrypt_api_key(ENCRYPTION_KEY, "new-access-token-xyz")?;
  let (new_enc_rt, new_salt_rt, new_nonce_rt) =
    encrypt_api_key(ENCRYPTION_KEY, "new-refresh-token-uvw")?;
  let updated_at = ctx.now + chrono::Duration::seconds(100);
  let updated_row = McpOAuthTokenRow {
    encrypted_access_token: new_enc_at,
    access_token_salt: new_salt_at,
    access_token_nonce: new_nonce_at,
    encrypted_refresh_token: Some(new_enc_rt),
    refresh_token_salt: Some(new_salt_rt),
    refresh_token_nonce: Some(new_nonce_rt),
    expires_at: Some(ctx.now + chrono::Duration::seconds(7200)),
    updated_at,
    ..token.clone()
  };
  let result = ctx.service.update_mcp_oauth_token(&updated_row).await?;
  assert_eq!("ot-1", result.id);
  assert_eq!(
    Some(ctx.now + chrono::Duration::seconds(7200)),
    result.expires_at
  );
  assert_eq!(updated_at, result.updated_at);

  // Verify decrypted values match new plaintext via decrypt methods
  let (header_key, header_value) = ctx
    .service
    .get_decrypted_oauth_bearer("ot-1")
    .await?
    .unwrap();
  assert_eq!("Authorization", header_key);
  assert_eq!("Bearer new-access-token-xyz", header_value);

  let decrypted_rt = ctx
    .service
    .get_decrypted_refresh_token("ot-1")
    .await?
    .unwrap();
  assert_eq!("new-refresh-token-uvw", decrypted_rt);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp_oauth_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;

  ctx.service.delete_mcp_oauth_token("user-1", "ot-1").await?;
  let gone = ctx.service.get_mcp_oauth_token("user-1", "ot-1").await?;
  assert_eq!(None, gone);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp_oauth_token_wrong_user_noop(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;

  // Wrong user should be a no-op
  ctx.service.delete_mcp_oauth_token("user-2", "ot-1").await?;
  let still_exists = ctx.service.get_mcp_oauth_token("user-1", "ot-1").await?;
  assert!(still_exists.is_some());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_oauth_tokens_by_config(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-2", ctx.now))
    .await?;

  ctx.service.delete_oauth_tokens_by_config("oc-1").await?;
  let gone1 = ctx.service.get_mcp_oauth_token("user-1", "ot-1").await?;
  let gone2 = ctx.service.get_mcp_oauth_token("user-1", "ot-2").await?;
  assert_eq!(None, gone1);
  assert_eq!(None, gone2);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_oauth_tokens_by_config_and_user(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  // user-1 token
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;
  // user-2 token
  ctx
    .service
    .create_mcp_oauth_token(&McpOAuthTokenRow {
      id: "ot-2".to_string(),
      created_by: "user-2".to_string(),
      ..make_oauth_token_row("oc-1", "ot-2", ctx.now)
    })
    .await?;

  ctx
    .service
    .delete_oauth_tokens_by_config_and_user("oc-1", "user-1")
    .await?;

  let gone = ctx.service.get_mcp_oauth_token("user-1", "ot-1").await?;
  assert_eq!(None, gone);
  // user-2 token should still exist
  let still = ctx.service.get_mcp_oauth_token("user-2", "ot-2").await?;
  assert!(still.is_some());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_decrypted_oauth_bearer(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;

  let result = ctx.service.get_decrypted_oauth_bearer("ot-1").await?;
  assert_eq!(
    Some((
      "Authorization".to_string(),
      "Bearer access-token-abc".to_string()
    )),
    result
  );

  let missing = ctx
    .service
    .get_decrypted_oauth_bearer("nonexistent")
    .await?;
  assert_eq!(None, missing);
  Ok(())
}
