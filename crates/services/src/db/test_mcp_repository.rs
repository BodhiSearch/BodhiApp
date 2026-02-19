use crate::{
  db::{encryption::encrypt_api_key, McpAuthHeaderRow, McpRepository, McpRow, McpServerRow},
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

  let fetched = service.get_mcp_auth_header("ah-crud-1").await?;
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

  service.delete_mcp_auth_header("ah-crud-1").await?;
  let gone = service.get_mcp_auth_header("ah-crud-1").await?;
  assert!(gone.is_none());

  Ok(())
}
