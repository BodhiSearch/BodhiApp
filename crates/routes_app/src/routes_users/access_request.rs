use crate::{
  AccessRequestError, ApproveUserAccessRequest, PaginatedUserAccessResponse, PaginationSortParams,
  UserAccessStatusResponse, ENDPOINT_ACCESS_REQUESTS_ALL, ENDPOINT_ACCESS_REQUESTS_PENDING,
  ENDPOINT_USER_REQUEST_ACCESS, ENDPOINT_USER_REQUEST_STATUS,
};
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::Json,
  Extension,
};
use objs::{ApiError, OpenAIApiError, API_TAG_AUTH};
use server_core::RouterState;
use services::{db::UserAccessRequestStatus, extract_claims, Claims};
use std::sync::Arc;
use tracing::{debug, error, info};

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
        (status = 201, description = "Access request created successfully"),
        (status = 409, description = "Pending request already exists", body = OpenAIApiError),
        (status = 422, description = "User already has role", body = OpenAIApiError),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn user_request_access_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<StatusCode, ApiError> {
  // Session auth: extract username, user_id, and role
  let AuthContext::Session {
    ref user_id,
    ref username,
    ref role,
    ..
  } = auth_context
  else {
    return Err(AccessRequestError::AlreadyHasAccess)?;
  };

  // Check if user already has a role
  if let Some(role) = role {
    debug!("User {} already has role: {}", username, role);
    return Err(AccessRequestError::AlreadyHasAccess)?;
  }

  let db_service = state.app_service().db_service();

  // Check for existing pending request
  if db_service
    .get_pending_request(user_id.clone())
    .await?
    .is_some()
  {
    debug!("User {} already has pending request", username);
    return Err(AccessRequestError::AlreadyPending)?;
  }

  // Create new access request
  let _ = db_service
    .insert_pending_request(username.to_string(), user_id.clone())
    .await?;

  debug!("Access request created for user {}", username);
  Ok(StatusCode::CREATED)
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
        (status = 404, description = "Request not found", body = OpenAIApiError),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn request_status_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<UserAccessStatusResponse>, ApiError> {
  let Some(user_id) = auth_context.user_id() else {
    return Err(AccessRequestError::PendingRequestNotFound)?;
  };
  debug!("Checking access request status for user {}", user_id);
  let db_service = state.app_service().db_service();
  if let Some(request) = db_service.get_pending_request(user_id.to_string()).await? {
    Ok(Json(UserAccessStatusResponse::from(request)))
  } else {
    Err(AccessRequestError::PendingRequestNotFound)?
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
    ),
    security(
        ("session_auth" = ["resource_manager"])
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
    ),
    security(
        ("session_auth" = ["resource_manager"])
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
    .map_err(|e| AccessRequestError::FetchFailed(e.to_string()))?;

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
        (status = 404, description = "Request not found", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_manager"])
    )
)]
pub async fn approve_request_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<i64>,
  Json(request): Json<ApproveUserAccessRequest>,
) -> Result<StatusCode, ApiError> {
  let AuthContext::Session {
    ref username,
    role: Some(ref approver_role),
    ref token,
    ..
  } = auth_context
  else {
    return Err(AccessRequestError::InsufficientPrivileges)?;
  };
  let approver_username = username;

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
    return Err(AccessRequestError::InsufficientPrivileges)?;
  }

  let db_service = state.app_service().db_service();

  // Get the request details to obtain the user's email
  let access_request = db_service
    .get_request_by_id(id)
    .await?
    .ok_or(AccessRequestError::RequestNotFound(id))?;

  // Update request status to approved
  db_service
    .update_request_status(
      id,
      UserAccessRequestStatus::Approved,
      approver_username.to_string(),
    )
    .await?;

  // Call auth service to assign role to user
  let auth_service = state.app_service().auth_service();
  let role_name = request.role.to_string();

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
        (status = 404, description = "Request not found", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["resource_manager"])
    )
)]
pub async fn reject_request_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
  let token = auth_context.token().expect("requires auth middleware");
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
