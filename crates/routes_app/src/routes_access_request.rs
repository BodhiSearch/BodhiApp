use crate::{
  EmptyResponse, PaginationSortParams, ENDPOINT_ACCESS_REQUESTS_ALL,
  ENDPOINT_ACCESS_REQUESTS_PENDING, ENDPOINT_USER_REQUEST_ACCESS, ENDPOINT_USER_REQUEST_STATUS,
};
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USERNAME,
  KEY_HEADER_BODHIAPP_USER_ID,
};
use axum::{
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
  response::Json,
};
use chrono::{DateTime, Utc};
use objs::{
  ApiError, BadRequestError, ConflictError, InternalServerError, NotFoundError, OpenAIApiError,
  Role, UnauthorizedError, UnprocessableEntityError, API_TAG_AUTH,
};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  db::{UserAccessRequest, UserAccessRequestStatus},
  extract_claims, Claims,
};
use std::sync::Arc;
use tracing::{debug, error, info};
use utoipa::ToSchema;

// DTOs for access request endpoints

/// Response for checking access request status
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "status": "pending",
    "created_at": "2024-01-01T12:00:00Z"
}))]
pub struct UserAccessStatusResponse {
  /// Username of the requesting user
  pub username: String,
  /// Current status of the request (pending, approved, rejected)
  pub status: UserAccessRequestStatus,
  /// Creation timestamp
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<UserAccessRequest> for UserAccessStatusResponse {
  fn from(request: UserAccessRequest) -> Self {
    Self {
      username: request.username,
      status: request.status,
      created_at: request.created_at,
      updated_at: request.updated_at,
    }
  }
}

/// Request body for approving access with role assignment
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "role": "resource_user"
}))]
pub struct ApproveUserAccessRequest {
  /// Role to assign to the user
  pub role: Role,
}

/// Paginated response for access requests
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "requests": [{
        "id": 1,
        "email": "user@example.com",
        "reviewer": null,
        "status": "pending",
        "created_at": "2024-01-01T12:00:00Z",
        "updated_at": "2024-01-01T12:00:00Z"
    }],
    "total": 1,
    "page": 1,
    "page_size": 20
}))]
pub struct PaginatedUserAccessResponse {
  /// List of access requests
  pub requests: Vec<UserAccessRequest>,
  /// Total number of requests
  pub total: usize,
  /// Current page number
  pub page: usize,
  /// Number of items per page
  pub page_size: usize,
}

// User endpoints

/// Request access to the system
#[utoipa::path(
    post,
    path = ENDPOINT_USER_REQUEST_ACCESS,
    tag = API_TAG_AUTH,
    operation_id = "requestUserAccess",
    summary = "Request User Access",
    description = "Authenticated users without roles can request access to the system. Only one pending request is allowed per user.",
    responses(
        (status = 201, description = "Access request created successfully", body = EmptyResponse),
        (status = 409, description = "Pending request already exists", body = OpenAIApiError),
        (status = 422, description = "User already has role", body = OpenAIApiError),
        (status = 401, description = "Not authenticated", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn user_request_access_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<(StatusCode, Json<EmptyResponse>), ApiError> {
  // Extract username from headers
  let Some(username) = headers.get(KEY_HEADER_BODHIAPP_USERNAME) else {
    return Err(BadRequestError::new(
      "User logged in information not present".to_string(),
    ))?;
  };

  let username = username
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  if username.is_empty() {
    return Err(BadRequestError::new(
      "User logged in information is empty".to_string(),
    ))?;
  }

  // Extract user_id from headers
  let Some(user_id) = headers.get(KEY_HEADER_BODHIAPP_USER_ID) else {
    return Err(BadRequestError::new("User ID not present".to_string()))?;
  };

  let user_id = user_id
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  // Check if user already has a role
  if let Some(role_header) = headers.get(KEY_HEADER_BODHIAPP_ROLE) {
    let role = role_header
      .to_str()
      .map_err(|err| BadRequestError::new(err.to_string()))?;

    if !role.is_empty() {
      debug!("User {} already has role: {}", username, role);
      return Err(UnprocessableEntityError::new(
        "User already has access".to_string(),
      ))?;
    }
  }

  let db_service = state.app_service().db_service();

  // Check for existing pending request
  if db_service
    .get_pending_request(user_id.to_string())
    .await?
    .is_some()
  {
    debug!("User {} already has pending request", username);
    return Err(ConflictError::new(
      "Access request already pending".to_string(),
    ))?;
  }

  // Create new access request
  let _ = db_service
    .insert_pending_request(username.to_string(), user_id.to_string())
    .await?;

  debug!("Access request created for user {}", username);
  Ok((StatusCode::CREATED, Json(EmptyResponse {})))
}

/// Check access request status
#[utoipa::path(
    get,
    path = ENDPOINT_USER_REQUEST_STATUS,
    tag = API_TAG_AUTH,
    operation_id = "getUserAccessStatus",
    summary = "Get Access Request Status",
    description = "Check the status of the current user's access request.",
    responses(
        (status = 200, description = "Request status retrieved", body = UserAccessStatusResponse),
        (status = 400, description = "Bad Request", body = OpenAIApiError),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 404, description = "Request not found", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn request_status_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<UserAccessStatusResponse>, ApiError> {
  let Some(user_id) = headers.get(KEY_HEADER_BODHIAPP_USER_ID) else {
    return Err(UnauthorizedError::new("user not found".to_string()))?;
  };
  let user_id = user_id
    .to_str()
    .map_err(|err| InternalServerError::new(err.to_string()))?;
  debug!("Checking access request status for user {}", user_id);
  let db_service = state.app_service().db_service();
  if let Some(request) = db_service.get_pending_request(user_id.to_string()).await? {
    Ok(Json(UserAccessStatusResponse::from(request)))
  } else {
    Err(NotFoundError::new(
      "pending access request for user not found".to_string(),
    ))?
  }
}

// Admin/Manager endpoints

/// List pending access requests
#[utoipa::path(
    get,
    path = ENDPOINT_ACCESS_REQUESTS_PENDING,
    tag = API_TAG_AUTH,
    operation_id = "listPendingAccessRequests",
    summary = "List Pending Access Requests",
    description = "List all pending access requests. Requires manager or admin role.",
    params(PaginationSortParams),
    responses(
        (status = 200, description = "Pending requests retrieved", body = PaginatedUserAccessResponse),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:manager", "role:admin"])
    )
)]
pub async fn list_pending_requests_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedUserAccessResponse>, ApiError> {
  debug!(
    "Listing pending access requests with pagination: {:?}",
    params
  );

  let db_service = state.app_service().db_service();
  let page_size = params.page_size.min(100);
  let page = params.page as u32;

  // Get pending requests with pagination
  let (requests, total) = db_service
    .list_pending_requests(page, page_size as u32)
    .await?;

  Ok(Json(PaginatedUserAccessResponse {
    requests,
    total,
    page: params.page,
    page_size,
  }))
}

/// List all access requests
#[utoipa::path(
    get,
    path = ENDPOINT_ACCESS_REQUESTS_ALL,
    tag = API_TAG_AUTH,
    operation_id = "listAllAccessRequests",
    summary = "List All Access Requests",
    description = "List all access requests regardless of status. Requires manager or admin role.",
    params(PaginationSortParams),
    responses(
        (status = 200, description = "All requests retrieved", body = PaginatedUserAccessResponse),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:manager", "role:admin"])
    )
)]
pub async fn list_all_requests_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedUserAccessResponse>, ApiError> {
  debug!("Listing all access requests with pagination: {:?}", params);

  let db_service = state.app_service().db_service();
  let (requests, total) = db_service
    .list_all_requests(params.page as u32, params.page_size as u32)
    .await
    .map_err(|e| InternalServerError::new(format!("Failed to fetch all access requests: {}", e)))?;

  Ok(Json(PaginatedUserAccessResponse {
    page: params.page,
    page_size: params.page_size,
    total,
    requests,
  }))
}

/// Approve access request
#[utoipa::path(
    post,
    path = ENDPOINT_ACCESS_REQUESTS_ALL.to_owned() + "/{id}/approve",
    tag = API_TAG_AUTH,
    operation_id = "approveAccessRequest",
    summary = "Approve Access Request",
    description = "Approve an access request and assign a role. Requires manager or admin role.",
    params(
        ("id" = i64, Path, description = "Access request ID")
    ),
    request_body(
        content = ApproveUserAccessRequest,
        description = "Role to assign to the user"
    ),
    responses(
        (status = 200, description = "Request approved successfully"),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError),
        (status = 404, description = "Request not found", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:manager", "role:admin"])
    )
)]
pub async fn approve_request_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<i64>,
  Json(request): Json<ApproveUserAccessRequest>,
) -> Result<StatusCode, ApiError> {
  // Extract approver's role from headers
  let role_header = headers
    .get(KEY_HEADER_BODHIAPP_ROLE)
    .ok_or_else(|| BadRequestError::new("No role header present".to_string()))?;

  let approver_role_str = role_header
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let approver_role = approver_role_str.parse::<Role>()?;

  // Extract approver's username from headers
  let approver_username = headers
    .get(KEY_HEADER_BODHIAPP_USERNAME)
    .ok_or_else(|| BadRequestError::new("No username header present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  info!(
    "User {} with role {:?} approving request {} with role {:?}",
    approver_username, approver_role, id, request.role
  );

  // Validate role hierarchy - users can only assign roles equal to or lower than their own
  if !approver_role.has_access_to(&request.role) {
    error!(
      "User {} with role {:?} cannot assign role {:?}",
      approver_username, approver_role, request.role
    );
    return Err(BadRequestError::new(
      "Insufficient privileges to assign this role".to_string(),
    ))?;
  }

  let db_service = state.app_service().db_service();

  // Get the request details to obtain the user's email
  let access_request = db_service
    .get_request_by_id(id)
    .await?
    .ok_or_else(|| BadRequestError::new(format!("Access request {} not found", id)))?;

  // Update request status to approved
  db_service
    .update_request_status(
      id,
      UserAccessRequestStatus::Approved,
      approver_username.to_string(),
    )
    .await?;

  // Phase 4 - Call auth service to assign role to user
  let auth_service = state.app_service().auth_service();
  let role_name = request.role.to_string();

  // Get token from header for auth service call
  let token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  auth_service
    .assign_user_role(token, &access_request.user_id, &role_name)
    .await?;

  // Clear existing sessions for the user to ensure new role is applied
  let session_service = state.app_service().session_service();

  let cleared_sessions = session_service
    .clear_sessions_for_user(&access_request.user_id)
    .await?;

  info!(
    "Access request {} approved by {}, user {} assigned role {}, cleared {} sessions",
    id, approver_username, access_request.username, role_name, cleared_sessions
  );
  Ok(StatusCode::OK)
}

/// Reject access request
#[utoipa::path(
    post,
    path = ENDPOINT_ACCESS_REQUESTS_ALL.to_owned() + "/{id}/reject",
    tag = API_TAG_AUTH,
    operation_id = "rejectAccessRequest",
    summary = "Reject Access Request",
    description = "Reject an access request. Requires manager or admin role.",
    params(
        ("id" = i64, Path, description = "Access request ID")
    ),
    responses(
        (status = 200, description = "Request rejected successfully"),
        (status = 401, description = "Not authenticated", body = OpenAIApiError),
        (status = 403, description = "Insufficient permissions", body = OpenAIApiError),
        (status = 404, description = "Request not found", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["role:manager", "role:admin"])
    )
)]
pub async fn reject_request_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
  // Extract rejector's email for logging
  let token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let claims: Claims = extract_claims::<Claims>(token)?;

  info!(
    "User {} rejecting access request {}",
    claims.preferred_username, id
  );

  // Update request status to rejected
  let db_service = state.app_service().db_service();
  db_service
    .update_request_status(
      id,
      UserAccessRequestStatus::Rejected,
      claims.preferred_username.clone(),
    )
    .await?;

  info!(
    "Access request {} rejected by {}",
    id, claims.preferred_username
  );
  Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow_trace::anyhow_trace;
  use auth_middleware::{
    KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USERNAME,
    KEY_HEADER_BODHIAPP_USER_ID,
  };
  use axum::{body::Body, http::Request, routing::post, Router};
  use objs::test_utils::{setup_l10n, temp_bodhi_home};
  use rstest::rstest;
  use serde_json::json;
  use server_core::{DefaultRouterState, MockSharedContext};
  use services::{
    db::DbService,
    test_utils::{test_db_service_with_temp_dir, AppServiceStubBuilder, SecretServiceStub},
    AppRegInfo, MockAuthService, SessionService, SqliteSessionService,
  };
  use std::{collections::HashMap, fs::File, sync::Arc};
  use tempfile::TempDir;
  use time::OffsetDateTime;
  use tower::ServiceExt;
  use tower_sessions::{
    session::{Id, Record},
    SessionStore,
  };

  #[test]
  fn test_user_access_status_response_from_user_access_request() {
    // Test DTO conversion
    let request = UserAccessRequest {
      id: 1,
      username: "test@example.com".to_string(),
      user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
      reviewer: None,
      status: UserAccessRequestStatus::Pending,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    };

    let response = UserAccessStatusResponse::from(request.clone());

    assert_eq!(response.username, request.username);
    assert_eq!(response.status, request.status);
    assert_eq!(response.created_at, request.created_at);
    assert_eq!(response.updated_at, request.updated_at);
  }

  #[test]
  fn test_approve_user_access_request_serde() {
    // Test request deserialization
    let json = r#"{"role": "resource_user"}"#;
    let request: ApproveUserAccessRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.role, Role::User);

    let json = r#"{"role": "resource_admin"}"#;
    let request: ApproveUserAccessRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.role, Role::Admin);
  }

  #[test]
  fn test_paginated_user_access_response_serde() {
    // Test response serialization
    let response = PaginatedUserAccessResponse {
      requests: vec![],
      total: 0,
      page: 1,
      page_size: 20,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"requests\":[]"));
    assert!(json.contains("\"total\":0"));
    assert!(json.contains("\"page\":1"));
    assert!(json.contains("\"page_size\":20"));
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_approve_request_clears_user_sessions(
    #[from(setup_l10n)] _setup_l10n: &std::sync::Arc<objs::FluentLocalizationService>,
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    // 1. Setup: Create real databases for both app and session
    let session_db = temp_bodhi_home.path().join("session.sqlite");

    // 2. Create services with real databases
    File::create(&session_db)?;

    let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
    let session_service =
      Arc::new(SqliteSessionService::build_session_service(session_db.clone()).await);

    // 3. Create a pending access request for a user
    let user_id = "test-user-123";
    let username = "testuser@example.com";
    let access_request = db_service
      .insert_pending_request(username.to_string(), user_id.to_string())
      .await?;

    // 4. Simulate user having multiple active sessions
    // (as if they logged in from different devices/browsers)
    for i in 0..3 {
      let id = Id::default();
      let mut data = HashMap::new();
      data.insert(
        "user_id".to_string(),
        serde_json::Value::String(user_id.to_string()),
      );
      data.insert(
        "access_token".to_string(),
        serde_json::Value::String(format!("token_{}", i)),
      );
      data.insert(
        "device".to_string(),
        serde_json::Value::String(format!("device_{}", i)),
      );

      let record = Record {
        id: id.clone(),
        data,
        expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
      };

      SessionStore::save(&session_service.session_store, &record).await?;
    }

    // 5. Verify sessions exist before approval
    let count_before = session_service
      .session_store
      .count_sessions_for_user(user_id)
      .await?;

    assert_eq!(3, count_before, "User should have 3 active sessions");

    // 6. Setup mock auth service for role assignment
    let mut mock_auth = MockAuthService::default();
    mock_auth
      .expect_assign_user_role()
      .times(1)
      .withf(|_token, uid, role| uid == "test-user-123" && role == "resource_user")
      .return_once(|_, _, _| Ok(()));

    // 7. Setup secret service with app registration info
    let secret_service = SecretServiceStub::default().with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_secret".to_string(),
    });

    // 8. Build complete app service
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_sqlite_session_service(session_service.clone())
      .auth_service(Arc::new(mock_auth))
      .secret_service(Arc::new(secret_service))
      .build()?;

    // 9. Create router with approve endpoint
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service),
    ));

    // 9. Create router with approve endpoint (no auth middleware needed)
    let router = Router::new()
      .route(
        &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
        post(approve_request_handler),
      )
      .with_state(state.clone());

    // 10. Make HTTP request with required headers (simulating authenticated admin)
    let request = Request::post(&format!(
      "{}/{}/approve",
      ENDPOINT_ACCESS_REQUESTS_ALL, access_request.id
    ))
    .header(KEY_HEADER_BODHIAPP_ROLE, "resource_manager")
    .header(KEY_HEADER_BODHIAPP_TOKEN, "dummy-admin-token")
    .header(KEY_HEADER_BODHIAPP_USERNAME, "admin@example.com")
    .header(KEY_HEADER_BODHIAPP_USER_ID, "admin-user-id")
    .header("content-type", "application/json")
    .body(Body::from(serde_json::to_string(
      &json!({ "role": "resource_user" }),
    )?))
    .unwrap();

    // Send request through the router
    let response = router.oneshot(request).await?;

    // Verify the handler succeeded
    assert_eq!(
      axum::http::StatusCode::OK,
      response.status(),
      "Handler should return OK status"
    );

    // 12. Verify all user sessions were cleared
    let session_store = session_service.get_session_store();
    let count_after = session_store.count_sessions_for_user(user_id).await?;

    assert_eq!(
      0, count_after,
      "All user sessions should be cleared after role assignment"
    );

    // 13. Verify request status was updated
    let updated_request = state
      .app_service()
      .db_service()
      .get_request_by_id(access_request.id)
      .await?
      .unwrap();
    assert_eq!(UserAccessRequestStatus::Approved, updated_request.status);
    assert_eq!(
      Some("admin@example.com".to_string()),
      updated_request.reviewer
    );

    Ok(())
  }
}
