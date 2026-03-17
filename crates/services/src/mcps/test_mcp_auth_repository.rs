use crate::db::encryption::encrypt_api_key;
use crate::mcps::{
  McpAuthType, McpEntity, McpOAuthConfigDetailEntity, McpOAuthTokenEntity, McpRepository,
  McpServerRepository, RegistrationType,
};
use crate::test_utils::{sea_context, setup_env, TEST_TENANT_ID, TEST_USER_ID};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

use crate::mcps::test_helpers::{
  make_auth_config_param_row, make_auth_config_row, make_auth_param_row, make_mcp, make_server,
  ENCRYPTION_KEY,
};

// ============================================================================
// Helper: make an OAuth token entity with encrypted fields
// ============================================================================

fn make_oauth_token(
  id: &str,
  mcp_id: Option<&str>,
  auth_config_id: &str,
  user_id: &str,
  access_token: &str,
  refresh_token: Option<&str>,
  now: chrono::DateTime<chrono::Utc>,
) -> McpOAuthTokenEntity {
  let (enc_at, salt_at, nonce_at) = encrypt_api_key(ENCRYPTION_KEY, access_token).unwrap();
  let (enc_rt, salt_rt, nonce_rt) = match refresh_token {
    Some(rt) => {
      let (e, s, n) = encrypt_api_key(ENCRYPTION_KEY, rt).unwrap();
      (Some(e), Some(s), Some(n))
    }
    None => (None, None, None),
  };
  McpOAuthTokenEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    mcp_id: mcp_id.map(|s| s.to_string()),
    auth_config_id: auth_config_id.to_string(),
    user_id: user_id.to_string(),
    encrypted_access_token: enc_at,
    access_token_salt: salt_at,
    access_token_nonce: nonce_at,
    encrypted_refresh_token: enc_rt,
    refresh_token_salt: salt_rt,
    refresh_token_nonce: nonce_rt,
    scopes_granted: Some("read write".to_string()),
    expires_at: Some(now + chrono::Duration::hours(1)),
    created_at: now,
    updated_at: now,
  }
}

fn make_oauth_config_detail(
  auth_config_id: &str,
  now: chrono::DateTime<chrono::Utc>,
) -> McpOAuthConfigDetailEntity {
  McpOAuthConfigDetailEntity {
    auth_config_id: auth_config_id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    registration_type: RegistrationType::PreRegistered,
    client_id: "my-client-id".to_string(),
    encrypted_client_secret: None,
    client_secret_salt: None,
    client_secret_nonce: None,
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    registration_endpoint: None,
    encrypted_registration_access_token: None,
    registration_access_token_salt: None,
    registration_access_token_nonce: None,
    client_id_issued_at: None,
    token_endpoint_auth_method: None,
    scopes: Some("read write".to_string()),
    created_at: now,
    updated_at: now,
  }
}

// ============================================================================
// Auth Config Composite Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_auth_config_header_with_params(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-1", "s1", "header", ctx.now);
  let params = vec![
    make_auth_config_param_row("acp-1", "ac-1", "header", "Authorization", ctx.now),
    make_auth_config_param_row("acp-2", "ac-1", "query", "api_key", ctx.now),
  ];

  let created = ctx
    .service
    .create_auth_config_header(TEST_TENANT_ID, &config, params)
    .await?;

  assert_eq!("ac-1", created.id);
  assert_eq!("header", created.config_type);

  // Verify params were stored
  let stored_params = ctx
    .service
    .list_mcp_auth_config_params(TEST_TENANT_ID, "ac-1")
    .await?;
  assert_eq!(2, stored_params.len());
  assert_eq!("Authorization", stored_params[0].param_key);
  assert_eq!("header", stored_params[0].param_type);
  assert_eq!("api_key", stored_params[1].param_key);
  assert_eq!("query", stored_params[1].param_type);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_auth_config_oauth_with_detail(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-oauth-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-oauth-1", ctx.now);

  let (created_config, created_detail) = ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  assert_eq!("ac-oauth-1", created_config.id);
  assert_eq!("oauth", created_config.config_type);
  assert_eq!("ac-oauth-1", created_detail.auth_config_id);
  assert_eq!("my-client-id", created_detail.client_id);
  assert_eq!(
    "https://auth.example.com/authorize",
    created_detail.authorization_endpoint
  );
  assert_eq!(
    "https://auth.example.com/token",
    created_detail.token_endpoint
  );

  // Verify we can read it back via get_mcp_oauth_config_detail
  let fetched = ctx
    .service
    .get_mcp_oauth_config_detail(TEST_TENANT_ID, "ac-oauth-1")
    .await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("my-client-id", fetched.client_id);
  assert_eq!(
    "https://auth.example.com/authorize",
    fetched.authorization_endpoint
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_auth_configs_by_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://s1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s2", "https://s2.example.com", ctx.now),
    )
    .await?;

  // Create 2 configs for server s1
  let config1 = make_auth_config_row("ac-1", "s1", "header", ctx.now);
  ctx
    .service
    .create_auth_config_header(TEST_TENANT_ID, &config1, vec![])
    .await?;
  let config2 = make_auth_config_row("ac-2", "s1", "header", ctx.now);
  ctx
    .service
    .create_auth_config_header(TEST_TENANT_ID, &config2, vec![])
    .await?;

  // Create 1 config for server s2
  let config3 = make_auth_config_row("ac-3", "s2", "header", ctx.now);
  ctx
    .service
    .create_auth_config_header(TEST_TENANT_ID, &config3, vec![])
    .await?;

  // List for s1 should return 2
  let s1_configs = ctx
    .service
    .list_mcp_auth_configs_by_server(TEST_TENANT_ID, "s1")
    .await?;
  assert_eq!(2, s1_configs.len());

  // List for s2 should return 1
  let s2_configs = ctx
    .service
    .list_mcp_auth_configs_by_server(TEST_TENANT_ID, "s2")
    .await?;
  assert_eq!(1, s2_configs.len());
  assert_eq!("ac-3", s2_configs[0].id);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_auth_config(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-get-1", "s1", "header", ctx.now);
  ctx
    .service
    .create_auth_config_header(TEST_TENANT_ID, &config, vec![])
    .await?;

  let fetched = ctx
    .service
    .get_mcp_auth_config(TEST_TENANT_ID, "ac-get-1")
    .await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("ac-get-1", fetched.id);
  assert_eq!("s1", fetched.mcp_server_id);
  assert_eq!("header", fetched.config_type);
  assert_eq!("Config ac-get-1", fetched.name);

  // Non-existent ID returns None
  let missing = ctx
    .service
    .get_mcp_auth_config(TEST_TENANT_ID, "no-such-config")
    .await?;
  assert_eq!(None, missing);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_auth_config_cascades(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-del-1", "s1", "header", ctx.now);
  let params = vec![
    make_auth_config_param_row("acp-d1", "ac-del-1", "header", "X-Api-Key", ctx.now),
    make_auth_config_param_row("acp-d2", "ac-del-1", "header", "X-Secret", ctx.now),
  ];
  ctx
    .service
    .create_auth_config_header(TEST_TENANT_ID, &config, params)
    .await?;

  // Verify params exist
  let params_before = ctx
    .service
    .list_mcp_auth_config_params(TEST_TENANT_ID, "ac-del-1")
    .await?;
  assert_eq!(2, params_before.len());

  // Delete the config
  ctx
    .service
    .delete_mcp_auth_config(TEST_TENANT_ID, "ac-del-1")
    .await?;

  // Config should be gone
  let gone = ctx
    .service
    .get_mcp_auth_config(TEST_TENANT_ID, "ac-del-1")
    .await?;
  assert_eq!(None, gone);

  // Params should be cascaded away
  let params_after = ctx
    .service
    .list_mcp_auth_config_params(TEST_TENANT_ID, "ac-del-1")
    .await?;
  assert_eq!(0, params_after.len());
  Ok(())
}

// ============================================================================
// Instance Composite Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_mcp_with_credentials(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let mcp = McpEntity {
    auth_type: McpAuthType::Header,
    ..make_mcp("m1", "s1", "cred-mcp", TEST_USER_ID, ctx.now)
  };

  let auth_params = vec![
    make_auth_param_row(
      "ap-1",
      "m1",
      "header",
      "Authorization",
      "Bearer tok123",
      ctx.now,
    ),
    make_auth_param_row("ap-2", "m1", "query", "api_key", "secret456", ctx.now),
  ];

  let created = ctx
    .service
    .create_mcp_with_auth(TEST_TENANT_ID, &mcp, Some(auth_params), None, TEST_USER_ID)
    .await?;

  assert_eq!("m1", created.id);

  // Verify auth params were created
  let params = ctx
    .service
    .list_mcp_auth_params(TEST_TENANT_ID, "m1")
    .await?;
  assert_eq!(2, params.len());

  // Decrypt and verify values
  let decrypted = ctx
    .service
    .get_decrypted_auth_params(TEST_TENANT_ID, "m1")
    .await?;
  assert!(decrypted.is_some());
  let decrypted = decrypted.unwrap();
  assert_eq!(1, decrypted.headers.len());
  assert_eq!("Authorization", decrypted.headers[0].0);
  assert_eq!("Bearer tok123", decrypted.headers[0].1);
  assert_eq!(1, decrypted.query_params.len());
  assert_eq!("api_key", decrypted.query_params[0].0);
  assert_eq!("secret456", decrypted.query_params[0].1);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_mcp_with_oauth_token_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  // Create auth config for the OAuth token FK
  let config = make_auth_config_row("ac-oauth-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-oauth-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  // Create an OAuth token without mcp_id
  let token = make_oauth_token(
    "tok-1",
    None,
    "ac-oauth-1",
    TEST_USER_ID,
    "access-abc",
    Some("refresh-xyz"),
    ctx.now,
  );
  ctx.service.create_mcp_oauth_token(&token).await?;

  // Create MCP with oauth_token_id
  let mcp = McpEntity {
    auth_type: McpAuthType::Oauth,
    auth_config_id: Some("ac-oauth-1".to_string()),
    ..make_mcp("m1", "s1", "oauth-mcp", TEST_USER_ID, ctx.now)
  };

  let created = ctx
    .service
    .create_mcp_with_auth(
      TEST_TENANT_ID,
      &mcp,
      None,
      Some("tok-1".to_string()),
      TEST_USER_ID,
    )
    .await?;

  assert_eq!("m1", created.id);

  // Verify the token's mcp_id is now set
  let fetched_token = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-1")
    .await?;
  assert!(fetched_token.is_some());
  assert_eq!(Some("m1".to_string()), fetched_token.unwrap().mcp_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp_replaces_credentials(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let mcp = McpEntity {
    auth_type: McpAuthType::Header,
    ..make_mcp("m1", "s1", "cred-mcp", TEST_USER_ID, ctx.now)
  };

  // Create with initial credentials
  let old_params = vec![make_auth_param_row(
    "ap-old",
    "m1",
    "header",
    "Authorization",
    "Bearer old-token",
    ctx.now,
  )];
  ctx
    .service
    .create_mcp_with_auth(TEST_TENANT_ID, &mcp, Some(old_params), None, TEST_USER_ID)
    .await?;

  // Update with new credentials
  let updated_at = ctx.now + chrono::Duration::seconds(30);
  let updated_mcp = McpEntity {
    updated_at,
    ..mcp.clone()
  };
  let new_params = vec![
    make_auth_param_row(
      "ap-new-1",
      "m1",
      "header",
      "Authorization",
      "Bearer new-token",
      updated_at,
    ),
    make_auth_param_row(
      "ap-new-2",
      "m1",
      "query",
      "key",
      "new-key-value",
      updated_at,
    ),
  ];

  ctx
    .service
    .update_mcp_with_auth(
      TEST_TENANT_ID,
      &updated_mcp,
      Some(new_params),
      None,
      TEST_USER_ID,
    )
    .await?;

  // Verify old params deleted and new params present
  let params = ctx
    .service
    .list_mcp_auth_params(TEST_TENANT_ID, "m1")
    .await?;
  assert_eq!(2, params.len());

  let decrypted = ctx
    .service
    .get_decrypted_auth_params(TEST_TENANT_ID, "m1")
    .await?;
  assert!(decrypted.is_some());
  let decrypted = decrypted.unwrap();
  assert_eq!(1, decrypted.headers.len());
  assert_eq!("Bearer new-token", decrypted.headers[0].1);
  assert_eq!(1, decrypted.query_params.len());
  assert_eq!("new-key-value", decrypted.query_params[0].1);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp_changes_oauth_token(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  // Create auth config for OAuth
  let config = make_auth_config_row("ac-oauth-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-oauth-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  // Create the MCP instance first (FK target for oauth token)
  let mcp = McpEntity {
    auth_type: McpAuthType::Oauth,
    auth_config_id: Some("ac-oauth-1".to_string()),
    ..make_mcp("m1", "s1", "oauth-mcp", TEST_USER_ID, ctx.now)
  };
  ctx.service.create_mcp(TEST_TENANT_ID, &mcp).await?;

  // Create old token linked to the MCP
  let old_token = make_oauth_token(
    "tok-old",
    Some("m1"),
    "ac-oauth-1",
    TEST_USER_ID,
    "old-access",
    None,
    ctx.now,
  );
  ctx.service.create_mcp_oauth_token(&old_token).await?;

  // Create a new unlinked token
  let new_token_time = ctx.now + chrono::Duration::seconds(10);
  let new_token = make_oauth_token(
    "tok-new",
    None,
    "ac-oauth-1",
    TEST_USER_ID,
    "new-access",
    Some("new-refresh"),
    new_token_time,
  );
  ctx.service.create_mcp_oauth_token(&new_token).await?;

  // Update MCP to use the new token
  let updated_at = ctx.now + chrono::Duration::seconds(30);
  let updated_mcp = McpEntity {
    updated_at,
    ..mcp.clone()
  };

  ctx
    .service
    .update_mcp_with_auth(
      TEST_TENANT_ID,
      &updated_mcp,
      None,
      Some("tok-new".to_string()),
      TEST_USER_ID,
    )
    .await?;

  // Old token should be deleted (update_mcp_with_auth deletes by mcp_id)
  let old_fetched = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-old")
    .await?;
  assert_eq!(None, old_fetched);

  // New token should be linked
  let new_fetched = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-new")
    .await?;
  assert!(new_fetched.is_some());
  assert_eq!(Some("m1".to_string()), new_fetched.unwrap().mcp_id);
  Ok(())
}

// ============================================================================
// Encryption Round-Trip Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_auth_param_encryption_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp("m1", "s1", "enc-mcp", TEST_USER_ID, ctx.now),
    )
    .await?;

  let param = make_auth_param_row(
    "ap-enc-1",
    "m1",
    "header",
    "X-Custom-Key",
    "super-secret-value-123!@#",
    ctx.now,
  );
  ctx.service.create_mcp_auth_param(&param).await?;

  // Read back via decryption
  let decrypted = ctx
    .service
    .get_decrypted_auth_params(TEST_TENANT_ID, "m1")
    .await?;
  assert!(decrypted.is_some());
  let decrypted = decrypted.unwrap();
  assert_eq!(1, decrypted.headers.len());
  assert_eq!("X-Custom-Key", decrypted.headers[0].0);
  assert_eq!("super-secret-value-123!@#", decrypted.headers[0].1);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_oauth_token_encryption_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  // Need an auth config for FK
  let config = make_auth_config_row("ac-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  let token = make_oauth_token(
    "tok-enc-1",
    None,
    "ac-1",
    TEST_USER_ID,
    "my-access-token-abc",
    Some("my-refresh-token-xyz"),
    ctx.now,
  );
  ctx.service.create_mcp_oauth_token(&token).await?;

  // Decrypt access token
  let access = ctx
    .service
    .get_decrypted_oauth_access_token(TEST_TENANT_ID, "tok-enc-1")
    .await?;
  assert_eq!(Some("my-access-token-abc".to_string()), access);

  // Decrypt refresh token
  let refresh = ctx
    .service
    .get_decrypted_refresh_token(TEST_TENANT_ID, "tok-enc-1")
    .await?;
  assert_eq!(Some("my-refresh-token-xyz".to_string()), refresh);
  Ok(())
}

// ============================================================================
// OAuth Token Lifecycle Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_store_oauth_token_null_mcp_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  let token = make_oauth_token(
    "tok-null-mcp",
    None,
    "ac-1",
    TEST_USER_ID,
    "access-no-mcp",
    None,
    ctx.now,
  );

  let created = ctx
    .service
    .store_oauth_token(TEST_TENANT_ID, None, TEST_USER_ID, &token)
    .await?;

  assert_eq!("tok-null-mcp", created.id);
  assert_eq!(None, created.mcp_id);

  let fetched = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-null-mcp")
    .await?;
  assert!(fetched.is_some());
  assert_eq!(None, fetched.unwrap().mcp_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_store_oauth_token_replaces_existing(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp("m1", "s1", "store-mcp", TEST_USER_ID, ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  // Store first token with mcp_id
  let token1 = make_oauth_token(
    "tok-1",
    Some("m1"),
    "ac-1",
    TEST_USER_ID,
    "access-first",
    None,
    ctx.now,
  );
  ctx
    .service
    .store_oauth_token(
      TEST_TENANT_ID,
      Some("m1".to_string()),
      TEST_USER_ID,
      &token1,
    )
    .await?;

  // Store second token with same mcp_id — should delete first
  let token2_time = ctx.now + chrono::Duration::seconds(10);
  let token2 = make_oauth_token(
    "tok-2",
    Some("m1"),
    "ac-1",
    TEST_USER_ID,
    "access-second",
    None,
    token2_time,
  );
  ctx
    .service
    .store_oauth_token(
      TEST_TENANT_ID,
      Some("m1".to_string()),
      TEST_USER_ID,
      &token2,
    )
    .await?;

  // First token should be gone
  let first = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-1")
    .await?;
  assert_eq!(None, first);

  // Second token should exist
  let second = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-2")
    .await?;
  assert!(second.is_some());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_link_oauth_token_to_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp("m1", "s1", "link-mcp", TEST_USER_ID, ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  // Create token without mcp_id
  let token = make_oauth_token(
    "tok-link",
    None,
    "ac-1",
    TEST_USER_ID,
    "access-link",
    None,
    ctx.now,
  );
  ctx.service.create_mcp_oauth_token(&token).await?;

  // Link it
  ctx
    .service
    .link_oauth_token_to_mcp(TEST_TENANT_ID, "tok-link", TEST_USER_ID, "m1")
    .await?;

  // Verify mcp_id is set
  let fetched = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_ID, TEST_USER_ID, "tok-link")
    .await?;
  assert!(fetched.is_some());
  assert_eq!(Some("m1".to_string()), fetched.unwrap().mcp_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_link_oauth_token_wrong_user_fails(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp("m1", "s1", "link-fail-mcp", TEST_USER_ID, ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  // Create token owned by TEST_USER_ID
  let token = make_oauth_token(
    "tok-wrong-user",
    None,
    "ac-1",
    TEST_USER_ID,
    "access-wrong",
    None,
    ctx.now,
  );
  ctx.service.create_mcp_oauth_token(&token).await?;

  // Try to link as a different user — should fail
  let result = ctx
    .service
    .link_oauth_token_to_mcp(TEST_TENANT_ID, "tok-wrong-user", "other-user-id", "m1")
    .await;
  assert!(result.is_err());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_latest_oauth_token_by_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp("m1", "s1", "latest-mcp", TEST_USER_ID, ctx.now),
    )
    .await?;

  let config = make_auth_config_row("ac-1", "s1", "oauth", ctx.now);
  let detail = make_oauth_config_detail("ac-1", ctx.now);
  ctx
    .service
    .create_auth_config_oauth(TEST_TENANT_ID, &config, &detail)
    .await?;

  // Create two tokens for the same MCP
  let token1 = make_oauth_token(
    "tok-older",
    Some("m1"),
    "ac-1",
    TEST_USER_ID,
    "access-older",
    None,
    ctx.now,
  );
  ctx.service.create_mcp_oauth_token(&token1).await?;

  let later = ctx.now + chrono::Duration::seconds(60);
  let token2 = make_oauth_token(
    "tok-newer",
    Some("m1"),
    "ac-1",
    TEST_USER_ID,
    "access-newer",
    None,
    later,
  );
  ctx.service.create_mcp_oauth_token(&token2).await?;

  // Should return the newest one
  let latest = ctx
    .service
    .get_latest_oauth_token_by_mcp(TEST_TENANT_ID, "m1")
    .await?;
  assert!(latest.is_some());
  assert_eq!("tok-newer", latest.unwrap().id);
  Ok(())
}

// ============================================================================
// Auth Params Read Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_decrypted_auth_params_headers_and_queries(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server("s1", "https://mcp.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp("m1", "s1", "mixed-params", TEST_USER_ID, ctx.now),
    )
    .await?;

  // Create mixed header + query params
  let header_param = make_auth_param_row(
    "ap-h1",
    "m1",
    "header",
    "Authorization",
    "Bearer h-token",
    ctx.now,
  );
  let query_param1 = make_auth_param_row("ap-q1", "m1", "query", "api_key", "qk-value-1", ctx.now);
  let query_param2 = make_auth_param_row("ap-q2", "m1", "query", "secret", "qk-value-2", ctx.now);

  ctx.service.create_mcp_auth_param(&header_param).await?;
  ctx.service.create_mcp_auth_param(&query_param1).await?;
  ctx.service.create_mcp_auth_param(&query_param2).await?;

  let decrypted = ctx
    .service
    .get_decrypted_auth_params(TEST_TENANT_ID, "m1")
    .await?;
  assert!(decrypted.is_some());
  let decrypted = decrypted.unwrap();

  // Verify headers
  assert_eq!(1, decrypted.headers.len());
  assert_eq!("Authorization", decrypted.headers[0].0);
  assert_eq!("Bearer h-token", decrypted.headers[0].1);

  // Verify query params
  assert_eq!(2, decrypted.query_params.len());
  let query_keys: Vec<&str> = decrypted
    .query_params
    .iter()
    .map(|(k, _)| k.as_str())
    .collect();
  assert!(query_keys.contains(&"api_key"));
  assert!(query_keys.contains(&"secret"));

  // Verify no params returns None
  let empty = ctx
    .service
    .get_decrypted_auth_params(TEST_TENANT_ID, "no-such-mcp")
    .await?;
  assert_eq!(None, empty);
  Ok(())
}
