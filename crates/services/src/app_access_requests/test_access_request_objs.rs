use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::{
  mcp_proxy_path, ApprovalStatus, ApprovedResources, ApprovedResourcesV1, McpApproval, McpGrant,
  McpInstance, ModelGrant, RequestedResources, RequestedResourcesV1, ResourceGrants,
};

fn approval(id: &str, status: ApprovalStatus) -> McpApproval {
  McpApproval {
    url: format!("https://mcp.example.com/{id}"),
    status,
    instance: Some(McpInstance {
      id: id.to_string(),
      path: mcp_proxy_path(id),
    }),
  }
}

#[rstest]
// models = All ⇒ every model inferable + listable
#[case(ModelGrant::All, false, "any-model", true, true)]
// Specific contains ⇒ inferable + listable even with list off
#[case(ModelGrant::Specific { ids: vec!["m1".into()] }, false, "m1", true, true)]
// Specific misses ⇒ not inferable, and not listable when list off
#[case(ModelGrant::Specific { ids: vec!["m1".into()] }, false, "m2", false, false)]
// Specific misses but list on ⇒ listable (not inferable)
#[case(ModelGrant::Specific { ids: vec!["m1".into()] }, true, "m2", false, true)]
// empty Specific ⇒ no model access
#[case(ModelGrant::Specific { ids: vec![] }, false, "m1", false, false)]
fn approved_model_predicates(
  #[case] models: ModelGrant,
  #[case] list_models: bool,
  #[case] model: &str,
  #[case] expect_infer: bool,
  #[case] expect_listable: bool,
) {
  let grants = ApprovedResourcesV1 {
    list_models,
    models,
    ..Default::default()
  };
  assert_eq!(expect_infer, grants.allows_model_inference(model));
  assert_eq!(expect_listable, grants.model_listable(model));
}

#[rstest]
// url-tied approved instance ⇒ connectable
#[case(vec![approval("a1", ApprovalStatus::Approved)], McpGrant::Specific { ids: vec![] }, false, "a1", true, true)]
// url-tied denied instance ⇒ not connectable
#[case(vec![approval("a1", ApprovalStatus::Denied)], McpGrant::Specific { ids: vec![] }, false, "a1", false, false)]
// owner-extra grant covers an instance the app never requested
#[case(vec![], McpGrant::Specific { ids: vec!["x9".into()] }, false, "x9", true, true)]
// extra = All ⇒ everything connectable
#[case(vec![], McpGrant::All, false, "whatever", true, true)]
// not granted anywhere, list off ⇒ hidden
#[case(vec![approval("a1", ApprovalStatus::Approved)], McpGrant::Specific { ids: vec![] }, false, "a2", false, false)]
// not granted but list on ⇒ listable (not connectable)
#[case(vec![approval("a1", ApprovalStatus::Approved)], McpGrant::Specific { ids: vec![] }, true, "a2", false, true)]
fn approved_mcp_predicates(
  #[case] mcps: Vec<McpApproval>,
  #[case] mcps_extra: McpGrant,
  #[case] list_mcps: bool,
  #[case] mcp: &str,
  #[case] expect_connect: bool,
  #[case] expect_listable: bool,
) {
  let grants = ApprovedResourcesV1 {
    list_mcps,
    mcps,
    mcps_extra,
    ..Default::default()
  };
  assert_eq!(expect_connect, grants.allows_mcp_connect(mcp));
  assert_eq!(expect_listable, grants.mcp_listable(mcp));
}

#[test]
fn approved_resources_default_is_least_privilege_mcp_but_all_models() {
  // Models default to All (preserves the legacy "apps get all models"); MCP extra
  // defaults to none, NOT the all-access McpGrant::default().
  let v1 = ApprovedResourcesV1::default();
  assert_eq!(ModelGrant::All, v1.models);
  assert_eq!(McpGrant::Specific { ids: vec![] }, v1.mcps_extra);
}

#[test]
fn approved_resources_serde_round_trip() {
  let original = ApprovedResources::V1(ApprovedResourcesV1 {
    list_models: true,
    models: ModelGrant::Specific {
      ids: vec!["m1".into()],
    },
    list_mcps: false,
    mcps: vec![approval("a1", ApprovalStatus::Approved)],
    mcps_extra: McpGrant::Specific {
      ids: vec!["x9".into()],
    },
  });
  let json = serde_json::to_string(&original).unwrap();
  let parsed: ApprovedResources = serde_json::from_str(&json).unwrap();
  assert_eq!(original, parsed);
}

#[test]
fn approved_resources_legacy_json_deserializes_with_defaults() {
  // Pre-grants approval JSON carried only `mcps`. It must still deserialize, with
  // the new fields taking their defaults.
  let legacy =
    r#"{"version":"1","mcps":[{"url":"https://m/x","status":"approved","instance":{"id":"a1"}}]}"#;
  let parsed: ApprovedResources = serde_json::from_str(legacy).unwrap();
  let v1 = parsed.v1();
  assert!(!v1.list_models);
  assert_eq!(ModelGrant::All, v1.models);
  assert!(!v1.list_mcps);
  assert_eq!(McpGrant::Specific { ids: vec![] }, v1.mcps_extra);
  assert_eq!(1, v1.mcps.len());
}

#[test]
fn requested_resources_legacy_json_deserializes_with_default_flags() {
  let legacy = r#"{"version":"1","mcp_servers":[{"url":"https://m/x"}]}"#;
  let parsed: RequestedResources = serde_json::from_str(legacy).unwrap();
  let RequestedResources::V1(v1) = parsed;
  assert_eq!(
    RequestedResourcesV1 {
      mcp_servers: v1.mcp_servers.clone(),
      ..Default::default()
    },
    v1
  );
  assert!(!v1.models_list && !v1.models_access && !v1.mcps_list && !v1.mcps_access);
}
