use crate::routes_mcp::{
  oauth_login_handler, oauth_token_exchange_handler, OAuthLoginRequest, OAuthLoginResponse,
  OAuthTokenExchangeRequest, OAuthTokenResponse,
};
use crate::test_utils::fixed_dt;
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::Router;
use objs::{McpOAuthConfig, McpOAuthToken, McpServer, RegistrationType, ResourceRole};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{
  test_utils::AppServiceStubBuilder, AppService, DefaultSessionService, McpError, MockMcpService,
  SessionService,
};
use std::collections::HashMap;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use tower_sessions::session::{Id, Record};
use tower_sessions::SessionStore;
use url::Url;

fn test_oauth_config() -> McpOAuthConfig {
  McpOAuthConfig {
    id: "oauth-config-1".to_string(),
    name: "Test OAuth".to_string(),
    mcp_server_id: "server-1".to_string(),
    registration_type: RegistrationType::PreRegistered,
    client_id: "test-client-id".to_string(),
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    registration_endpoint: None,
    client_id_issued_at: None,
    token_endpoint_auth_method: None,
    scopes: Some("openid profile".to_string()),
    has_client_secret: true,
    has_registration_access_token: false,
    created_by: "user123".to_string(),
    created_at: fixed_dt(),
    updated_at: fixed_dt(),
  }
}

fn test_mcp_server() -> McpServer {
  McpServer {
    id: "server-1".to_string(),
    url: "https://mcp.example.com".to_string(),
    name: "Test Server".to_string(),
    description: None,
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: fixed_dt(),
    updated_at: fixed_dt(),
  }
}

/// Builds a router with session layer for OAuth flow tests.
/// Returns (Router, Arc<DefaultSessionService>) for session inspection.
async fn build_oauth_flow_router(
  mock_mcp_service: MockMcpService,
) -> anyhow::Result<(Router, Arc<DefaultSessionService>)> {
  let temp_dir = tempfile::TempDir::new()?;
  let dbfile = temp_dir.path().join("test-session.sqlite");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let mcp_svc: Arc<dyn services::McpService> = Arc::new(mock_mcp_service);
  let app_service: Arc<dyn AppService> = Arc::new(
    AppServiceStubBuilder::default()
      .mcp_service(mcp_svc)
      .with_default_session_service(session_service.clone())
      .build()
      .await?,
  );

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/mcps/auth-configs/{id}/login", post(oauth_login_handler))
    .route(
      "/mcps/auth-configs/{id}/token",
      post(oauth_token_exchange_handler),
    )
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  // Keep temp_dir alive by leaking it (tests are short-lived)
  std::mem::forget(temp_dir);

  Ok((router, session_service))
}

// ============================================================================
// Test 1: OAuth login success - verify auth URL params including resource
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_login_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();
  let config = test_oauth_config();
  let server = test_mcp_server();

  mock
    .expect_get_oauth_config()
    .withf(|id| id == "oauth-config-1")
    .times(1)
    .returning(move |_| Ok(Some(config.clone())));

  mock
    .expect_get_mcp_server()
    .withf(|id| id == "server-1")
    .times(1)
    .returning(move |_| Ok(Some(server.clone())));

  let (app, _session_service) = build_oauth_flow_router(mock).await?;

  let body = serde_json::to_string(&OAuthLoginRequest {
    redirect_uri: "http://localhost:3000/callback".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-configs/oauth-config-1/login")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: OAuthLoginResponse = response.json().await?;

  // Parse the authorization URL and verify all expected parameters
  let auth_url = Url::parse(&body.authorization_url)?;
  let params: HashMap<_, _> = auth_url.query_pairs().into_owned().collect();

  assert_eq!(
    Some("code"),
    params.get("response_type").map(|s| s.as_str())
  );
  assert_eq!(
    Some("test-client-id"),
    params.get("client_id").map(|s| s.as_str())
  );
  assert_eq!(
    Some("http://localhost:3000/callback"),
    params.get("redirect_uri").map(|s| s.as_str())
  );
  assert_eq!(
    Some("S256"),
    params.get("code_challenge_method").map(|s| s.as_str())
  );
  assert!(params.contains_key("code_challenge"));
  assert!(params.contains_key("state"));
  assert_eq!(
    Some("openid profile"),
    params.get("scope").map(|s| s.as_str())
  );
  // N14: Verify resource parameter is present with the MCP server URL
  assert_eq!(
    Some("https://mcp.example.com"),
    params.get("resource").map(|s| s.as_str())
  );

  Ok(())
}

// ============================================================================
// Test 2: OAuth login - config not found returns 404
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_login_config_not_found() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_get_oauth_config()
    .withf(|id| id == "nonexistent-config")
    .times(1)
    .returning(|_| Ok(None));

  let (app, _session_service) = build_oauth_flow_router(mock).await?;

  let body = serde_json::to_string(&OAuthLoginRequest {
    redirect_uri: "http://localhost:3000/callback".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-configs/nonexistent-config/login")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

// ============================================================================
// Test 3: OAuth token exchange success - pre-populated session
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_token_exchange_success() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_exchange_oauth_token()
    .withf(|user_id, config_id, code, redirect_uri, code_verifier| {
      user_id == "user123"
        && config_id == "oauth-config-1"
        && code == "auth-code-xyz"
        && redirect_uri == "http://localhost:3000/callback"
        && code_verifier == "test-code-verifier"
    })
    .times(1)
    .returning(|_, _, _, _, _| {
      let now = fixed_dt();
      Ok(McpOAuthToken {
        id: "token-uuid-1".to_string(),
        mcp_oauth_config_id: "oauth-config-1".to_string(),
        scopes_granted: Some("openid profile".to_string()),
        expires_at: Some(1700000000),
        has_access_token: true,
        has_refresh_token: true,
        created_by: "user123".to_string(),
        created_at: now,
        updated_at: now,
      })
    });

  let (app, session_service) = build_oauth_flow_router(mock).await?;

  // Pre-populate session with OAuth state data (as oauth_login_handler would)
  let session_id = Id::default();
  // Use FrozenTimeService default: 2025-01-01T00:00:00Z -> timestamp = 1735689600
  let created_at = fixed_dt().timestamp();
  let mut record = Record {
    id: session_id,
    data: maplit::hashmap! {
      "mcp_oauth_oauth-config-1".to_string() => json!({
        "code_verifier": "test-code-verifier",
        "state": "test-csrf-state",
        "created_at": created_at,
      }),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let body = serde_json::to_string(&OAuthTokenExchangeRequest {
    code: "auth-code-xyz".to_string(),
    redirect_uri: "http://localhost:3000/callback".to_string(),
    state: "test-csrf-state".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-configs/oauth-config-1/token")
    .header("content-type", "application/json")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: OAuthTokenResponse = response.json().await?;
  assert_eq!("token-uuid-1", body.id);
  assert_eq!("oauth-config-1", body.mcp_oauth_config_id);
  assert_eq!(Some("openid profile".to_string()), body.scopes_granted);
  assert!(body.has_access_token);
  assert!(body.has_refresh_token);
  Ok(())
}

// ============================================================================
// Test 4: OAuth token exchange - CSRF state mismatch returns 400
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_token_exchange_state_mismatch() -> anyhow::Result<()> {
  let mock = MockMcpService::new();

  let (app, session_service) = build_oauth_flow_router(mock).await?;

  // Pre-populate session with one state
  let session_id = Id::default();
  let created_at = fixed_dt().timestamp();
  let mut record = Record {
    id: session_id,
    data: maplit::hashmap! {
      "mcp_oauth_oauth-config-1".to_string() => json!({
        "code_verifier": "test-code-verifier",
        "state": "correct-state",
        "created_at": created_at,
      }),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  // Send request with a DIFFERENT state
  let body = serde_json::to_string(&OAuthTokenExchangeRequest {
    code: "auth-code-xyz".to_string(),
    redirect_uri: "http://localhost:3000/callback".to_string(),
    state: "wrong-state".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-configs/oauth-config-1/token")
    .header("content-type", "application/json")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body: Value = response.json().await?;
  assert_eq!(
    "mcp_validation_error-csrf_state_mismatch",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// Test 5: OAuth token exchange - missing session data returns 400
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_token_exchange_missing_session() -> anyhow::Result<()> {
  let mock = MockMcpService::new();

  let (app, _session_service) = build_oauth_flow_router(mock).await?;

  // No session pre-populated -- no Cookie header sent
  let body = serde_json::to_string(&OAuthTokenExchangeRequest {
    code: "auth-code-xyz".to_string(),
    redirect_uri: "http://localhost:3000/callback".to_string(),
    state: "some-state".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-configs/oauth-config-1/token")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body: Value = response.json().await?;
  assert_eq!(
    "mcp_validation_error-session_data_missing",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// Test 6: OAuth token exchange - service error propagation
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_oauth_token_exchange_service_error() -> anyhow::Result<()> {
  let mut mock = MockMcpService::new();

  mock
    .expect_exchange_oauth_token()
    .times(1)
    .returning(|_, _, _, _, _| {
      Err(McpError::OAuthTokenExchangeFailed(
        "token endpoint returned 401".to_string(),
      ))
    });

  let (app, session_service) = build_oauth_flow_router(mock).await?;

  // Pre-populate session
  let session_id = Id::default();
  let created_at = fixed_dt().timestamp();
  let mut record = Record {
    id: session_id,
    data: maplit::hashmap! {
      "mcp_oauth_oauth-config-1".to_string() => json!({
        "code_verifier": "test-code-verifier",
        "state": "test-state",
        "created_at": created_at,
      }),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let body = serde_json::to_string(&OAuthTokenExchangeRequest {
    code: "auth-code-xyz".to_string(),
    redirect_uri: "http://localhost:3000/callback".to_string(),
    state: "test-state".to_string(),
  })?;

  let request = Request::builder()
    .method("POST")
    .uri("/mcps/auth-configs/oauth-config-1/token")
    .header("content-type", "application/json")
    .header("Cookie", format!("bodhiapp_session_id={}", session_id))
    .body(Body::from(body))?;
  let request = request.with_auth_context(AuthContext::test_session(
    "user123",
    "testuser",
    ResourceRole::User,
  ));
  let response = app.oneshot(request).await?;

  // McpError::TokenExchangeFailed maps to InternalServer (500) via ErrorMeta
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
  Ok(())
}
