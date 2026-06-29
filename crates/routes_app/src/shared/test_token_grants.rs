use crate::{AccessPolicy, TokenGrantError};
use pretty_assertions::assert_eq;
use rstest::rstest;
use services::{
  AppError, AuthContext, DeploymentMode, McpGrant, ModelGrant, TokenGrants, TokenGrantsV1,
  TokenScope,
};

fn token(models: ModelGrant, list_models: bool, mcps: McpGrant, list_mcps: bool) -> AuthContext {
  AuthContext::ApiToken {
    client_id: "c".to_string(),
    tenant_id: "t".to_string(),
    user_id: "u".to_string(),
    role: TokenScope::User,
    token: "tok".to_string(),
    grants: TokenGrants::V1(TokenGrantsV1 {
      list_models,
      models,
      list_mcps,
      mcps,
    }),
  }
}

fn specific(ids: &[&str]) -> ModelGrant {
  ModelGrant::Specific {
    ids: ids.iter().map(|s| s.to_string()).collect(),
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
// list_models on: non-granted model is listable but NOT inferable.
#[case(specific(&["a"]), true, "b", true, false)]
#[case(ModelGrant::All, false, "z", true, true)]
fn model_policy_matrix(
  #[case] models: ModelGrant,
  #[case] list_models: bool,
  #[case] model: &str,
  #[case] expect_listable: bool,
  #[case] expect_inferable: bool,
) {
  let ctx = token(models, list_models, McpGrant::All, false);
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
#[case(McpGrant::None, false, "x", false, false)]
// list_mcps on: non-granted mcp is listable but NOT connectable.
#[case(McpGrant::None, true, "x", true, false)]
#[case(McpGrant::Specific { ids: vec!["x".to_string()] }, false, "x", true, true)]
#[case(McpGrant::Specific { ids: vec!["x".to_string()] }, false, "y", false, false)]
fn mcp_policy_matrix(
  #[case] mcps: McpGrant,
  #[case] list_mcps: bool,
  #[case] mcp: &str,
  #[case] expect_listable: bool,
  #[case] expect_connectable: bool,
) {
  let ctx = token(ModelGrant::All, false, mcps, list_mcps);
  let policy = AccessPolicy::of(&ctx);
  assert_eq!(expect_listable, policy.mcp_listable(mcp));
  assert_eq!(expect_connectable, policy.ensure_mcp_connect(mcp).is_ok());
}
