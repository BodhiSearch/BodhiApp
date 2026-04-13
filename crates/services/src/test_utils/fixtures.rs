use super::ModelMetadataEntityBuilder;
use crate::models::{
  AliasSource, ApiAlias, ApiAliasBuilder, ApiModel, ContextLimits, ModelCapabilities,
  ToolCapabilities, UserAlias,
};
use async_openai::types::models::Model as OpenAIModel;

pub fn openai_model(id: &str) -> ApiModel {
  ApiModel::OpenAI(OpenAIModel {
    id: id.to_string(),
    object: "model".to_string(),
    created: 0,
    owned_by: "openai".to_string(),
  })
}

pub fn gemini_model(id: impl Into<String>) -> crate::models::GeminiModel {
  let id = id.into();
  crate::models::GeminiModel {
    name: format!("models/{}", id),
    version: Some("001".to_string()),
    display_name: None,
    description: None,
    input_token_limit: None,
    output_token_limit: None,
    supported_generation_methods: vec![],
    temperature: None,
    max_temperature: None,
    top_p: None,
    top_k: None,
    thinking: None,
  }
}

pub fn anthropic_model(id: &str) -> ApiModel {
  use crate::models::AnthropicModel;
  ApiModel::Anthropic(AnthropicModel {
    id: id.to_string(),
    display_name: id.to_string(),
    created_at: "2024-01-01T00:00:00Z".to_string(),
    capabilities: None,
    max_input_tokens: None,
    max_tokens: None,
    model_type: "model".to_string(),
  })
}

use crate::{AppStatus, Tenant};
use chrono::{DateTime, Utc};
use rstest::fixture;

use super::{TEST_CLIENT_ID, TEST_CLIENT_SECRET, TEST_TENANT_ID, TEST_USER_ID};

/// Fixed deterministic timestamp matching `FrozenTimeService` default (2025-01-01T00:00:00Z).
pub fn fixed_dt() -> DateTime<Utc> {
  chrono::TimeZone::with_ymd_and_hms(&Utc, 2025, 1, 1, 0, 0, 0).unwrap()
}

impl Tenant {
  pub fn test_default() -> Self {
    let now = fixed_dt();
    Self {
      id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
      client_id: TEST_CLIENT_ID.to_string(),
      client_secret: TEST_CLIENT_SECRET.to_string(),
      name: "Test App".to_string(),
      description: None,
      status: AppStatus::Ready,
      created_by: Some(TEST_USER_ID.to_string()),
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

  pub fn test_tenant_b() -> Self {
    let now = fixed_dt();
    Self {
      id: super::TEST_TENANT_B_ID.to_string(),
      client_id: "test-client-b".to_string(),
      client_secret: "test-client-secret-b".to_string(),
      name: "Test App B".to_string(),
      description: None,
      status: AppStatus::Ready,
      created_by: Some(TEST_USER_ID.to_string()),
      created_at: now,
      updated_at: now,
    }
  }
}

#[fixture]
pub fn tenant() -> Tenant {
  Tenant::test_default()
}

/// Create a test ApiModelAlias with incrementing timestamps for sorting tests
pub fn create_test_api_model_alias(
  alias: &str,
  models: Vec<ApiModel>,
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
  models: Vec<ApiModel>,
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
  seed_test_api_models_for_user(db_service, base_time, TEST_USER_ID).await
}

pub async fn seed_test_api_models_for_user(
  db_service: &dyn crate::db::DbService,
  base_time: DateTime<Utc>,
  user_id: &str,
) -> anyhow::Result<Vec<ApiAlias>> {
  let aliases = vec![
    create_test_api_model_alias("openai-gpt4", vec![openai_model("gpt-4")], base_time),
    create_test_api_model_alias(
      "openai-gpt35-turbo",
      vec![openai_model("gpt-3.5-turbo")],
      base_time - chrono::Duration::seconds(10),
    ),
    create_test_api_model_alias(
      "openai-gpt4-turbo",
      vec![openai_model("gpt-4-turbo")],
      base_time - chrono::Duration::seconds(20),
    ),
    create_test_api_model_alias(
      "openai-gpt4-vision",
      vec![openai_model("gpt-4-vision-preview")],
      base_time - chrono::Duration::seconds(30),
    ),
    create_test_api_model_alias(
      "openai-multi-model",
      vec![openai_model("gpt-4"), openai_model("gpt-3.5-turbo")],
      base_time - chrono::Duration::seconds(40),
    ),
    // Add prefix test data with separators
    create_test_api_model_alias_with_prefix(
      "azure-openai",
      vec![openai_model("gpt-4"), openai_model("gpt-3.5-turbo")],
      Some("azure/".to_string()),
      base_time - chrono::Duration::seconds(50),
    ),
    create_test_api_model_alias_with_prefix(
      "custom-alias",
      vec![openai_model("custom-model-1")],
      Some("my.custom_".to_string()),
      base_time - chrono::Duration::seconds(60),
    ),
  ];

  for alias in &aliases {
    db_service
      .create_api_model_alias(
        TEST_TENANT_ID,
        user_id,
        alias,
        Some("sk-test-key-123456789".to_string()),
      )
      .await?;
  }

  Ok(aliases)
}

// =============================================================================
// ModelMetadataEntity Test Factories
// =============================================================================

/// Creates a ModelMetadataEntityBuilder pre-configured with required timestamps.
/// Use this as a starting point and chain additional builder methods.
pub fn model_metadata_builder(now: DateTime<Utc>) -> ModelMetadataEntityBuilder {
  let mut builder = ModelMetadataEntityBuilder::default();
  builder.extracted_at(now).created_at(now).updated_at(now);
  builder
}

/// Creates a test ModelMetadataEntity for a local GGUF model file.
/// source is always 'model' since UserAlias and ModelAlias both reference the same physical file.
pub fn create_test_model_metadata(
  repo: &str,
  filename: &str,
  snapshot: &str,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataEntity {
  let mut builder = model_metadata_builder(now);
  builder
    .source(AliasSource::Model)
    .repo(repo)
    .filename(filename)
    .snapshot(snapshot);
  builder
    .build()
    .expect("Failed to build test ModelMetadataEntity")
}

/// Creates a test ModelMetadataEntity for a local GGUF model with capabilities.
pub fn create_test_model_metadata_with_capabilities(
  repo: &str,
  filename: &str,
  snapshot: &str,
  vision: bool,
  thinking: bool,
  function_calling: bool,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataEntity {
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
    .expect("Failed to build test ModelMetadataEntity with capabilities")
}

/// Creates a test ModelMetadataEntity for an API model (e.g., OpenAI GPT-4).
pub fn create_test_api_model_metadata(
  api_model_id: &str,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataEntity {
  let mut builder = model_metadata_builder(now);
  builder.source(AliasSource::Api).api_model_id(api_model_id);
  builder
    .build()
    .expect("Failed to build test API ModelMetadataEntity")
}

/// Creates a test ModelMetadataEntity for an API model with context limits.
pub fn create_test_api_model_metadata_with_context(
  api_model_id: &str,
  max_input_tokens: u64,
  max_output_tokens: u64,
  now: DateTime<Utc>,
) -> crate::models::ModelMetadataEntity {
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
    .expect("Failed to build test API ModelMetadataEntity with context")
}

/// Seed database with test user aliases (replaces YAML fixture files)
pub async fn seed_test_user_aliases(
  db_service: &dyn crate::db::DbService,
) -> anyhow::Result<Vec<UserAlias>> {
  use crate::test_utils::db::{TEST_TENANT_ID, TEST_USER_ID};
  let now = db_service.now();
  let aliases = vec![
    UserAlias::llama3_with_time(now),
    UserAlias::testalias_exists_with_time(now),
    UserAlias::tinyllama_with_time(now),
  ];

  for alias in &aliases {
    db_service
      .create_user_alias(TEST_TENANT_ID, TEST_USER_ID, alias)
      .await?;
  }

  Ok(aliases)
}
