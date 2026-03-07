use serde::{Deserialize, Serialize};
use services::TokenScope;
use services::UserInfo;
use utoipa::ToSchema;

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

/// Envelope wrapping UserResponse with additional session info
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserInfoEnvelope {
  /// Core user authentication response
  #[serde(flatten)]
  pub user: UserResponse,
  /// Whether the user has an active dashboard session (only present when true)
  #[serde(default, skip_serializing_if = "is_false")]
  pub has_dashboard_session: bool,
}

fn is_false(v: &bool) -> bool {
  !v
}

// === From routes_users_list.rs ===

/// List users query parameters. Intentionally omits sort fields (unlike PaginationSortParams)
/// because user listing is fetched from the auth service which handles its own ordering.
#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct ListUsersParams {
  #[schema(example = 1)]
  pub page: Option<u32>,
  #[schema(example = 10)]
  pub page_size: Option<u32>,
}
