use crate::{
  models::{ApiAlias, ApiAliasRepository, ApiFormat, ApiModel},
  new_ulid,
  test_utils::{openai_model, sea_context, setup_env},
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
