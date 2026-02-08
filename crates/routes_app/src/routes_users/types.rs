use chrono::{DateTime, Utc};
use objs::{
  AppError, ErrorType, ResourceRole, TokenScope, UserInfo,
};
use serde::{Deserialize, Serialize};
use services::db::{UserAccessRequest, UserAccessRequestStatus};
use utoipa::ToSchema;

// === From routes_user.rs ===

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum UserRouteError {
  #[error("Invalid header value: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidHeader(String),
  #[error("Injected token is empty.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  EmptyToken,
  #[error("Failed to list users: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ListFailed(String),
  #[error("Failed to change user role: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RoleChangeFailed(String),
  #[error("Failed to remove user: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RemoveFailed(String),
}

/// Token Type
/// `session` - token stored in cookie based http session
/// `bearer` - token received from http authorization header as bearer token
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
  Session,
  Bearer,
}

/// Role Source
/// `role` - client level user role
/// `scope_token` - scope granted token role
/// `scope_user` - scope granted user role
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleSource {
  Role,
  ScopeToken,
  ScopeUser,
}

/// API Token information response
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct TokenInfo {
  pub role: TokenScope,
}

/// User authentication response with discriminated union
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(tag = "auth_status")]
#[schema(example = json!({
    "auth_status": "logged_in",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "user@example.com",
    "role": "resource_user"
}))]
pub enum UserResponse {
  /// User is not authenticated
  #[serde(rename = "logged_out")]
  LoggedOut,
  /// User is authenticated with details
  #[serde(rename = "logged_in")]
  LoggedIn(UserInfo),
  /// API token authentication
  #[serde(rename = "api_token")]
  Token(TokenInfo),
}

// === From routes_users_list.rs ===

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

// === From routes_access_request.rs ===

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AccessRequestError {
  #[error("Access request already pending.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyPending,

  #[error("User already has access.")]
  #[error_meta(error_type = ErrorType::UnprocessableEntity)]
  AlreadyHasAccess,

  #[error("Pending access request for user not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  PendingRequestNotFound,

  #[error("Access request {0} not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  RequestNotFound(i64),

  #[error("Insufficient privileges to assign this role.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InsufficientPrivileges,

  #[error("Failed to fetch access requests: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FetchFailed(String),
}

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
  pub role: ResourceRole,
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
