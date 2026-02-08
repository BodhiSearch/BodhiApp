use crate::{change_user_role_handler, list_users_handler};
use auth_middleware::{
  KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USERNAME, KEY_HEADER_BODHIAPP_USER_ID,
};
use axum::{
  body::Body,
  http::Request,
  routing::{get, put},
  Router,
};
use chrono::{Duration, Utc};
use mockall::predicate::{always, eq};
use objs::{test_utils::temp_bodhi_home, AppRole, ResourceRole, UserInfo};
use rstest::rstest;
use server_core::{DefaultRouterState, MockSharedContext};
use services::{
  test_utils::{build_token_with_exp, AppServiceStubBuilder},
  MockAuthService, MockSessionService, UserListResponse,
};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

#[rstest]
#[tokio::test]
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
    .build()?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users", get(list_users_handler))
    .with_state(state);

  // Make request with required headers
  let request = Request::get("/bodhi/v1/users?page=1&page_size=10")
    .header(KEY_HEADER_BODHIAPP_TOKEN, "test-token")
    .header(KEY_HEADER_BODHIAPP_USERNAME, "admin@example.com")
    .header(KEY_HEADER_BODHIAPP_USER_ID, "admin-user-id")
    .body(Body::empty())
    .unwrap();

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify successful response
  assert_eq!(axum::http::StatusCode::OK, response.status());

  // Verify response body
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let response_body: UserListResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(response_body.client_id, "test-client-id");
  assert_eq!(response_body.users.len(), 2);
  assert_eq!(response_body.users[0].username, "admin@example.com");
  assert_eq!(response_body.users[1].username, "user@example.com");

  Ok(())
}

#[rstest]
#[tokio::test]
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
    .build()?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users", get(list_users_handler))
    .with_state(state);

  // Make request with required headers
  let request = Request::get("/bodhi/v1/users?page=1&page_size=10")
    .header(KEY_HEADER_BODHIAPP_TOKEN, "invalid-token")
    .header(KEY_HEADER_BODHIAPP_USERNAME, "user@example.com")
    .header(KEY_HEADER_BODHIAPP_USER_ID, "user-id")
    .body(Body::empty())
    .unwrap();

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify error response
  assert_eq!(
    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    response.status()
  );

  // Verify error message contains auth service error
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let body_text = String::from_utf8(body_bytes.to_vec())?;
  assert!(body_text.contains("Failed to list users"));

  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_list_users_handler_missing_token(_temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  // Build app service (auth service won't be called)
  let app_service = AppServiceStubBuilder::default().build()?;

  // Create router with handler
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route("/bodhi/v1/users", get(list_users_handler))
    .with_state(state);

  // Make request without token header
  let request = Request::get("/bodhi/v1/users?page=1&page_size=10")
    .body(Body::empty())
    .unwrap();

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify bad request response
  assert_eq!(axum::http::StatusCode::BAD_REQUEST, response.status());

  // Verify error message
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let body_text = String::from_utf8(body_bytes.to_vec())?;
  assert!(body_text.contains("Required header"));

  Ok(())
}

#[rstest]
#[tokio::test]
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
    .build()?;

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
    .header(KEY_HEADER_BODHIAPP_TOKEN, "test-token")
    .header(KEY_HEADER_BODHIAPP_USERNAME, "admin@example.com")
    .header(KEY_HEADER_BODHIAPP_USER_ID, "admin-user-id")
    .body(Body::empty())
    .unwrap();

  // Send request through router
  let response = router.oneshot(request).await?;

  // Verify successful response
  assert_eq!(axum::http::StatusCode::OK, response.status());

  // Verify response contains correct pagination
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let response_body: UserListResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(response_body.page, 2);
  assert_eq!(response_body.page_size, 5);
  assert_eq!(response_body.total_users, 15);
  assert_eq!(response_body.has_next, true);
  assert_eq!(response_body.has_previous, true);

  Ok(())
}

#[rstest]
#[tokio::test]
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
    .build()?;

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
    .header(KEY_HEADER_BODHIAPP_TOKEN, test_token)
    .header("Content-Type", "application/json")
    .body(Body::from(r#"{"role": "resource_power_user"}"#))
    .unwrap();

  // Send request
  let response = router.oneshot(request).await?;

  // Verify success
  assert_eq!(axum::http::StatusCode::OK, response.status());

  // The mock expectations will verify that both assign_user_role
  // AND clear_sessions_for_user were called
  Ok(())
}
