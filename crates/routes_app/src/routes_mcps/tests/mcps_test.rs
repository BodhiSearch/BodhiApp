use crate::{
  create_mcp_handler, delete_mcp_handler, enable_mcp_server_handler, execute_mcp_tool_handler,
  get_mcp_handler, list_mcp_servers_handler, list_mcp_tools_handler, list_mcps_handler,
  refresh_mcp_tools_handler, update_mcp_handler, CreateMcpRequest, EnableMcpServerRequest,
  ListMcpServersResponse, ListMcpsResponse, McpExecuteRequest, McpExecuteResponse, McpResponse,
  McpToolsResponse, UpdateMcpRequest,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::Router;
use chrono::Utc;
use objs::{Mcp, McpServer};
use objs::{McpExecutionResponse, McpTool, ResourceRole};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{test_utils::AppServiceStubBuilder, MockMcpService};
use std::sync::Arc;
use tower::ServiceExt;

fn test_mcp_server() -> McpServer {
  let now = Utc::now();
  McpServer {
    id: "server-uuid-1".to_string(),
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    enabled: true,
    updated_by: "admin-user".to_string(),
    created_at: now,
    updated_at: now,
  }
}

#[fixture]
fn test_mcp_instance() -> Mcp {
  let now = Utc::now();
  Mcp {
    id: "mcp-uuid-1".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    slug: "deepwiki".to_string(),
    name: "DeepWiki MCP".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    created_at: now,
    updated_at: now,
  }
}

async fn test_router_for_crud(mock_mcp_service: MockMcpService) -> anyhow::Result<Router> {
  let mcp_svc: Arc<dyn services::McpService> = Arc::new(mock_mcp_service);
  let app_service = AppServiceStubBuilder::default()
    .mcp_service(mcp_svc)
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));

  Ok(
    Router::new()
      .route("/mcps", get(list_mcps_handler))
      .route("/mcps", post(create_mcp_handler))
      .route("/mcps/{id}", get(get_mcp_handler))
      .route("/mcps/{id}", put(update_mcp_handler))
      .route("/mcps/{id}", delete(delete_mcp_handler))
      .route("/mcps/{id}/tools", get(list_mcp_tools_handler))
      .route("/mcps/{id}/tools/refresh", post(refresh_mcp_tools_handler))
      .route(
        "/mcps/{id}/tools/{tool_name}/execute",
        post(execute_mcp_tool_handler),
      )
      .route("/mcp_servers", get(list_mcp_servers_handler))
      .route("/mcp_servers", put(enable_mcp_server_handler))
      .with_state(state),
  )
}

// ============================================================================
// POST /mcps - Create MCP instance
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_success(test_mcp_instance: Mcp) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let instance = test_mcp_instance.clone();

  mock
    .expect_create()
    .withf(|user_id, name, slug, url, _, _| {
      user_id == "user123"
        && name == "DeepWiki MCP"
        && slug == "deepwiki"
        && url == "https://mcp.deepwiki.com/mcp"
    })
    .times(1)
    .returning(move |_, _, _, _, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let body: McpResponse = response.json().await?;
  assert_eq!("mcp-uuid-1", body.id);
  assert_eq!("deepwiki", body.slug);
  assert_eq!("DeepWiki MCP", body.name);
  assert_eq!("https://mcp.deepwiki.com/mcp", body.url);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_empty_name_returns_400() -> anyhow::Result<()> {
  let mock = MockMcpService::new();
  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "".to_string(),
    slug: "deepwiki".to_string(),
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    description: None,
    enabled: true,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  Ok(())
}

// ============================================================================
// GET /mcps - List MCP instances
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcps_success(test_mcp_instance: Mcp) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let instance = test_mcp_instance.clone();

  mock
    .expect_list()
    .withf(|user_id| user_id == "user123")
    .times(1)
    .returning(move |_| Ok(vec![instance.clone()]));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcps")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: ListMcpsResponse = response.json().await?;
  assert_eq!(1, body.mcps.len());
  assert_eq!("mcp-uuid-1", body.mcps[0].id);
  Ok(())
}

// ============================================================================
// GET /mcps/{id} - Get MCP instance
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_mcp_success(test_mcp_instance: Mcp) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let instance = test_mcp_instance.clone();

  mock
    .expect_get()
    .withf(|user_id, id| user_id == "user123" && id == "mcp-uuid-1")
    .times(1)
    .returning(move |_, _| Ok(Some(instance.clone())));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcps/mcp-uuid-1")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpResponse = response.json().await?;
  assert_eq!("mcp-uuid-1", body.id);
  assert_eq!("deepwiki", body.slug);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_mcp_not_found() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_get()
    .withf(|user_id, id| user_id == "user123" && id == "nonexistent")
    .times(1)
    .returning(|_, _| Ok(None));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcps/nonexistent")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

// ============================================================================
// PUT /mcps/{id} - Update MCP instance
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_success(test_mcp_instance: Mcp) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_instance.clone();
  updated.name = "Updated Name".to_string();

  mock
    .expect_update()
    .withf(|user_id, id, name, slug, _, _, _| {
      user_id == "user123" && id == "mcp-uuid-1" && name == "Updated Name" && slug == "deepwiki"
    })
    .times(1)
    .returning(move |_, _, _, _, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpRequest {
    name: "Updated Name".to_string(),
    slug: "deepwiki".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    tools_filter: None,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcps/mcp-uuid-1")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpResponse = response.json().await?;
  assert_eq!("Updated Name", body.name);
  Ok(())
}

// ============================================================================
// DELETE /mcps/{id} - Delete MCP instance
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_mcp_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_delete()
    .withf(|user_id, id| user_id == "user123" && id == "mcp-uuid-1")
    .times(1)
    .returning(|_, _| Ok(()));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("DELETE")
    .uri("/mcps/mcp-uuid-1")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NO_CONTENT, response.status());
  Ok(())
}

// ============================================================================
// GET /mcp_servers - List and query MCP servers
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcp_servers_all() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_list_mcp_servers()
    .times(1)
    .returning(move || Ok(vec![server.clone()]));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcp_servers")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: ListMcpServersResponse = response.json().await?;
  assert_eq!(1, body.mcp_servers.len());
  assert_eq!("https://mcp.deepwiki.com/mcp", body.mcp_servers[0].url);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcp_servers_by_url_found() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_get_mcp_server_by_url()
    .withf(|url| url == "https://mcp.deepwiki.com/mcp")
    .times(1)
    .returning(move |_| Ok(Some(server.clone())));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcp_servers?url=https%3A%2F%2Fmcp.deepwiki.com%2Fmcp")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: ListMcpServersResponse = response.json().await?;
  assert_eq!(1, body.mcp_servers.len());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcp_servers_by_url_not_found() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_get_mcp_server_by_url()
    .withf(|url| url == "https://unknown.server.com/mcp")
    .times(1)
    .returning(|_| Ok(None));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcp_servers?url=https%3A%2F%2Funknown.server.com%2Fmcp")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: ListMcpServersResponse = response.json().await?;
  assert_eq!(0, body.mcp_servers.len());
  Ok(())
}

// ============================================================================
// PUT /mcp_servers - Enable MCP server
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_enable_mcp_server_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_set_mcp_server_enabled()
    .withf(|url, enabled, updated_by| {
      url == "https://mcp.deepwiki.com/mcp" && *enabled && updated_by == "admin-user"
    })
    .times(1)
    .returning(move |_, _, _| Ok(server.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&EnableMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    enabled: true,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcp_servers")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "admin-user",
    "admin",
    ResourceRole::Admin,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// GET /mcps/{id}/tools - List cached tools
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcp_tools_success(test_mcp_instance: Mcp) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  let tools = vec![McpTool {
    name: "read_wiki_structure".to_string(),
    description: Some("Read wiki structure".to_string()),
    input_schema: None,
  }];
  let mut instance = test_mcp_instance;
  instance.tools_cache = Some(tools);

  mock
    .expect_get()
    .times(1)
    .returning(move |_, _| Ok(Some(instance.clone())));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcps/mcp-uuid-1/tools")
    .body(Body::empty())?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpToolsResponse = response.json().await?;
  assert_eq!(1, body.tools.len());
  assert_eq!("read_wiki_structure", body.tools[0].name);
  Ok(())
}

// ============================================================================
// POST /mcps/{id}/tools/refresh - Refresh tools from MCP server
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_mcp_tools_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  let tools = vec![
    McpTool {
      name: "read_wiki_structure".to_string(),
      description: Some("Read wiki structure".to_string()),
      input_schema: None,
    },
    McpTool {
      name: "ask_question".to_string(),
      description: Some("Ask a question".to_string()),
      input_schema: None,
    },
  ];

  mock
    .expect_fetch_tools()
    .withf(|user_id, id| user_id == "user123" && id == "mcp-uuid-1")
    .times(1)
    .returning(move |_, _| Ok(tools.clone()));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/mcp-uuid-1/tools/refresh")
    .body(Body::empty())?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpToolsResponse = response.json().await?;
  assert_eq!(2, body.tools.len());
  Ok(())
}

// ============================================================================
// POST /mcps/{id}/tools/{tool_name}/execute - Execute tool
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_execute_mcp_tool_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_execute()
    .withf(|user_id, id, tool_name, _request| {
      user_id == "user123" && id == "mcp-uuid-1" && tool_name == "read_wiki_structure"
    })
    .times(1)
    .returning(|_, _, _, _| {
      Ok(McpExecutionResponse {
        result: Some(serde_json::json!({"content": "wiki data"})),
        error: None,
      })
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&McpExecuteRequest {
    params: serde_json::json!({"repo_name": "BodhiSearch/BodhiApp"}),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/mcp-uuid-1/tools/read_wiki_structure/execute")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpExecuteResponse = response.json().await?;
  assert!(body.result.is_some());
  assert!(body.error.is_none());
  assert_eq!("wiki data", body.result.unwrap()["content"]);
  Ok(())
}
