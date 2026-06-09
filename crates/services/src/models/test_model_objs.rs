use crate::models::{
  Alias, AliasResponse, AliasSource, ApiAlias, ApiFormat, FallbackConfig, ModelAlias,
  ModelRouterAlias, OAIRequestParams, Repo, RouterTarget, RoutingStrategyConfig, UserAlias,
};
use crate::test_utils::{fixed_dt, openai_model};
use crate::UserAliasBuilder;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;

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
    "test-name",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec![openai_model("gpt-4")],
    None,
    false,
    fixed_dt(),
    None,
    None,
  );
  let alias = Alias::Api(api_alias);
  assert_eq!(expected, alias.can_serve(model));
}

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
  #[case] model_ids: Vec<&str>,
  #[case] forward_all: bool,
  #[case] expected: bool,
) {
  let api_alias = ApiAlias::new(
    "test-api",
    "test-name",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    model_ids
      .into_iter()
      .map(|id| openai_model(id))
      .collect::<Vec<_>>(),
    prefix.map(|s| s.to_string()),
    forward_all,
    fixed_dt(),
    None,
    None,
  );
  assert_eq!(expected, api_alias.supports_model(model));
}

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
  #[case] model_ids: Vec<&str>,
  #[case] forward_all: bool,
  #[case] expected: Vec<&str>,
) {
  let api_alias = ApiAlias::new(
    "test-api",
    "test-name",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    model_ids
      .into_iter()
      .map(|id| openai_model(id))
      .collect::<Vec<_>>(),
    prefix.map(|s| s.to_string()),
    forward_all,
    fixed_dt(),
    None,
    None,
  );
  let matchable: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
  assert_eq!(matchable, api_alias.matchable_models());
}

#[rstest]
fn test_user_alias_serde_roundtrip() {
  let alias = UserAliasBuilder::testalias().build_test().unwrap();
  let json = serde_json::to_string(&alias).expect("serialize");
  let back: UserAlias = serde_json::from_str(&json).expect("deserialize");
  assert_eq!(alias, back);
}

#[rstest]
#[case::openai(ApiFormat::OpenAI, r#""openai""#)]
#[case::openai_responses(ApiFormat::OpenAIResponses, r#""openai_responses""#)]
#[case::anthropic(ApiFormat::Anthropic, r#""anthropic""#)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth, r#""anthropic_oauth""#)]
#[case::gemini(ApiFormat::Gemini, r#""gemini""#)]
fn test_api_format_serde_roundtrip(#[case] format: ApiFormat, #[case] expected_json: &str) {
  let serialized = serde_json::to_string(&format).expect("serialize");
  assert_eq!(expected_json, serialized);
  let deserialized: ApiFormat = serde_json::from_str(&serialized).expect("deserialize");
  assert_eq!(format, deserialized);
}

fn test_router() -> ModelRouterAlias {
  ModelRouterAlias {
    id: "router-1".to_string(),
    alias: "my-stack".to_string(),
    targets: vec![
      RouterTarget {
        alias: "openai-gpt".to_string(),
        model: "gpt-4o".to_string(),
        enabled: true,
        weight: None,
      },
      RouterTarget {
        alias: "claude".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        enabled: false,
        weight: None,
      },
    ],
    strategy: RoutingStrategyConfig::default(),
    created_at: fixed_dt(),
    updated_at: fixed_dt(),
  }
}

#[rstest]
fn test_alias_source_model_router_snake_case() {
  // The source tag must serialize as snake_case, never kebab-case.
  let serialized = serde_json::to_string(&AliasSource::ModelRouter).expect("serialize");
  assert_eq!(r#""model_router""#, serialized);
}

#[rstest]
fn test_alias_model_router_source_tag() {
  let alias = Alias::ModelRouter(test_router());
  let value = serde_json::to_value(&alias).expect("serialize");
  assert_eq!("model_router", value["source"].as_str().unwrap());
  let back: Alias = serde_json::from_value(value).expect("deserialize");
  assert_eq!(alias, back);
}

#[rstest]
fn test_routing_strategy_config_fallback_tag() {
  let strategy = RoutingStrategyConfig::Fallback(FallbackConfig::default());
  let value = serde_json::to_value(&strategy).expect("serialize");
  assert_eq!("fallback", value["strategy"].as_str().unwrap());
  assert_eq!(30, value["cooldown_secs"].as_u64().unwrap());
  assert_eq!(0, value["max_attempts"].as_u64().unwrap());
  assert_eq!(true, value["honor_retry_after"].as_bool().unwrap());
}

#[rstest]
fn test_router_target_enabled_defaults_true() {
  let json = json!({ "alias": "openai-gpt", "model": "gpt-4o" });
  let target: RouterTarget = serde_json::from_value(json).expect("deserialize");
  assert!(target.enabled);
  assert_eq!(None, target.weight);
}

#[rstest]
fn test_alias_can_serve_model_router_exact_match() {
  let alias = Alias::ModelRouter(test_router());
  assert!(alias.can_serve("my-stack"));
  assert!(!alias.can_serve("my-stack-other"));
  assert!(alias.is_model_router());
  assert_eq!("my-stack", alias.alias_name());
  assert_eq!(AliasSource::ModelRouter, alias.source());
}

#[rstest]
fn test_alias_response_model_router_untagged_roundtrip() {
  let response: AliasResponse = AliasResponse::from(Alias::ModelRouter(test_router()));
  let json = serde_json::to_string(&response).expect("serialize");
  let back: AliasResponse = serde_json::from_str(&json).expect("deserialize");
  assert_eq!(response, back);
  // A model-router response must not be mis-deserialized as another untagged variant.
  assert!(matches!(back, AliasResponse::ModelRouter(_)));
}

#[rstest]
#[case::openai(ApiFormat::OpenAI, true)]
#[case::anthropic(ApiFormat::Anthropic, true)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth, true)]
#[case::openai_responses(ApiFormat::OpenAIResponses, false)]
#[case::gemini(ApiFormat::Gemini, false)]
#[case::llm_liberty(ApiFormat::LlmLibertyOauth, false)]
fn test_api_format_supports_chat_completions(#[case] format: ApiFormat, #[case] expected: bool) {
  assert_eq!(expected, format.supports_chat_completions());
}
