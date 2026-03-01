use crate::auth::{ResourceRole, RoleError, UserScope};
use errmeta::AppError;
use pretty_assertions::assert_eq;
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
#[case(ResourceRole::User, UserScope::User)]
#[case(ResourceRole::PowerUser, UserScope::PowerUser)]
#[case(ResourceRole::Manager, UserScope::PowerUser)]
#[case(ResourceRole::Admin, UserScope::PowerUser)]
fn test_max_user_scope(#[case] role: ResourceRole, #[case] expected: UserScope) {
  assert_eq!(expected, role.max_user_scope());
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
  let err = input.parse::<ResourceRole>().unwrap_err();
  assert_eq!("role_error-invalid_role_name", err.code());
}
