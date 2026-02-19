use crate::db::{DbService, McpServerRow, TimeService};
use crate::mcp_service::{DefaultMcpService, McpService};
use crate::test_utils::{test_db_service, FrozenTimeService, TestDbService};
use anyhow_trace::anyhow_trace;
use mcp_client::MockMcpClient;
use mockall::predicate::eq;
use objs::{McpExecutionRequest, McpTool};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use std::sync::Arc;

async fn setup_server(db: &dyn DbService) -> McpServerRow {
  let now = db.now().timestamp();
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
      Some("Authorization".to_string()),
      Some("Bearer sk-secret-123".to_string()),
    )
    .await?;

  assert_eq!("header", mcp.auth_type);
  assert_eq!(Some("Authorization".to_string()), mcp.auth_header_key);
  assert_eq!(true, mcp.has_auth_header_value);

  let auth = db.get_mcp_auth_header(&mcp.id).await?;
  assert_eq!(
    Some((
      "Authorization".to_string(),
      "Bearer sk-secret-123".to_string()
    )),
    auth
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
      None,
      None,
    )
    .await?;

  assert_eq!("public", mcp.auth_type);
  assert_eq!(None, mcp.auth_header_key);
  assert_eq!(false, mcp.has_auth_header_value);

  let auth = db.get_mcp_auth_header(&mcp.id).await?;
  assert_eq!(None, auth);
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
      "user-1", "My MCP", "my-mcp", "server-1", None, true, None, None, None, None,
    )
    .await?;
  assert_eq!("public", mcp.auth_type);

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
      Some("X-Api-Key".to_string()),
      Some("key-abc-123".to_string()),
      false,
    )
    .await?;

  assert_eq!("header", updated.auth_type);
  assert_eq!(Some("X-Api-Key".to_string()), updated.auth_header_key);
  assert_eq!(true, updated.has_auth_header_value);

  let auth = db.get_mcp_auth_header(&updated.id).await?;
  assert_eq!(
    Some(("X-Api-Key".to_string(), "key-abc-123".to_string())),
    auth
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_mcp_service_update_switch_header_to_public(
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
      "Auth MCP",
      "auth-mcp",
      "server-1",
      None,
      true,
      None,
      None,
      Some("Authorization".to_string()),
      Some("Bearer token".to_string()),
    )
    .await?;
  assert_eq!("header", mcp.auth_type);

  let updated = service
    .update(
      "user-1", &mcp.id, "Auth MCP", "auth-mcp", None, true, None, None, None, None, false,
    )
    .await?;

  assert_eq!("public", updated.auth_type);
  assert_eq!(None, updated.auth_header_key);
  assert_eq!(false, updated.has_auth_header_value);

  let auth = db.get_mcp_auth_header(&updated.id).await?;
  assert_eq!(None, auth);
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
      Some("Authorization".to_string()),
      Some("Bearer original-token".to_string()),
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
      None,
      None,
      true, // auth_keep = true
    )
    .await?;

  assert_eq!("Renamed MCP", updated.name);
  assert_eq!("header", updated.auth_type);
  assert_eq!(Some("Authorization".to_string()), updated.auth_header_key);
  assert_eq!(true, updated.has_auth_header_value);

  let auth = db.get_mcp_auth_header(&updated.id).await?;
  assert_eq!(
    Some((
      "Authorization".to_string(),
      "Bearer original-token".to_string()
    )),
    auth
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
      Some("Authorization".to_string()),
      Some("Bearer sk-secret".to_string()),
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
      Some("Authorization".to_string()),
      Some("Bearer exec-token".to_string()),
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
