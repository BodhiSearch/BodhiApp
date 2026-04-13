use crate::models::{
  ApiAliasRepository, ApiFormat, ApiKeyUpdate, ApiModel, ApiModelRequest, ApiModelService,
  DefaultApiModelService,
};
use crate::test_utils::{
  gemini_model, openai_model, test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID,
  TEST_USER_ID,
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

/// Returns two provider models appropriate for the given format.
fn two_models_for(format: &ApiFormat) -> Vec<ApiModel> {
  match format {
    ApiFormat::Gemini => vec![
      ApiModel::Gemini(gemini_model("gemini-2.5-flash")),
      ApiModel::Gemini(gemini_model("gemini-2.5-pro")),
    ],
    _ => vec![openai_model("model-a"), openai_model("model-b")],
  }
}

/// Returns two named provider models for selection tests.
fn two_named_models_for(format: &ApiFormat, id_a: &str, id_b: &str) -> Vec<ApiModel> {
  match format {
    ApiFormat::Gemini => vec![
      ApiModel::Gemini(gemini_model(id_a)),
      ApiModel::Gemini(gemini_model(id_b)),
    ],
    _ => vec![openai_model(id_a), openai_model(id_b)],
  }
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
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
  let expected_models = two_models_for(&api_format);
  let expected_models_clone = expected_models.clone();

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(move |_, _, _, _, _| Ok(expected_models_clone.clone()));

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
  let stored_ids: Vec<&str> = result.models.iter().map(|m| m.id()).collect();
  let expected_stored_ids: Vec<String> =
    expected_models.iter().map(|m| m.id().to_string()).collect();
  assert_eq!(
    expected_stored_ids,
    stored_ids.iter().map(|s| s.to_string()).collect::<Vec<_>>()
  );
  assert_eq!(extra_headers, result.extra_headers);
  assert_eq!(extra_body, result.extra_body);

  // Verify models stored correctly in DB (proves ApiModelVec round-trip for the format)
  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?
    .expect("alias should exist");
  assert_eq!(result.models, alias.models.to_vec());
  assert_eq!(extra_headers, alias.extra_headers);
  assert_eq!(extra_body, alias.extra_body);

  Ok(())
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
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
  let provider_models = two_named_models_for(&api_format, "model-x", "model-y");
  let expected_model = provider_models[0].clone();
  let provider_models_clone = provider_models.clone();

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(move |_, _, _, _, _| Ok(provider_models_clone.clone()));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format,
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["model-x".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert!(!result.forward_all_with_prefix);
  assert_eq!(vec![expected_model], result.models);

  Ok(())
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
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
  let expected_models = two_models_for(&api_format);
  let expected_models_clone = expected_models.clone();

  let mut mock_ai = MockAiApiService::new();
  // create() fetches once, update() fetches once
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(move |_, _, _, _, _| Ok(expected_models_clone.clone()));

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
  let stored_ids: Vec<&str> = result.models.iter().map(|m| m.id()).collect();
  let expected_stored_ids: Vec<String> =
    expected_models.iter().map(|m| m.id().to_string()).collect();
  assert_eq!(
    expected_stored_ids,
    stored_ids.iter().map(|s| s.to_string()).collect::<Vec<_>>()
  );
  assert_eq!(extra_headers, result.extra_headers);
  assert_eq!(extra_body, result.extra_body);

  Ok(())
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
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
  let provider_models = two_named_models_for(&api_format, "model-p", "model-q");
  let provider_models_clone = provider_models.clone();

  let mut mock_ai = MockAiApiService::new();
  // create() calls fetch_models once, update() calls fetch_models once
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(move |_, _, _, _, _| Ok(provider_models_clone.clone()));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let create_form = ApiModelRequest {
    api_format: api_format.clone(),
    base_url: "https://api.example.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["model-p".to_string()],
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
    models: vec!["model-p".to_string(), "model-q".to_string()],
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
  assert_eq!(vec!["model-p", "model-q"], model_ids);

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

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_gemini_preserves_bare_name(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);
  let provider_models = vec![ApiModel::Gemini(gemini_model("gemini-2.5-flash"))];
  let provider_models_clone = provider_models.clone();

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(move |_, _, _, _, _| Ok(provider_models_clone.clone()));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format: ApiFormat::Gemini,
    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gemini-2.5-flash".to_string()],
    prefix: Some("gmn/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert_eq!(1, result.models.len());
  match &result.models[0] {
    ApiModel::Gemini(m) => {
      assert_eq!("models/gemini-2.5-flash", m.name);
      assert_eq!("gemini-2.5-flash", m.model_id());
    }
    other => panic!("expected ApiModel::Gemini, got {:?}", other),
  }

  // Verify DB round-trip keeps bare name and matchable_models returns single-prefix form.
  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?
    .expect("alias should exist");
  match &alias.models[0] {
    ApiModel::Gemini(m) => {
      assert_eq!("models/gemini-2.5-flash", m.name);
      assert_eq!("gemini-2.5-flash", m.model_id());
    }
    other => panic!(
      "expected ApiModel::Gemini after DB roundtrip, got {:?}",
      other
    ),
  }
  assert_eq!(
    vec!["gmn/gemini-2.5-flash".to_string()],
    alias.matchable_models()
  );
  assert!(alias.supports_model("gmn/gemini-2.5-flash"));
  assert!(!alias.supports_model("gmn/gmn/gemini-2.5-flash"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_gemini_preserves_bare_name(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);
  let provider_models = vec![ApiModel::Gemini(gemini_model("gemini-2.5-flash"))];
  let provider_models_clone = provider_models.clone();

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(2)
    .returning(move |_, _, _, _, _| Ok(provider_models_clone.clone()));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let create_form = ApiModelRequest {
    api_format: ApiFormat::Gemini,
    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gemini-2.5-flash".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };
  let created = service
    .create(TEST_TENANT_ID, TEST_USER_ID, create_form)
    .await?;

  let update_form = ApiModelRequest {
    api_format: ApiFormat::Gemini,
    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gemini-2.5-flash".to_string()],
    prefix: Some("gmn/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };
  let result = service
    .update(TEST_TENANT_ID, TEST_USER_ID, &created.id, update_form)
    .await?;
  assert_eq!(1, result.models.len());
  match &result.models[0] {
    ApiModel::Gemini(m) => {
      assert_eq!("models/gemini-2.5-flash", m.name);
      assert_eq!("gemini-2.5-flash", m.model_id());
    }
    other => panic!("expected ApiModel::Gemini, got {:?}", other),
  }

  let alias = db_service
    .get_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &created.id)
    .await?
    .expect("alias should exist");
  assert_eq!(
    vec!["gmn/gemini-2.5-flash".to_string()],
    alias.matchable_models()
  );
  assert!(alias.supports_model("gmn/gemini-2.5-flash"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_openai_with_prefix_no_mutation(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .times(1)
    .returning(|_, _, _, _, _| Ok(vec![openai_model("gpt-4")]));

  let time_service = Arc::new(FrozenTimeService::default());
  let service = DefaultApiModelService::new(db_service.clone(), time_service, Arc::new(mock_ai));

  let form = ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: Some("oai/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let result = service.create(TEST_TENANT_ID, TEST_USER_ID, form).await?;
  assert_eq!(1, result.models.len());
  // OpenAI id must remain bare — prefix is not baked in for non-Gemini formats.
  assert_eq!("gpt-4", result.models[0].id());

  Ok(())
}

// Extend the extra-headers forbidden-key test to cover x-goog-api-key.
#[rstest]
#[case::x_goog_api_key(json!({"x-goog-api-key": "AIza-x"}), "x-goog-api-key")]
#[case::x_goog_api_key_capitalized(json!({"X-Goog-Api-Key": "AIza-x"}), "X-Goog-Api-Key")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_rejects_extra_headers_x_goog_api_key(
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
    api_format: ApiFormat::Gemini,
    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gemini-2.5-flash".to_string()],
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
