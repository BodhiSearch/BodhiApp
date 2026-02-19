use crate::{
  db::{encryption::encrypt_api_key, McpRepository, McpRow, McpServerRow},
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
    user_id: "user-1".to_string(),
    mcp_server_id: "server-1".to_string(),
    name: "Public MCP".to_string(),
    slug: "public-mcp".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "public".to_string(),
    auth_header_key: None,
    encrypted_auth_header_value: None,
    auth_header_salt: None,
    auth_header_nonce: None,
    created_at: now,
    updated_at: now,
  }
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_get_mcp_auth_header_roundtrip(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();
  service.create_mcp_server(&test_mcp_server_row(now)).await?;

  let (encrypted, salt, nonce) =
    encrypt_api_key(&service.encryption_key, "Bearer sk-secret-token-123")?;
  let row = McpRow {
    id: "mcp-auth-1".to_string(),
    user_id: "user-1".to_string(),
    mcp_server_id: "server-1".to_string(),
    name: "Auth MCP".to_string(),
    slug: "auth-mcp".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "header".to_string(),
    auth_header_key: Some("Authorization".to_string()),
    encrypted_auth_header_value: Some(encrypted),
    auth_header_salt: Some(salt),
    auth_header_nonce: Some(nonce),
    created_at: now,
    updated_at: now,
  };
  service.create_mcp(&row).await?;

  let result = service.get_mcp_auth_header("mcp-auth-1").await?;
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
async fn test_db_service_get_mcp_auth_header_returns_none_for_public(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();
  service.create_mcp_server(&test_mcp_server_row(now)).await?;
  service.create_mcp(&test_mcp_row_public(now)).await?;

  let result = service.get_mcp_auth_header("mcp-pub-1").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_db_service_list_mcps_with_server_shows_has_auth_header_value(
  #[future]
  #[from(test_db_service)]
  service: TestDbService,
) -> anyhow::Result<()> {
  let now = service.now().timestamp();
  service.create_mcp_server(&test_mcp_server_row(now)).await?;
  service.create_mcp(&test_mcp_row_public(now)).await?;

  let (encrypted, salt, nonce) = encrypt_api_key(&service.encryption_key, "Bearer secret")?;
  let auth_row = McpRow {
    id: "mcp-auth-2".to_string(),
    user_id: "user-1".to_string(),
    mcp_server_id: "server-1".to_string(),
    name: "Header MCP".to_string(),
    slug: "header-mcp".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "header".to_string(),
    auth_header_key: Some("X-Api-Key".to_string()),
    encrypted_auth_header_value: Some(encrypted),
    auth_header_salt: Some(salt),
    auth_header_nonce: Some(nonce),
    created_at: now,
    updated_at: now,
  };
  service.create_mcp(&auth_row).await?;

  let mcps = service.list_mcps_with_server("user-1").await?;
  assert_eq!(2, mcps.len());

  let public_mcp = mcps
    .iter()
    .find(|m| m.id == "mcp-pub-1")
    .expect("public mcp");
  assert_eq!("public", public_mcp.auth_type);
  assert_eq!(false, public_mcp.has_auth_header_value);

  let header_mcp = mcps
    .iter()
    .find(|m| m.id == "mcp-auth-2")
    .expect("header mcp");
  assert_eq!("header", header_mcp.auth_type);
  assert_eq!(Some("X-Api-Key".to_string()), header_mcp.auth_header_key);
  assert_eq!(true, header_mcp.has_auth_header_value);

  Ok(())
}
