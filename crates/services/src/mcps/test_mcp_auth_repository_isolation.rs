use crate::db::encryption::encrypt_api_key;
use crate::mcps::test_helpers::ENCRYPTION_KEY;
use crate::mcps::{
  McpAuthHeaderEntity, McpAuthRepository, McpOAuthConfigEntity, McpOAuthTokenEntity,
  McpServerEntity, McpServerRepository, RegistrationType,
};
use crate::test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_server_for(tenant_id: &str, id: &str, url: &str, now: DateTime<Utc>) -> McpServerEntity {
  McpServerEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    url: url.to_string(),
    name: format!("Server {}", id),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn make_auth_header_for(
  tenant_id: &str,
  id: &str,
  server_id: &str,
  now: DateTime<Utc>,
) -> McpAuthHeaderEntity {
  let (encrypted, salt, nonce) =
    encrypt_api_key(ENCRYPTION_KEY, "Bearer sk-secret-token-123").expect("encryption failed");
  McpAuthHeaderEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    name: "Header".to_string(),
    mcp_server_id: server_id.to_string(),
    header_key: "Authorization".to_string(),
    encrypted_header_value: encrypted,
    header_value_salt: salt,
    header_value_nonce: nonce,
    created_at: now,
    updated_at: now,
  }
}

fn make_oauth_config_for(
  tenant_id: &str,
  id: &str,
  server_id: &str,
  now: DateTime<Utc>,
) -> McpOAuthConfigEntity {
  let (encrypted, salt, nonce) =
    encrypt_api_key(ENCRYPTION_KEY, "my-client-secret").expect("encryption failed");
  McpOAuthConfigEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    name: format!("OAuth Config {}", id),
    mcp_server_id: server_id.to_string(),
    registration_type: RegistrationType::PreRegistered,
    client_id: format!("client-{}", id),
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
    scopes: None,
    created_at: now,
    updated_at: now,
  }
}

fn make_oauth_token_for(
  tenant_id: &str,
  user_id: &str,
  config_id: &str,
  id: &str,
  now: DateTime<Utc>,
) -> McpOAuthTokenEntity {
  let (encrypted, salt, nonce) =
    encrypt_api_key(ENCRYPTION_KEY, "access-token-secret").expect("encryption failed");
  McpOAuthTokenEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    mcp_oauth_config_id: config_id.to_string(),
    encrypted_access_token: encrypted,
    access_token_salt: salt,
    access_token_nonce: nonce,
    encrypted_refresh_token: None,
    refresh_token_salt: None,
    refresh_token_nonce: None,
    scopes_granted: None,
    expires_at: None,
    user_id: user_id.to_string(),
    created_at: now,
    updated_at: now,
  }
}

// ============================================================================
// Cross-Tenant MCP Auth Header Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_mcp_auth_header_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create MCP server in tenant A, create auth header for it
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(TEST_TENANT_ID, "s-a1", "https://a1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&make_auth_header_for(
      TEST_TENANT_ID,
      "ah-a1",
      "s-a1",
      ctx.now,
    ))
    .await?;

  // Create MCP server in tenant B, create auth header for it
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_B_ID,
      &make_server_for(TEST_TENANT_B_ID, "s-b1", "https://b1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&make_auth_header_for(
      TEST_TENANT_B_ID,
      "ah-b1",
      "s-b1",
      ctx.now,
    ))
    .await?;

  // list_mcp_auth_headers_by_server(TENANT_A, server_a) -> only tenant A's header
  let headers_a = ctx
    .service
    .list_mcp_auth_headers_by_server(TEST_TENANT_ID, "s-a1")
    .await?;
  assert_eq!(1, headers_a.len());
  assert_eq!("ah-a1", headers_a[0].id);

  // get_mcp_auth_header(TENANT_B, header_a_id) -> None (cross-tenant)
  let cross = ctx
    .service
    .get_mcp_auth_header(TEST_TENANT_B_ID, "ah-a1")
    .await?;
  assert_eq!(None, cross);

  // get_decrypted_auth_header(TENANT_B, header_a_id) -> None (cross-tenant)
  let cross_decrypted = ctx
    .service
    .get_decrypted_auth_header(TEST_TENANT_B_ID, "ah-a1")
    .await?;
  assert_eq!(None, cross_decrypted);

  Ok(())
}

// ============================================================================
// Cross-Tenant MCP OAuth Config Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_mcp_oauth_config_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create server + oauth config in tenant A
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(TEST_TENANT_ID, "s-a1", "https://a1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_for(
      TEST_TENANT_ID,
      "oc-a1",
      "s-a1",
      ctx.now,
    ))
    .await?;

  // Create server + oauth config in tenant B
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_B_ID,
      &make_server_for(TEST_TENANT_B_ID, "s-b1", "https://b1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_for(
      TEST_TENANT_B_ID,
      "oc-b1",
      "s-b1",
      ctx.now,
    ))
    .await?;

  // list_mcp_oauth_configs_by_server(TENANT_A, server_a) -> only tenant A's config
  let configs_a = ctx
    .service
    .list_mcp_oauth_configs_by_server(TEST_TENANT_ID, "s-a1")
    .await?;
  assert_eq!(1, configs_a.len());
  assert_eq!("oc-a1", configs_a[0].id);

  // get_mcp_oauth_config(TENANT_B, config_a_id) -> None
  let cross = ctx
    .service
    .get_mcp_oauth_config(TEST_TENANT_B_ID, "oc-a1")
    .await?;
  assert_eq!(None, cross);

  Ok(())
}

// ============================================================================
// Cross-Tenant MCP OAuth Token Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_mcp_oauth_token_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create server + config + token in tenant A (user_id = TEST_USER_ID)
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(TEST_TENANT_ID, "s-a1", "https://a1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_for(
      TEST_TENANT_ID,
      "oc-a1",
      "s-a1",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_for(
      TEST_TENANT_ID,
      TEST_USER_ID,
      "oc-a1",
      "ot-a1",
      ctx.now,
    ))
    .await?;

  // Create server + config + token in tenant B (user_id = TEST_USER_ID)
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_B_ID,
      &make_server_for(TEST_TENANT_B_ID, "s-b1", "https://b1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&make_oauth_config_for(
      TEST_TENANT_B_ID,
      "oc-b1",
      "s-b1",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&make_oauth_token_for(
      TEST_TENANT_B_ID,
      TEST_USER_ID,
      "oc-b1",
      "ot-b1",
      ctx.now,
    ))
    .await?;

  // get_mcp_oauth_token(TENANT_B, TEST_USER_ID, token_a_id) -> None
  let cross = ctx
    .service
    .get_mcp_oauth_token(TEST_TENANT_B_ID, TEST_USER_ID, "ot-a1")
    .await?;
  assert_eq!(None, cross);

  // get_latest_oauth_token_by_config(TENANT_B, config_a_id) -> None
  let cross_latest = ctx
    .service
    .get_latest_oauth_token_by_config(TEST_TENANT_B_ID, "oc-a1")
    .await?;
  assert_eq!(None, cross_latest);

  // get_decrypted_oauth_bearer(TENANT_B, token_a_id) -> None
  let cross_bearer = ctx
    .service
    .get_decrypted_oauth_bearer(TEST_TENANT_B_ID, "ot-a1")
    .await?;
  assert_eq!(None, cross_bearer);

  Ok(())
}
