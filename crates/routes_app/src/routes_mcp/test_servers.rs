use crate::routes_mcp::{
  create_mcp_server_handler, get_mcp_server_handler, list_mcp_servers_handler,
  update_mcp_server_handler, CreateMcpServerRequest, McpServerResponse,
};
use crate::test_utils::{build_mcp_test_state, fixed_dt};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post, put};
use axum::Router;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::ResponseTestExt;
use services::ResourceRole;
use services::{CreateMcpAuthConfigRequest, McpAuthConfigResponse, McpServer, RegistrationType};
use services::{McpServerError, MockMcpService};
use tower::ServiceExt;

fn test_mcp_server() -> McpServer {
  let now = fixed_dt();
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

async fn test_router_for_mcp_servers(mock_mcp_service: MockMcpService) -> anyhow::Result<Router> {
  let state = build_mcp_test_state(mock_mcp_service).await?;
  Ok(
    Router::new()
      .route("/mcps/servers", get(list_mcp_servers_handler))
      .route("/mcps/servers", post(create_mcp_server_handler))
      .route("/mcps/servers/{id}", get(get_mcp_server_handler))
      .route("/mcps/servers/{id}", put(update_mcp_server_handler))
      .with_state(state),
  )
}

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

  let app = test_router_for_mcp_servers(mock).await?;

  let body = serde_json::to_string(&CreateMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "DeepWiki".to_string(),
    description: Some("DeepWiki MCP server".to_string()),
    enabled: true,
    auth_config: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/servers")
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

  let app = test_router_for_mcp_servers(mock).await?;

  let body = serde_json::to_string(&CreateMcpServerRequest {
    url: "https://mcp.deepwiki.com/mcp".to_string(),
    name: "DeepWiki".to_string(),
    description: None,
    enabled: true,
    auth_config: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/servers")
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

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_server_with_header_auth_config() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_create_mcp_server()
    .times(1)
    .returning(move |_, _, _, _, _| Ok(server.clone()));

  mock
    .expect_create_auth_config()
    .withf(|user_id, server_id, request| {
      user_id == "admin-user"
        && server_id == "server-uuid-1"
        && matches!(request, CreateMcpAuthConfigRequest::Header { name, header_key, .. } if name == "My Header" && header_key == "Authorization")
    })
    .times(1)
    .returning(|_, _, _| {
      Ok(McpAuthConfigResponse::Header {
        id: "auth-config-uuid-1".to_string(),
        name: "My Header".to_string(),
        mcp_server_id: "server-uuid-1".to_string(),
        header_key: "Authorization".to_string(),
        has_header_value: true,
        created_by: "admin-user".to_string(),
        created_at: fixed_dt(),
        updated_at: fixed_dt(),
      })
    });

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((0, 0)));

  let app = test_router_for_mcp_servers(mock).await?;

  let body = serde_json::to_string(&json!({
    "url": "https://mcp.example.com/mcp",
    "name": "Example",
    "enabled": true,
    "auth_config": {
      "type": "header",
      "name": "My Header",
      "header_key": "Authorization",
      "header_value": "Bearer token123"
    }
  }))?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/servers")
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
  assert!(body.auth_config.is_some());
  match body.auth_config.unwrap() {
    McpAuthConfigResponse::Header {
      id,
      name,
      header_key,
      ..
    } => {
      assert_eq!("auth-config-uuid-1", id);
      assert_eq!("My Header", name);
      assert_eq!("Authorization", header_key);
    }
    _ => panic!("expected Header auth config"),
  }
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_server_with_oauth_prereg_auth_config() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_create_mcp_server()
    .times(1)
    .returning(move |_, _, _, _, _| Ok(server.clone()));

  mock
    .expect_create_auth_config()
    .times(1)
    .returning(|_, _, _| {
      Ok(McpAuthConfigResponse::Oauth {
        id: "oauth-config-uuid-1".to_string(),
        name: "My OAuth".to_string(),
        mcp_server_id: "server-uuid-1".to_string(),
        registration_type: RegistrationType::PreRegistered,
        client_id: "client-123".to_string(),
        authorization_endpoint: "https://auth.example.com/authorize".to_string(),
        token_endpoint: "https://auth.example.com/token".to_string(),
        registration_endpoint: None,
        scopes: Some("openid".to_string()),
        client_id_issued_at: None,
        token_endpoint_auth_method: None,
        has_client_secret: true,
        has_registration_access_token: false,
        created_by: "admin-user".to_string(),
        created_at: fixed_dt(),
        updated_at: fixed_dt(),
      })
    });

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((0, 0)));

  let app = test_router_for_mcp_servers(mock).await?;

  let body = serde_json::to_string(&json!({
    "url": "https://mcp.example.com/mcp",
    "name": "Example",
    "enabled": true,
    "auth_config": {
      "type": "oauth",
      "name": "My OAuth",
      "client_id": "client-123",
      "client_secret": "secret-456",
      "authorization_endpoint": "https://auth.example.com/authorize",
      "token_endpoint": "https://auth.example.com/token",
      "scopes": "openid"
    }
  }))?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/servers")
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
  assert!(body.auth_config.is_some());
  match body.auth_config.unwrap() {
    McpAuthConfigResponse::Oauth { id, client_id, .. } => {
      assert_eq!("oauth-config-uuid-1", id);
      assert_eq!("client-123", client_id);
    }
    _ => panic!("expected Oauth auth config"),
  }
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_server_without_auth_config_backwards_compat() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let server = test_mcp_server();

  mock
    .expect_create_mcp_server()
    .times(1)
    .returning(move |_, _, _, _, _| Ok(server.clone()));

  mock
    .expect_count_mcps_for_server()
    .returning(|_| Ok((0, 0)));

  let app = test_router_for_mcp_servers(mock).await?;

  // Send JSON without auth_config field - should still work
  let body = serde_json::to_string(&json!({
    "url": "https://mcp.deepwiki.com/mcp",
    "name": "DeepWiki",
    "enabled": true
  }))?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/servers")
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
  assert!(body.auth_config.is_none());
  Ok(())
}
