use super::ModelMetadataRowBuilder;
use crate::models::{
  AliasSource, ApiAlias, ApiAliasBuilder, ContextLimits, ModelCapabilities, ToolCapabilities,
  UserAlias,
};
use crate::{AppInstance, AppStatus};
use chrono::{DateTime, Utc};
use rstest::fixture;

use super::{TEST_CLIENT_ID, TEST_CLIENT_SECRET};

/// Fixed deterministic timestamp matching `FrozenTimeService` default (2025-01-01T00:00:00Z).
pub fn fixed_dt() -> DateTime<Utc> {
  chrono::TimeZone::with_ymd_and_hms(&Utc, 2025, 1, 1, 0, 0, 0).unwrap()
}

impl AppInstance {
  pub fn test_default() -> Self {
    let now = fixed_dt();
    Self {
      client_id: TEST_CLIENT_ID.to_string(),
      client_secret: TEST_CLIENT_SECRET.to_string(),
      status: AppStatus::Ready,
      created_at: now,
      updated_at: now,
    }
  }

  pub fn test_with_status(status: AppStatus) -> Self {
    Self {
      status,
      ..Self::test_default()
    }
  }
}

#[fixture]
pub fn app_instance() -> AppInstance {
  AppInstance::test_default()
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

// =============================================================================
// ModelMetadataRow Test Factories
// =============================================================================

/// Creates a ModelMetadataRowBuilder pre-configured with required timestamps.
/// Use this as a starting point and chain additional builder methods.
pub fn model_metadata_builder(now: DateTime<Utc>) -> ModelMetadataRowBuilder {
  let mut builder = ModelMetadataRowBuilder::default();
  builder.extracted_at(now).created_at(now).updated_at(now);
  builder
}

/// Creates a test ModelMetadataRow for a local GGUF model file.
/// source is always 'model' since UserAlias and ModelAlias both reference the same physical file.
pub fn create_test_model_metadata(
  repo: &str,
  filename: &str,
  snapshot: &str,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataRow {
  let mut builder = model_metadata_builder(now);
  builder
    .source(AliasSource::Model)
    .repo(repo)
    .filename(filename)
    .snapshot(snapshot);
  builder
    .build()
    .expect("Failed to build test ModelMetadataRow")
}

/// Creates a test ModelMetadataRow for a local GGUF model with capabilities.
pub fn create_test_model_metadata_with_capabilities(
  repo: &str,
  filename: &str,
  snapshot: &str,
  vision: bool,
  thinking: bool,
  function_calling: bool,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataRow {
  let mut builder = model_metadata_builder(now);
  builder
    .source(AliasSource::Model)
    .repo(repo)
    .filename(filename)
    .snapshot(snapshot)
    .capabilities(ModelCapabilities {
      vision: Some(vision),
      audio: None,
      thinking: Some(thinking),
      tools: ToolCapabilities {
        function_calling: Some(function_calling),
        structured_output: None,
      },
    });
  builder
    .build()
    .expect("Failed to build test ModelMetadataRow with capabilities")
}

/// Creates a test ModelMetadataRow for an API model (e.g., OpenAI GPT-4).
pub fn create_test_api_model_metadata(
  api_model_id: &str,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataRow {
  let mut builder = model_metadata_builder(now);
  builder.source(AliasSource::Api).api_model_id(api_model_id);
  builder
    .build()
    .expect("Failed to build test API ModelMetadataRow")
}

/// Creates a test ModelMetadataRow for an API model with context limits.
pub fn create_test_api_model_metadata_with_context(
  api_model_id: &str,
  max_input_tokens: u64,
  max_output_tokens: u64,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataRow {
  let mut builder = model_metadata_builder(now);
  builder
    .source(AliasSource::Api)
    .api_model_id(api_model_id)
    .context(ContextLimits {
      max_input_tokens: Some(max_input_tokens),
      max_output_tokens: Some(max_output_tokens),
    });
  builder
    .build()
    .expect("Failed to build test API ModelMetadataRow with context")
}

/// Seed database with test user aliases (replaces YAML fixture files)
pub async fn seed_test_user_aliases(
  db_service: &dyn crate::db::DbService,
) -> anyhow::Result<Vec<UserAlias>> {
  let now = db_service.now();
  let aliases = vec![
    UserAlias::llama3_with_time(now),
    UserAlias::testalias_exists_with_time(now),
    UserAlias::tinyllama_with_time(now),
  ];

  for alias in &aliases {
    db_service.create_user_alias(alias).await?;
  }

  Ok(aliases)
}
