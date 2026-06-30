use crate::{AccessPolicy, TokenGrantError};
use pretty_assertions::assert_eq;
use rstest::rstest;
use services::{
  AppError, ApprovedResources, ApprovedResourcesV1, AuthContext, DeploymentMode, McpGrant,
  ModelGrant, TokenGrants, TokenGrantsV1, TokenScope, UserScope,
};

fn token(models: ModelGrant, models_list: bool, mcps: McpGrant, mcps_list: bool) -> AuthContext {
  AuthContext::ApiToken {
    client_id: "c".to_string(),
    tenant_id: "t".to_string(),
    user_id: "u".to_string(),
    role: TokenScope::User,
    token: "tok".to_string(),
    grants: TokenGrants::V1(TokenGrantsV1 {
      models_list,
      models,
      mcps_list,
      mcps,
    }),
  }
}

fn specific(ids: &[&str]) -> ModelGrant {
  ModelGrant::Specific {
    ids: ids.iter().map(|s| s.to_string()).collect(),
  }
}

fn external_app(grants: Option<ApprovedResources>) -> AuthContext {
  AuthContext::ExternalApp {
    client_id: "c".to_string(),
    tenant_id: "t".to_string(),
    user_id: "u".to_string(),
    role: Some(UserScope::User),
    token: "tok".to_string(),
    external_app_token: "ext".to_string(),
    app_client_id: "app".to_string(),
    access_request_id: Some("ar".to_string()),
    grants,
  }
}

#[test]
fn unrestricted_principal_passes_everything() {
  let ctx = AuthContext::Anonymous {
    deployment: DeploymentMode::Standalone,
  };
  let policy = AccessPolicy::of(&ctx);
  assert!(policy.model_listable("anything"));
  assert!(policy.mcp_listable("anything"));
  assert!(policy.ensure_model_inference("anything").is_ok());
  assert!(policy.ensure_mcp_connect("anything").is_ok());
}

#[rstest]
// All grant: everything listable + inferable regardless of list flag.
#[case(specific(&["a"]), false, "a", true, true)]
#[case(specific(&["a"]), false, "b", false, false)]
// models_list on: non-granted model is listable but NOT inferable.
#[case(specific(&["a"]), true, "b", true, false)]
#[case(ModelGrant::All, false, "z", true, true)]
fn model_policy_matrix(
  #[case] models: ModelGrant,
  #[case] models_list: bool,
  #[case] model: &str,
  #[case] expect_listable: bool,
  #[case] expect_inferable: bool,
) {
  let ctx = token(models, models_list, McpGrant::All, false);
  let policy = AccessPolicy::of(&ctx);
  assert_eq!(expect_listable, policy.model_listable(model));
  assert_eq!(
    expect_inferable,
    policy.ensure_model_inference(model).is_ok()
  );
}

#[test]
fn model_forbidden_has_forbidden_code() {
  let ctx = token(specific(&["a"]), false, McpGrant::All, false);
  let err = AccessPolicy::of(&ctx)
    .ensure_model_inference("b")
    .unwrap_err();
  assert!(matches!(err, TokenGrantError::ModelForbidden(_)));
  assert_eq!("token_grant_error-model_forbidden", err.code());
}

#[rstest]
#[case(McpGrant::All, false, "x", true, true)]
#[case(McpGrant::Specific { ids: vec![] }, false, "x", false, false)]
// mcps_list on: non-granted mcp is listable but NOT connectable.
#[case(McpGrant::Specific { ids: vec![] }, true, "x", true, false)]
#[case(McpGrant::Specific { ids: vec!["x".to_string()] }, false, "x", true, true)]
#[case(McpGrant::Specific { ids: vec!["x".to_string()] }, false, "y", false, false)]
fn mcp_policy_matrix(
  #[case] mcps: McpGrant,
  #[case] mcps_list: bool,
  #[case] mcp: &str,
  #[case] expect_listable: bool,
  #[case] expect_connectable: bool,
) {
  let ctx = token(ModelGrant::All, false, mcps, mcps_list);
  let policy = AccessPolicy::of(&ctx);
  assert_eq!(expect_listable, policy.mcp_listable(mcp));
  assert_eq!(expect_connectable, policy.ensure_mcp_connect(mcp).is_ok());
}

#[test]
fn external_app_without_grants_is_denied() {
  // No bound access request ⇒ fail closed: no models, no MCPs (not the old
  // pre-grants all-access). An unbound external app can do nothing until approved.
  let ctx = external_app(None);
  let policy = AccessPolicy::of(&ctx);
  assert!(!policy.model_listable("anything"));
  assert!(!policy.mcp_listable("anything"));
  let model_err = policy.ensure_model_inference("anything").unwrap_err();
  assert_eq!("token_grant_error-model_forbidden", model_err.code());
  let mcp_err = policy.ensure_mcp_connect("anything").unwrap_err();
  assert_eq!("token_grant_error-mcp_forbidden", mcp_err.code());
}

#[test]
fn external_app_grants_enforce_like_a_token() {
  // App approved: specific model "a", list off; owner-extra MCP "x9", list off.
  let approved = ApprovedResources::V1(ApprovedResourcesV1 {
    models_list: false,
    models_access: specific(&["a"]),
    mcps_list: false,
    mcps: vec![],
    mcps_access: McpGrant::Specific {
      ids: vec!["x9".to_string()],
    },
  });
  let ctx = external_app(Some(approved));
  let policy = AccessPolicy::of(&ctx);

  assert!(policy.ensure_model_inference("a").is_ok());
  assert!(policy.model_listable("a"));
  let err = policy.ensure_model_inference("b").unwrap_err();
  assert_eq!("token_grant_error-model_forbidden", err.code());
  assert!(!policy.model_listable("b"));

  assert!(policy.ensure_mcp_connect("x9").is_ok());
  let err = policy.ensure_mcp_connect("y").unwrap_err();
  assert_eq!("token_grant_error-mcp_forbidden", err.code());
  assert!(!policy.mcp_listable("y"));
}
