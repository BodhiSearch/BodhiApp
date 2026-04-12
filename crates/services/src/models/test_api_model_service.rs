use crate::models::{
  ApiAliasRepository, ApiFormat, ApiKeyUpdate, ApiModelRequest, ApiModelService,
  DefaultApiModelService,
};
use crate::test_utils::{
  openai_model, test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID, TEST_USER_ID,
};
use crate::MockAiApiService;
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use std::sync::Arc;

fn extra_headers_for(format: &ApiFormat) -> Option<serde_json::Value> {
  match format {
    ApiFormat::AnthropicOAuth => {
      Some(json!({"anthropic-beta": "oauth-2025-04-20", "user-agent": "claude-cli/2.1.80"}))
    }
    _ => None,
  }
}

fn extra_body_for(format: &ApiFormat) -> Option<serde_json::Value> {
  match format {
    ApiFormat::AnthropicOAuth => Some(json!({
      "max_tokens": 32000,
      "system": [{"type": "text", "text": "You are Claude Code"}]
    })),
    _ => None,
  }
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_forward_all_stores_all_models(
  #[case] api_format: ApiFormat,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _, _, _| Ok(vec![openai_model("model-a"), openai_model("model-b")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let extra_headers = extra_headers_for(&api_format);
  let extra_body = extra_body_for(&api_format);
  let form = ApiModelRequest {
    api_format,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("test".to_string()),
    forward_all_with_prefix: true,
    extra_headers: extra_headers.clone(),
    extra_body: extra_body.clone(),
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert!(result.forward_all_with_prefix);
  assert_eq!(
    vec![openai_model("model-a"), openai_model("model-b")],
    result.models
  );
  assert_eq!(extra_headers, result.extra_headers);
  assert_eq!(extra_body, result.extra_body);

  // Verify extra fields stored in DB
  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?
    .expect("alias should exist");
  assert_eq!(
    vec![openai_model("model-a"), openai_model("model-b")],
    alias.models.to_vec()
  );

  Ok(())
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_non_forward_all_validates_and_filters(
  #[case] api_format: ApiFormat,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _, _, _| Ok(vec![openai_model("gpt-4"), openai_model("gpt-3.5")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert!(!result.forward_all_with_prefix);
  assert_eq!(vec![openai_model("gpt-4")], result.models);

  Ok(())
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_forward_all_stores_all_models(
  #[case] api_format: ApiFormat,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  // create() fetches once, update() fetches once
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(|_, _, _, _, _| Ok(vec![openai_model("remote-1"), openai_model("remote-2")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let extra_headers = extra_headers_for(&api_format);
  let extra_body = extra_body_for(&api_format);
  let create_form = ApiModelRequest {
    api_format: api_format.clone(),
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("initial".to_string()),
    forward_all_with_prefix: true,
    extra_headers: extra_headers.clone(),
    extra_body: extra_body.clone(),
  };
  let created = service
    .create(TEST_TENANT_ID, TEST_USER_ID, create_form)
    .await?;

  let update_form = ApiModelRequest {
    api_format,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("test".to_string()),
    forward_all_with_prefix: true,
    extra_headers: extra_headers.clone(),
    extra_body: extra_body.clone(),
  };
  let result = service
    .update(TEST_TENANT_ID, TEST_USER_ID, &created.id, update_form)
    .await?;
  assert!(result.forward_all_with_prefix);
  assert_eq!(
    vec![openai_model("remote-1"), openai_model("remote-2")],
    result.models
  );
  assert_eq!(extra_headers, result.extra_headers);
  assert_eq!(extra_body, result.extra_body);

  Ok(())
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_non_forward_all_validates_and_filters(
  #[case] api_format: ApiFormat,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  // create() calls fetch_models once, update() calls fetch_models once
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(|_, _, _, _, _| Ok(vec![openai_model("gpt-4"), openai_model("gpt-3.5")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let create_form = ApiModelRequest {
    api_format: api_format.clone(),
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };
  let created = service
    .create(TEST_TENANT_ID, TEST_USER_ID, create_form)
    .await?;

  let update_form = ApiModelRequest {
    api_format,
    base_url: "https://api.example.com/v2".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string(), "gpt-3.5".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };
  let result = service
    .update(TEST_TENANT_ID, TEST_USER_ID, &created.id, update_form)
    .await?;
  assert!(!result.forward_all_with_prefix);
  let mut model_ids: Vec<&str> = result.models.iter().map(|m| m.id()).collect();
  model_ids.sort();
  assert_eq!(vec!["gpt-3.5", "gpt-4"], model_ids);

  Ok(())
}

// Pass-through auth rejection at service layer (defense-in-depth on top of DTO validator).
#[rstest]
#[case::authorization(json!({"Authorization": "Bearer x"}), "Authorization")]
#[case::authorization_lower(json!({"authorization": "Bearer x"}), "authorization")]
#[case::x_api_key(json!({"x-api-key": "sk-x"}), "x-api-key")]
#[case::x_api_key_camel(json!({"X-Api-Key": "sk-x"}), "X-Api-Key")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_rejects_extra_headers_pass_through_auth(
  #[case] extra_headers: serde_json::Value,
  #[case] forbidden_key: &str,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);
  let mock_ai = MockAiApiService::new();
  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service, time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format: ApiFormat::AnthropicOAuth,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["claude-3".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: Some(extra_headers),
    extra_body: None,
  };

  let err = service
    .create(TEST_TENANT_ID, TEST_USER_ID, form)
    .await
    .expect_err("must reject forbidden auth header");
  let msg = format!("{}", err);
  assert!(
    msg.contains("Cannot have pass-through authorization headers") && msg.contains(forbidden_key),
    "expected forbidden-key error mentioning `{}`, got: {}",
    forbidden_key,
    msg
  );
  Ok(())
}

// api_format change + ApiKeyUpdate::Keep must be rejected (stored key is provider-specific).
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_rejects_api_format_change_with_keep_key(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);
  let mut mock_ai = MockAiApiService::new();
  // Only create() fetches; update() must bail before fetch_models.
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _, _, _| Ok(vec![openai_model("gpt-4")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service, time_service, Arc::new(mock_ai));

  let create_form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };
  let created = service
    .create(TEST_TENANT_ID, TEST_USER_ID, create_form)
    .await?;

  let update_form = ApiModelRequest {
    api_format: ApiFormat::AnthropicOAuth,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("anth/".to_string()),
    forward_all_with_prefix: true,
    extra_headers: None,
    extra_body: None,
  };
  let err = service
    .update(TEST_TENANT_ID, TEST_USER_ID, &created.id, update_form)
    .await
    .expect_err("must reject Keep when api_format changes");
  let msg = format!("{}", err);
  assert!(
    msg.contains("Changing api_format requires a new api_key"),
    "expected ApiFormatChangedRequiresNewKey, got: {}",
    msg
  );
  Ok(())
}
