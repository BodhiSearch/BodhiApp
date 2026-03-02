use crate::users::error::UsersRouteError;
use crate::users::users_api_schemas::{ChangeRoleRequest, ListUsersParams};
use crate::{ApiError, AuthScope, OpenAIApiError, API_TAG_AUTH};
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  Json,
};
use services::UserListResponse;
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
pub async fn users_index(
  auth_scope: AuthScope,
  Query(params): Query<ListUsersParams>,
) -> Result<Json<UserListResponse>, ApiError> {
  let users = auth_scope
    .users()
    .list_users(params.page, params.page_size)
    .await
    .map_err(|e| {
      error!("Failed to list users from auth service: {}", e);
      UsersRouteError::ListFailed(e.to_string())
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
pub async fn users_change_role(
  auth_scope: AuthScope,
  Path(user_id): Path<String>,
  Json(request): Json<ChangeRoleRequest>,
) -> Result<StatusCode, ApiError> {
  let requester_id = auth_scope.require_user_id()?;

  info!(
    "User {} changing role for user {} to role {}",
    requester_id, user_id, request.role
  );

  auth_scope
    .users()
    .assign_user_role(&user_id, &request.role)
    .await
    .map_err(|e| {
      error!("Failed to change user role: {}", e);
      UsersRouteError::RoleChangeFailed(e.to_string())
    })?;

  // Clear existing sessions for the user to ensure new role is applied
  // Note: We don't fail the operation if session clearing fails, just log it
  match auth_scope.session_service().clear_sessions_for_user(&user_id).await {
    Ok(cleared_count) => {
      info!(
        "Successfully changed role for user {} to {} by {}, cleared {} sessions",
        user_id, request.role, requester_id, cleared_count
      );
    }
    Err(e) => {
      warn!(
        "Changed role for user {} to {} by {}, but failed to clear sessions: {}",
        user_id, request.role, requester_id, e
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
pub async fn users_destroy(
  auth_scope: AuthScope,
  Path(user_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let requester_id = auth_scope.require_user_id()?;

  info!("User {} removing user {}", requester_id, user_id);

  auth_scope
    .users()
    .remove_user(&user_id)
    .await
    .map_err(|e| {
      error!("Failed to remove user: {}", e);
      UsersRouteError::RemoveFailed(e.to_string())
    })?;

  info!("Successfully removed user {} by {}", user_id, requester_id);
  Ok(StatusCode::OK)
}

#[cfg(test)]
#[path = "test_management_crud.rs"]
mod test_management_crud;
