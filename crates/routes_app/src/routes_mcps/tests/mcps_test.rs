use crate::{
  create_mcp_handler, create_mcp_server_handler, delete_mcp_handler, execute_mcp_tool_handler,
  fetch_mcp_tools_handler, get_mcp_handler, get_mcp_server_handler, list_mcp_servers_handler,
  list_mcp_tools_handler, list_mcps_handler, refresh_mcp_tools_handler, update_mcp_handler,
  update_mcp_server_handler, CreateMcpRequest, CreateMcpServerRequest, FetchMcpToolsRequest,
  ListMcpServersResponse, ListMcpsResponse, McpAuth, McpExecuteRequest, McpExecuteResponse,
  McpResponse, McpServerResponse, McpToolsResponse, UpdateMcpRequest, UpdateMcpServerRequest,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::Router;
use chrono::Utc;
use objs::{Mcp, McpServer, McpServerInfo};
use objs::{McpExecutionResponse, McpTool, ResourceRole};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{test_utils::AppServiceStubBuilder, McpServerError, MockMcpService};
use std::sync::Arc;
use tower::ServiceExt;

fn test_mcp_server() -> McpServer {
  let now = Utc::now();
  McpServer {
    id: "server-uuid-1".to_string(),
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "DeepWiki".to_string(),
    description: Some("DeepWiki MCP server".to_string()),
    enabled: true,
    created_by: "admin-user".to_string(),
    updated_by: "admin-user".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn test_server_info() -> McpServerInfo {
  McpServerInfo {
    id: "server-uuid-1".to_string(),
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "DeepWiki".to_string(),
    enabled: true,
  }
}

#[fixture]
fn test_mcp_instance() -> Mcp {
  let now = Utc::now();
  Mcp {
    id: "mcp-uuid-1".to_string(),
    mcp_server: test_server_info(),
    slug: "deepwiki".to_string(),
    name: "DeepWiki MCP".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "public".to_string(),
    auth_header_key: None,
    has_auth_header_value: false,
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
      .route("/mcps/fetch-tools", post(fetch_mcp_tools_handler))
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
      .route("/mcp_servers", post(create_mcp_server_handler))
      .route("/mcp_servers/{id}", get(get_mcp_server_handler))
      .route("/mcp_servers/{id}", put(update_mcp_server_handler))
      .with_state(state),
  )
}

// ============================================================================
// POST /mcp_servers - Create MCP server
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_server_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_create_mcp_server()
    .withf(|name, url, _, enabled, created_by| {
      name == "DeepWiki"
        && url == "https://mcp.deepwiki.com/mcp"
        && *enabled
        && created_by == "admin-user"
    })
    .times(1)
    .returning(move |_, _, _, _, _| Ok(server.clone()));

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((0, 0)));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "DeepWiki".to_string(),
    description: Some("DeepWiki MCP server".to_string()),
    enabled: true,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcp_servers")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "admin-user",
    "admin",
    ResourceRole::Admin,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let body: McpServerResponse = response.json().await?;
  assert_eq!("server-uuid-1", body.id);
  assert_eq!("DeepWiki", body.name);
  assert_eq!("https://mcp.deepwiki.com/mcp", body.url);
  assert_eq!(0, body.enabled_mcp_count);
  assert_eq!(0, body.disabled_mcp_count);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_server_duplicate_url_returns_409() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_create_mcp_server()
    .times(1)
    .returning(|_, _, _, _, _| {
      Err(McpServerError::UrlAlreadyExists(
        "https://mcp.deepwiki.com/mcp".to_string(),
      ))
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "DeepWiki".to_string(),
    description: None,
    enabled: true,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcp_servers")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "admin-user",
    "admin",
    ResourceRole::Admin,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CONFLICT, response.status());
  Ok(())
}

// ============================================================================
// PUT /mcp_servers/{id} - Update MCP server
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_server_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut server = test_mcp_server();
  server.name = "Updated DeepWiki".to_string();

  mock
    .expect_update_mcp_server()
    .withf(|id, name, _, _, _, _| id == "server-uuid-1" && name == "Updated DeepWiki")
    .times(1)
    .returning(move |_, _, _, _, _, _| Ok(server.clone()));

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((2, 1)));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "Updated DeepWiki".to_string(),
    description: Some("Updated description".to_string()),
    enabled: true,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcp_servers/server-uuid-1")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "admin-user",
    "admin",
    ResourceRole::Admin,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpServerResponse = response.json().await?;
  assert_eq!("Updated DeepWiki", body.name);
  assert_eq!(2, body.enabled_mcp_count);
  assert_eq!(1, body.disabled_mcp_count);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_server_not_found_returns_404() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_update_mcp_server()
    .times(1)
    .returning(|_, _, _, _, _, _| {
      Err(McpServerError::McpServerNotFound("nonexistent".to_string()))
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "Test".to_string(),
    description: None,
    enabled: true,
  })?;

  let request = Request::builder()
    .method("PUT")
    .uri("/mcp_servers/nonexistent")
    .header("content-type", "application/json")
    .body(Body::from(body))?;

  let request = request.with_auth_context(AuthContext::test_session(
    "admin-user",
    "admin",
    ResourceRole::Admin,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

// ============================================================================
// GET /mcp_servers - List MCP servers
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcp_servers_all() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_list_mcp_servers()
    .withf(|enabled| enabled.is_none())
    .times(1)
    .returning(move |_| Ok(vec![server.clone()]));

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((3, 1)));

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
  assert_eq!(3, body.mcp_servers[0].enabled_mcp_count);
  assert_eq!(1, body.mcp_servers[0].disabled_mcp_count);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_mcp_servers_filtered_by_enabled() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_list_mcp_servers()
    .withf(|enabled| *enabled == Some(true))
    .times(1)
    .returning(move |_| Ok(vec![server.clone()]));

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((0, 0)));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcp_servers?enabled=true")
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

// ============================================================================
// GET /mcp_servers/{id} - Get MCP server
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_mcp_server_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_get_mcp_server()
    .withf(|id| id == "server-uuid-1")
    .times(1)
    .returning(move |_| Ok(Some(server.clone())));

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((2, 0)));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcp_servers/server-uuid-1")
    .body(Body::empty())?;

  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpServerResponse = response.json().await?;
  assert_eq!("server-uuid-1", body.id);
  assert_eq!("DeepWiki", body.name);
  assert_eq!(2, body.enabled_mcp_count);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_mcp_server_not_found() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_get_mcp_server()
    .times(1)
    .returning(|_| Ok(None));

  let app = test_router_for_crud(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcp_servers/nonexistent")
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
    .withf(|user_id, name, slug, mcp_server_id, _, _, _, _, _, _| {
      user_id == "user123"
        && name == "DeepWiki MCP"
        && slug == "deepwiki"
        && mcp_server_id == "server-uuid-1"
    })
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth: McpAuth::Public,
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
  assert_eq!("https://mcp.deepwiki.com/mcp", body.mcp_server.url);
  assert_eq!("server-uuid-1", body.mcp_server.id);
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
    mcp_server_id: "server-uuid-1".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth: McpAuth::Public,
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

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_empty_mcp_server_id_returns_400() -> anyhow::Result<()> {
  let mock = MockMcpService::new();
  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "Test".to_string(),
    slug: "test".to_string(),
    mcp_server_id: "".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth: McpAuth::Public,
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
  assert_eq!("https://mcp.deepwiki.com/mcp", body.mcps[0].mcp_server.url);
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
  assert_eq!("server-uuid-1", body.mcp_server.id);
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
    .withf(|user_id, id, name, slug, _, _, _, _, _, _, _| {
      user_id == "user123" && id == "mcp-uuid-1" && name == "Updated Name" && slug == "deepwiki"
    })
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpRequest {
    name: "Updated Name".to_string(),
    slug: "deepwiki".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    tools_filter: None,
    tools_cache: None,
    auth: None,
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
// POST /mcps/fetch-tools - Fetch tools from MCP server (stateless)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_mcp_tools_success() -> anyhow::Result<()> {
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
    .expect_fetch_tools_for_server()
    .withf(|server_id, _, _| server_id == "server-uuid-1")
    .times(1)
    .returning(move |_, _, _| Ok(tools.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "server-uuid-1".to_string(),
    auth: McpAuth::Public,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/fetch-tools")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());

  let body: McpToolsResponse = response.json().await?;
  assert_eq!(2, body.tools.len());
  assert_eq!("read_wiki_structure", body.tools[0].name);
  assert_eq!("ask_question", body.tools[1].name);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_mcp_tools_empty_server_id_returns_400() -> anyhow::Result<()> {
  let mock = MockMcpService::new();
  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "".to_string(),
    auth: McpAuth::Public,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/fetch-tools")
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

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_mcp_tools_server_not_found_returns_404() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_fetch_tools_for_server()
    .times(1)
    .returning(|_, _, _| {
      Err(services::McpError::McpServerNotFound(
        "nonexistent".to_string(),
      ))
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "nonexistent".to_string(),
    auth: McpAuth::Public,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/fetch-tools")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_mcp_tools_server_disabled_returns_400() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_fetch_tools_for_server()
    .times(1)
    .returning(|_, _, _| Err(services::McpError::McpDisabled));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "server-uuid-1".to_string(),
    auth: McpAuth::Public,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/fetch-tools")
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

// ============================================================================
// Auth Header Tests - Create with header auth
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_header_auth() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut instance = test_mcp_instance();
  instance.auth_type = "header".to_string();
  instance.auth_header_key = Some("Authorization".to_string());
  instance.has_auth_header_value = true;
  let ret = instance.clone();

  mock
    .expect_create()
    .withf(
      |_user_id, _name, _slug, _server_id, _desc, _enabled, _tc, _tf, auth_key, auth_val| {
        auth_key.as_deref() == Some("Authorization")
          && auth_val.as_deref() == Some("Bearer secret123")
      },
    )
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _| Ok(ret.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth: McpAuth::Header {
      header_key: "Authorization".to_string(),
      header_value: "Bearer secret123".to_string(),
    },
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
  assert_eq!("header", body.auth_type);
  assert_eq!(Some("Authorization".to_string()), body.auth_header_key);
  assert!(body.has_auth_header_value);
  Ok(())
}

// ============================================================================
// Auth Header Tests - Update to header auth
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_set_header_auth() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_instance();
  updated.auth_type = "header".to_string();
  updated.auth_header_key = Some("X-API-Key".to_string());
  updated.has_auth_header_value = true;
  let ret = updated.clone();

  mock
    .expect_update()
    .withf(
      |_user_id, _id, _name, _slug, _desc, _enabled, _tf, _tc, auth_key, auth_val, auth_keep| {
        auth_key.as_deref() == Some("X-API-Key")
          && auth_val.as_deref() == Some("my-api-key")
          && !auth_keep
      },
    )
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _, _| Ok(ret.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    description: None,
    enabled: true,
    tools_filter: None,
    tools_cache: None,
    auth: Some(McpAuth::Header {
      header_key: "X-API-Key".to_string(),
      header_value: "my-api-key".to_string(),
    }),
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
  assert_eq!("header", body.auth_type);
  assert_eq!(Some("X-API-Key".to_string()), body.auth_header_key);
  assert!(body.has_auth_header_value);
  Ok(())
}

// ============================================================================
// Auth Header Tests - Update to public (clear auth)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_switch_to_public() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_instance();
  updated.auth_type = "public".to_string();
  updated.auth_header_key = None;
  updated.has_auth_header_value = false;
  let ret = updated.clone();

  mock
    .expect_update()
    .withf(
      |_user_id, _id, _name, _slug, _desc, _enabled, _tf, _tc, auth_key, auth_val, auth_keep| {
        auth_key.is_none() && auth_val.is_none() && !auth_keep
      },
    )
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _, _| Ok(ret.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    description: None,
    enabled: true,
    tools_filter: None,
    tools_cache: None,
    auth: Some(McpAuth::Public),
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
  assert_eq!("public", body.auth_type);
  assert_eq!(None, body.auth_header_key);
  assert!(!body.has_auth_header_value);
  Ok(())
}

// ============================================================================
// Auth Header Tests - Update with auth_keep (no auth in request body)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_keep_existing_auth() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut updated = test_mcp_instance();
  updated.auth_type = "header".to_string();
  updated.auth_header_key = Some("Authorization".to_string());
  updated.has_auth_header_value = true;
  let ret = updated.clone();

  mock
    .expect_update()
    .withf(
      |_user_id, _id, _name, _slug, _desc, _enabled, _tf, _tc, auth_key, auth_val, auth_keep| {
        auth_key.is_none() && auth_val.is_none() && *auth_keep
      },
    )
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _, _| Ok(ret.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpRequest {
    name: "Updated Name".to_string(),
    slug: "deepwiki".to_string(),
    description: None,
    enabled: true,
    tools_filter: None,
    tools_cache: None,
    auth: None,
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
  assert_eq!("header", body.auth_type);
  assert!(body.has_auth_header_value);
  Ok(())
}

// ============================================================================
// Auth Header Tests - Fetch tools with header auth
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_tools_with_header_auth() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_fetch_tools_for_server()
    .withf(|server_id, auth_key, auth_val| {
      server_id == "server-uuid-1"
        && auth_key.as_deref() == Some("Authorization")
        && auth_val.as_deref() == Some("Bearer test-key")
    })
    .times(1)
    .returning(|_, _, _| {
      Ok(vec![McpTool {
        name: "tavily_search".to_string(),
        description: Some("Search the web".to_string()),
        input_schema: None,
      }])
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "server-uuid-1".to_string(),
    auth: McpAuth::Header {
      header_key: "Authorization".to_string(),
      header_value: "Bearer test-key".to_string(),
    },
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/fetch-tools")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: McpToolsResponse = response.json().await?;
  assert_eq!(1, body.tools.len());
  assert_eq!("tavily_search", body.tools[0].name);
  Ok(())
}

// ============================================================================
// Auth Header Tests - Create with public auth response fields
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_response_includes_auth_fields() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let instance = test_mcp_instance();
  let ret = instance.clone();

  mock
    .expect_create()
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _| Ok(ret.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "Test".to_string(),
    slug: "test".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth: McpAuth::Public,
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
  assert_eq!("public", body.auth_type);
  assert_eq!(None, body.auth_header_key);
  assert!(!body.has_auth_header_value);
  Ok(())
}

// ============================================================================
// Real-DB integration tests for MCP auth (via build_test_router)
// ============================================================================

use crate::test_utils::{
  build_test_router, create_authenticated_session, session_request_with_body,
};
use serde_json::{json, Value};

async fn setup_mcp_server_in_db(router: &Router, admin_cookie: &str) -> anyhow::Result<String> {
  let body = json!({
    "name": "Test MCP Server",
    "url": "https://mcp.example.com/mcp",
    "description": "Integration test server",
    "enabled": true
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcp_servers",
      admin_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let server: Value = response.json().await?;
  Ok(server["id"].as_str().unwrap().to_string())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_create_mcp_with_header_auth() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let body = json!({
    "name": "Tavily Auth",
    "slug": "tavily-auth",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "header", "header_key": "Authorization", "header_value": "Bearer sk-test-secret" }
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let mcp: Value = response.json().await?;
  assert_eq!("header", mcp["auth_type"]);
  assert_eq!("Authorization", mcp["auth_header_key"]);
  assert_eq!(true, mcp["has_auth_header_value"]);
  assert!(mcp.get("auth_header_value").is_none());
  assert!(mcp.get("encrypted_auth_header_value").is_none());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_create_mcp_with_public_auth() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let body = json!({
    "name": "Public MCP",
    "slug": "public-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "public" }
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let mcp: Value = response.json().await?;
  assert_eq!("public", mcp["auth_type"]);
  assert_eq!(Value::Null, mcp["auth_header_key"]);
  assert_eq!(false, mcp["has_auth_header_value"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_update_mcp_switch_public_to_header() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let create_body = json!({
    "name": "My MCP",
    "slug": "my-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "public" }
  });
  let create_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&create_body)?),
    ))
    .await?;
  let created: Value = create_resp.json().await?;
  let mcp_id = created["id"].as_str().unwrap();

  let update_body = json!({
    "name": "My MCP",
    "slug": "my-mcp",
    "enabled": true,
    "auth": { "type": "header", "header_key": "X-Api-Key", "header_value": "key-123" }
  });
  let update_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "PUT",
      &format!("/bodhi/v1/mcps/{}", mcp_id),
      &user_cookie,
      Body::from(serde_json::to_string(&update_body)?),
    ))
    .await?;
  assert_eq!(StatusCode::OK, update_resp.status());

  let updated: Value = update_resp.json().await?;
  assert_eq!("header", updated["auth_type"]);
  assert_eq!("X-Api-Key", updated["auth_header_key"]);
  assert_eq!(true, updated["has_auth_header_value"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_update_mcp_switch_header_to_public() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let create_body = json!({
    "name": "Auth MCP",
    "slug": "auth-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "header", "header_key": "Authorization", "header_value": "Bearer secret" }
  });
  let create_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&create_body)?),
    ))
    .await?;
  let created: Value = create_resp.json().await?;
  let mcp_id = created["id"].as_str().unwrap();
  assert_eq!("header", created["auth_type"]);

  let update_body = json!({
    "name": "Auth MCP",
    "slug": "auth-mcp",
    "enabled": true,
    "auth": { "type": "public" }
  });
  let update_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "PUT",
      &format!("/bodhi/v1/mcps/{}", mcp_id),
      &user_cookie,
      Body::from(serde_json::to_string(&update_body)?),
    ))
    .await?;
  assert_eq!(StatusCode::OK, update_resp.status());

  let updated: Value = update_resp.json().await?;
  assert_eq!("public", updated["auth_type"]);
  assert_eq!(Value::Null, updated["auth_header_key"]);
  assert_eq!(false, updated["has_auth_header_value"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_update_mcp_keep_existing_auth() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let create_body = json!({
    "name": "Keep Auth MCP",
    "slug": "keep-auth-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "header", "header_key": "Authorization", "header_value": "Bearer keep-me" }
  });
  let create_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&create_body)?),
    ))
    .await?;
  let created: Value = create_resp.json().await?;
  let mcp_id = created["id"].as_str().unwrap();

  // Update name without sending auth (omitted = keep existing)
  let update_body = json!({
    "name": "Renamed MCP",
    "slug": "keep-auth-mcp",
    "enabled": true
  });
  let update_resp = router
    .clone()
    .oneshot(session_request_with_body(
      "PUT",
      &format!("/bodhi/v1/mcps/{}", mcp_id),
      &user_cookie,
      Body::from(serde_json::to_string(&update_body)?),
    ))
    .await?;
  assert_eq!(StatusCode::OK, update_resp.status());

  let updated: Value = update_resp.json().await?;
  assert_eq!("Renamed MCP", updated["name"]);
  assert_eq!("header", updated["auth_type"]);
  assert_eq!("Authorization", updated["auth_header_key"]);
  assert_eq!(true, updated["has_auth_header_value"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_list_mcps_shows_auth_info() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;

  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let public_body = json!({
    "name": "Public One",
    "slug": "public-one",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "public" }
  });
  router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&public_body)?),
    ))
    .await?;

  let header_body = json!({
    "name": "Header One",
    "slug": "header-one",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth": { "type": "header", "header_key": "X-Api-Key", "header_value": "secret-val" }
  });
  router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps",
      &user_cookie,
      Body::from(serde_json::to_string(&header_body)?),
    ))
    .await?;

  use crate::test_utils::session_request;
  let list_resp = router
    .clone()
    .oneshot(session_request("GET", "/bodhi/v1/mcps", &user_cookie))
    .await?;
  assert_eq!(StatusCode::OK, list_resp.status());

  let list: Value = list_resp.json().await?;
  let mcps = list["mcps"].as_array().unwrap();
  assert_eq!(2, mcps.len());

  let public_mcp = mcps.iter().find(|m| m["slug"] == "public-one").unwrap();
  assert_eq!("public", public_mcp["auth_type"]);
  assert_eq!(false, public_mcp["has_auth_header_value"]);

  let header_mcp = mcps.iter().find(|m| m["slug"] == "header-one").unwrap();
  assert_eq!("header", header_mcp["auth_type"]);
  assert_eq!("X-Api-Key", header_mcp["auth_header_key"]);
  assert_eq!(true, header_mcp["has_auth_header_value"]);
  Ok(())
}
