use crate::AppRegInfo;
use chrono::{DateTime, Utc};
use objs::{ApiAlias, ApiAliasBuilder};
use rstest::fixture;

#[fixture]
pub fn app_reg_info() -> AppRegInfo {
  AppRegInfo {
    client_id: "test-client".to_string(),
    client_secret: "test-secret".to_string(),
  }
}

/// Create a test ApiModelAlias with incrementing timestamps for sorting tests
pub fn create_test_api_model_alias(
  alias: &str,
  models: Vec<String>,
  created_at: DateTime<Utc>,
) -> ApiAlias {
  ApiAliasBuilder::test_default()
    .id(alias)
    .base_url("https://api.openai.com/v1")
    .models(models)
    .build_with_time(created_at)
    .expect("Failed to build test ApiAlias")
}

/// Create a test ApiModelAlias with prefix for prefix-based routing tests
pub fn create_test_api_model_alias_with_prefix(
  alias: &str,
  models: Vec<String>,
  prefix: Option<String>,
  created_at: DateTime<Utc>,
) -> ApiAlias {
  let mut builder = ApiAliasBuilder::test_default();
  builder
    .id(alias)
    .base_url("https://api.openai.com/v1")
    .models(models);
  if let Some(p) = prefix {
    builder.prefix(p);
  }
  builder
    .build_with_time(created_at)
    .expect("Failed to build test ApiAlias with prefix")
}

/// Seed database with test API model aliases
pub async fn seed_test_api_models(
  db_service: &dyn crate::db::DbService,
  base_time: DateTime<Utc>,
) -> anyhow::Result<Vec<ApiAlias>> {
  let aliases = vec![
    create_test_api_model_alias("openai-gpt4", vec!["gpt-4".to_string()], base_time),
    create_test_api_model_alias(
      "openai-gpt35-turbo",
      vec!["gpt-3.5-turbo".to_string()],
      base_time - chrono::Duration::seconds(10),
    ),
    create_test_api_model_alias(
      "openai-gpt4-turbo",
      vec!["gpt-4-turbo".to_string()],
      base_time - chrono::Duration::seconds(20),
    ),
    create_test_api_model_alias(
      "openai-gpt4-vision",
      vec!["gpt-4-vision-preview".to_string()],
      base_time - chrono::Duration::seconds(30),
    ),
    create_test_api_model_alias(
      "openai-multi-model",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      base_time - chrono::Duration::seconds(40),
    ),
    // Add prefix test data with separators
    create_test_api_model_alias_with_prefix(
      "azure-openai",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      Some("azure/".to_string()),
      base_time - chrono::Duration::seconds(50),
    ),
    create_test_api_model_alias_with_prefix(
      "custom-alias",
      vec!["custom-model-1".to_string()],
      Some("my.custom_".to_string()),
      base_time - chrono::Duration::seconds(60),
    ),
  ];

  for alias in &aliases {
    db_service
      .create_api_model_alias(alias, Some("sk-test-key-123456789".to_string()))
      .await?;
  }

  Ok(aliases)
}
