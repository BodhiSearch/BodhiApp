use crate::test_utils::{
  build_test_router, create_authenticated_session, create_header_auth_config_in_db,
  session_request, session_request_with_body, setup_mcp_server_in_db,
};
use anyhow_trace::anyhow_trace;
use axum::body::Body;
use axum::http::StatusCode;
use objs::{McpAuthConfigResponse, RegistrationType};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use tower::ServiceExt;

// ============================================================================
// POST /bodhi/v1/mcps/auth-configs - Create header auth config
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_auth_config_header_success() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let body = json!({
    "mcp_server_id": server_id,
    "type": "header",
    "name": "My Header",
    "header_key": "Authorization",
    "header_value": "Bearer secret"
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/auth-configs",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: McpAuthConfigResponse = response.json().await?;
  match body {
    McpAuthConfigResponse::Header {
      name,
      mcp_server_id,
      header_key,
      has_header_value,
      ..
    } => {
      assert_eq!("My Header", name);
      assert_eq!(server_id, mcp_server_id);
      assert_eq!("Authorization", header_key);
      assert!(has_header_value);
    }
    _ => panic!("expected Header auth config"),
  }
  Ok(())
}

// ============================================================================
// GET /bodhi/v1/mcps/auth-configs/{id}
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_auth_config_success() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
  let auth_id = create_header_auth_config_in_db(
    &router,
    &user_cookie,
    &server_id,
    "Authorization",
    "Bearer s",
  )
  .await?;

  let response = router
    .clone()
    .oneshot(session_request(
      "GET",
      &format!("/bodhi/v1/mcps/auth-configs/{}", auth_id),
      &user_cookie,
    ))
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: McpAuthConfigResponse = response.json().await?;
  match body {
    McpAuthConfigResponse::Header { id, .. } => {
      assert_eq!(auth_id, id);
    }
    _ => panic!("expected Header auth config"),
  }
  Ok(())
}

// ============================================================================
// DELETE /bodhi/v1/mcps/auth-configs/{id}
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_auth_config_success() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
  let auth_id =
    create_header_auth_config_in_db(&router, &user_cookie, &server_id, "X-Api-Key", "key-val")
      .await?;

  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "DELETE",
      &format!("/bodhi/v1/mcps/auth-configs/{}", auth_id),
      &user_cookie,
      Body::empty(),
    ))
    .await?;

  assert_eq!(StatusCode::NO_CONTENT, response.status());
  Ok(())
}

// ============================================================================
// GET /bodhi/v1/mcps/auth-configs?mcp_server_id=...
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_auth_configs_success() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
  create_header_auth_config_in_db(
    &router,
    &user_cookie,
    &server_id,
    "Authorization",
    "Bearer s",
  )
  .await?;

  let response = router
    .clone()
    .oneshot(session_request(
      "GET",
      &format!("/bodhi/v1/mcps/auth-configs?mcp_server_id={}", server_id),
      &user_cookie,
    ))
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body: Value = response.json().await?;
  let configs = body["auth_configs"].as_array().unwrap();
  assert_eq!(1, configs.len());
  Ok(())
}

// ============================================================================
// POST /bodhi/v1/mcps/auth-configs - Create OAuth pre-registered auth config
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_auth_config_oauth_prereg_success() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let body = json!({
    "mcp_server_id": server_id,
    "type": "oauth",
    "name": "My OAuth PreReg",
    "client_id": "client-123",
    "client_secret": "secret-456",
    "authorization_endpoint": "https://auth.example.com/authorize",
    "token_endpoint": "https://auth.example.com/token",
    "scopes": "openid",
    "registration_type": "pre-registered"
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/auth-configs",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: McpAuthConfigResponse = response.json().await?;
  match body {
    McpAuthConfigResponse::Oauth {
      name,
      mcp_server_id,
      registration_type,
      client_id,
      authorization_endpoint,
      token_endpoint,
      has_client_secret,
      has_registration_access_token,
      scopes,
      ..
    } => {
      assert_eq!("My OAuth PreReg", name);
      assert_eq!(server_id, mcp_server_id);
      assert_eq!(RegistrationType::PreRegistered, registration_type);
      assert_eq!("client-123", client_id);
      assert_eq!("https://auth.example.com/authorize", authorization_endpoint);
      assert_eq!("https://auth.example.com/token", token_endpoint);
      assert!(has_client_secret);
      assert!(!has_registration_access_token);
      assert_eq!(Some("openid".to_string()), scopes);
    }
    _ => panic!("expected Oauth auth config"),
  }
  Ok(())
}

// ============================================================================
// POST /bodhi/v1/mcps/auth-configs - Create OAuth DCR auth config
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_create_auth_config_oauth_dcr_success() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let admin_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let user_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;
  let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;

  let body = json!({
    "mcp_server_id": server_id,
    "type": "oauth",
    "name": "My OAuth DCR",
    "client_id": "dcr-client-id",
    "authorization_endpoint": "https://auth.example.com/authorize",
    "token_endpoint": "https://auth.example.com/token",
    "scopes": "openid profile",
    "registration_type": "dynamic-registration",
    "registration_access_token": "dcr-reg-token",
    "registration_endpoint": "https://auth.example.com/register",
    "token_endpoint_auth_method": "none",
    "client_id_issued_at": 1700000000
  });
  let response = router
    .clone()
    .oneshot(session_request_with_body(
      "POST",
      "/bodhi/v1/mcps/auth-configs",
      &user_cookie,
      Body::from(serde_json::to_string(&body)?),
    ))
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());
  let body: McpAuthConfigResponse = response.json().await?;
  match body {
    McpAuthConfigResponse::Oauth {
      name,
      mcp_server_id,
      registration_type,
      client_id,
      registration_endpoint,
      scopes,
      client_id_issued_at,
      token_endpoint_auth_method,
      has_client_secret,
      has_registration_access_token,
      ..
    } => {
      assert_eq!("My OAuth DCR", name);
      assert_eq!(server_id, mcp_server_id);
      assert_eq!(RegistrationType::DynamicRegistration, registration_type);
      assert_eq!("dcr-client-id", client_id);
      assert_eq!(
        Some("https://auth.example.com/register".to_string()),
        registration_endpoint
      );
      assert_eq!(Some("openid profile".to_string()), scopes);
      assert_eq!(Some(1700000000), client_id_issued_at);
      assert_eq!(Some("none".to_string()), token_endpoint_auth_method);
      assert!(!has_client_secret);
      assert!(has_registration_access_token);
    }
    _ => panic!("expected Oauth auth config"),
  }
  Ok(())
}
