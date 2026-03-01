use errmeta::{AppError, ErrorType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use utoipa::ToSchema;

// ============================================================================
// ResourceRole - Session-based role hierarchy
// ============================================================================

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  strum::Display,
  strum::EnumIter,
  Serialize,
  Deserialize,
  ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ResourceRole {
  #[serde(rename = "resource_user")]
  #[strum(serialize = "resource_user")]
  User,
  #[serde(rename = "resource_power_user")]
  #[strum(serialize = "resource_power_user")]
  PowerUser,
  #[serde(rename = "resource_manager")]
  #[strum(serialize = "resource_manager")]
  Manager,
  #[serde(rename = "resource_admin")]
  #[strum(serialize = "resource_admin")]
  Admin,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, PartialEq)]
#[error_meta(trait_to_impl = AppError)]
pub enum RoleError {
  #[error("invalid_role_name")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRoleName(String),
}

impl ResourceRole {
  /// Checks if this role has access to the required role level
  /// Higher roles automatically have access to lower role endpoints
  pub fn has_access_to(&self, required: &ResourceRole) -> bool {
    // Since we derive PartialOrd, we can use >= for comparison
    // Admin > Manager > PowerUser > User
    self >= required
  }

  /// Get the resource role name for this role
  pub fn resource_role(&self) -> String {
    self.to_string()
  }

  /// Get all roles that this role has access to (including itself)
  pub fn included_roles(&self) -> Vec<ResourceRole> {
    // Use iterator to get all roles up to and including this role
    ResourceRole::iter()
      .filter(|r| r <= self)
      .rev() // Reverse to get highest role first
      .collect()
  }

  /// Returns the maximum `UserScope` that this resource role is allowed to grant.
  ///
  /// - `PowerUser`, `Manager`, `Admin` → `UserScope::PowerUser`
  /// - `User` → `UserScope::User`
  pub fn max_user_scope(&self) -> UserScope {
    if self >= &ResourceRole::PowerUser {
      UserScope::PowerUser
    } else {
      UserScope::User
    }
  }

  /// Parse the highest role from a slice of resource role strings.
  /// Returns the highest valid role found, or an error if no valid roles are present.
  pub fn from_resource_role<T: AsRef<str>>(resource_roles: &[T]) -> Result<Self, RoleError> {
    resource_roles
      .iter()
      .filter_map(|s| s.as_ref().parse::<ResourceRole>().ok())
      .max()
      .ok_or_else(|| RoleError::InvalidRoleName("no valid resource roles found".to_string()))
  }
}

impl FromStr for ResourceRole {
  type Err = RoleError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    // Match base role names first
    match s {
      "resource_user" => Ok(ResourceRole::User),
      "resource_power_user" => Ok(ResourceRole::PowerUser),
      "resource_manager" => Ok(ResourceRole::Manager),
      "resource_admin" => Ok(ResourceRole::Admin),
      _ => Err(RoleError::InvalidRoleName(s.to_string())),
    }
  }
}

// ============================================================================
// TokenScope - API token scope hierarchy
// ============================================================================

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  strum::Display,
  strum::EnumIter,
  Serialize,
  Deserialize,
  ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TokenScope {
  #[strum(serialize = "scope_token_user")]
  #[serde(rename = "scope_token_user")]
  User,
  #[strum(serialize = "scope_token_power_user")]
  #[serde(rename = "scope_token_power_user")]
  PowerUser,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, PartialEq)]
#[error_meta(trait_to_impl = AppError)]
pub enum TokenScopeError {
  #[error("invalid_token_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidTokenScope(String),
}

impl TokenScope {
  pub fn has_access_to(&self, required: &TokenScope) -> bool {
    self >= required
  }

  pub fn scope_token(&self) -> String {
    self.to_string()
  }

  pub fn included_scopes(&self) -> Vec<TokenScope> {
    TokenScope::iter().filter(|s| s <= self).rev().collect()
  }
}

impl FromStr for TokenScope {
  type Err = TokenScopeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "scope_token_user" => Ok(TokenScope::User),
      "scope_token_power_user" => Ok(TokenScope::PowerUser),
      _ => Err(TokenScopeError::InvalidTokenScope(s.to_string())),
    }
  }
}

// ============================================================================
// UserScope - External app scope hierarchy
// ============================================================================

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  strum::Display,
  strum::EnumIter,
  Serialize,
  Deserialize,
  ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserScope {
  #[strum(serialize = "scope_user_user")]
  #[serde(rename = "scope_user_user")]
  User,
  #[strum(serialize = "scope_user_power_user")]
  #[serde(rename = "scope_user_power_user")]
  PowerUser,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, PartialEq)]
#[error_meta(trait_to_impl = AppError)]
pub enum UserScopeError {
  #[error("invalid_user_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidUserScope(String),
}

impl UserScope {
  pub fn has_access_to(&self, required: &UserScope) -> bool {
    self >= required
  }

  pub fn scope_user(&self) -> String {
    self.to_string()
  }

  pub fn included_scopes(&self) -> Vec<UserScope> {
    UserScope::iter().filter(|s| s <= self).rev().collect()
  }
}

impl FromStr for UserScope {
  type Err = UserScopeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "scope_user_user" => Ok(UserScope::User),
      "scope_user_power_user" => Ok(UserScope::PowerUser),
      _ => Err(UserScopeError::InvalidUserScope(s.to_string())),
    }
  }
}

// ============================================================================
// AppRole - Union type across authentication contexts
// ============================================================================

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(untagged)]
pub enum AppRole {
  Session(ResourceRole),
  ApiToken(TokenScope),
  ExchangedToken(UserScope),
}

// ============================================================================
// UserInfo - Authenticated user information
// ============================================================================

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserInfo {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub user_id: String,
  #[schema(example = "user@example.com")]
  pub username: String,
  #[schema(example = "John")]
  pub first_name: Option<String>,
  #[schema(example = "Doe")]
  pub last_name: Option<String>,
  #[schema(example = "resource_user")]
  pub role: Option<AppRole>,
}

#[cfg(test)]
#[path = "test_auth_objs_role.rs"]
mod test_auth_objs_role;

#[cfg(test)]
#[path = "test_auth_objs_token_scope.rs"]
mod test_auth_objs_token_scope;

#[cfg(test)]
#[path = "test_auth_objs_user_scope.rs"]
mod test_auth_objs_user_scope;
