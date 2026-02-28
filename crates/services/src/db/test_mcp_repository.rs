use crate::{
  db::{
    encryption::encrypt_api_key, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow,
    McpRepository, McpRow, McpServerRow,
  },
  test_utils::{sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use objs::{AppError, McpAuthType, RegistrationType};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

const ENCRYPTION_KEY: &[u8] = b"01234567890123456789012345678901";

fn test_mcp_server_row(id: &str, url: &str, now: DateTime<Utc>) -> McpServerRow {
  McpServerRow {
    id: id.to_string(),
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

fn test_mcp_row(
  id: &str,
  server_id: &str,
  slug: &str,
  user_id: &str,
  now: DateTime<Utc>,
) -> McpRow {
  McpRow {
    id: id.to_string(),
    created_by: user_id.to_string(),
    mcp_server_id: server_id.to_string(),
    name: format!("MCP {}", id),
    slug: slug.to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: McpAuthType::Public,
    auth_uuid: None,
    created_at: now,
    updated_at: now,
  }
}

fn test_auth_header_row(id: &str, server_id: &str, now: DateTime<Utc>) -> McpAuthHeaderRow {
  let (encrypted, salt, nonce) =
    encrypt_api_key(ENCRYPTION_KEY, "Bearer sk-secret-token-123").expect("encryption failed");
  McpAuthHeaderRow {
    id: id.to_string(),
    name: "Header".to_string(),
    mcp_server_id: server_id.to_string(),
    header_key: "Authorization".to_string(),
    encrypted_header_value: encrypted,
    header_value_salt: salt,
    header_value_nonce: nonce,
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn test_oauth_config_row(id: &str, server_id: &str, now: DateTime<Utc>) -> McpOAuthConfigRow {
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

fn test_oauth_token_row(config_id: &str, id: &str, now: DateTime<Utc>) -> McpOAuthTokenRow {
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
// MCP Server Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let row = test_mcp_server_row("s1", "https://mcp.example.com", ctx.now);
  let created = ctx.service.create_mcp_server(&row).await?;
  assert_eq!(row.id, created.id);
  assert_eq!(row.url, created.url);
  assert_eq!(row.name, created.name);
  assert_eq!(row.description, created.description);
  assert_eq!(row.enabled, created.enabled);

  let fetched = ctx.service.get_mcp_server("s1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("s1", fetched.id);
  assert_eq!("https://mcp.example.com", fetched.url);
  assert_eq!(ctx.now, fetched.created_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_server_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_mcp_server("nonexistent").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let row = test_mcp_server_row("s1", "https://mcp.example.com", ctx.now);
  ctx.service.create_mcp_server(&row).await?;

  let updated_at = ctx.now + chrono::Duration::seconds(60);
  let updated_row = McpServerRow {
    url: "https://mcp-updated.example.com".to_string(),
    name: "Updated Server".to_string(),
    description: Some("Updated description".to_string()),
    enabled: false,
    updated_by: "admin2".to_string(),
    updated_at,
    ..row
  };
  let updated = ctx.service.update_mcp_server(&updated_row).await?;
  assert_eq!("https://mcp-updated.example.com", updated.url);
  assert_eq!("Updated Server", updated.name);
  assert_eq!(false, updated.enabled);
  assert_eq!(updated_at, updated.updated_at);

  let fetched = ctx.service.get_mcp_server("s1").await?.unwrap();
  assert_eq!("https://mcp-updated.example.com", fetched.url);
  assert_eq!(false, fetched.enabled);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_server_by_url(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let row = test_mcp_server_row("s1", "https://mcp.example.com/api", ctx.now);
  ctx.service.create_mcp_server(&row).await?;

  let found = ctx
    .service
    .get_mcp_server_by_url("https://mcp.example.com/api")
    .await?;
  assert!(found.is_some());
  assert_eq!("s1", found.unwrap().id);

  let not_found = ctx
    .service
    .get_mcp_server_by_url("https://other.example.com")
    .await?;
  assert_eq!(None, not_found);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcp_servers(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://one.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_server(&McpServerRow {
      enabled: false,
      ..test_mcp_server_row("s2", "https://two.example.com", ctx.now)
    })
    .await?;

  let all = ctx.service.list_mcp_servers(None).await?;
  assert_eq!(2, all.len());

  let enabled = ctx.service.list_mcp_servers(Some(true)).await?;
  assert_eq!(1, enabled.len());
  assert_eq!("s1", enabled[0].id);

  let disabled = ctx.service.list_mcp_servers(Some(false)).await?;
  assert_eq!(1, disabled.len());
  assert_eq!("s2", disabled[0].id);
  Ok(())
}

// ============================================================================
// MCP Instance Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;

  let mcp = test_mcp_row("m1", "s1", "my-mcp", "user-1", ctx.now);
  let created = ctx.service.create_mcp(&mcp).await?;
  assert_eq!("m1", created.id);
  assert_eq!("my-mcp", created.slug);

  let fetched = ctx.service.get_mcp("user-1", "m1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("m1", fetched.id);
  assert_eq!("user-1", fetched.created_by);
  assert_eq!("s1", fetched.mcp_server_id);
  assert_eq!(ctx.now, fetched.created_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_wrong_user(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m1", "s1", "my-mcp", "user-1", ctx.now))
    .await?;

  let result = ctx.service.get_mcp("user-2", "m1").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_by_slug(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m1", "s1", "my-mcp-slug", "user-1", ctx.now))
    .await?;

  let found = ctx.service.get_mcp_by_slug("user-1", "my-mcp-slug").await?;
  assert!(found.is_some());
  assert_eq!("m1", found.unwrap().id);

  let wrong_user = ctx.service.get_mcp_by_slug("user-2", "my-mcp-slug").await?;
  assert_eq!(None, wrong_user);

  let wrong_slug = ctx
    .service
    .get_mcp_by_slug("user-1", "no-such-slug")
    .await?;
  assert_eq!(None, wrong_slug);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  let mcp = test_mcp_row("m1", "s1", "my-mcp", "user-1", ctx.now);
  ctx.service.create_mcp(&mcp).await?;

  let updated_at = ctx.now + chrono::Duration::seconds(30);
  let updated = McpRow {
    name: "Updated MCP".to_string(),
    slug: "updated-mcp".to_string(),
    description: Some("Now with description".to_string()),
    enabled: false,
    tools_cache: Some("{\"tools\":[]}".to_string()),
    tools_filter: Some("[\"tool1\"]".to_string()),
    auth_type: McpAuthType::Header,
    auth_uuid: Some("auth-1".to_string()),
    updated_at,
    ..mcp
  };
  let result = ctx.service.update_mcp(&updated).await?;
  assert_eq!("Updated MCP", result.name);
  assert_eq!("updated-mcp", result.slug);
  assert_eq!(false, result.enabled);
  assert_eq!(Some("{\"tools\":[]}".to_string()), result.tools_cache);
  assert_eq!(updated_at, result.updated_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m1", "s1", "my-mcp", "user-1", ctx.now))
    .await?;

  ctx.service.delete_mcp("user-1", "m1").await?;
  let gone = ctx.service.get_mcp("user-1", "m1").await?;
  assert_eq!(None, gone);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp_wrong_user_noop(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m1", "s1", "my-mcp", "user-1", ctx.now))
    .await?;

  // delete by wrong user should be a no-op
  ctx.service.delete_mcp("user-2", "m1").await?;
  let still_exists = ctx.service.get_mcp("user-1", "m1").await?;
  assert!(still_exists.is_some());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcps_with_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m1", "s1", "mcp-one", "user-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m2", "s1", "mcp-two", "user-1", ctx.now))
    .await?;
  // Different user — should not appear
  ctx
    .service
    .create_mcp(&test_mcp_row("m3", "s1", "mcp-three", "user-2", ctx.now))
    .await?;

  let results = ctx.service.list_mcps_with_server("user-1").await?;
  assert_eq!(2, results.len());
  assert_eq!("https://mcp.example.com", results[0].server_url);
  assert_eq!("Server s1", results[0].server_name);
  assert_eq!(true, results[0].server_enabled);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcps_with_server_with_auth(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&test_auth_header_row("ah-1", "s1", ctx.now))
    .await?;

  let mcp = McpRow {
    auth_type: McpAuthType::Header,
    auth_uuid: Some("ah-1".to_string()),
    ..test_mcp_row("m1", "s1", "mcp-with-auth", "user-1", ctx.now)
  };
  ctx.service.create_mcp(&mcp).await?;

  let results = ctx.service.list_mcps_with_server("user-1").await?;
  assert_eq!(1, results.len());
  assert_eq!(McpAuthType::Header, results[0].auth_type);
  assert_eq!(Some("ah-1".to_string()), results[0].auth_uuid);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_count_mcps_by_server_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;

  // Create 2 enabled + 1 disabled
  ctx
    .service
    .create_mcp(&test_mcp_row("m1", "s1", "mcp-one", "user-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp(&test_mcp_row("m2", "s1", "mcp-two", "user-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp(&McpRow {
      enabled: false,
      ..test_mcp_row("m3", "s1", "mcp-three", "user-1", ctx.now)
    })
    .await?;

  let (enabled, disabled) = ctx.service.count_mcps_by_server_id("s1").await?;
  assert_eq!(2, enabled);
  assert_eq!(1, disabled);

  // Non-existent server returns (0, 0)
  let (e, d) = ctx.service.count_mcps_by_server_id("nonexistent").await?;
  assert_eq!(0, e);
  assert_eq!(0, d);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_clear_mcp_tools_by_server_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s2",
      "https://other.example.com",
      ctx.now,
    ))
    .await?;

  // Create MCPs with tools on both servers
  ctx
    .service
    .create_mcp(&McpRow {
      tools_cache: Some("{\"tools\":[]}".to_string()),
      tools_filter: Some("[\"tool1\"]".to_string()),
      ..test_mcp_row("m1", "s1", "mcp-one", "user-1", ctx.now)
    })
    .await?;
  ctx
    .service
    .create_mcp(&McpRow {
      tools_cache: Some("{\"tools\":[\"t2\"]}".to_string()),
      ..test_mcp_row("m2", "s1", "mcp-two", "user-1", ctx.now)
    })
    .await?;
  ctx
    .service
    .create_mcp(&McpRow {
      tools_cache: Some("{\"other\":[]}".to_string()),
      ..test_mcp_row("m3", "s2", "mcp-three", "user-1", ctx.now)
    })
    .await?;

  let affected = ctx.service.clear_mcp_tools_by_server_id("s1").await?;
  assert_eq!(2, affected);

  // Verify s1 MCPs cleared
  let m1 = ctx.service.get_mcp("user-1", "m1").await?.unwrap();
  assert_eq!(None, m1.tools_cache);
  assert_eq!(None, m1.tools_filter);

  let m2 = ctx.service.get_mcp("user-1", "m2").await?.unwrap();
  assert_eq!(None, m2.tools_cache);

  // Verify s2 MCP untouched
  let m3 = ctx.service.get_mcp("user-1", "m3").await?.unwrap();
  assert_eq!(Some("{\"other\":[]}".to_string()), m3.tools_cache);
  Ok(())
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;

  let row = test_auth_header_row("ah-1", "s1", ctx.now);
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  let row = test_auth_header_row("ah-1", "s1", ctx.now);
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&test_auth_header_row("ah-1", "s1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_server(&test_mcp_server_row(
      "s2",
      "https://other.example.com",
      ctx.now,
    ))
    .await?;

  ctx
    .service
    .create_mcp_auth_header(&test_auth_header_row("ah-1", "s1", ctx.now))
    .await?;
  let later = ctx.now + chrono::Duration::seconds(10);
  ctx
    .service
    .create_mcp_auth_header(&McpAuthHeaderRow {
      id: "ah-2".to_string(),
      name: "Header 2".to_string(),
      created_at: later,
      updated_at: later,
      ..test_auth_header_row("ah-2", "s1", later)
    })
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&test_auth_header_row("ah-3", "s2", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_auth_header(&test_auth_header_row("ah-1", "s1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;

  let row = test_oauth_config_row("oc-1", "s1", ctx.now);
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;

  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
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
      ..test_oauth_config_row("oc-2", "s1", later)
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-2", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  let token = test_oauth_token_row("oc-1", "ot-1", ctx.now);
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  let older = test_oauth_token_row("oc-1", "ot-1", ctx.now);
  ctx.service.create_mcp_oauth_token(&older).await?;

  let later = ctx.now + chrono::Duration::seconds(100);
  let newer = McpOAuthTokenRow {
    id: "ot-2".to_string(),
    created_at: later,
    updated_at: later,
    ..test_oauth_token_row("oc-1", "ot-2", later)
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  let token = test_oauth_token_row("oc-1", "ot-1", ctx.now);
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-2", ctx.now))
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;

  // user-1 token
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
    .await?;
  // user-2 token
  ctx
    .service
    .create_mcp_oauth_token(&McpOAuthTokenRow {
      id: "ot-2".to_string(),
      created_by: "user-2".to_string(),
      ..test_oauth_token_row("oc-1", "ot-2", ctx.now)
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
    .create_mcp_server(&test_mcp_server_row(
      "s1",
      "https://mcp.example.com",
      ctx.now,
    ))
    .await?;
  ctx
    .service
    .create_mcp_oauth_config(&test_oauth_config_row("oc-1", "s1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_oauth_token(&test_oauth_token_row("oc-1", "ot-1", ctx.now))
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
