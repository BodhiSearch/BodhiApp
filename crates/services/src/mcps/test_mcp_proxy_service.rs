use crate::db::{DbService, TimeService};
use crate::mcps::{
  DefaultMcpService, McpAuthParamInput, McpAuthParamType, McpAuthType, McpError, McpRequest,
  McpServerRequest, McpService,
};
use crate::test_utils::{
  test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID, TEST_USER_ID,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn default_time_service() -> Arc<dyn TimeService> {
  Arc::new(FrozenTimeService::default())
}

fn make_service(db: TestDbService) -> Result<DefaultMcpService, McpError> {
  let db: Arc<dyn DbService> = Arc::new(db);
  let mcp_client: Arc<dyn mcp_client::McpClient> = Arc::new(mcp_client::MockMcpClient::new());
  DefaultMcpService::new(db, mcp_client, default_time_service())
}

fn server_request(url: &str, name: &str) -> McpServerRequest {
  McpServerRequest {
    url: url.to_string(),
    name: name.to_string(),
    description: None,
    enabled: true,
    auth_config: None,
  }
}

fn mcp_request(slug: &str, server_id: &str) -> McpRequest {
  McpRequest {
    name: format!("Test MCP {}", slug),
    slug: slug.to_string(),
    mcp_server_id: Some(server_id.to_string()),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  }
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_mcp_not_found(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db)?;

  let result = service
    .resolve_auth_params(TEST_TENANT_ID, TEST_USER_ID, "nonexistent-id")
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert!(matches!(err, McpError::McpNotFound(ref id) if id == "nonexistent-id"));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_public(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db)?;

  let server = service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;

  let _mcp = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request("proxy-pub", &server.id),
    )
    .await?;

  let auth_params = service
    .resolve_auth_params(TEST_TENANT_ID, TEST_USER_ID, &_mcp.id)
    .await?;

  assert!(auth_params.is_none());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_disabled_instance(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db)?;

  let server = service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;

  let mut request = mcp_request("proxy-dis", &server.id);
  request.enabled = false;

  let mcp = service
    .create(TEST_TENANT_ID, TEST_USER_ID, request)
    .await?;

  // resolve_auth_params succeeds even for disabled instances —
  // enabled check is the caller's responsibility
  let auth_params = service
    .resolve_auth_params(TEST_TENANT_ID, TEST_USER_ID, &mcp.id)
    .await?;

  assert!(auth_params.is_none());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_with_header_auth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db)?;

  let server = service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;

  let credentials = vec![McpAuthParamInput {
    param_type: McpAuthParamType::Header,
    param_key: "Authorization".to_string(),
    value: "Bearer secret-token".to_string(),
  }];

  let mut request = mcp_request("proxy-hdr", &server.id);
  request.auth_type = McpAuthType::Header;
  request.credentials = Some(credentials);

  let mcp = service
    .create(TEST_TENANT_ID, TEST_USER_ID, request)
    .await?;

  let auth_params = service
    .resolve_auth_params(TEST_TENANT_ID, TEST_USER_ID, &mcp.id)
    .await?;

  assert!(auth_params.is_some());
  let auth = auth_params.unwrap();
  assert_eq!(1, auth.headers.len());
  assert_eq!("Authorization", auth.headers[0].0);
  assert_eq!("Bearer secret-token", auth.headers[0].1);
  Ok(())
}
