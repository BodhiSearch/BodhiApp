use crate::{
  EmptyResponse, PaginationSortParams, ENDPOINT_ACCESS_REQUESTS_ALL,
  ENDPOINT_ACCESS_REQUESTS_PENDING, ENDPOINT_USER_REQUEST_ACCESS, ENDPOINT_USER_REQUEST_STATUS,
};
use auth_middleware::{KEY_RESOURCE_ROLE, KEY_RESOURCE_TOKEN, KEY_RESOURCE_USER_ID};
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
  /// Email of the requesting user
  pub email: String,
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
      email: request.email,
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
  // Extract token from headers
  let Some(token) = headers.get(KEY_RESOURCE_TOKEN) else {
    return Err(BadRequestError::new(
      "No authentication token present".to_string(),
    ))?;
  };

  let token = token
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  if token.is_empty() {
    return Err(BadRequestError::new(
      "Authentication token is empty".to_string(),
    ))?;
  }

  let claims: Claims = extract_claims::<Claims>(token)?;
  let email = claims.email.clone();
  let user_id = claims.sub.clone();

  info!("User {} requesting access", email);

  // Check if user already has a role
  if let Some(role_header) = headers.get(KEY_RESOURCE_ROLE) {
    let role = role_header
      .to_str()
      .map_err(|err| BadRequestError::new(err.to_string()))?;

    if !role.is_empty() {
      debug!("User {} already has role: {}", email, role);
      return Err(UnprocessableEntityError::new(
        "User already has access".to_string(),
      ))?;
    }
  }

  let db_service = state.app_service().db_service();

  // Check for existing pending request
  if db_service
    .get_pending_request(user_id.clone())
    .await?
    .is_some()
  {
    debug!("User {} already has pending request", email);
    return Err(ConflictError::new(
      "Access request already pending".to_string(),
    ))?;
  }

  // Create new access request
  let _ = db_service
    .insert_pending_request(email.clone(), user_id.clone())
    .await?;

  info!("Access request created for user {}", email);
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
  let Some(user_id) = headers.get(KEY_RESOURCE_USER_ID) else {
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

  // For now, this uses the same method as pending requests
  // In production, we'd need a separate method to get all requests
  list_pending_requests_handler(State(state), Query(params)).await
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
    .get(KEY_RESOURCE_ROLE)
    .ok_or_else(|| BadRequestError::new("No role header present".to_string()))?;

  let approver_role_str = role_header
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let approver_role = approver_role_str.parse::<Role>()?;

  // Extract approver's email for logging
  let token = headers
    .get(KEY_RESOURCE_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let claims: Claims = extract_claims::<Claims>(token)?;

  info!(
    "User {} with role {:?} approving request {} with role {:?}",
    claims.email, approver_role, id, request.role
  );

  // Validate role hierarchy - users can only assign roles equal to or lower than their own
  if !approver_role.has_access_to(&request.role) {
    error!(
      "User {} with role {:?} cannot assign role {:?}",
      claims.email, approver_role, request.role
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
    .update_request_status(id, UserAccessRequestStatus::Approved, claims.email.clone())
    .await?;

  // Phase 4 - Call auth service to assign role to user
  let auth_service = state.app_service().auth_service();
  let role_name = request.role.to_string();

  auth_service
    .assign_user_role(token, &access_request.user_id, &role_name)
    .await?;

  info!(
    "Access request {} approved by {}, user {} assigned role {}",
    id, claims.email, access_request.email, role_name
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
    .get(KEY_RESOURCE_TOKEN)
    .ok_or_else(|| BadRequestError::new("No authentication token present".to_string()))?
    .to_str()
    .map_err(|err| BadRequestError::new(err.to_string()))?;

  let claims: Claims = extract_claims::<Claims>(token)?;

  info!("User {} rejecting access request {}", claims.email, id);

  // Update request status to rejected
  let db_service = state.app_service().db_service();
  db_service
    .update_request_status(id, UserAccessRequestStatus::Rejected, claims.email.clone())
    .await?;

  info!("Access request {} rejected by {}", id, claims.email);
  Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_user_access_status_response_from_user_access_request() {
    // Test DTO conversion
    let request = UserAccessRequest {
      id: 1,
      email: "test@example.com".to_string(),
      user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
      reviewer: None,
      status: UserAccessRequestStatus::Pending,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    };

    let response = UserAccessStatusResponse::from(request.clone());

    assert_eq!(response.email, request.email);
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
}
