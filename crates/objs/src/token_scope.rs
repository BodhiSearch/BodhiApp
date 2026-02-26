use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use utoipa::ToSchema;

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

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(TokenScope::PowerUser, TokenScope::User, true)]
  #[case(TokenScope::PowerUser, TokenScope::PowerUser, false)]
  #[case(TokenScope::User, TokenScope::PowerUser, false)]
  #[case(TokenScope::User, TokenScope::User, false)]
  fn test_token_scope_ordering_explicit(
    #[case] left: TokenScope,
    #[case] right: TokenScope,
    #[case] is_greater: bool,
  ) {
    assert_eq!(left > right, is_greater);
    assert_eq!(left >= right, is_greater || left == right);
    assert_eq!(left < right, !is_greater && left != right);
    assert_eq!(left <= right, !is_greater || left == right);
  }

  #[rstest]
  #[case(TokenScope::User)]
  #[case(TokenScope::PowerUser)]
  fn test_token_scope_has_access_to_self(#[case] scope: TokenScope) {
    assert!(scope.has_access_to(&scope));
  }

  #[rstest]
  #[case(TokenScope::User, "scope_token_user")]
  #[case(TokenScope::PowerUser, "scope_token_power_user")]
  fn test_token_scope_string_formats(#[case] scope: TokenScope, #[case] as_str: &str) {
    assert_eq!(scope.to_string(), as_str);
    assert_eq!(scope.scope_token(), as_str);

    let serialized = serde_json::to_string(&scope).unwrap();
    assert_eq!(serialized, format!("\"{}\"", as_str));

    let deserialized: TokenScope = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, scope);
  }

  #[rstest]
  #[case(TokenScope::PowerUser, vec![TokenScope::PowerUser, TokenScope::User])]
  #[case(TokenScope::User, vec![TokenScope::User])]
  fn test_included_scopes_explicit(#[case] scope: TokenScope, #[case] expected: Vec<TokenScope>) {
    let included = scope.included_scopes();
    assert_eq!(included, expected);

    if !included.is_empty() {
      assert_eq!(*included.first().unwrap(), scope);
      assert_eq!(*included.last().unwrap(), TokenScope::User);
      for window in included.windows(2) {
        assert!(window[0] > window[1]);
      }
    }
  }

  #[rstest]
  #[case("scope_token_user", Ok(TokenScope::User))]
  #[case("scope_token_power_user", Ok(TokenScope::PowerUser))]
  fn test_token_scope_parse_valid(
    #[case] input: &str,
    #[case] expected: Result<TokenScope, TokenScopeError>,
  ) {
    assert_eq!(input.parse::<TokenScope>(), expected);
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
  #[case("Scope_Token_Admin")]
  #[case("resource-user")]
  #[case("scope-token-admin")]
  #[case("resource_")]
  #[case("scope_token_")]
  #[case("_user")]
  #[case("_admin")]
  #[case("resource_unknown")]
  #[case("scope_token_unknown")]
  #[case("scope_token_manager")]
  #[case("scope_token_admin")]
  fn test_token_scope_parse_invalid(#[case] input: &str) {
    let result = input.parse::<TokenScope>();
    assert!(result.is_err());
    assert!(matches!(result, Err(TokenScopeError::InvalidTokenScope(_))));
    assert_eq!(result.unwrap_err().to_string(), "invalid_token_scope");
  }

  #[test]
  fn test_token_scope_serialization() {
    assert_eq!(
      serde_json::to_string(&TokenScope::User).unwrap(),
      "\"scope_token_user\""
    );
    assert_eq!(
      serde_json::to_string(&TokenScope::PowerUser).unwrap(),
      "\"scope_token_power_user\""
    );
  }

  #[test]
  fn test_token_scope_deserialization() {
    assert_eq!(
      serde_json::from_str::<TokenScope>("\"scope_token_user\"").unwrap(),
      TokenScope::User
    );
    assert_eq!(
      serde_json::from_str::<TokenScope>("\"scope_token_power_user\"").unwrap(),
      TokenScope::PowerUser
    );
  }
}
