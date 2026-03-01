use crate::{change_user_role_handler, list_users_handler, remove_user_handler};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{delete, get, put},
  Router,
};
use chrono::{Duration, Utc};
use mockall::predicate::{always, eq};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
use services::test_utils::temp_bodhi_home;
use services::{
  test_utils::{build_token_with_exp, AppServiceStubBuilder},
  MockAuthService, MockSessionService, UserListResponse,
};
use services::{AppRole, ResourceRole, UserInfo};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_users_handler_success(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  // Mock successful auth service response
  let mut mock_auth = MockAuthService::default();
  let expected_response = UserListResponse {
    client_id: "test-client-id".to_string(),
    users: vec![
      UserInfo {
        user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        username: "admin@example.com".to_string(),
        first_name: Some("Admin".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::Session(ResourceRole::Admin)),
      },
      UserInfo {
        user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
        username: "user@example.com".to_string(),
        first_name: Some("Regular".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::Session(ResourceRole::User)),
      },
    ],
    page: 1,
    page_size: 10,
    total_pages: 1,
    total_users: 2,
    has_next: false,
    has_previous: false,
  };

  mock_auth
    .expect_list_users()
    .times(1)
    .withf(|token, page, page_size| {
      token == "test-token" && *page == Some(1) && *page_size == Some(10)
    })
    .return_once(|_, _, _| Ok(expected_response));

  // Build app service with mock auth service
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users", get(list_users_handler))
    .with_state(state);

  // Make request with auth context
  let request = Request::get("/bodhi/v1/users?page=1&page_size=10")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "admin-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      "test-token",
    ));

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify successful response
  assert_eq!(StatusCode::OK, response.status());

  // Verify response body
  let response_body = response.json::<UserListResponse>().await?;

  assert_eq!("test-client-id", response_body.client_id);
  assert_eq!(2, response_body.users.len());
  assert_eq!("admin@example.com", response_body.users[0].username);
  assert_eq!("user@example.com", response_body.users[1].username);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_users_handler_auth_error(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  // Mock auth service returning error
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_list_users()
    .times(1)
    .return_once(|_, _, _| {
      Err(services::AuthServiceError::AuthServiceApiError {
        status: 500,
        body: "Invalid session [RequestID: 123456]".to_string(),
      })
    });

  // Build app service with mock auth service
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users", get(list_users_handler))
    .with_state(state);

  // Make request with auth context
  let request = Request::get("/bodhi/v1/users?page=1&page_size=10")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "user-id",
      "user@example.com",
      ResourceRole::Admin,
      "invalid-token",
    ));

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify error response
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());

  // Verify error code
  let response_json = response.json::<Value>().await?;
  assert_eq!(
    "user_route_error-list_failed",
    response_json["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_users_handler_pagination_parameters(
  _temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  // Mock auth service to verify pagination parameters
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_list_users()
    .times(1)
    .withf(|_, page, page_size| *page == Some(2) && *page_size == Some(5))
    .return_once(|_, _, _| {
      Ok(UserListResponse {
        client_id: "test-client-id".to_string(),
        users: vec![],
        page: 2,
        page_size: 5,
        total_pages: 3,
        total_users: 15,
        has_next: true,
        has_previous: true,
      })
    });

  // Build app service with mock auth service
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users", get(list_users_handler))
    .with_state(state);

  // Make request with custom pagination
  let request = Request::get("/bodhi/v1/users?page=2&page_size=5")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "admin-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      "test-token",
    ));

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify successful response
  assert_eq!(StatusCode::OK, response.status());

  // Verify response contains correct pagination
  let response_body = response.json::<UserListResponse>().await?;

  assert_eq!(2, response_body.page);
  assert_eq!(5, response_body.page_size);
  assert_eq!(15, response_body.total_users);
  assert_eq!(true, response_body.has_next);
  assert_eq!(true, response_body.has_previous);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_change_user_role_clears_sessions(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  // Create a valid JWT token for testing
  let (test_token, _) = build_token_with_exp((Utc::now() + Duration::hours(1)).timestamp())?;

  // Mock auth service for role assignment
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_assign_user_role()
    .times(1)
    .with(always(), eq("user-123"), eq("resource_power_user"))
    .return_once(|_, _, _| Ok(()));

  // Mock session service to verify sessions are cleared
  let mut mock_session = MockSessionService::default();
  mock_session
    .expect_clear_sessions_for_user()
    .times(1)
    .with(eq("user-123"))
    .return_once(|_| Ok(3)); // Return that 3 sessions were cleared

  // Build app service with mocks
  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .session_service(Arc::new(mock_session))
    .build()
    .await?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      "/bodhi/v1/users/{user_id}/role",
      put(change_user_role_handler),
    )
    .with_state(state);

  // Make request
  let request = Request::put("/bodhi/v1/users/user-123/role")
    .header("Content-Type", "application/json")
    .body(Body::from(r#"{"role": "resource_power_user"}"#))?
    .with_auth_context(AuthContext::test_session_with_token(
      "test-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      &test_token,
    ));

  // Send request
  let response = router.oneshot(request).await?;

  // Verify success
  assert_eq!(StatusCode::OK, response.status());

  // The mock expectations will verify that both assign_user_role
  // AND clear_sessions_for_user were called
  Ok(())
}

// ============================================================================
// remove_user_handler tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_remove_user_handler_success(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let (test_token, _) = build_token_with_exp((Utc::now() + Duration::hours(1)).timestamp())?;

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_remove_user()
    .times(1)
    .with(always(), eq("user-to-remove"))
    .return_once(|_, _| Ok(()));

  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users/{user_id}", delete(remove_user_handler))
    .with_state(state);

  let request = Request::delete("/bodhi/v1/users/user-to-remove")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "test-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      &test_token,
    ));

  let response = router.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_remove_user_handler_auth_error(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let (test_token, _) = build_token_with_exp((Utc::now() + Duration::hours(1)).timestamp())?;

  let mut mock_auth = MockAuthService::default();
  mock_auth.expect_remove_user().times(1).return_once(|_, _| {
    Err(services::AuthServiceError::AuthServiceApiError {
      status: 404,
      body: "User not found".to_string(),
    })
  });

  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users/{user_id}", delete(remove_user_handler))
    .with_state(state);

  let request = Request::delete("/bodhi/v1/users/nonexistent-user")
    .body(Body::empty())?
    .with_auth_context(AuthContext::test_session_with_token(
      "test-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      &test_token,
    ));

  let response = router.oneshot(request).await?;

  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "user_route_error-remove_failed",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// change_user_role_handler - role change failure test
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_change_user_role_handler_auth_error(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let (test_token, _) = build_token_with_exp((Utc::now() + Duration::hours(1)).timestamp())?;

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_assign_user_role()
    .times(1)
    .return_once(|_, _, _| {
      Err(services::AuthServiceError::AuthServiceApiError {
        status: 500,
        body: "Role assignment failed".to_string(),
      })
    });

  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      "/bodhi/v1/users/{user_id}/role",
      put(change_user_role_handler),
    )
    .with_state(state);

  let request = Request::put("/bodhi/v1/users/user-123/role")
    .header("Content-Type", "application/json")
    .body(Body::from(r#"{"role": "resource_admin"}"#))?
    .with_auth_context(AuthContext::test_session_with_token(
      "test-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      &test_token,
    ));

  let response = router.oneshot(request).await?;

  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "user_route_error-role_change_failed",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// change_user_role_handler - session clear failure doesn't fail operation
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_change_user_role_session_clear_failure_still_succeeds(
  _temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let (test_token, _) = build_token_with_exp((Utc::now() + Duration::hours(1)).timestamp())?;

  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_assign_user_role()
    .times(1)
    .return_once(|_, _, _| Ok(()));

  let mut mock_session = MockSessionService::default();
  mock_session
    .expect_clear_sessions_for_user()
    .times(1)
    .return_once(|_| {
      Err(services::SessionServiceError::SessionStoreError(
        tower_sessions::session_store::Error::Backend("Session store unavailable".to_string()),
      ))
    });

  let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .session_service(Arc::new(mock_session))
    .build()
    .await?;

  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      "/bodhi/v1/users/{user_id}/role",
      put(change_user_role_handler),
    )
    .with_state(state);

  let request = Request::put("/bodhi/v1/users/user-123/role")
    .header("Content-Type", "application/json")
    .body(Body::from(r#"{"role": "resource_user"}"#))?
    .with_auth_context(AuthContext::test_session_with_token(
      "test-user-id",
      "admin@example.com",
      ResourceRole::Admin,
      &test_token,
    ));

  let response = router.oneshot(request).await?;

  // Should still succeed even though session clearing failed
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
