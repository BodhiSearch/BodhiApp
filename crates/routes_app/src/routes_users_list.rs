use auth_middleware::KEY_HEADER_BODHIAPP_TOKEN;
use axum::{
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
  Json,
};
use objs::{ApiError, BadRequestError, InternalServerError, OpenAIApiError, API_TAG_AUTH};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{extract_claims, Claims, UserListResponse};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;

/// List users query parameters
#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct ListUsersParams {
  #[schema(example = 1)]
  pub page: Option<u32>,
  #[schema(example = 10)]
  pub page_size: Option<u32>,
}

/// Change user role request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangeRoleRequest {
  /// Role to assign to the user
  #[schema(example = "resource_manager")]
  pub role: String,
}

/// List users
#[utoipa::path(
    get,
    path = "/bodhi/v1/users",
    tag = API_TAG_AUTH,
    operation_id = "listUsers",
    summary = "List users",
    description = "List all users with roles and status information. Available to managers and admins.",
    params(
        ("page" = Option<u32>, Query, description = "Page number (1-based)", example = 1),
        ("page_size" = Option<u32>, Query, description = "Number of users per page", example = 10)
    ),
    responses(
        (status = 200, description = "Users retrieved successfully", body = UserListResponse),
        (status = 400, description = "Invalid request parameters", body = OpenAIApiError),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:manager", "role:admin"])
    )
)]
pub async fn list_users_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<ListUsersParams>,
) -> Result<Json<UserListResponse>, ApiError> {
  // Get reviewer token from headers
  let token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  // Call auth service to list users
  let auth_service = state.app_service().auth_service();
  let users = auth_service
    .list_users(token, params.page, params.page_size)
    .await
    .map_err(|e| {
      error!("Failed to list users from auth service: {}", e);
      InternalServerError::new(format!("Failed to list users: {}", e))
    })?;

  info!("Successfully retrieved {} users", users.users.len());
  Ok(Json(users))
}

/// Change user role
#[utoipa::path(
    put,
    path = "/bodhi/v1/users/{user_id}/role",
    tag = API_TAG_AUTH,
    operation_id = "changeUserRole",
    summary = "Change user role",
    description = "Assign a new role to a user. Admins can assign any role, managers can assign user/power_user/manager roles.",
    params(
        ("user_id" = String, Path, description = "User ID to change role for")
    ),
    request_body = ChangeRoleRequest,
    responses(
        (status = 200, description = "Role changed successfully"),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError),
        (status = 404, description = "User not found", body = OpenAIApiError),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:manager", "role:admin"])
    )
)]
pub async fn change_user_role_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Path(user_id): Path<String>,
  Json(request): Json<ChangeRoleRequest>,
) -> Result<StatusCode, ApiError> {
  let token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let claims: Claims = extract_claims::<Claims>(token)?;

  info!(
    "User {} changing role for user {} to role {}",
    claims.preferred_username, user_id, request.role
  );

  let auth_service = state.app_service().auth_service();
  auth_service
    .assign_user_role(token, &user_id, &request.role)
    .await
    .map_err(|e| {
      error!("Failed to change user role: {}", e);
      InternalServerError::new(format!("Failed to change user role: {}", e))
    })?;

  info!(
    "Successfully changed role for user {} to {} by {}",
    user_id, request.role, claims.preferred_username
  );
  Ok(StatusCode::OK)
}

/// Remove user
#[utoipa::path(
    delete,
    path = "/bodhi/v1/users/{user_id}",
    tag = API_TAG_AUTH,
    operation_id = "removeUser",
    summary = "Remove user access",
    description = "Remove a user's access to the application. Only admins can remove users.",
    params(
        ("user_id" = String, Path, description = "User ID to remove")
    ),
    responses(
        (status = 200, description = "User removed successfully"),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError),
        (status = 404, description = "User not found", body = OpenAIApiError),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:admin"])
    )
)]
pub async fn remove_user_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Path(user_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let claims: Claims = extract_claims::<Claims>(token)?;

  info!(
    "User {} removing user {}",
    claims.preferred_username, user_id
  );

  let auth_service = state.app_service().auth_service();
  auth_service
    .remove_user(token, &user_id)
    .await
    .map_err(|e| {
      error!("Failed to remove user: {}", e);
      InternalServerError::new(format!("Failed to remove user: {}", e))
    })?;

  info!(
    "Successfully removed user {} by {}",
    user_id, claims.preferred_username
  );
  Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
  use super::*;
  use auth_middleware::{
    KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USERNAME, KEY_HEADER_BODHIAPP_USER_ID,
  };
  use axum::{body::Body, http::Request, routing::get, Router};
  use objs::test_utils::temp_bodhi_home;
  use rstest::rstest;
  use server_core::{DefaultRouterState, MockSharedContext};
  use services::{test_utils::AppServiceStubBuilder, MockAuthService, UserInfoResponse};
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
        UserInfoResponse {
          user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
          username: "admin@example.com".to_string(),
          first_name: Some("Admin".to_string()),
          last_name: Some("User".to_string()),
          role: "resource_admin".to_string(),
        },
        UserInfoResponse {
          user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
          username: "user@example.com".to_string(),
          first_name: Some("Regular".to_string()),
          last_name: Some("User".to_string()),
          role: "resource_user".to_string(),
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
        Err(services::AuthServiceError::AuthServiceApiError(
          "Invalid session [RequestID: 123456]".to_string(),
        ))
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
    assert!(body_text.contains("No authentication token present"));

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
}
