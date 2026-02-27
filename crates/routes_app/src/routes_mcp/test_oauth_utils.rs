use crate::routes_mcp::{
  delete_oauth_token_handler, get_oauth_token_handler, oauth_discover_as_handler,
  oauth_discover_mcp_handler, standalone_dynamic_register_handler, DynamicRegisterRequest,
  DynamicRegisterResponse, OAuthDiscoverAsRequest, OAuthDiscoverAsResponse,
  OAuthDiscoverMcpRequest, OAuthDiscoverMcpResponse,
};
use crate::test_utils::{build_mcp_test_state, build_mcp_test_state_with_app_service, fixed_dt};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, post};
use axum::Router;
use objs::{McpOAuthToken, RegistrationType, ResourceRole};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use services::MockMcpService;
use tower::ServiceExt;

async fn test_router_for_oauth_discovery(
  mock_mcp_service: MockMcpService,
) -> anyhow::Result<Router> {
  let state = build_mcp_test_state(mock_mcp_service).await?;
  Ok(
    Router::new()
      .route("/mcps/oauth/discover-as", post(oauth_discover_as_handler))
      .route("/mcps/oauth/discover-mcp", post(oauth_discover_mcp_handler))
      .route(
        "/mcps/oauth/dynamic-register",
        post(standalone_dynamic_register_handler),
      )
      .with_state(state),
  )
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_discover_as_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_discover_oauth_metadata()
    .withf(|url| url == "https://mcp.example.com")
    .times(1)
    .returning(|_| {
      Ok(json!({
        "authorization_endpoint": "https://auth.example.com/authorize",
        "token_endpoint": "https://auth.example.com/token",
        "scopes_supported": ["openid", "profile", "email"]
      }))
    });

  let app = test_router_for_oauth_discovery(mock).await?;

  let body = serde_json::to_string(&OAuthDiscoverAsRequest {
    url: "https://mcp.example.com".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/oauth/discover-as")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: OAuthDiscoverAsResponse = response.json().await?;
  assert_eq!(
    "https://auth.example.com/authorize",
    body.authorization_endpoint
  );
  assert_eq!("https://auth.example.com/token", body.token_endpoint);
  assert_eq!(
    Some(vec![
      "openid".to_string(),
      "profile".to_string(),
      "email".to_string()
    ]),
    body.scopes_supported
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_discover_mcp_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_discover_mcp_oauth_metadata()
    .withf(|url| url == "https://mcp.example.com")
    .times(1)
    .returning(|_| {
      Ok(json!({
        "authorization_endpoint": "https://auth.example.com/authorize",
        "token_endpoint": "https://auth.example.com/token",
        "registration_endpoint": "https://auth.example.com/register",
        "scopes_supported": ["openid", "profile"],
        "resource": "https://mcp.example.com",
        "authorization_server_url": "https://auth.example.com"
      }))
    });

  let app = test_router_for_oauth_discovery(mock).await?;

  let body = serde_json::to_string(&OAuthDiscoverMcpRequest {
    mcp_server_url: "https://mcp.example.com".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/oauth/discover-mcp")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: OAuthDiscoverMcpResponse = response.json().await?;
  assert_eq!(
    Some("https://auth.example.com/authorize".to_string()),
    body.authorization_endpoint
  );
  assert_eq!(
    Some("https://auth.example.com/token".to_string()),
    body.token_endpoint
  );
  assert_eq!(
    Some("https://auth.example.com/register".to_string()),
    body.registration_endpoint
  );
  assert_eq!(Some("https://mcp.example.com".to_string()), body.resource);
  assert_eq!(
    Some("https://auth.example.com".to_string()),
    body.authorization_server_url
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_standalone_dynamic_register_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_dynamic_register_client()
    .withf(|endpoint, redirect_uri, scopes| {
      endpoint == "https://auth.example.com/register"
        && redirect_uri == "http://localhost:3000/callback"
        && scopes.as_deref() == Some("openid profile")
    })
    .times(1)
    .returning(|_, _, _| {
      Ok(json!({
        "client_id": "standalone-client-id",
        "client_secret": "standalone-secret",
        "client_id_issued_at": 1700000000,
        "token_endpoint_auth_method": "none",
        "registration_access_token": "standalone-reg-token"
      }))
    });

  let app = test_router_for_oauth_discovery(mock).await?;

  let body = serde_json::to_string(&DynamicRegisterRequest {
    registration_endpoint: "https://auth.example.com/register".to_string(),
    redirect_uri: "http://localhost:3000/callback".to_string(),
    scopes: Some("openid profile".to_string()),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/oauth/dynamic-register")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: DynamicRegisterResponse = response.json().await?;
  assert_eq!("standalone-client-id", body.client_id);
  assert_eq!(Some("standalone-secret".to_string()), body.client_secret);
  assert_eq!(Some(1700000000), body.client_id_issued_at);
  assert_eq!(Some("none".to_string()), body.token_endpoint_auth_method);
  assert_eq!(
    Some("standalone-reg-token".to_string()),
    body.registration_access_token
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_standalone_dynamic_register_missing_fields() -> anyhow::Result<()> {
  let mock = MockMcpService::new();
  let app = test_router_for_oauth_discovery(mock).await?;

  let body = serde_json::to_string(&DynamicRegisterRequest {
    registration_endpoint: "".to_string(),
    redirect_uri: "http://localhost:3000/callback".to_string(),
    scopes: None,
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/oauth/dynamic-register")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  Ok(())
}

// ============================================================================
// OAuth Token Handler Tests
// ============================================================================

fn test_oauth_token() -> McpOAuthToken {
  let now = fixed_dt();
  McpOAuthToken {
    id: "token-uuid-1".to_string(),
    mcp_oauth_config_id: "oauth-config-uuid-1".to_string(),
    scopes_granted: Some("openid profile".to_string()),
    expires_at: Some(1700000000),
    has_access_token: true,
    has_refresh_token: true,
    created_by: "user123".to_string(),
    created_at: now,
    updated_at: now,
  }
}

async fn test_router_for_oauth_tokens(mock_mcp_service: MockMcpService) -> anyhow::Result<Router> {
  let state = build_mcp_test_state(mock_mcp_service).await?;
  Ok(
    Router::new()
      .route(
        "/mcps/oauth-tokens/{token_id}",
        get(get_oauth_token_handler),
      )
      .route(
        "/mcps/oauth-tokens/{token_id}",
        delete(delete_oauth_token_handler),
      )
      .with_state(state),
  )
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_oauth_token_handler_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let token = test_oauth_token();

  mock
    .expect_get_oauth_token()
    .withf(|user_id, token_id| user_id == "user123" && token_id == "token-uuid-1")
    .times(1)
    .returning(move |_, _| Ok(Some(token.clone())));

  let app = test_router_for_oauth_tokens(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcps/oauth-tokens/token-uuid-1")
    .body(Body::empty())?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: Value = response.json().await?;
  assert_eq!("token-uuid-1", body["id"].as_str().unwrap());
  assert_eq!(
    "oauth-config-uuid-1",
    body["mcp_oauth_config_id"].as_str().unwrap()
  );
  assert_eq!("openid profile", body["scopes_granted"].as_str().unwrap());
  assert!(body["has_access_token"].as_bool().unwrap());
  assert!(body["has_refresh_token"].as_bool().unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_oauth_token_handler_wrong_user_returns_404() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_get_oauth_token()
    .withf(|user_id, token_id| user_id == "wrong-user" && token_id == "token-uuid-1")
    .times(1)
    .returning(|_, _| Ok(None));

  let app = test_router_for_oauth_tokens(mock).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/mcps/oauth-tokens/token-uuid-1")
    .body(Body::empty())?;
  let request = request.with_auth_context(AuthContext::test_session(
    "wrong-user",
    "wronguser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_oauth_token_handler_success() -> anyhow::Result<()> {
  use services::db::{McpOAuthConfigRow, McpOAuthTokenRow, McpServerRow};

  let mock = MockMcpService::new();
  let (state, app_service) = build_mcp_test_state_with_app_service(mock).await?;
  let db_service = app_service.db_service();

  let now_ts = fixed_dt();

  // Insert prerequisite rows (server -> config -> token)
  let server_row = McpServerRow {
    id: "server-uuid-1".to_string(),
    url: "https://mcp.example.com".to_string(),
    name: "Test Server".to_string(),
    description: None,
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: now_ts,
    updated_at: now_ts,
  };
  db_service.create_mcp_server(&server_row).await?;

  let config_row = McpOAuthConfigRow {
    id: "oauth-config-uuid-1".to_string(),
    name: "Test OAuth".to_string(),
    mcp_server_id: "server-uuid-1".to_string(),
    registration_type: RegistrationType::PreRegistered,
    client_id: "client-123".to_string(),
    encrypted_client_secret: None,
    client_secret_salt: None,
    client_secret_nonce: None,
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    registration_endpoint: None,
    encrypted_registration_access_token: None,
    registration_access_token_salt: None,
    registration_access_token_nonce: None,
    client_id_issued_at: None,
    token_endpoint_auth_method: None,
    scopes: Some("openid".to_string()),
    created_by: "user123".to_string(),
    created_at: now_ts,
    updated_at: now_ts,
  };
  db_service.create_mcp_oauth_config(&config_row).await?;

  let token_row = McpOAuthTokenRow {
    id: "token-uuid-1".to_string(),
    mcp_oauth_config_id: "oauth-config-uuid-1".to_string(),
    encrypted_access_token: "enc_at".to_string(),
    access_token_salt: "salt".to_string(),
    access_token_nonce: "nonce".to_string(),
    encrypted_refresh_token: None,
    refresh_token_salt: None,
    refresh_token_nonce: None,
    scopes_granted: Some("openid".to_string()),
    expires_at: None,
    created_by: "user123".to_string(),
    created_at: now_ts,
    updated_at: now_ts,
  };
  db_service.create_mcp_oauth_token(&token_row).await?;

  // Verify the token exists before delete
  let before = db_service
    .get_mcp_oauth_token("user123", "token-uuid-1")
    .await?;
  assert!(before.is_some(), "token should exist before delete");

  let app = Router::new()
    .route(
      "/mcps/oauth-tokens/{token_id}",
      delete(delete_oauth_token_handler),
    )
    .with_state(state);

  let request = Request::builder()
    .method("DELETE")
    .uri("/mcps/oauth-tokens/token-uuid-1")
    .body(Body::empty())?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NO_CONTENT, response.status());

  // Verify the token was actually deleted from the database
  let after = db_service
    .get_mcp_oauth_token("user123", "token-uuid-1")
    .await?;
  assert!(
    after.is_none(),
    "token should be deleted after handler call"
  );
  Ok(())
}
