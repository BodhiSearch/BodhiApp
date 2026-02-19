use crate::{
  create_auth_header_handler, create_mcp_handler, create_mcp_server_handler,
  delete_auth_header_handler, delete_mcp_handler, execute_mcp_tool_handler,
  fetch_mcp_tools_handler, get_auth_header_handler, get_mcp_handler, get_mcp_server_handler,
  list_mcp_servers_handler, list_mcp_tools_handler, list_mcps_handler, refresh_mcp_tools_handler,
  update_auth_header_handler, update_mcp_handler, update_mcp_server_handler, AuthHeaderResponse,
  CreateAuthHeaderRequest, CreateMcpRequest, CreateMcpServerRequest, FetchMcpToolsRequest, McpAuth,
  McpExecuteRequest, McpExecuteResponse, McpResponse, McpServerResponse, McpToolsResponse,
  UpdateMcpRequest,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::Router;
use chrono::Utc;
use objs::{Mcp, McpAuthHeader, McpServer, McpServerInfo};
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
    auth_uuid: None,
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
      .route("/mcps/auth-headers", post(create_auth_header_handler))
      .route("/mcps/auth-headers/{id}", get(get_auth_header_handler))
      .route("/mcps/auth-headers/{id}", put(update_auth_header_handler))
      .route(
        "/mcps/auth-headers/{id}",
        delete(delete_auth_header_handler),
      )
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
    .withf(
      |user_id, name, slug, mcp_server_id, _, _, _, _, auth_type, _| {
        user_id == "user123"
          && name == "DeepWiki MCP"
          && slug == "deepwiki"
          && mcp_server_id == "server-uuid-1"
          && auth_type == "public"
      },
    )
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
    auth_type: "public".to_string(),
    auth_uuid: None,
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
  assert_eq!("public", body.auth_type);
  assert_eq!(None, body.auth_uuid);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_auth_uuid(test_mcp_instance: Mcp) -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let mut instance = test_mcp_instance.clone();
  instance.auth_type = "header".to_string();
  instance.auth_uuid = Some("auth-uuid-1".to_string());

  mock
    .expect_create()
    .withf(|_, _, _, _, _, _, _, _, auth_type, auth_uuid| {
      auth_type == "header" && auth_uuid.as_deref() == Some("auth-uuid-1")
    })
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _| Ok(instance.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateMcpRequest {
    name: "DeepWiki MCP".to_string(),
    slug: "deepwiki".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: "header".to_string(),
    auth_uuid: Some("auth-uuid-1".to_string()),
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
  assert_eq!(Some("auth-uuid-1".to_string()), body.auth_uuid);
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
    .withf(|user_id, id, name, slug, _, _, _, _, _, _| {
      user_id == "user123" && id == "mcp-uuid-1" && name == "Updated Name" && slug == "deepwiki"
    })
    .times(1)
    .returning(move |_, _, _, _, _, _, _, _, _, _| Ok(updated.clone()));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&UpdateMcpRequest {
    name: "Updated Name".to_string(),
    slug: "deepwiki".to_string(),
    description: Some("Deep wiki search".to_string()),
    enabled: true,
    tools_filter: None,
    tools_cache: None,
    auth_type: None,
    auth_uuid: None,
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
// DELETE /mcps/{id}
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
// POST /mcps/fetch-tools
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_mcp_tools_with_inline_auth() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_fetch_tools_for_server()
    .withf(|server_id, auth_key, auth_val, _| {
      server_id == "server-uuid-1"
        && auth_key.as_deref() == Some("Authorization")
        && auth_val.as_deref() == Some("Bearer test-key")
    })
    .times(1)
    .returning(|_, _, _, _| {
      Ok(vec![McpTool {
        name: "search".to_string(),
        description: None,
        input_schema: None,
      }])
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "server-uuid-1".to_string(),
    auth: Some(McpAuth::Header {
      header_key: "Authorization".to_string(),
      header_value: "Bearer test-key".to_string(),
    }),
    auth_uuid: None,
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
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_fetch_mcp_tools_with_auth_uuid() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_fetch_tools_for_server()
    .withf(|server_id, auth_key, auth_val, auth_uuid| {
      server_id == "server-uuid-1"
        && auth_key.is_none()
        && auth_val.is_none()
        && auth_uuid.as_deref() == Some("uuid-123")
    })
    .times(1)
    .returning(|_, _, _, _| Ok(vec![]));

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&FetchMcpToolsRequest {
    mcp_server_id: "server-uuid-1".to_string(),
    auth: None,
    auth_uuid: Some("uuid-123".to_string()),
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
  Ok(())
}

// ============================================================================
// POST /mcps/{id}/tools/{tool_name}/execute
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_execute_mcp_tool_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_execute()
    .withf(|user_id, id, tool_name, _| {
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
  Ok(())
}

// ============================================================================
// Auth Header CRUD handlers
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_auth_header_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_create_auth_header()
    .withf(|user_id, key, value| {
      user_id == "user123" && key == "Authorization" && value == "Bearer secret"
    })
    .times(1)
    .returning(|_, _, _| {
      Ok(McpAuthHeader {
        id: "ah-uuid-1".to_string(),
        header_key: "Authorization".to_string(),
        has_header_value: true,
        created_by: "user123".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
      })
    });

  let app = test_router_for_crud(mock).await?;

  let body = serde_json::to_string(&CreateAuthHeaderRequest {
    header_key: "Authorization".to_string(),
    header_value: "Bearer secret".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-headers")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: AuthHeaderResponse = response.json().await?;
  assert_eq!("ah-uuid-1", body.id);
  assert_eq!("Authorization", body.header_key);
  assert!(body.has_header_value);
  Ok(())
}

// ============================================================================
// Integration tests (real DB)
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

async fn create_auth_header_in_db(
  router: &Router,
  user_cookie: &str,
  header_key: &str,
  header_value: &str,
) -> anyhow::Result<String> {
  let body = json!({
    "header_key": header_key,
    "header_value": header_value
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/auth-headers",
      user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let auth: Value = response.json().await?;
  Ok(auth["id"].as_str().unwrap().to_string())
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
  let auth_id = create_auth_header_in_db(
    &router,
    &user_cookie,
    "Authorization",
    "Bearer sk-test-secret",
  )
  .await?;

  let body = json!({
    "name": "Tavily Auth",
    "slug": "tavily-auth",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "header",
    "auth_uuid": auth_id
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
  assert_eq!(auth_id, mcp["auth_uuid"]);
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
    "auth_type": "public"
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
  assert_eq!(Value::Null, mcp["auth_uuid"]);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_integration_update_mcp_switch_auth_type() -> anyhow::Result<()> {
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
    "auth_type": "public"
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

  let auth_id = create_auth_header_in_db(&router, &user_cookie, "X-Api-Key", "key-123").await?;

  let update_body = json!({
    "name": "My MCP",
    "slug": "my-mcp",
    "enabled": true,
    "auth_type": "header",
    "auth_uuid": auth_id
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
  assert_eq!(auth_id, updated["auth_uuid"]);
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
  let auth_id =
    create_auth_header_in_db(&router, &user_cookie, "Authorization", "Bearer keep-me").await?;

  let create_body = json!({
    "name": "Keep Auth MCP",
    "slug": "keep-auth-mcp",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "header",
    "auth_uuid": auth_id
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
  assert_eq!(auth_id, updated["auth_uuid"].as_str().unwrap());
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
  let auth_id = create_auth_header_in_db(&router, &user_cookie, "X-Api-Key", "secret-val").await?;

  let public_body = json!({
    "name": "Public One",
    "slug": "public-one",
    "mcp_server_id": server_id,
    "enabled": true,
    "auth_type": "public"
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
    "auth_type": "header",
    "auth_uuid": auth_id
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

  let header_mcp = mcps.iter().find(|m| m["slug"] == "header-one").unwrap();
  assert_eq!("header", header_mcp["auth_type"]);
  assert_eq!(auth_id, header_mcp["auth_uuid"].as_str().unwrap());
  Ok(())
}
