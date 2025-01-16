use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;

use crate::{AppError, ErrorType};

// Add constants for prefixes
const SCOPE_TOKEN_PREFIX: &str = "scope_token_";
const RESOURCE_PREFIX: &str = "resource_";

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
  strum::Display,
  strum::EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Role {
  User,
  PowerUser,
  Manager,
  Admin,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, PartialEq)]
#[error_meta(trait_to_impl = AppError)]
pub enum RoleError {
  #[error("invalid_role_name")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRoleName(String),

  #[error("missing_offline_access")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingOfflineAccess,

  #[error("missing_role_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingRoleScope,
}

impl Role {
  /// Checks if this role has access to the required role level
  /// Higher roles automatically have access to lower role endpoints
  pub fn has_access_to(&self, required: &Role) -> bool {
    // Since we derive PartialOrd, we can use >= for comparison
    // Admin > Manager > PowerUser > User
    self >= required
  }

  /// Get the scope token name for this role
  pub fn scope_token(&self) -> String {
    format!("{}{}", SCOPE_TOKEN_PREFIX, self)
  }

  /// Get the resource role name for this role
  pub fn resource_role(&self) -> String {
    format!("{}{}", RESOURCE_PREFIX, self)
  }

  /// Get all roles that this role has access to (including itself)
  pub fn included_roles(&self) -> Vec<Role> {
    // Use iterator to get all roles up to and including this role
    Role::iter()
      .filter(|r| r <= self)
      .rev() // Reverse to get highest role first
      .collect()
  }

  /// Parse the highest role from a space-separated scope string
  pub fn from_scope(scope: &str) -> Result<Self, RoleError> {
    let scopes: Vec<&str> = scope.split_whitespace().collect();

    if !scopes.contains(&"offline_access") {
      return Err(RoleError::MissingOfflineAccess);
    }

    // Find the highest role scope by checking all possible roles
    let highest_role = Role::iter()
      .filter(|role| scopes.contains(&role.scope_token().as_str()))
      .max();

    highest_role.ok_or(RoleError::MissingRoleScope)
  }

  /// Parse the highest role from a slice of resource role strings
  /// Returns the highest valid role found, or an error if no valid roles are present
  pub fn from_resource_role<T: AsRef<str>>(resource_roles: &[T]) -> Result<Self, RoleError> {
    let mut highest_role = None;
    for resource_role in resource_roles {
      match resource_role.as_ref() {
        "resource_user" => {
          highest_role = Some(highest_role.map_or(Role::User, |current: Role| {
            if Role::User > current {
              Role::User
            } else {
              current
            }
          }));
        }
        "resource_power_user" => {
          highest_role = Some(highest_role.map_or(Role::PowerUser, |current: Role| {
            if Role::PowerUser > current {
              Role::PowerUser
            } else {
              current
            }
          }));
        }
        "resource_manager" => {
          highest_role = Some(highest_role.map_or(Role::Manager, |current: Role| {
            if Role::Manager > current {
              Role::Manager
            } else {
              current
            }
          }));
        }
        "resource_admin" => {
          highest_role = Some(highest_role.map_or(Role::Admin, |current: Role| {
            if Role::Admin > current {
              Role::Admin
            } else {
              current
            }
          }));
        }
        _ => continue,
      }
    }

    highest_role
      .ok_or_else(|| RoleError::InvalidRoleName("no valid resource roles found".to_string()))
  }
}

impl FromStr for Role {
  type Err = RoleError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    // Try to match both scope token and resource role formats
    for role in Role::iter() {
      if s == role.scope_token() || s == role.resource_role() {
        return Ok(role);
      }
    }
    Err(RoleError::InvalidRoleName(s.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(Role::Admin, Role::Manager, true)]
  #[case(Role::Admin, Role::PowerUser, true)]
  #[case(Role::Admin, Role::User, true)]
  #[case(Role::Manager, Role::Admin, false)]
  #[case(Role::Manager, Role::Manager, false)]
  #[case(Role::Manager, Role::PowerUser, true)]
  #[case(Role::Manager, Role::User, true)]
  #[case(Role::PowerUser, Role::Admin, false)]
  #[case(Role::PowerUser, Role::Manager, false)]
  #[case(Role::PowerUser, Role::PowerUser, false)]
  #[case(Role::PowerUser, Role::User, true)]
  #[case(Role::User, Role::Admin, false)]
  #[case(Role::User, Role::Manager, false)]
  #[case(Role::User, Role::PowerUser, false)]
  #[case(Role::User, Role::User, false)]
  fn test_role_ordering_explicit(
    #[case] left: Role,
    #[case] right: Role,
    #[case] is_greater: bool,
  ) {
    // Test greater than
    assert_eq!(left > right, is_greater);

    // Test greater than or equal
    assert_eq!(left >= right, is_greater || left == right);

    // Test less than (inverse of greater than, unless equal)
    assert_eq!(left < right, !is_greater && left != right);

    // Test less than or equal (inverse of greater than, or equal)
    assert_eq!(left <= right, !is_greater || left == right);
  }

  #[rstest]
  #[case(Role::User, "user", "scope_token_user", "resource_user")]
  #[case(
    Role::PowerUser,
    "power_user",
    "scope_token_power_user",
    "resource_power_user"
  )]
  #[case(Role::Manager, "manager", "scope_token_manager", "resource_manager")]
  #[case(Role::Admin, "admin", "scope_token_admin", "resource_admin")]
  fn test_role_string_formats(
    #[case] role: Role,
    #[case] display: &str,
    #[case] scope_token: &str,
    #[case] resource_role: &str,
  ) {
    // Test Display format
    assert_eq!(role.to_string(), display);

    // Test scope token format
    assert_eq!(role.scope_token(), scope_token);

    // Test resource role format
    assert_eq!(role.resource_role(), resource_role);

    // Test serialization
    let serialized = serde_json::to_string(&role).unwrap();
    assert_eq!(serialized, format!("\"{}\"", display));

    // Test deserialization
    let deserialized: Role = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, role);
  }

  #[rstest]
  #[case(Role::Admin, vec![Role::Admin, Role::Manager, Role::PowerUser, Role::User])]
  #[case(Role::Manager, vec![Role::Manager, Role::PowerUser, Role::User])]
  #[case(Role::PowerUser, vec![Role::PowerUser, Role::User])]
  #[case(Role::User, vec![Role::User])]
  fn test_included_roles_explicit(#[case] role: Role, #[case] expected: Vec<Role>) {
    let included = role.included_roles();
    assert_eq!(included, expected);

    // Verify ordering properties
    if !included.is_empty() {
      assert_eq!(*included.first().unwrap(), role);
      assert_eq!(*included.last().unwrap(), Role::User);
      for window in included.windows(2) {
        assert!(window[0] > window[1]);
      }
    }
  }

  #[rstest]
  #[case(Role::Admin, Role::User, true)]
  #[case(Role::Admin, Role::PowerUser, true)]
  #[case(Role::Admin, Role::Manager, true)]
  #[case(Role::Admin, Role::Admin, true)]
  #[case(Role::Manager, Role::User, true)]
  #[case(Role::Manager, Role::PowerUser, true)]
  #[case(Role::Manager, Role::Manager, true)]
  #[case(Role::Manager, Role::Admin, false)]
  #[case(Role::PowerUser, Role::User, true)]
  #[case(Role::PowerUser, Role::PowerUser, true)]
  #[case(Role::PowerUser, Role::Manager, false)]
  #[case(Role::PowerUser, Role::Admin, false)]
  #[case(Role::User, Role::User, true)]
  #[case(Role::User, Role::PowerUser, false)]
  #[case(Role::User, Role::Manager, false)]
  #[case(Role::User, Role::Admin, false)]
  fn test_role_has_access_to(#[case] role: Role, #[case] required: Role, #[case] expected: bool) {
    assert_eq!(role.has_access_to(&required), expected);
  }

  #[rstest]
  #[case(Role::User, "scope_token_user")]
  #[case(Role::PowerUser, "scope_token_power_user")]
  #[case(Role::Manager, "scope_token_manager")]
  #[case(Role::Admin, "scope_token_admin")]
  fn test_scope_token(#[case] role: Role, #[case] expected: &str) {
    assert_eq!(role.scope_token(), expected);
  }

  #[rstest]
  #[case(Role::User, "resource_user")]
  #[case(Role::PowerUser, "resource_power_user")]
  #[case(Role::Manager, "resource_manager")]
  #[case(Role::Admin, "resource_admin")]
  fn test_resource_role(#[case] role: Role, #[case] expected: &str) {
    assert_eq!(role.resource_role(), expected);
  }

  #[rstest]
  #[case("resource_user", Ok(Role::User))]
  #[case("scope_token_user", Ok(Role::User))]
  #[case("resource_power_user", Ok(Role::PowerUser))]
  #[case("scope_token_power_user", Ok(Role::PowerUser))]
  #[case("resource_manager", Ok(Role::Manager))]
  #[case("scope_token_manager", Ok(Role::Manager))]
  #[case("resource_admin", Ok(Role::Admin))]
  #[case("scope_token_admin", Ok(Role::Admin))]
  #[case("invalid_role", Err(RoleError::InvalidRoleName("invalid_role".to_string())))]
  fn test_role_from_str(#[case] input: &str, #[case] expected: Result<Role, RoleError>) {
    assert_eq!(
      Role::from_str(input).map_err(|e| e.to_string()),
      expected.map_err(|e| e.to_string())
    );
  }

  #[rstest]
  #[case(Role::User, vec![Role::User])]
  #[case(Role::PowerUser, vec![Role::PowerUser, Role::User])]
  #[case(Role::Manager, vec![Role::Manager, Role::PowerUser, Role::User])]
  #[case(Role::Admin, vec![Role::Admin, Role::Manager, Role::PowerUser, Role::User])]
  fn test_included_roles(#[case] role: Role, #[case] expected: Vec<Role>) {
    assert_eq!(role.included_roles(), expected);
  }

  #[rstest]
  #[case(Role::User, "\"user\"")]
  #[case(Role::PowerUser, "\"power_user\"")]
  #[case(Role::Manager, "\"manager\"")]
  #[case(Role::Admin, "\"admin\"")]
  fn test_role_serde_format(#[case] role: Role, #[case] expected_json: &str) {
    // Test serialization
    let serialized = serde_json::to_string(&role).unwrap();
    assert_eq!(serialized, expected_json);

    // Test deserialization
    let deserialized: Role = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, role);
  }

  #[rstest]
  #[case("offline_access scope_token_user", Ok(Role::User))]
  #[case(
    "offline_access scope_token_power_user scope_token_user",
    Ok(Role::PowerUser)
  )]
  #[case(
    "offline_access scope_token_manager scope_token_power_user scope_token_user",
    Ok(Role::Manager)
  )]
  #[case(
    "offline_access scope_token_admin scope_token_manager scope_token_power_user scope_token_user",
    Ok(Role::Admin)
  )]
  #[case("offline_access scope_token_user openid profile email", Ok(Role::User))]
  #[case(
    "offline_access openid profile email",
    Err(RoleError::MissingRoleScope)
  )]
  #[case(
    "scope_token_admin scope_token_user",
    Err(RoleError::MissingOfflineAccess)
  )]
  #[case(
    "offline_access scope_token_power_user scope_token_admin",
    Ok(Role::Admin)
  )]
  #[case(
    "offline_access scope_token_user scope_token_manager",
    Ok(Role::Manager)
  )]
  fn test_role_from_scope(#[case] scope: &str, #[case] expected: Result<Role, RoleError>) {
    assert_eq!(
      Role::from_scope(scope).map_err(|e| e.to_string()),
      expected.map_err(|e| e.to_string())
    );
  }

  #[rstest]
  #[case("offline_access SCOPE_TOKEN_ADMIN", Err(RoleError::MissingRoleScope))]
  #[case(
    "OFFLINE_ACCESS scope_token_admin",
    Err(RoleError::MissingOfflineAccess)
  )]
  #[case("offline_access Scope_Token_User", Err(RoleError::MissingRoleScope))]
  #[case(
    "Offline_Access SCOPE_TOKEN_MANAGER",
    Err(RoleError::MissingOfflineAccess)
  )]
  fn test_role_from_scope_case_sensitivity(
    #[case] scope: &str,
    #[case] expected: Result<Role, RoleError>,
  ) {
    assert_eq!(
      Role::from_scope(scope).map_err(|e| e.to_string()),
      expected.map_err(|e| e.to_string())
    );
  }

  #[rstest]
  #[case(&["resource_user"], Role::User)]
  #[case(&["resource_power_user"], Role::PowerUser)]
  #[case(&["resource_manager"], Role::Manager)]
  #[case(&["resource_admin"], Role::Admin)]
  #[case(&["resource_user", "resource_power_user"], Role::PowerUser)]
  #[case(&["resource_user", "resource_manager"], Role::Manager)]
  #[case(&["resource_power_user", "resource_admin"], Role::Admin)]
  #[case(&["resource_user", "resource_power_user", "resource_manager"], Role::Manager)]
  #[case(&["resource_user", "resource_admin", "resource_manager"], Role::Admin)]
  fn test_role_from_resource_role_success(#[case] input: &[&str], #[case] expected: Role) {
    assert_eq!(Role::from_resource_role(input).unwrap(), expected);
  }

  #[rstest]
  #[case(&["user"])]
  #[case(&["power_user", "manager"])]
  #[case(&["resource_invalid", "invalid_role"])]
  #[case(&["RESOURCE_USER", "Resource_Manager"])]
  #[case(&[])]
  fn test_role_from_resource_role_failure(#[case] input: &[&str]) {
    assert_eq!(
      Role::from_resource_role(input).unwrap_err(),
      RoleError::InvalidRoleName("no valid resource roles found".to_string())
    );
  }

  #[rstest]
  #[case(&["resource_user", "invalid_role"], Role::User)]
  #[case(&["invalid_role", "resource_manager", "bad_role"], Role::Manager)]
  #[case(&["resource_power_user", "RESOURCE_ADMIN"], Role::PowerUser)]
  #[case(&["bad_role", "resource_admin", "invalid"], Role::Admin)]
  fn test_role_from_resource_role_mixed(#[case] input: &[&str], #[case] expected: Role) {
    assert_eq!(Role::from_resource_role(input).unwrap(), expected);
  }
}
