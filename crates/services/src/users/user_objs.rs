use crate::{ResourceRole, UserAccessRequestEntity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// UserAccessRequestStatus - Status for user-initiated access requests
// ============================================================================

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  PartialEq,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserAccessRequestStatus {
  Pending,
  Approved,
  Rejected,
}

// ============================================================================
// Request/Response types for user access request domain
// ============================================================================

/// Change user role request
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ChangeRoleRequest {
  /// Role to assign to the user
  #[schema(example = "resource_manager")]
  pub role: ResourceRole,
}

/// Response for checking access request status
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "username": "user@example.com",
    "status": "pending",
    "created_at": "2024-01-01T12:00:00Z",
    "updated_at": "2024-01-01T12:00:00Z"
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

impl From<UserAccessRequestEntity> for UserAccessStatusResponse {
  fn from(request: UserAccessRequestEntity) -> Self {
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

/// User access request output type for API responses
#[derive(Debug, Serialize, ToSchema)]
pub struct UserAccessRequest {
  pub id: String,
  pub username: String,
  pub user_id: String,
  #[serde(default)]
  pub reviewer: Option<String>,
  pub status: UserAccessRequestStatus,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<UserAccessRequestEntity> for UserAccessRequest {
  fn from(entity: UserAccessRequestEntity) -> Self {
    Self {
      id: entity.id,
      username: entity.username,
      user_id: entity.user_id,
      reviewer: entity.reviewer,
      status: entity.status,
      created_at: entity.created_at,
      updated_at: entity.updated_at,
    }
  }
}

/// Paginated response for access requests
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "requests": [{
        "id": "01HXXXXXX",
        "username": "user@example.com",
        "user_id": "auth0|123",
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
