use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use utoipa::ToSchema;

use crate::{AppError, ErrorType, TokenScope, UserScope};

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

  /// Parse the highest role from a slice of resource role strings
  /// Returns the highest valid role found, or an error if no valid roles are present
  pub fn from_resource_role<T: AsRef<str>>(resource_roles: &[T]) -> Result<Self, RoleError> {
    let mut highest_role = None;
    for resource_role in resource_roles {
      match resource_role.as_ref() {
        "resource_user" => {
          highest_role = Some(
            highest_role.map_or(ResourceRole::User, |current: ResourceRole| {
              if ResourceRole::User > current {
                ResourceRole::User
              } else {
                current
              }
            }),
          );
        }
        "resource_power_user" => {
          highest_role = Some(highest_role.map_or(
            ResourceRole::PowerUser,
            |current: ResourceRole| {
              if ResourceRole::PowerUser > current {
                ResourceRole::PowerUser
              } else {
                current
              }
            },
          ));
        }
        "resource_manager" => {
          highest_role = Some(highest_role.map_or(
            ResourceRole::Manager,
            |current: ResourceRole| {
              if ResourceRole::Manager > current {
                ResourceRole::Manager
              } else {
                current
              }
            },
          ));
        }
        "resource_admin" => {
          highest_role = Some(
            highest_role.map_or(ResourceRole::Admin, |current: ResourceRole| {
              if ResourceRole::Admin > current {
                ResourceRole::Admin
              } else {
                current
              }
            }),
          );
        }
        _ => continue,
      }
    }

    highest_role
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

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(untagged)]
pub enum AppRole {
  Session(ResourceRole),
  ApiToken(TokenScope),
  ExchangedToken(UserScope),
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(ResourceRole::Admin, ResourceRole::Manager, true)]
  #[case(ResourceRole::Admin, ResourceRole::PowerUser, true)]
  #[case(ResourceRole::Admin, ResourceRole::User, true)]
  #[case(ResourceRole::Manager, ResourceRole::Admin, false)]
  #[case(ResourceRole::Manager, ResourceRole::Manager, false)]
  #[case(ResourceRole::Manager, ResourceRole::PowerUser, true)]
  #[case(ResourceRole::Manager, ResourceRole::User, true)]
  #[case(ResourceRole::PowerUser, ResourceRole::Admin, false)]
  #[case(ResourceRole::PowerUser, ResourceRole::Manager, false)]
  #[case(ResourceRole::PowerUser, ResourceRole::PowerUser, false)]
  #[case(ResourceRole::PowerUser, ResourceRole::User, true)]
  #[case(ResourceRole::User, ResourceRole::Admin, false)]
  #[case(ResourceRole::User, ResourceRole::Manager, false)]
  #[case(ResourceRole::User, ResourceRole::PowerUser, false)]
  #[case(ResourceRole::User, ResourceRole::User, false)]
  fn test_role_ordering_explicit(
    #[case] left: ResourceRole,
    #[case] right: ResourceRole,
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
  #[case(ResourceRole::User, "resource_user")]
  #[case(ResourceRole::PowerUser, "resource_power_user")]
  #[case(ResourceRole::Manager, "resource_manager")]
  #[case(ResourceRole::Admin, "resource_admin")]
  fn test_role_string_formats(#[case] role: ResourceRole, #[case] as_str: &str) {
    // Test Display format
    assert_eq!(role.to_string(), as_str);

    // Test resource role format
    assert_eq!(role.resource_role(), as_str);

    // Test serialization
    let serialized = serde_json::to_string(&role).unwrap();
    assert_eq!(serialized, format!("\"{}\"", as_str));

    // Test deserialization
    let deserialized: ResourceRole = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, role);
  }

  #[rstest]
  #[case(ResourceRole::Admin, vec![ResourceRole::Admin, ResourceRole::Manager, ResourceRole::PowerUser, ResourceRole::User])]
  #[case(ResourceRole::Manager, vec![ResourceRole::Manager, ResourceRole::PowerUser, ResourceRole::User])]
  #[case(ResourceRole::PowerUser, vec![ResourceRole::PowerUser, ResourceRole::User])]
  #[case(ResourceRole::User, vec![ResourceRole::User])]
  fn test_included_roles_explicit(#[case] role: ResourceRole, #[case] expected: Vec<ResourceRole>) {
    let included = role.included_roles();
    assert_eq!(included, expected);

    // Verify ordering properties
    if !included.is_empty() {
      assert_eq!(*included.first().unwrap(), role);
      assert_eq!(*included.last().unwrap(), ResourceRole::User);
      for window in included.windows(2) {
        assert!(window[0] > window[1]);
      }
    }
  }

  #[rstest]
  #[case(ResourceRole::Admin, ResourceRole::User, true)]
  #[case(ResourceRole::Admin, ResourceRole::PowerUser, true)]
  #[case(ResourceRole::Admin, ResourceRole::Manager, true)]
  #[case(ResourceRole::Admin, ResourceRole::Admin, true)]
  #[case(ResourceRole::Manager, ResourceRole::User, true)]
  #[case(ResourceRole::Manager, ResourceRole::PowerUser, true)]
  #[case(ResourceRole::Manager, ResourceRole::Manager, true)]
  #[case(ResourceRole::Manager, ResourceRole::Admin, false)]
  #[case(ResourceRole::PowerUser, ResourceRole::User, true)]
  #[case(ResourceRole::PowerUser, ResourceRole::PowerUser, true)]
  #[case(ResourceRole::PowerUser, ResourceRole::Manager, false)]
  #[case(ResourceRole::PowerUser, ResourceRole::Admin, false)]
  #[case(ResourceRole::User, ResourceRole::User, true)]
  #[case(ResourceRole::User, ResourceRole::PowerUser, false)]
  #[case(ResourceRole::User, ResourceRole::Manager, false)]
  #[case(ResourceRole::User, ResourceRole::Admin, false)]
  fn test_role_has_access_to(
    #[case] role: ResourceRole,
    #[case] required: ResourceRole,
    #[case] expected: bool,
  ) {
    assert_eq!(role.has_access_to(&required), expected);
  }

  #[rstest]
  #[case(ResourceRole::User, "resource_user")]
  #[case(ResourceRole::PowerUser, "resource_power_user")]
  #[case(ResourceRole::Manager, "resource_manager")]
  #[case(ResourceRole::Admin, "resource_admin")]
  fn test_resource_role(#[case] role: ResourceRole, #[case] expected: &str) {
    assert_eq!(role.resource_role(), expected);
  }

  #[rstest]
  #[case(ResourceRole::User, vec![ResourceRole::User])]
  #[case(ResourceRole::PowerUser, vec![ResourceRole::PowerUser, ResourceRole::User])]
  #[case(ResourceRole::Manager, vec![ResourceRole::Manager, ResourceRole::PowerUser, ResourceRole::User])]
  #[case(ResourceRole::Admin, vec![ResourceRole::Admin, ResourceRole::Manager, ResourceRole::PowerUser, ResourceRole::User])]
  fn test_included_roles(#[case] role: ResourceRole, #[case] expected: Vec<ResourceRole>) {
    assert_eq!(role.included_roles(), expected);
  }

  #[rstest]
  #[case(ResourceRole::User, "\"resource_user\"")]
  #[case(ResourceRole::PowerUser, "\"resource_power_user\"")]
  #[case(ResourceRole::Manager, "\"resource_manager\"")]
  #[case(ResourceRole::Admin, "\"resource_admin\"")]
  fn test_role_serde_format(#[case] role: ResourceRole, #[case] expected_json: &str) {
    // Test serialization
    let serialized = serde_json::to_string(&role).unwrap();
    assert_eq!(serialized, expected_json);

    // Test deserialization
    let deserialized: ResourceRole = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, role);
  }

  #[rstest]
  #[case(&["resource_user"], ResourceRole::User)]
  #[case(&["resource_power_user"], ResourceRole::PowerUser)]
  #[case(&["resource_manager"], ResourceRole::Manager)]
  #[case(&["resource_admin"], ResourceRole::Admin)]
  #[case(&["resource_user", "resource_power_user"], ResourceRole::PowerUser)]
  #[case(&["resource_user", "resource_manager"], ResourceRole::Manager)]
  #[case(&["resource_power_user", "resource_admin"], ResourceRole::Admin)]
  #[case(&["resource_user", "resource_power_user", "resource_manager"], ResourceRole::Manager)]
  #[case(&["resource_user", "resource_admin", "resource_manager"], ResourceRole::Admin)]
  fn test_role_from_resource_role_success(#[case] input: &[&str], #[case] expected: ResourceRole) {
    assert_eq!(ResourceRole::from_resource_role(input).unwrap(), expected);
  }

  #[rstest]
  #[case(&["user"])]
  #[case(&["power_user", "manager"])]
  #[case(&["resource_invalid", "invalid_role"])]
  #[case(&["RESOURCE_USER", "Resource_Manager"])]
  #[case(&[])]
  fn test_role_from_resource_role_failure(#[case] input: &[&str]) {
    assert_eq!(
      ResourceRole::from_resource_role(input).unwrap_err(),
      RoleError::InvalidRoleName("no valid resource roles found".to_string())
    );
  }

  #[rstest]
  #[case(&["resource_user", "invalid_role"], ResourceRole::User)]
  #[case(&["invalid_role", "resource_manager", "bad_role"], ResourceRole::Manager)]
  #[case(&["resource_power_user", "RESOURCE_ADMIN"], ResourceRole::PowerUser)]
  #[case(&["bad_role", "resource_admin", "invalid"], ResourceRole::Admin)]
  fn test_role_from_resource_role_mixed(#[case] input: &[&str], #[case] expected: ResourceRole) {
    assert_eq!(ResourceRole::from_resource_role(input).unwrap(), expected);
  }

  #[rstest]
  #[case("resource_user", Ok(ResourceRole::User))]
  #[case("resource_power_user", Ok(ResourceRole::PowerUser))]
  #[case("resource_manager", Ok(ResourceRole::Manager))]
  #[case("resource_admin", Ok(ResourceRole::Admin))]
  fn test_role_parse_valid(#[case] input: &str, #[case] expected: Result<ResourceRole, RoleError>) {
    assert_eq!(input.parse::<ResourceRole>(), expected);
  }

  #[rstest]
  #[case("")]
  #[case("scope_token_user")]
  #[case("scope_token_power_user")]
  #[case("scope_token_manager")]
  #[case("scope_token_admin")]
  #[case("user")]
  #[case("power_user")]
  #[case("manager")]
  #[case("admin")]
  #[case("invalid")]
  #[case("USER")]
  #[case("ADMIN")]
  #[case("Resource_User")]
  #[case("Scope_Token_Admin")]
  #[case("resource-user")]
  #[case("scope-token-admin")]
  #[case("resource_")]
  #[case("scope_token_")]
  #[case("_user")]
  #[case("_admin")]
  #[case("resource_unknown")]
  #[case("scope_token_unknown")]
  fn test_role_parse_invalid(#[case] input: &str) {
    let result = input.parse::<ResourceRole>();
    assert!(result.is_err());
    assert!(matches!(result, Err(RoleError::InvalidRoleName(_))));
    assert_eq!(result.unwrap_err().to_string(), "invalid_role_name");
  }

  #[test]
  fn test_role_serialization() {
    assert_eq!(
      serde_json::to_string(&ResourceRole::User).unwrap(),
      "\"resource_user\""
    );
    assert_eq!(
      serde_json::to_string(&ResourceRole::PowerUser).unwrap(),
      "\"resource_power_user\""
    );
    assert_eq!(
      serde_json::to_string(&ResourceRole::Manager).unwrap(),
      "\"resource_manager\""
    );
    assert_eq!(
      serde_json::to_string(&ResourceRole::Admin).unwrap(),
      "\"resource_admin\""
    );
  }

  #[test]
  fn test_role_deserialization() {
    assert_eq!(
      serde_json::from_str::<ResourceRole>("\"resource_user\"").unwrap(),
      ResourceRole::User
    );
    assert_eq!(
      serde_json::from_str::<ResourceRole>("\"resource_power_user\"").unwrap(),
      ResourceRole::PowerUser
    );
    assert_eq!(
      serde_json::from_str::<ResourceRole>("\"resource_manager\"").unwrap(),
      ResourceRole::Manager
    );
    assert_eq!(
      serde_json::from_str::<ResourceRole>("\"resource_admin\"").unwrap(),
      ResourceRole::Admin
    );
  }
}
