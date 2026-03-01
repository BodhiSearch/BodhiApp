use crate::models::{Alias, ApiAlias, ApiFormat, ModelAlias, OAIRequestParams, Repo, UserAlias};
use crate::test_utils::fixed_dt;
use crate::UserAliasBuilder;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;

// =============================================================================
// Alias::can_serve
// =============================================================================

#[rstest]
#[case::user_alias_exact_match("testalias:instruct", true)]
#[case::user_alias_no_match("other:model", false)]
fn test_alias_can_serve_user(#[case] model: &str, #[case] expected: bool) {
  let user_alias = UserAliasBuilder::testalias().build_test().unwrap();
  let alias = Alias::User(user_alias);
  assert_eq!(expected, alias.can_serve(model));
}

#[rstest]
#[case::model_alias_exact_match("llama3:instruct", true)]
#[case::model_alias_no_match("other:model", false)]
fn test_alias_can_serve_model(#[case] model: &str, #[case] expected: bool) {
  let model_alias = ModelAlias {
    alias: "llama3:instruct".to_string(),
    repo: Repo::new("meta-llama", "Llama-3"),
    filename: "llama3.gguf".to_string(),
    snapshot: "main".to_string(),
  };
  let alias = Alias::Model(model_alias);
  assert_eq!(expected, alias.can_serve(model));
}

#[rstest]
#[case::api_alias_matches_model("gpt-4", true)]
#[case::api_alias_no_match("gpt-3.5-turbo", false)]
fn test_alias_can_serve_api(#[case] model: &str, #[case] expected: bool) {
  let api_alias = ApiAlias::new(
    "openai-api",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    fixed_dt(),
  );
  let alias = Alias::Api(api_alias);
  assert_eq!(expected, alias.can_serve(model));
}

// =============================================================================
// ApiAlias::supports_model
// =============================================================================

#[rstest]
#[case::exact_match_no_prefix("gpt-4", None, vec!["gpt-4"], false, true)]
#[case::no_match_no_prefix("gpt-3.5-turbo", None, vec!["gpt-4"], false, false)]
#[case::prefix_match("azure/gpt-4", Some("azure/"), vec!["gpt-4"], false, true)]
#[case::prefix_no_match("gpt-4", Some("azure/"), vec!["gpt-4"], false, false)]
#[case::forward_all_with_prefix("azure/gpt-4", Some("azure/"), vec![], true, true)]
#[case::forward_all_no_match("other/gpt-4", Some("azure/"), vec![], true, false)]
fn test_api_alias_supports_model(
  #[case] model: &str,
  #[case] prefix: Option<&str>,
  #[case] models: Vec<&str>,
  #[case] forward_all: bool,
  #[case] expected: bool,
) {
  let api_alias = ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    models
      .into_iter()
      .map(|s| s.to_string())
      .collect::<Vec<_>>(),
    prefix.map(|s| s.to_string()),
    forward_all,
    fixed_dt(),
  );
  assert_eq!(expected, api_alias.supports_model(model));
}

// =============================================================================
// ApiAlias new() initializes cache_fetched_at to UNIX_EPOCH
// =============================================================================

#[rstest]
fn test_api_alias_new_epoch_sentinel() {
  let api_alias = ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    fixed_dt(),
  );
  assert_eq!(chrono::DateTime::UNIX_EPOCH, api_alias.cache_fetched_at);
}

// =============================================================================
// OAIRequestParams::apply_to_value
// =============================================================================

#[rstest]
fn test_oai_request_params_apply_to_value_sets_missing_fields() {
  let params = OAIRequestParams {
    temperature: Some(0.7),
    max_tokens: Some(512),
    ..Default::default()
  };
  let mut value = json!({
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello"}]
  });
  params.apply_to_value(&mut value);
  // temperature is set from f32 (0.7f32), so check approximate presence
  assert!(
    value["temperature"].is_number(),
    "temperature should be a number"
  );
  assert_eq!(json!(512), value["max_completion_tokens"]);
}

#[rstest]
fn test_oai_request_params_apply_to_value_does_not_override_existing() {
  let params = OAIRequestParams {
    temperature: Some(0.7),
    ..Default::default()
  };
  let mut value = json!({
    "model": "gpt-4",
    "temperature": 0.2,
    "messages": [{"role": "user", "content": "Hello"}]
  });
  params.apply_to_value(&mut value);
  // temperature already present — should not be overridden
  assert_eq!(json!(0.2), value["temperature"]);
}

#[rstest]
#[case::no_prefix_unchanged(
  "gpt-4",
  None,
  vec!["gpt-4"],
  false,
  vec!["gpt-4"]
)]
#[case::prefix_prepended(
  "azure/",
  Some("azure/"),
  vec!["gpt-4", "gpt-3.5-turbo"],
  false,
  vec!["azure/gpt-4", "azure/gpt-3.5-turbo"]
)]
fn test_api_alias_matchable_models(
  #[case] _label: &str,
  #[case] prefix: Option<&str>,
  #[case] models: Vec<&str>,
  #[case] forward_all: bool,
  #[case] expected: Vec<&str>,
) {
  let api_alias = ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    models
      .into_iter()
      .map(|s| s.to_string())
      .collect::<Vec<_>>(),
    prefix.map(|s| s.to_string()),
    forward_all,
    fixed_dt(),
  );
  let matchable: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
  assert_eq!(matchable, api_alias.matchable_models());
}

// =============================================================================
// UserAlias serialization/deserialization round-trip
// =============================================================================

#[rstest]
fn test_user_alias_serde_roundtrip() {
  let alias = UserAliasBuilder::testalias().build_test().unwrap();
  let json = serde_json::to_string(&alias).expect("serialize");
  let back: UserAlias = serde_json::from_str(&json).expect("deserialize");
  assert_eq!(alias, back);
}
