use objs::{AppError, ErrorType, TokenScope, UserScope};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceScope {
  Token(TokenScope),
  User(Option<UserScope>),
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ResourceScopeError {
  #[error("invalid resource scope: {0}")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidScope(String),
}

impl ResourceScope {
  pub fn try_parse(scope_str: &str) -> Result<Self, ResourceScopeError> {
    // Try to parse as TokenScope first
    if let Ok(token_scope) = scope_str.parse::<TokenScope>() {
      return Ok(ResourceScope::Token(token_scope));
    }

    // Try to parse as UserScope
    if let Ok(user_scope) = scope_str.parse::<UserScope>() {
      return Ok(ResourceScope::User(Some(user_scope)));
    }

    Err(ResourceScopeError::InvalidScope(scope_str.to_string()))
  }
}

impl std::fmt::Display for ResourceScope {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ResourceScope::Token(scope) => scope.fmt(f),
      ResourceScope::User(Some(scope)) => scope.fmt(f),
      ResourceScope::User(None) => write!(f, "user:none"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(ResourceScope::Token(TokenScope::User), "scope_token_user")]
  #[case(ResourceScope::Token(TokenScope::PowerUser), "scope_token_power_user")]
  #[case(ResourceScope::Token(TokenScope::Manager), "scope_token_manager")]
  #[case(ResourceScope::Token(TokenScope::Admin), "scope_token_admin")]
  #[case(ResourceScope::User(Some(UserScope::User)), "scope_user_user")]
  #[case(ResourceScope::User(Some(UserScope::PowerUser)), "scope_user_power_user")]
  #[case(ResourceScope::User(Some(UserScope::Manager)), "scope_user_manager")]
  #[case(ResourceScope::User(Some(UserScope::Admin)), "scope_user_admin")]
  fn test_resource_scope_display(#[case] resource_scope: ResourceScope, #[case] expected: &str) {
    assert_eq!(resource_scope.to_string(), expected);
  }
}
