use crate::auth::{UserScope, UserScopeError};
use errmeta::AppError;
use pretty_assertions::assert_eq;
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
  let err = input.parse::<UserScope>().unwrap_err();
  assert_eq!("user_scope_error-invalid_user_scope", err.code());
}
