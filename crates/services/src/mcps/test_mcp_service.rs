use crate::db::{DbService, McpServerRow, TimeService};
use crate::mcps::{DefaultMcpService, McpService};
use crate::test_utils::{test_db_service, FrozenTimeService, TestDbService};
use anyhow_trace::anyhow_trace;
use mcp_client::MockMcpClient;
use mockall::predicate::eq;
use objs::{AppError, McpAuthType, McpExecutionRequest, McpTool, RegistrationType};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use std::sync::Arc;

async fn setup_server(db: &dyn DbService) -> McpServerRow {
  let now = db.now();
  let row = McpServerRow {
    id: "server-1".to_string(),
    url: "https://mcp.example.com/mcp".to_string(),
    name: "Test MCP Server".to_string(),
    description: Some("Test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: now,
    updated_at: now,
  };
  db.create_mcp_server(&row).await.unwrap();
  row
}

fn default_time_service() -> Arc<dyn TimeService> {
  Arc::new(FrozenTimeService::default())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_create_with_header_auth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header(
      "user-1",
      "Header",
      "server-1",
      "Authorization",
      "Bearer sk-secret-123",
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "My Tavily",
      "my-tavily",
      "server-1",
      Some("Tavily search".to_string()),
      true,
      None,
      None,
      McpAuthType::Header,
      Some(auth_header.id.clone()),
    )
    .await?;

  assert_eq!(McpAuthType::Header, mcp.auth_type);
  assert_eq!(Some(auth_header.id.clone()), mcp.auth_uuid);

  let decrypted = db.get_decrypted_auth_header(&auth_header.id).await?;
  assert_eq!(
    Some((
      "Authorization".to_string(),
      "Bearer sk-secret-123".to_string()
    )),
    decrypted
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_create_with_public_auth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let mcp = service
    .create(
      "user-1",
      "Public MCP",
      "public-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Public,
      None,
    )
    .await?;

  assert_eq!(McpAuthType::Public, mcp.auth_type);
  assert_eq!(None, mcp.auth_uuid);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_update_switch_public_to_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let mcp = service
    .create(
      "user-1",
      "My MCP",
      "my-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Public,
      None,
    )
    .await?;
  assert_eq!(McpAuthType::Public, mcp.auth_type);

  let auth_header = service
    .create_auth_header("user-1", "Header", "server-1", "X-Api-Key", "key-abc-123")
    .await?;

  let updated = service
    .update(
      "user-1",
      &mcp.id,
      "My MCP",
      "my-mcp",
      None,
      true,
      None,
      None,
      Some(McpAuthType::Header),
      Some(auth_header.id.clone()),
    )
    .await?;

  assert_eq!(McpAuthType::Header, updated.auth_type);
  assert_eq!(Some(auth_header.id.clone()), updated.auth_uuid);

  let decrypted = db.get_decrypted_auth_header(&auth_header.id).await?;
  assert_eq!(
    Some(("X-Api-Key".to_string(), "key-abc-123".to_string())),
    decrypted
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_update_switch_header_to_public_preserves_auth_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header(
      "user-1",
      "Header",
      "server-1",
      "Authorization",
      "Bearer token",
    )
    .await?;
  let auth_id = auth_header.id.clone();

  let mcp = service
    .create(
      "user-1",
      "Auth MCP",
      "auth-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Header,
      Some(auth_id.clone()),
    )
    .await?;
  assert_eq!(McpAuthType::Header, mcp.auth_type);

  let updated = service
    .update(
      "user-1",
      &mcp.id,
      "Auth MCP",
      "auth-mcp",
      None,
      true,
      None,
      None,
      Some(McpAuthType::Public),
      None,
    )
    .await?;

  assert_eq!(McpAuthType::Public, updated.auth_type);
  assert_eq!(None, updated.auth_uuid);

  // Auth headers are admin-managed resources - they should persist
  // even when MCP instances stop using them, so they can be reused
  let preserved = db.get_mcp_auth_header(&auth_id).await?;
  assert!(
    preserved.is_some(),
    "auth header should be preserved for reuse"
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_update_keep_existing_auth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header(
      "user-1",
      "Header",
      "server-1",
      "Authorization",
      "Bearer original-token",
    )
    .await?;
  let auth_id = auth_header.id.clone();

  let mcp = service
    .create(
      "user-1",
      "Auth MCP",
      "keep-auth",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Header,
      Some(auth_id.clone()),
    )
    .await?;

  let updated = service
    .update(
      "user-1",
      &mcp.id,
      "Renamed MCP",
      "keep-auth",
      Some("new desc".to_string()),
      true,
      None,
      None,
      None, // auth_type = None means keep existing
      None,
    )
    .await?;

  assert_eq!("Renamed MCP", updated.name);
  assert_eq!(McpAuthType::Header, updated.auth_type);
  assert_eq!(Some(auth_id.clone()), updated.auth_uuid);

  let decrypted = db.get_decrypted_auth_header(&auth_id).await?;
  assert_eq!(
    Some((
      "Authorization".to_string(),
      "Bearer original-token".to_string()
    )),
    decrypted
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_delete_preserves_auth_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header(
      "user-1",
      "Header",
      "server-1",
      "Authorization",
      "Bearer token",
    )
    .await?;
  let auth_id = auth_header.id.clone();

  let mcp = service
    .create(
      "user-1",
      "Delete Me",
      "delete-me",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Header,
      Some(auth_id.clone()),
    )
    .await?;

  service.delete("user-1", &mcp.id).await?;

  // Auth headers are admin-managed shared resources - they should persist
  // even when MCP instances are deleted, so they can be reused
  let preserved = db.get_mcp_auth_header(&auth_id).await?;
  assert!(
    preserved.is_some(),
    "auth header should be preserved for reuse on MCP delete"
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_fetch_tools_passes_auth_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut mock_client = MockMcpClient::new();
  mock_client
    .expect_fetch_tools()
    .with(
      eq("https://mcp.example.com/mcp"),
      eq(Some((
        "Authorization".to_string(),
        "Bearer sk-secret".to_string(),
      ))),
    )
    .returning(|_, _| {
      Ok(vec![McpTool {
        name: "search".to_string(),
        description: Some("Search the web".to_string()),
        input_schema: Some(json!({"type": "object", "properties": {"query": {"type": "string"}}})),
      }])
    });

  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header(
      "user-1",
      "Header",
      "server-1",
      "Authorization",
      "Bearer sk-secret",
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "Tavily",
      "tavily",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Header,
      Some(auth_header.id.clone()),
    )
    .await?;

  let tools = service.fetch_tools("user-1", &mcp.id).await?;
  assert_eq!(1, tools.len());
  assert_eq!("search", tools[0].name);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_fetch_tools_for_server_passes_inline_auth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut mock_client = MockMcpClient::new();
  mock_client
    .expect_fetch_tools()
    .with(
      eq("https://mcp.example.com/mcp"),
      eq(Some((
        "X-Api-Key".to_string(),
        "inline-key-value".to_string(),
      ))),
    )
    .returning(|_, _| Ok(vec![]));

  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let tools = service
    .fetch_tools_for_server(
      "server-1",
      Some("X-Api-Key".to_string()),
      Some("inline-key-value".to_string()),
      None,
    )
    .await?;
  assert_eq!(0, tools.len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_execute_passes_auth_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut mock_client = MockMcpClient::new();
  mock_client.expect_fetch_tools().returning(|_, _| {
    Ok(vec![McpTool {
      name: "tavily_search".to_string(),
      description: Some("Search".to_string()),
      input_schema: Some(json!({"type": "object"})),
    }])
  });
  mock_client
    .expect_call_tool()
    .with(
      eq("https://mcp.example.com/mcp"),
      eq("tavily_search"),
      eq(json!({"query": "test"})),
      eq(Some((
        "Authorization".to_string(),
        "Bearer exec-token".to_string(),
      ))),
    )
    .returning(|_, _, _, _| Ok(json!({"results": []})));

  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header(
      "user-1",
      "Header",
      "server-1",
      "Authorization",
      "Bearer exec-token",
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "Tavily Exec",
      "tavily-exec",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Header,
      Some(auth_header.id.clone()),
    )
    .await?;

  service.fetch_tools("user-1", &mcp.id).await?;

  let response = service
    .execute(
      "user-1",
      &mcp.id,
      "tavily_search",
      McpExecutionRequest {
        params: json!({"query": "test"}),
      },
    )
    .await?;

  assert!(response.error.is_none());
  assert_eq!(Some(json!({"results": []})), response.result);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_fetch_tools_for_server_with_oauth_auth_uuid(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut mock_client = MockMcpClient::new();
  mock_client
    .expect_fetch_tools()
    .with(
      eq("https://mcp.example.com/mcp"),
      eq(Some((
        "Authorization".to_string(),
        "Bearer test-oauth-access-token".to_string(),
      ))),
    )
    .returning(|_, _| {
      Ok(vec![McpTool {
        name: "oauth-tool".to_string(),
        description: Some("OAuth protected tool".to_string()),
        input_schema: None,
      }])
    });

  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "my-client-id",
      Some("my-client-secret".to_string()),
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      Some("read write".to_string()),
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "test-oauth-access-token",
      None,
      Some("read write".to_string()),
      Some(3600),
    )
    .await?;

  let tools = service
    .fetch_tools_for_server("server-1", None, None, Some(token.id.clone()))
    .await?;

  assert_eq!(1, tools.len());
  assert_eq!("oauth-tool", tools[0].name);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_execute_with_oauth_auth_type(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut mock_client = MockMcpClient::new();
  mock_client.expect_fetch_tools().returning(|_, _| {
    Ok(vec![McpTool {
      name: "oauth_search".to_string(),
      description: Some("Search".to_string()),
      input_schema: Some(json!({"type": "object"})),
    }])
  });
  mock_client
    .expect_call_tool()
    .with(
      eq("https://mcp.example.com/mcp"),
      eq("oauth_search"),
      eq(json!({"query": "test"})),
      eq(Some((
        "Authorization".to_string(),
        "Bearer test-oauth-access-token".to_string(),
      ))),
    )
    .returning(|_, _, _, _| Ok(json!({"results": []})));

  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "client-id",
      Some("client-secret".to_string()),
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      None,
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "test-oauth-access-token",
      None,
      None,
      Some(3600),
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "OAuth MCP",
      "oauth-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Oauth,
      Some(token.id.clone()),
    )
    .await?;

  assert_eq!(McpAuthType::Oauth, mcp.auth_type);
  assert_eq!(Some(token.id.clone()), mcp.auth_uuid);

  service.fetch_tools("user-1", &mcp.id).await?;

  let response = service
    .execute(
      "user-1",
      &mcp.id,
      "oauth_search",
      McpExecutionRequest {
        params: json!({"query": "test"}),
      },
    )
    .await?;

  assert!(response.error.is_none());
  assert_eq!(Some(json!({"results": []})), response.result);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_update_orphan_cleanup_oauth_to_public(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "client-id",
      Some("client-secret".to_string()),
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      None,
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "access-token",
      Some("refresh-token".to_string()),
      None,
      Some(3600),
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "OAuth MCP",
      "oauth-cleanup",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Oauth,
      Some(token.id.clone()),
    )
    .await?;

  assert_eq!(McpAuthType::Oauth, mcp.auth_type);

  let updated = service
    .update(
      "user-1",
      &mcp.id,
      "OAuth MCP",
      "oauth-cleanup",
      None,
      true,
      None,
      None,
      Some(McpAuthType::Public),
      None,
    )
    .await?;

  assert_eq!(McpAuthType::Public, updated.auth_type);
  assert_eq!(None, updated.auth_uuid);

  let config = db.get_mcp_oauth_config(&oauth_config.id).await?;
  assert!(
    config.is_some(),
    "OAuth config should be preserved on auth type change"
  );

  let token_row = db.get_mcp_oauth_token("user-1", &token.id).await?;
  assert!(
    token_row.is_none(),
    "OAuth token should be deleted on auth type change"
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_delete_cleans_up_oauth_config(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "client-id",
      Some("client-secret".to_string()),
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      None,
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "access-token",
      Some("refresh-token".to_string()),
      None,
      Some(3600),
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "Delete OAuth",
      "delete-oauth",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Oauth,
      Some(token.id.clone()),
    )
    .await?;

  service.delete("user-1", &mcp.id).await?;

  let config = db.get_mcp_oauth_config(&oauth_config.id).await?;
  assert!(
    config.is_some(),
    "OAuth config should be preserved when MCP is deleted"
  );

  let token_row = db.get_mcp_oauth_token("user-1", &token.id).await?;
  assert!(
    token_row.is_none(),
    "OAuth token should be deleted when MCP is deleted"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_discover_oauth_metadata_success() -> anyhow::Result<()> {
  let mut server = mockito::Server::new_async().await;
  let metadata = json!({
    "issuer": server.url(),
    "authorization_endpoint": format!("{}/authorize", server.url()),
    "token_endpoint": format!("{}/token", server.url()),
    "response_types_supported": ["code"],
    "grant_types_supported": ["authorization_code", "refresh_token"]
  });

  let mock = server
    .mock("GET", "/.well-known/oauth-authorization-server")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(metadata.to_string())
    .create_async()
    .await;

  let db = Arc::new(crate::test_utils::MockDbService::new());
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db, Arc::new(mock_client), default_time_service());

  let result = service.discover_oauth_metadata(&server.url()).await?;

  assert_eq!(server.url(), result["issuer"]);
  assert!(result["authorization_endpoint"].as_str().is_some());
  assert!(result["token_endpoint"].as_str().is_some());

  mock.assert_async().await;
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_discover_oauth_metadata_not_found() -> anyhow::Result<()> {
  let mut server = mockito::Server::new_async().await;

  let mock = server
    .mock("GET", "/.well-known/oauth-authorization-server")
    .with_status(404)
    .with_body("Not Found")
    .create_async()
    .await;

  let db = Arc::new(crate::test_utils::MockDbService::new());
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db, Arc::new(mock_client), default_time_service());

  let result = service.discover_oauth_metadata(&server.url()).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("mcp_error-o_auth_discovery_failed", err.code());

  mock.assert_async().await;
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_discover_mcp_oauth_metadata_success() -> anyhow::Result<()> {
  let mut mcp_server = mockito::Server::new_async().await;
  let mut as_server = mockito::Server::new_async().await;

  let prs_metadata = json!({
    "resource": mcp_server.url(),
    "authorization_servers": [as_server.url()]
  });
  let prs_mock = mcp_server
    .mock("GET", "/.well-known/oauth-protected-resource")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(prs_metadata.to_string())
    .create_async()
    .await;

  let as_metadata = json!({
    "issuer": as_server.url(),
    "authorization_endpoint": format!("{}/authorize", as_server.url()),
    "token_endpoint": format!("{}/token", as_server.url()),
    "registration_endpoint": format!("{}/register", as_server.url()),
    "scopes_supported": ["openid", "profile"]
  });
  let as_mock = as_server
    .mock("GET", "/.well-known/oauth-authorization-server")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(as_metadata.to_string())
    .create_async()
    .await;

  let db = Arc::new(crate::test_utils::MockDbService::new());
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db, Arc::new(mock_client), default_time_service());

  let result = service
    .discover_mcp_oauth_metadata(&mcp_server.url())
    .await?;

  assert_eq!(
    format!("{}/authorize", as_server.url()),
    result["authorization_endpoint"]
  );
  assert_eq!(
    format!("{}/token", as_server.url()),
    result["token_endpoint"]
  );
  assert_eq!(
    format!("{}/register", as_server.url()),
    result["registration_endpoint"]
  );
  assert_eq!(mcp_server.url(), result["resource"]);
  assert_eq!(as_server.url(), result["authorization_server_url"]);

  prs_mock.assert_async().await;
  as_mock.assert_async().await;
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_discover_mcp_oauth_metadata_prs_404() -> anyhow::Result<()> {
  let mut mcp_server = mockito::Server::new_async().await;

  let prs_mock = mcp_server
    .mock("GET", "/.well-known/oauth-protected-resource")
    .with_status(404)
    .with_body("Not Found")
    .create_async()
    .await;

  let db = Arc::new(crate::test_utils::MockDbService::new());
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db, Arc::new(mock_client), default_time_service());

  let result = service.discover_mcp_oauth_metadata(&mcp_server.url()).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("mcp_error-o_auth_discovery_failed", err.code());

  prs_mock.assert_async().await;
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_dynamic_register_client_success() -> anyhow::Result<()> {
  let mut server = mockito::Server::new_async().await;

  let reg_response = json!({
    "client_id": "dynamic-client-123",
    "client_secret": "dynamic-secret-456",
    "client_id_issued_at": 1700000000,
    "token_endpoint_auth_method": "none",
    "registration_access_token": "reg-access-token-789"
  });

  let reg_mock = server
    .mock("POST", "/register")
    .with_status(201)
    .with_header("content-type", "application/json")
    .with_body(reg_response.to_string())
    .create_async()
    .await;

  let db = Arc::new(crate::test_utils::MockDbService::new());
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db, Arc::new(mock_client), default_time_service());

  let result = service
    .dynamic_register_client(
      &format!("{}/register", server.url()),
      "http://localhost:3000/callback",
      Some("openid profile".to_string()),
    )
    .await?;

  assert_eq!("dynamic-client-123", result["client_id"]);
  assert_eq!("dynamic-secret-456", result["client_secret"]);
  assert_eq!(1700000000, result["client_id_issued_at"]);
  assert_eq!("none", result["token_endpoint_auth_method"]);
  assert_eq!("reg-access-token-789", result["registration_access_token"]);

  reg_mock.assert_async().await;
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_dynamic_register_client_failure() -> anyhow::Result<()> {
  let mut server = mockito::Server::new_async().await;

  let reg_mock = server
    .mock("POST", "/register")
    .with_status(400)
    .with_body(r#"{"error": "invalid_client_metadata"}"#)
    .create_async()
    .await;

  let db = Arc::new(crate::test_utils::MockDbService::new());
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db, Arc::new(mock_client), default_time_service());

  let result = service
    .dynamic_register_client(
      &format!("{}/register", server.url()),
      "http://localhost:3000/callback",
      None::<String>,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("mcp_error-o_auth_discovery_failed", err.code());

  reg_mock.assert_async().await;
  Ok(())
}

// ============================================================================
// C2: Token Refresh Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_oauth_token_expired_with_refresh_success(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut token_server = mockito::Server::new_async().await;

  // Create service to set up OAuth config and token
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  // Create OAuth config with token_endpoint pointing to mockito
  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "client-id",
      Some("client-secret".to_string()),
      "https://auth.example.com/authorize",
      &format!("{}/token", token_server.url()),
      None,
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  // Store token with short expiry (30s). FrozenTimeService is at 1735689600.
  // expires_at = 1735689600 + 30 = 1735689630
  // Expiry check: now(1735689600) >= expires_at(1735689630) - 60 = 1735689570 => TRUE (expired)
  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "old-access-token",
      Some("my-refresh-token".to_string()),
      None,
      Some(30),
    )
    .await?;

  // Create MCP instance using OAuth auth
  let mcp = service
    .create(
      "user-1",
      "Refresh MCP",
      "refresh-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Oauth,
      Some(token.id.clone()),
    )
    .await?;

  // Drop service so we can create a new one with a MockMcpClient that has expectations
  drop(service);

  // Set up mockito to return new tokens on refresh
  let token_mock = token_server
    .mock("POST", "/token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "access_token": "new-access-token-refreshed",
        "refresh_token": "new-refresh-token-refreshed",
        "token_type": "Bearer",
        "expires_in": 3600
      })
      .to_string(),
    )
    .create_async()
    .await;

  // MockMcpClient expects fetch_tools with the NEW refreshed access token
  let mut mock_client2 = MockMcpClient::new();
  mock_client2
    .expect_fetch_tools()
    .with(
      eq("https://mcp.example.com/mcp"),
      eq(Some((
        "Authorization".to_string(),
        "Bearer new-access-token-refreshed".to_string(),
      ))),
    )
    .returning(|_, _| {
      Ok(vec![McpTool {
        name: "refreshed-tool".to_string(),
        description: Some("Tool after refresh".to_string()),
        input_schema: None,
      }])
    });

  let service2 = DefaultMcpService::new(db.clone(), Arc::new(mock_client2), default_time_service());

  let tools = service2.fetch_tools("user-1", &mcp.id).await?;
  assert_eq!(1, tools.len());
  assert_eq!("refreshed-tool", tools[0].name);

  token_mock.assert_async().await;
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_oauth_token_expired_no_refresh_returns_error(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "client-id",
      Some("client-secret".to_string()),
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      None,
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  // Store token with short expiry and NO refresh token
  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "expired-access-token",
      None, // no refresh token
      None,
      Some(30), // triggers expiry
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "NoRefresh MCP",
      "norefresh-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Oauth,
      Some(token.id.clone()),
    )
    .await?;

  let result = service.fetch_tools("user-1", &mcp.id).await;
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("mcp_error-o_auth_token_expired", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_oauth_token_expired_refresh_http_failure(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mut token_server = mockito::Server::new_async().await;

  // Create service to set up OAuth config and token
  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let oauth_config = service
    .create_oauth_config(
      "user-1",
      "OAuth",
      "server-1",
      "client-id",
      Some("client-secret".to_string()),
      "https://auth.example.com/authorize",
      &format!("{}/token", token_server.url()),
      None,
      RegistrationType::default(),
      None,
      None,
      None,
      None,
    )
    .await?;

  // Store token with short expiry and a refresh token
  let token = service
    .store_oauth_token(
      "user-1",
      &oauth_config.id,
      "expired-access-token",
      Some("my-refresh-token".to_string()),
      None,
      Some(30), // triggers expiry
    )
    .await?;

  let mcp = service
    .create(
      "user-1",
      "FailRefresh MCP",
      "failrefresh-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      McpAuthType::Oauth,
      Some(token.id.clone()),
    )
    .await?;

  // Drop service and recreate (refresh fails before reaching mcp_client.fetch_tools)
  drop(service);

  // Set up mockito to return 401 on refresh attempt
  let token_mock = token_server
    .mock("POST", "/token")
    .with_status(401)
    .with_body(r#"{"error": "invalid_grant"}"#)
    .create_async()
    .await;

  let mock_client2 = MockMcpClient::new();
  let service2 = DefaultMcpService::new(db.clone(), Arc::new(mock_client2), default_time_service());

  let result = service2.fetch_tools("user-1", &mcp.id).await;
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("mcp_error-o_auth_refresh_failed", err.code());

  token_mock.assert_async().await;
  Ok(())
}
