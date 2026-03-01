use crate::{
  models::{ApiAlias, ApiAliasRepository, ApiFormat},
  test_utils::{sea_context, setup_env},
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
    id: ulid::Ulid::new().to_string(),
    api_format: ApiFormat::OpenAI,
    base_url: "https://api.example.com".to_string(),
    models: vec!["gpt-4".to_string()].into(),
    prefix: Some("test".to_string()),
    forward_all_with_prefix: false,
    models_cache: vec![].into(),
    cache_fetched_at: ctx.now,
    created_at: ctx.now,
    updated_at: ctx.now,
  };

  ctx
    .service
    .create_api_model_alias(&alias, Some("sk-test-key".to_string()))
    .await?;

  let fetched = ctx.service.get_api_model_alias(&alias.id).await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!(alias.id, fetched.id);
  assert_eq!(alias.base_url, fetched.base_url);

  let api_key = ctx.service.get_api_key_for_alias(&alias.id).await?;
  assert_eq!(Some("sk-test-key".to_string()), api_key);

  ctx.service.delete_api_model_alias(&alias.id).await?;
  let deleted = ctx.service.get_api_model_alias(&alias.id).await?;
  assert!(deleted.is_none());
  Ok(())
}
