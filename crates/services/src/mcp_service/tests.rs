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

  let auth_header = service
    .create_auth_header("user-1", "Authorization", "Bearer sk-secret-123")
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
      "header",
      Some(auth_header.id.clone()),
    )
    .await?;

  assert_eq!("header", mcp.auth_type);
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
      "public",
      None,
    )
    .await?;

  assert_eq!("public", mcp.auth_type);
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
      "user-1", "My MCP", "my-mcp", "server-1", None, true, None, None, "public", None,
    )
    .await?;
  assert_eq!("public", mcp.auth_type);

  let auth_header = service
    .create_auth_header("user-1", "X-Api-Key", "key-abc-123")
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
      Some("header".to_string()),
      Some(auth_header.id.clone()),
    )
    .await?;

  assert_eq!("header", updated.auth_type);
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
async fn test_mcp_service_update_switch_header_to_public_orphan_cleanup(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header("user-1", "Authorization", "Bearer token")
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
      "header",
      Some(auth_id.clone()),
    )
    .await?;
  assert_eq!("header", mcp.auth_type);

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
      Some("public".to_string()),
      None,
    )
    .await?;

  assert_eq!("public", updated.auth_type);
  assert_eq!(None, updated.auth_uuid);

  let orphaned = db.get_mcp_auth_header(&auth_id).await?;
  assert!(orphaned.is_none(), "orphan auth header should be deleted");
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
    .create_auth_header("user-1", "Authorization", "Bearer original-token")
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
      "header",
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
  assert_eq!("header", updated.auth_type);
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
async fn test_mcp_service_delete_orphan_cleanup(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db: Arc<dyn DbService> = Arc::new(db);
  setup_server(db.as_ref()).await;

  let mock_client = MockMcpClient::new();
  let service = DefaultMcpService::new(db.clone(), Arc::new(mock_client), default_time_service());

  let auth_header = service
    .create_auth_header("user-1", "Authorization", "Bearer token")
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
      "header",
      Some(auth_id.clone()),
    )
    .await?;

  service.delete("user-1", &mcp.id).await?;

  let orphaned = db.get_mcp_auth_header(&auth_id).await?;
  assert!(
    orphaned.is_none(),
    "auth header should be deleted on MCP delete"
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
    .create_auth_header("user-1", "Authorization", "Bearer sk-secret")
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
      "header",
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
    .create_auth_header("user-1", "Authorization", "Bearer exec-token")
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
      "header",
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
