use crate::{AppError, ErrorType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use utoipa::ToSchema;

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

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(UserScope::PowerUser, UserScope::User, true)]
  #[case(UserScope::PowerUser, UserScope::PowerUser, false)]
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

  #[rstest]
  #[case(UserScope::User)]
  #[case(UserScope::PowerUser)]
  fn test_user_scope_has_access_to_self(#[case] scope: UserScope) {
    assert!(scope.has_access_to(&scope));
  }

  #[rstest]
  #[case(UserScope::User, "scope_user_user")]
  #[case(UserScope::PowerUser, "scope_user_power_user")]
  fn test_user_scope_string_formats(#[case] scope: UserScope, #[case] as_str: &str) {
    assert_eq!(scope.to_string(), as_str);
    assert_eq!(scope.scope_user(), as_str);

    let serialized = serde_json::to_string(&scope).unwrap();
    assert_eq!(serialized, format!("\"{}\"", as_str));

    let deserialized: UserScope = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, scope);
  }

  #[rstest]
  #[case(UserScope::PowerUser, vec![UserScope::PowerUser, UserScope::User])]
  #[case(UserScope::User, vec![UserScope::User])]
  fn test_included_scopes_explicit(#[case] scope: UserScope, #[case] expected: Vec<UserScope>) {
    let included = scope.included_scopes();
    assert_eq!(included, expected);

    if !included.is_empty() {
      assert_eq!(*included.first().unwrap(), scope);
      assert_eq!(*included.last().unwrap(), UserScope::User);
      for window in included.windows(2) {
        assert!(window[0] > window[1]);
      }
    }
  }

  #[rstest]
  #[case("scope_user_user", Ok(UserScope::User))]
  #[case("scope_user_power_user", Ok(UserScope::PowerUser))]
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
  #[case("scope_user_manager")]
  #[case("scope_user_admin")]
  fn test_user_scope_parse_invalid(#[case] input: &str) {
    let result = input.parse::<UserScope>();
    assert!(result.is_err());
    assert!(matches!(result, Err(UserScopeError::InvalidUserScope(_))));
    assert_eq!(result.unwrap_err().to_string(), "invalid_user_scope");
  }

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
  }
}
