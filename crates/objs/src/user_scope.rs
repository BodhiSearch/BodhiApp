use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;

use crate::{AppError, ErrorType};

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
  #[strum(serialize = "scope_user_manager")]
  #[serde(rename = "scope_user_manager")]
  Manager,
  #[strum(serialize = "scope_user_admin")]
  #[serde(rename = "scope_user_admin")]
  Admin,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, PartialEq)]
#[error_meta(trait_to_impl = AppError)]
pub enum UserScopeError {
  #[error("invalid_user_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidUserScope(String),

  #[error("missing_user_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingUserScope,
}

impl UserScope {
  /// Checks if this user scope has access to the required scope level
  /// Higher scopes automatically have access to lower scope endpoints
  pub fn has_access_to(&self, required: &UserScope) -> bool {
    self >= required
  }

  /// Get the scope token string for this scope
  pub fn scope_user(&self) -> String {
    self.to_string()
  }

  /// Get all scopes that this scope has access to (including itself)
  pub fn included_scopes(&self) -> Vec<UserScope> {
    UserScope::iter()
      .filter(|s| s <= self)
      .rev() // Highest first
      .collect()
  }

  /// Parse the highest scope from a space-separated scope string.
  /// Does NOT require `offline_access` to be present.
  pub fn from_scope(scope: &str) -> Result<Self, UserScopeError> {
    let scopes: Vec<&str> = scope.split_whitespace().collect();

    // Find the highest scope by checking all possible scopes
    let highest_scope = UserScope::iter()
      .filter(|s| scopes.contains(&s.scope_user().as_str()))
      .max();

    highest_scope.ok_or(UserScopeError::MissingUserScope)
  }
}

impl FromStr for UserScope {
  type Err = UserScopeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "scope_user_user" => Ok(UserScope::User),
      "scope_user_power_user" => Ok(UserScope::PowerUser),
      "scope_user_manager" => Ok(UserScope::Manager),
      "scope_user_admin" => Ok(UserScope::Admin),
      _ => Err(UserScopeError::InvalidUserScope(s.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  // --- Ordering & comparison tests ---
  #[rstest]
  #[case(UserScope::Admin, UserScope::Manager, true)]
  #[case(UserScope::Admin, UserScope::PowerUser, true)]
  #[case(UserScope::Admin, UserScope::User, true)]
  #[case(UserScope::Manager, UserScope::Admin, false)]
  #[case(UserScope::Manager, UserScope::Manager, false)]
  #[case(UserScope::Manager, UserScope::PowerUser, true)]
  #[case(UserScope::Manager, UserScope::User, true)]
  #[case(UserScope::PowerUser, UserScope::Admin, false)]
  #[case(UserScope::PowerUser, UserScope::Manager, false)]
  #[case(UserScope::PowerUser, UserScope::PowerUser, false)]
  #[case(UserScope::PowerUser, UserScope::User, true)]
  #[case(UserScope::User, UserScope::Admin, false)]
  #[case(UserScope::User, UserScope::Manager, false)]
  #[case(UserScope::User, UserScope::PowerUser, false)]
  #[case(UserScope::User, UserScope::User, false)]
  fn test_user_scope_ordering_explicit(
    #[case] left: UserScope,
    #[case] right: UserScope,
    #[case] is_greater: bool,
  ) {
    assert_eq!(left > right, is_greater);
    assert_eq!(left >= right, is_greater || left == right);
    assert_eq!(left < right, !is_greater && left != right);
    assert_eq!(left <= right, !is_greater || left == right);
  }

  // --- Serialization / string tests ---
  #[rstest]
  #[case(UserScope::User, "scope_user_user")]
  #[case(UserScope::PowerUser, "scope_user_power_user")]
  #[case(UserScope::Manager, "scope_user_manager")]
  #[case(UserScope::Admin, "scope_user_admin")]
  fn test_user_scope_string_formats(#[case] scope: UserScope, #[case] as_str: &str) {
    // Display
    assert_eq!(scope.to_string(), as_str);

    // Custom helper
    assert_eq!(scope.scope_user(), as_str);

    // Serde serialization
    let serialized = serde_json::to_string(&scope).unwrap();
    assert_eq!(serialized, format!("\"{}\"", as_str));

    // Deserialization
    let deserialized: UserScope = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, scope);
  }

  // --- included_scopes tests ---
  #[rstest]
  #[case(UserScope::Admin, vec![UserScope::Admin, UserScope::Manager, UserScope::PowerUser, UserScope::User])]
  #[case(UserScope::Manager, vec![UserScope::Manager, UserScope::PowerUser, UserScope::User])]
  #[case(UserScope::PowerUser, vec![UserScope::PowerUser, UserScope::User])]
  #[case(UserScope::User, vec![UserScope::User])]
  fn test_included_scopes_explicit(#[case] scope: UserScope, #[case] expected: Vec<UserScope>) {
    let included = scope.included_scopes();
    assert_eq!(included, expected);

    // Ordering properties
    if !included.is_empty() {
      assert_eq!(*included.first().unwrap(), scope);
      assert_eq!(*included.last().unwrap(), UserScope::User);
      for window in included.windows(2) {
        assert!(window[0] > window[1]);
      }
    }
  }

  // --- from_scope parsing tests ---
  #[rstest]
  #[case("scope_user_user", Ok(UserScope::User))]
  #[case("scope_user_power_user scope_user_user", Ok(UserScope::PowerUser))]
  #[case(
    "scope_user_manager scope_user_power_user scope_user_user",
    Ok(UserScope::Manager)
  )]
  #[case(
    "scope_user_admin scope_user_manager scope_user_power_user scope_user_user",
    Ok(UserScope::Admin)
  )]
  #[case("scope_user_user openid email", Ok(UserScope::User))]
  #[case("openid email", Err(UserScopeError::MissingUserScope))]
  fn test_user_scope_from_scope(
    #[case] scope: &str,
    #[case] expected: Result<UserScope, UserScopeError>,
  ) {
    assert_eq!(UserScope::from_scope(scope), expected);
  }

  // --- Case sensitivity tests ---
  #[rstest]
  #[case("SCOPE_USER_ADMIN", Err(UserScopeError::MissingUserScope))]
  #[case("scope_User_Power_User", Err(UserScopeError::MissingUserScope))]
  fn test_user_scope_case_sensitivity(
    #[case] scope: &str,
    #[case] expected: Result<UserScope, UserScopeError>,
  ) {
    assert_eq!(UserScope::from_scope(scope), expected);
  }

  // --- FromStr parsing tests ---
  #[rstest]
  #[case("scope_user_user", Ok(UserScope::User))]
  #[case("scope_user_power_user", Ok(UserScope::PowerUser))]
  #[case("scope_user_manager", Ok(UserScope::Manager))]
  #[case("scope_user_admin", Ok(UserScope::Admin))]
  fn test_user_scope_parse_valid(
    #[case] input: &str,
    #[case] expected: Result<UserScope, UserScopeError>,
  ) {
    assert_eq!(input.parse::<UserScope>(), expected);
  }

  #[rstest]
  #[case("")]
  #[case("user")]
  #[case("power_user")]
  #[case("manager")]
  #[case("admin")]
  #[case("invalid")]
  #[case("USER")]
  #[case("ADMIN")]
  #[case("Resource_User")]
  #[case("Scope_User_Admin")]
  #[case("resource-user")]
  #[case("scope-user-admin")]
  #[case("resource_")]
  #[case("scope_user_")]
  #[case("_user")]
  #[case("_admin")]
  #[case("resource_unknown")]
  #[case("scope_user_unknown")]
  fn test_user_scope_parse_invalid(#[case] input: &str) {
    let result = input.parse::<UserScope>();
    assert!(result.is_err());
    assert!(matches!(result, Err(UserScopeError::InvalidUserScope(_))));
    assert_eq!(result.unwrap_err().to_string(), "invalid_user_scope");
  }

  // --- Serde explicit tests ---
  #[test]
  fn test_user_scope_serialization() {
    assert_eq!(
      serde_json::to_string(&UserScope::User).unwrap(),
      "\"scope_user_user\""
    );
    assert_eq!(
      serde_json::to_string(&UserScope::PowerUser).unwrap(),
      "\"scope_user_power_user\""
    );
    assert_eq!(
      serde_json::to_string(&UserScope::Manager).unwrap(),
      "\"scope_user_manager\""
    );
    assert_eq!(
      serde_json::to_string(&UserScope::Admin).unwrap(),
      "\"scope_user_admin\""
    );
  }

  #[test]
  fn test_user_scope_deserialization() {
    assert_eq!(
      serde_json::from_str::<UserScope>("\"scope_user_user\"").unwrap(),
      UserScope::User
    );
    assert_eq!(
      serde_json::from_str::<UserScope>("\"scope_user_power_user\"").unwrap(),
      UserScope::PowerUser
    );
    assert_eq!(
      serde_json::from_str::<UserScope>("\"scope_user_manager\"").unwrap(),
      UserScope::Manager
    );
    assert_eq!(
      serde_json::from_str::<UserScope>("\"scope_user_admin\"").unwrap(),
      UserScope::Admin
    );
  }
}
