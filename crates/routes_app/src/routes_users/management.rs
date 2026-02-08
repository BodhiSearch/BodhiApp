use crate::{ChangeRoleRequest, ListUsersParams, UserRouteError};
use auth_middleware::ExtractToken;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};
use objs::{ApiError, OpenAIApiError, API_TAG_AUTH};
use server_core::RouterState;
use services::{extract_claims, Claims, UserListResponse};
use std::sync::Arc;
use tracing::{error, info, warn};

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
    ),
    security(
        ("session_auth" = ["resource_manager"])
    )
)]
pub async fn list_users_handler(
  ExtractToken(token): ExtractToken,
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<ListUsersParams>,
) -> Result<Json<UserListResponse>, ApiError> {
  // Call auth service to list users
  let auth_service = state.app_service().auth_service();
  let users = auth_service
    .list_users(&token, params.page, params.page_size)
    .await
    .map_err(|e| {
      error!("Failed to list users from auth service: {}", e);
      UserRouteError::ListFailed(e.to_string())
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
        (status = 404, description = "User not found", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_manager"])
    )
)]
pub async fn change_user_role_handler(
  ExtractToken(token): ExtractToken,
  State(state): State<Arc<dyn RouterState>>,
  Path(user_id): Path<String>,
  Json(request): Json<ChangeRoleRequest>,
) -> Result<StatusCode, ApiError> {
  let claims: Claims = extract_claims::<Claims>(&token)?;

  info!(
    "User {} changing role for user {} to role {}",
    claims.preferred_username, user_id, request.role
  );

  let auth_service = state.app_service().auth_service();
  auth_service
    .assign_user_role(&token, &user_id, &request.role)
    .await
    .map_err(|e| {
      error!("Failed to change user role: {}", e);
      UserRouteError::RoleChangeFailed(e.to_string())
    })?;

  // Clear existing sessions for the user to ensure new role is applied
  // Note: We don't fail the operation if session clearing fails, just log it
  let session_service = state.app_service().session_service();
  match session_service.clear_sessions_for_user(&user_id).await {
    Ok(cleared_count) => {
      info!(
        "Successfully changed role for user {} to {} by {}, cleared {} sessions",
        user_id, request.role, claims.preferred_username, cleared_count
      );
    }
    Err(e) => {
      warn!(
        "Changed role for user {} to {} by {}, but failed to clear sessions: {}",
        user_id, request.role, claims.preferred_username, e
      );
    }
  }

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
        (status = 404, description = "User not found", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn remove_user_handler(
  ExtractToken(token): ExtractToken,
  State(state): State<Arc<dyn RouterState>>,
  Path(user_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let claims: Claims = extract_claims::<Claims>(&token)?;

  info!(
    "User {} removing user {}",
    claims.preferred_username, user_id
  );

  let auth_service = state.app_service().auth_service();
  auth_service
    .remove_user(&token, &user_id)
    .await
    .map_err(|e| {
      error!("Failed to remove user: {}", e);
      UserRouteError::RemoveFailed(e.to_string())
    })?;

  info!(
    "Successfully removed user {} by {}",
    user_id, claims.preferred_username
  );
  Ok(StatusCode::OK)
}
