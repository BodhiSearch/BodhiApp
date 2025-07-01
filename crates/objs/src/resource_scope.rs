use crate::{TokenScope, UserScope};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceScope {
  Token(TokenScope),
  User(UserScope),
}

impl std::fmt::Display for ResourceScope {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ResourceScope::Token(scope) => scope.fmt(f),
      ResourceScope::User(scope) => scope.fmt(f),
    }
  }
}
#[cfg(test)]
mod tests {
  use crate::{ResourceScope, TokenScope, UserScope};
  use rstest::rstest;

  #[rstest]
  #[case(ResourceScope::Token(TokenScope::User), "scope_token_user")]
  #[case(ResourceScope::Token(TokenScope::PowerUser), "scope_token_power_user")]
  #[case(ResourceScope::Token(TokenScope::Manager), "scope_token_manager")]
  #[case(ResourceScope::Token(TokenScope::Admin), "scope_token_admin")]
  #[case(ResourceScope::User(UserScope::User), "scope_user_user")]
  #[case(ResourceScope::User(UserScope::PowerUser), "scope_user_power_user")]
  #[case(ResourceScope::User(UserScope::Manager), "scope_user_manager")]
  #[case(ResourceScope::User(UserScope::Admin), "scope_user_admin")]
  fn test_resource_scope_display(#[case] resource_scope: ResourceScope, #[case] expected: &str) {
    assert_eq!(resource_scope.to_string(), expected);
  }
}
