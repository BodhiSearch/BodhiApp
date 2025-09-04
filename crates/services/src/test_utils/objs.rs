use crate::AppRegInfo;
use chrono::{DateTime, Utc};
use objs::{AliasSource, ApiAlias};
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
  ApiAlias::new(
    alias,
    AliasSource::RemoteApi,
    "openai",
    "https://api.openai.com/v1",
    models,
    created_at,
  )
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
  ];

  for alias in &aliases {
    db_service
      .create_api_model_alias(alias, "sk-test-key-123456789")
      .await?;
  }

  Ok(aliases)
}
