use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::app_access_requests::{
  access_request_scope_value, compose_keycloak_scope, match_redirect_uri, parse_app_scope,
  AppScopeError, ParsedAppScope, RedirectUriValidation,
};
use crate::{ApprovalStatus, ApprovedResourcesV1, McpApproval, McpGrant, ModelGrant, UserScope};

fn parsed(role: UserScope, llms: bool, mcps: bool, passthrough: &[&str]) -> ParsedAppScope {
  ParsedAppScope {
    role,
    llms,
    mcps,
    passthrough: passthrough.iter().map(|s| s.to_string()).collect(),
  }
}

fn scope_flags(llms: bool, mcps: bool) -> ParsedAppScope {
  parsed(UserScope::User, llms, mcps, &[])
}

fn mcp_approval(id: &str) -> McpApproval {
  McpApproval {
    url: format!("https://mcp.example.com/{id}"),
    status: ApprovalStatus::Approved,
    instance: None,
  }
}

#[rstest]
#[case::empty_scope("", parsed(UserScope::User, true, true, &[]))]
#[case::role_user("scope_user_user", parsed(UserScope::User, true, true, &[]))]
#[case::role_power_user("scope_user_power_user", parsed(UserScope::PowerUser, true, true, &[]))]
#[case::both_roles_power_user_wins(
  "scope_user_user scope_user_power_user",
  parsed(UserScope::PowerUser, true, true, &[])
)]
#[case::both_roles_reversed(
  "scope_user_power_user scope_user_user",
  parsed(UserScope::PowerUser, true, true, &[])
)]
#[case::llms_bare("scope_apps:llms", parsed(UserScope::User, true, true, &[]))]
#[case::llms_true("scope_apps:llms:true", parsed(UserScope::User, true, true, &[]))]
#[case::llms_false("scope_apps:llms:false", parsed(UserScope::User, false, true, &[]))]
#[case::mcps_bare("scope_apps:mcps", parsed(UserScope::User, true, true, &[]))]
#[case::mcps_true("scope_apps:mcps:true", parsed(UserScope::User, true, true, &[]))]
#[case::mcps_false("scope_apps:mcps:false", parsed(UserScope::User, true, false, &[]))]
#[case::role_only_both_false(
  "scope_user_power_user scope_apps:llms:false scope_apps:mcps:false",
  parsed(UserScope::PowerUser, false, false, &[])
)]
#[case::same_value_duplicates_ok(
  "scope_apps:llms scope_apps:llms:true scope_apps:mcps:false scope_apps:mcps:false",
  parsed(UserScope::User, true, false, &[])
)]
#[case::passthrough_deduped_order_preserving(
  "openid profile offline_access arbitrary-scope openid",
  parsed(UserScope::User, true, true, &["openid", "profile", "offline_access", "arbitrary-scope"])
)]
#[case::passthrough_mixed_with_consumed_tokens(
  "openid scope_user_user scope_apps:llms:false profile",
  parsed(UserScope::User, false, true, &["openid", "profile"])
)]
fn test_parse_app_scope_ok(
  #[case] scope: &str,
  #[case] expected: ParsedAppScope,
) -> anyhow::Result<()> {
  let actual = parse_app_scope(scope)?;
  assert_eq!(expected, actual);
  Ok(())
}

#[rstest]
#[case::conflicting_llms(
  "scope_apps:llms scope_apps:llms:false",
  "app_scope_error-conflicting_scope_token"
)]
#[case::conflicting_mcps(
  "scope_apps:mcps:false scope_apps:mcps:true",
  "app_scope_error-conflicting_scope_token"
)]
#[case::malformed_apps_token("scope_apps:garbage", "app_scope_error-malformed_scope_token")]
#[case::unknown_user_scope("scope_user_admin", "app_scope_error-malformed_scope_token")]
#[case::reserved_access_request_token(
  "scope_access_request:x.y",
  "app_scope_error-reserved_scope_token"
)]
#[case::reserved_access_request_token_mixed_case(
  "Scope_Access_Request:x.y",
  "app_scope_error-reserved_scope_token"
)]
fn test_parse_app_scope_err(
  #[case] scope: &str,
  #[case] expected_code: &str,
) -> anyhow::Result<()> {
  let err = parse_app_scope(scope).unwrap_err();
  assert_eq!(expected_code, err.code());
  Ok(())
}

#[rstest]
fn test_access_request_scope_value_valid() -> anyhow::Result<()> {
  let actual = access_request_scope_value("resource-client", "01ARZ3NDEKTSV4RRFFQ69G5FAV")?;
  assert_eq!(
    "scope_access_request:resource-client.01ARZ3NDEKTSV4RRFFQ69G5FAV",
    actual
  );
  Ok(())
}

#[rstest]
#[case::empty_client("", "ar-1")]
#[case::empty_id("resource-client", "")]
#[case::id_contains_dot("resource-client", "ar.1")]
fn test_access_request_scope_value_invalid_composition(
  #[case] resource_client_id: &str,
  #[case] access_request_id: &str,
) -> anyhow::Result<()> {
  let err = access_request_scope_value(resource_client_id, access_request_id).unwrap_err();
  assert_eq!("app_scope_error-invalid_scope_composition", err.code());
  Ok(())
}

#[rstest]
#[case::no_passthrough("", "openid profile email roles scope_access_request:rc.ar1")]
#[case::openid_duplicate_collapsed(
  "openid profile custom-scope",
  "openid profile email roles custom-scope scope_access_request:rc.ar1"
)]
#[case::passthrough_appended_after_base(
  "offline_access",
  "openid profile email roles offline_access scope_access_request:rc.ar1"
)]
fn test_compose_keycloak_scope(#[case] scope: &str, #[case] expected: &str) -> anyhow::Result<()> {
  let parsed = parse_app_scope(scope)?;
  let actual = compose_keycloak_scope(&parsed, "scope_access_request:rc.ar1");
  assert_eq!(expected, actual);
  Ok(())
}

#[rstest]
#[case::llms_false_specific_models(
  scope_flags(false, true),
  ApprovedResourcesV1 {
    models_access: ModelGrant::Specific { ids: vec!["m1".to_string()] },
    ..Default::default()
  },
  Some("models")
)]
#[case::llms_false_all_models(
  scope_flags(false, true),
  ApprovedResourcesV1 { models_access: ModelGrant::All, ..Default::default() },
  Some("models")
)]
#[case::llms_false_models_list(
  scope_flags(false, true),
  ApprovedResourcesV1 { models_list: true, ..Default::default() },
  Some("models")
)]
#[case::llms_false_empty_grant_ok(scope_flags(false, true), ApprovedResourcesV1::default(), None)]
#[case::mcps_false_mcps_list(
  scope_flags(true, false),
  ApprovedResourcesV1 { mcps_list: true, ..Default::default() },
  Some("mcps")
)]
#[case::mcps_false_mcp_approvals(
  scope_flags(true, false),
  ApprovedResourcesV1 { mcps: vec![mcp_approval("a1")], ..Default::default() },
  Some("mcps")
)]
#[case::mcps_false_specific_mcps_access(
  scope_flags(true, false),
  ApprovedResourcesV1 {
    mcps_access: McpGrant::Specific { ids: vec!["x1".to_string()] },
    ..Default::default()
  },
  Some("mcps")
)]
#[case::mcps_false_all_mcps_access(
  scope_flags(true, false),
  ApprovedResourcesV1 { mcps_access: McpGrant::All, ..Default::default() },
  Some("mcps")
)]
#[case::mcps_false_empty_grant_ok(scope_flags(true, false), ApprovedResourcesV1::default(), None)]
#[case::scope_allowed_full_grant_ok(
  scope_flags(true, true),
  ApprovedResourcesV1 {
    models_list: true,
    models_access: ModelGrant::All,
    mcps_list: true,
    mcps: vec![mcp_approval("a1")],
    mcps_access: McpGrant::All,
  },
  None
)]
fn test_validate_grant_against_scope(
  #[case] parsed: ParsedAppScope,
  #[case] approved: ApprovedResourcesV1,
  #[case] expected_section: Option<&str>,
) -> anyhow::Result<()> {
  let result = crate::app_access_requests::validate_grant_against_scope(&parsed, &approved);
  match expected_section {
    None => assert!(result.is_ok(), "expected Ok, got {:?}", result),
    Some(section) => {
      let err = result.unwrap_err();
      assert_eq!("app_scope_error-grant_exceeds_scope", err.code());
      assert!(
        matches!(&err, AppScopeError::GrantExceedsScope { section: s } if s == section),
        "expected section '{}', got {:?}",
        section,
        err
      );
    }
  }
  Ok(())
}

#[rstest]
#[case::no_registered_list_unvalidated(None, RedirectUriValidation::Unvalidated)]
#[case::empty_registered_list_mismatch(Some(vec![]), RedirectUriValidation::Mismatch)]
#[case::exact_match_valid(
  Some(vec!["https://app.example.com/cb".to_string()]),
  RedirectUriValidation::Valid
)]
#[case::trailing_slash_mismatch(
  Some(vec!["https://app.example.com/cb/".to_string()]),
  RedirectUriValidation::Mismatch
)]
fn test_match_redirect_uri(
  #[case] registered: Option<Vec<String>>,
  #[case] expected: RedirectUriValidation,
) -> anyhow::Result<()> {
  let actual = match_redirect_uri("https://app.example.com/cb", registered.as_deref());
  assert_eq!(expected, actual);
  Ok(())
}
