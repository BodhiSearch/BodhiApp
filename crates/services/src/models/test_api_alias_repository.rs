use crate::{
  models::{ApiAlias, ApiAliasRepository, ApiFormat, ApiModel},
  new_ulid,
  test_utils::{gemini_model, openai_model, sea_context, setup_env},
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

// ===== API Model Alias Tests =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_api_model_alias_crud(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com".to_string(),
    models: vec![openai_model("gpt-4")].into(),
    prefix: Some("test".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias, Some("sk-test-key".to_string()))
    .await?;

  let fetched = ctx.service.get_api_model_alias("", "", &alias.id).await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(alias.id, fetched.id);
  assert_eq!(alias.base_url, fetched.base_url);

  let api_key = ctx.service.get_api_key_for_alias("", "", &alias.id).await?;
  assert_eq!(Some("sk-test-key".to_string()), api_key);

  ctx
    .service
    .delete_api_model_alias("", "", &alias.id)
    .await?;
  let deleted = ctx.service.get_api_model_alias("", "", &alias.id).await?;
  assert!(deleted.is_none());
  Ok(())
}

// ===== update_api_model_alias =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_api_model_alias(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com".to_string(),
    models: vec![openai_model("gpt-4")].into(),
    prefix: Some("orig".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias, Some("sk-old-key".to_string()))
    .await?;

  // Update model fields + replace API key
  let mut updated = alias.clone();
  updated.base_url = "https://api.updated.com".to_string();
  updated.models = vec![openai_model("gpt-4o")].into();
  updated.prefix = Some("upd".to_string());

  ctx
    .service
    .update_api_model_alias(
      "",
      "",
      &alias.id,
      &updated,
      crate::RawApiKeyUpdate::Set(Some("sk-new-key".to_string())),
    )
    .await?;

  let fetched = ctx
    .service
    .get_api_model_alias("", "", &alias.id)
    .await?
    .expect("should exist");
  assert_eq!("https://api.updated.com", fetched.base_url);
  assert_eq!(&vec![openai_model("gpt-4o")], &*fetched.models);
  assert_eq!(Some("upd".to_string()), fetched.prefix);

  let api_key = ctx.service.get_api_key_for_alias("", "", &alias.id).await?;
  assert_eq!(Some("sk-new-key".to_string()), api_key);

  // Update with Keep -> API key unchanged
  let mut updated2 = updated.clone();
  updated2.base_url = "https://api.final.com".to_string();
  ctx
    .service
    .update_api_model_alias("", "", &alias.id, &updated2, crate::RawApiKeyUpdate::Keep)
    .await?;

  let api_key = ctx.service.get_api_key_for_alias("", "", &alias.id).await?;
  assert_eq!(Some("sk-new-key".to_string()), api_key);

  // Update with Set(None) -> API key removed
  ctx
    .service
    .update_api_model_alias(
      "",
      "",
      &alias.id,
      &updated2,
      crate::RawApiKeyUpdate::Set(None),
    )
    .await?;

  let api_key = ctx.service.get_api_key_for_alias("", "", &alias.id).await?;
  assert_eq!(None, api_key);

  Ok(())
}

// ===== update_api_model_alias prefix conflict =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_api_model_alias_prefix_conflict(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  use errmeta::AppError;

  let ctx = sea_context(db_type).await;
  let alias1 = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api1.example.com".to_string(),
    models: vec![].into(),
    prefix: Some("pfx-one".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };
  let alias2 = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api2.example.com".to_string(),
    models: vec![].into(),
    prefix: Some("pfx-two".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias1, None)
    .await?;
  ctx
    .service
    .create_api_model_alias("", "", &alias2, None)
    .await?;

  // Try to update alias2's prefix to alias1's prefix -> conflict
  let mut conflict = alias2.clone();
  conflict.prefix = Some("pfx-one".to_string());
  let result = ctx
    .service
    .update_api_model_alias("", "", &alias2.id, &conflict, crate::RawApiKeyUpdate::Keep)
    .await;

  let err = result.unwrap_err();
  assert_eq!("db_error-prefix_exists", err.code());

  Ok(())
}

// ===== update_api_model_models =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_api_model_models(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let alias = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com".to_string(),
    models: vec![openai_model("gpt-4")].into(),
    prefix: Some("fwd/".to_string()),
    forward_all_with_prefix: true,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias, None)
    .await?;

  let new_models = vec![openai_model("gpt-4"), openai_model("gpt-4o")];
  ctx
    .service
    .update_api_model_models("", &alias.id, new_models.clone())
    .await?;

  let fetched = ctx
    .service
    .get_api_model_alias("", "", &alias.id)
    .await?
    .expect("should exist");
  assert_eq!(&new_models, &*fetched.models);

  Ok(())
}

// ===== extra_headers / extra_body round-trip =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_api_model_alias_extra_fields_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  use serde_json::json;

  let ctx = sea_context(db_type).await;

  // Round-trip with non-null extra fields
  let alias_with_extras = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::AnthropicOAuth,
    base_url: "https://api.anthropic.com/v1".to_string(),
    models: vec![].into(),
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: Some(json!({"x-custom": "v1"})),
    extra_body: Some(json!({"max_tokens": 32000})),
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias_with_extras, None)
    .await?;

  let fetched = ctx
    .service
    .get_api_model_alias("", "", &alias_with_extras.id)
    .await?
    .expect("should exist");
  assert_eq!(alias_with_extras.extra_headers, fetched.extra_headers);
  assert_eq!(alias_with_extras.extra_body, fetched.extra_body);
  assert_eq!(ApiFormat::AnthropicOAuth, fetched.api_format);

  // Round-trip with null extra fields
  let alias_null_extras = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::AnthropicOAuth,
    base_url: "https://api.anthropic.com/v1".to_string(),
    models: vec![].into(),
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias_null_extras, None)
    .await?;

  let fetched_null = ctx
    .service
    .get_api_model_alias("", "", &alias_null_extras.id)
    .await?
    .expect("should exist");
  assert_eq!(None, fetched_null.extra_headers);
  assert_eq!(None, fetched_null.extra_body);

  Ok(())
}

// ===== ApiModel::Gemini round-trip =====

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_api_model_alias_gemini_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let gemini = gemini_model("gemini-2.5-flash");
  let alias = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::Gemini,
    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
    models: vec![ApiModel::Gemini(gemini.clone())].into(),
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias, Some("AIza-test-key".to_string()))
    .await?;

  let fetched = ctx
    .service
    .get_api_model_alias("", "", &alias.id)
    .await?
    .expect("should exist");

  assert_eq!(ApiFormat::Gemini, fetched.api_format);
  assert_eq!(alias.base_url, fetched.base_url);
  assert_eq!(None, fetched.extra_headers);
  assert_eq!(None, fetched.extra_body);

  // Verify ApiModel::Gemini deserialized correctly from DB
  assert_eq!(1, fetched.models.len());
  match &fetched.models[0] {
    ApiModel::Gemini(m) => {
      assert_eq!("models/gemini-2.5-flash", m.name);
      assert_eq!("gemini-2.5-flash", m.model_id());
      assert_eq!(gemini.display_name, m.display_name);
      assert_eq!(gemini.input_token_limit, m.input_token_limit);
      assert_eq!(gemini.output_token_limit, m.output_token_limit);
    }
    other => panic!("expected ApiModel::Gemini, got {:?}", other),
  }

  let api_key = ctx.service.get_api_key_for_alias("", "", &alias.id).await?;
  assert_eq!(Some("AIza-test-key".to_string()), api_key);

  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_api_model_alias_gemini_prefixed_roundtrip(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let gemini = gemini_model("gemini-2.5-flash");

  let alias = ApiAlias {
    id: new_ulid(),
    api_format: ApiFormat::Gemini,
    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
    models: vec![ApiModel::Gemini(gemini.clone())].into(),
    prefix: Some("gmn/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias("", "", &alias, Some("AIza-test-key".to_string()))
    .await?;

  let fetched = ctx
    .service
    .get_api_model_alias("", "", &alias.id)
    .await?
    .expect("should exist");

  assert_eq!(1, fetched.models.len());
  match &fetched.models[0] {
    ApiModel::Gemini(m) => {
      assert_eq!("models/gemini-2.5-flash", m.name);
      assert_eq!("gemini-2.5-flash", m.model_id());
    }
    other => panic!("expected ApiModel::Gemini, got {:?}", other),
  }
  assert_eq!(
    vec!["gmn/gemini-2.5-flash".to_string()],
    fetched.matchable_models()
  );
  assert!(fetched.supports_model("gmn/gemini-2.5-flash"));

  Ok(())
}
