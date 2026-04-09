use crate::users::error::UsersRouteError;
use crate::users::users_api_schemas::ListUsersParams;
use crate::{ApiError, AuthScope, BodhiApiError, ValidatedJson, API_TAG_AUTH};
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  Json,
};
use services::{ChangeRoleRequest, UserListResponse};
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
        (status = 404, description = "User not found", body = BodhiApiError),
    ),
    security(
        ("session_auth" = ["resource_manager"])
    )
)]
pub async fn users_change_role(
  auth_scope: AuthScope,
  Path(user_id): Path<String>,
  ValidatedJson(request): ValidatedJson<ChangeRoleRequest>,
) -> Result<StatusCode, ApiError> {
  // Validate role hierarchy: caller's role must be >= target role
  let caller_role = auth_scope
    .auth_context()
    .resource_role()
    .ok_or(UsersRouteError::InsufficientPrivileges)?;
  // Reject Anonymous/Guest as assignment targets
  if !request.role.has_access_to(&services::ResourceRole::User) {
    return Err(UsersRouteError::InsufficientPrivileges)?;
  }
  if !caller_role.has_access_to(&request.role) {
    warn!(
      "Role hierarchy violation: caller role {:?} cannot assign {:?}",
      caller_role, request.role
    );
    return Err(UsersRouteError::InsufficientPrivileges)?;
  }

  let role_name = request.role.to_string();
  auth_scope
    .users()
    .assign_user_role(&user_id, &role_name)
    .await
    .map_err(|e| {
      error!("Failed to change user role: {}", e);
      UsersRouteError::RoleChangeFailed(e.to_string())
    })?;

  // Clear existing sessions for the user to ensure new role is applied
  // Note: We don't fail the operation if session clearing fails, just log it
  match auth_scope
    .session_service()
    .clear_sessions_for_user(&user_id)
    .await
  {
    Ok(cleared_count) => {
      info!(
        "Successfully changed role for user {} to {}, cleared {} sessions",
        user_id, role_name, cleared_count
      );
    }
    Err(e) => {
      warn!(
        "Changed role for user {} to {}, but failed to clear sessions: {}",
        user_id, role_name, e
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
    description = "Remove a user's access to the application. Only managers or above can remove users.",
    params(
        ("user_id" = String, Path, description = "User ID to remove")
    ),
    responses(
        (status = 200, description = "User removed successfully"),
        (status = 404, description = "User not found", body = BodhiApiError),
    ),
    security(
        ("session_auth" = ["resource_manager"])
    )
)]
pub async fn users_destroy(
  auth_scope: AuthScope,
  Path(user_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  // Role ceiling check: caller cannot delete users with higher privilege (AUTHZ-VULN-06)
  // Fetch the target user's role, then compare against caller's role
  let caller_role = auth_scope
    .auth_context()
    .resource_role()
    .ok_or(UsersRouteError::InsufficientPrivileges)?;
  let target_user = auth_scope.users().get_user(&user_id).await?;
  if let Some(target_user) = target_user {
    if let Some(services::AppRole::Session(target_role)) = &target_user.role {
      if !caller_role.has_access_to(target_role) {
        warn!(
          "Role ceiling violation: caller role {:?} cannot delete user with role {:?}",
          caller_role, target_role
        );
        return Err(UsersRouteError::InsufficientPrivileges.into());
      }
    }
  }
  // target_user is None: user not found in Keycloak — proceed with deletion
  // as orphan cleanup (by-design: stale local records should be removable)

  auth_scope
    .users()
    .remove_user(&user_id)
    .await
    .map_err(|e| {
      error!("Failed to remove user: {}", e);
      UsersRouteError::RemoveFailed(e.to_string())
    })?;

  info!("Successfully removed user {}", user_id);
  Ok(StatusCode::OK)
}

#[cfg(test)]
#[path = "test_management_crud.rs"]
mod test_management_crud;
