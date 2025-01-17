use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;

use crate::{AppError, ErrorType};

const SCOPE_TOKEN_PREFIX: &str = "scope_token_";

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
pub enum TokenScope {
  User,
  PowerUser,
  Manager,
  Admin,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, PartialEq)]
#[error_meta(trait_to_impl = AppError)]
pub enum TokenScopeError {
  #[error("invalid_token_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidTokenScope(String),

  #[error("missing_offline_access")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingOfflineAccess,

  #[error("missing_token_scope")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingTokenScope,
}

impl TokenScope {
  /// Checks if this token scope has access to the required scope level
  /// Higher scopes automatically have access to lower scope endpoints
  pub fn has_access_to(&self, required: &TokenScope) -> bool {
    self >= required
  }

  /// Get the scope token string for this scope
  pub fn scope_token(&self) -> String {
    format!("{}{}", SCOPE_TOKEN_PREFIX, self)
  }

  /// Get all scopes that this scope has access to (including itself)
  pub fn included_scopes(&self) -> Vec<TokenScope> {
    TokenScope::iter()
      .filter(|s| s <= self)
      .rev() // Reverse to get highest scope first
      .collect()
  }

  /// Parse the highest scope from a space-separated scope string
  /// Requires "offline_access" scope to be present
  pub fn from_scope(scope: &str) -> Result<Self, TokenScopeError> {
    let scopes: Vec<&str> = scope.split_whitespace().collect();

    if !scopes.contains(&"offline_access") {
      return Err(TokenScopeError::MissingOfflineAccess);
    }

    // Find the highest scope by checking all possible scopes
    let highest_scope = TokenScope::iter()
      .filter(|scope| scopes.contains(&scope.scope_token().as_str()))
      .max();

    highest_scope.ok_or(TokenScopeError::MissingTokenScope)
  }
}

impl FromStr for TokenScope {
  type Err = TokenScopeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    // Match base scope names first
    match s {
      "user" => Ok(TokenScope::User),
      "power_user" => Ok(TokenScope::PowerUser),
      "manager" => Ok(TokenScope::Manager),
      "admin" => Ok(TokenScope::Admin),
      _ => Err(TokenScopeError::InvalidTokenScope(s.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(TokenScope::Admin, TokenScope::Manager, true)]
  #[case(TokenScope::Admin, TokenScope::PowerUser, true)]
  #[case(TokenScope::Admin, TokenScope::User, true)]
  #[case(TokenScope::Manager, TokenScope::Admin, false)]
  #[case(TokenScope::Manager, TokenScope::Manager, false)]
  #[case(TokenScope::Manager, TokenScope::PowerUser, true)]
  #[case(TokenScope::Manager, TokenScope::User, true)]
  #[case(TokenScope::PowerUser, TokenScope::Admin, false)]
  #[case(TokenScope::PowerUser, TokenScope::Manager, false)]
  #[case(TokenScope::PowerUser, TokenScope::PowerUser, false)]
  #[case(TokenScope::PowerUser, TokenScope::User, true)]
  #[case(TokenScope::User, TokenScope::Admin, false)]
  #[case(TokenScope::User, TokenScope::Manager, false)]
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
  #[case(TokenScope::User, "user", "scope_token_user")]
  #[case(TokenScope::PowerUser, "power_user", "scope_token_power_user")]
  #[case(TokenScope::Manager, "manager", "scope_token_manager")]
  #[case(TokenScope::Admin, "admin", "scope_token_admin")]
  fn test_token_scope_string_formats(
    #[case] scope: TokenScope,
    #[case] display: &str,
    #[case] scope_token: &str,
  ) {
    // Test Display format
    assert_eq!(scope.to_string(), display);

    // Test scope token format
    assert_eq!(scope.scope_token(), scope_token);

    // Test serialization
    let serialized = serde_json::to_string(&scope).unwrap();
    assert_eq!(serialized, format!("\"{}\"", display));

    // Test deserialization
    let deserialized: TokenScope = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, scope);
  }

  #[rstest]
  #[case(TokenScope::Admin, vec![TokenScope::Admin, TokenScope::Manager, TokenScope::PowerUser, TokenScope::User])]
  #[case(TokenScope::Manager, vec![TokenScope::Manager, TokenScope::PowerUser, TokenScope::User])]
  #[case(TokenScope::PowerUser, vec![TokenScope::PowerUser, TokenScope::User])]
  #[case(TokenScope::User, vec![TokenScope::User])]
  fn test_included_scopes_explicit(#[case] scope: TokenScope, #[case] expected: Vec<TokenScope>) {
    let included = scope.included_scopes();
    assert_eq!(included, expected);

    // Verify ordering properties
    if !included.is_empty() {
      assert_eq!(*included.first().unwrap(), scope);
      assert_eq!(*included.last().unwrap(), TokenScope::User);
      for window in included.windows(2) {
        assert!(window[0] > window[1]);
      }
    }
  }

  #[rstest]
  #[case("offline_access scope_token_user", Ok(TokenScope::User))]
  #[case(
    "offline_access scope_token_power_user scope_token_user",
    Ok(TokenScope::PowerUser)
  )]
  #[case(
    "offline_access scope_token_manager scope_token_power_user scope_token_user",
    Ok(TokenScope::Manager)
  )]
  #[case(
    "offline_access scope_token_admin scope_token_manager scope_token_power_user scope_token_user",
    Ok(TokenScope::Admin)
  )]
  #[case("offline_access scope_token_user openid profile email", Ok(TokenScope::User))]
  #[case(
    "offline_access openid profile email",
    Err(TokenScopeError::MissingTokenScope)
  )]
  #[case(
    "scope_token_admin scope_token_user",
    Err(TokenScopeError::MissingOfflineAccess)
  )]
  fn test_token_scope_from_scope(
    #[case] scope: &str,
    #[case] expected: Result<TokenScope, TokenScopeError>,
  ) {
    assert_eq!(TokenScope::from_scope(scope), expected);
  }

  #[rstest]
  #[case("offline_access SCOPE_TOKEN_ADMIN", Err(TokenScopeError::MissingTokenScope))]
  #[case(
    "OFFLINE_ACCESS scope_token_admin",
    Err(TokenScopeError::MissingOfflineAccess)
  )]
  #[case("offline_access Scope_Token_User", Err(TokenScopeError::MissingTokenScope))]
  #[case(
    "Offline_Access SCOPE_TOKEN_MANAGER",
    Err(TokenScopeError::MissingOfflineAccess)
  )]
  fn test_token_scope_case_sensitivity(
    #[case] scope: &str,
    #[case] expected: Result<TokenScope, TokenScopeError>,
  ) {
    assert_eq!(TokenScope::from_scope(scope), expected);
  }

  #[rstest]
  #[case("user", Ok(TokenScope::User))]
  #[case("power_user", Ok(TokenScope::PowerUser))]
  #[case("manager", Ok(TokenScope::Manager))]
  #[case("admin", Ok(TokenScope::Admin))]
  fn test_token_scope_parse_valid(
    #[case] input: &str,
    #[case] expected: Result<TokenScope, TokenScopeError>,
  ) {
    assert_eq!(input.parse::<TokenScope>(), expected);
  }

  #[rstest]
  #[case("")]
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
  fn test_token_scope_parse_invalid(#[case] input: &str) {
    let result = input.parse::<TokenScope>();
    assert!(result.is_err());
    assert!(matches!(result, Err(TokenScopeError::InvalidTokenScope(_))));
    assert_eq!(result.unwrap_err().to_string(), "invalid_token_scope");
  }
} 