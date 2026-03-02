use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use services::{
  Alias, ApiAlias, ApiFormat, DownloadRequest, HubFile, ModelAlias, ModelMetadata,
  OAIRequestParams, UserAlias,
};
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAliasRequest {
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: Option<String>,

  pub request_params: Option<services::OAIRequestParams>,
  pub context_params: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateAliasRequest {
  pub repo: String,
  pub filename: String,
  pub snapshot: Option<String>,

  pub request_params: Option<services::OAIRequestParams>,
  pub context_params: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CopyAliasRequest {
  pub alias: String,
}

/// Request to pull a model file from HuggingFace
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
    "filename": "mistral-7b-instruct-v0.1.Q4_K_M.gguf"
}))]
pub struct NewDownloadRequest {
  /// HuggingFace repository name in format 'username/repository-name'
  #[schema(
    pattern = "^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$",
    example = "TheBloke/Mistral-7B-Instruct-v0.1-GGUF"
  )]
  pub repo: String,
  /// Model file name to download (typically .gguf format)
  #[schema(
    pattern = ".*\\.(gguf|bin|safetensors)$",
    example = "mistral-7b-instruct-v0.1.Q4_K_M.gguf"
  )]
  pub filename: String,
}

/// Source type discriminator for refresh requests
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RefreshSource {
  /// Refresh all local GGUF models (async)
  All,
  /// Refresh specific GGUF model (sync)
  Model,
  // Future: Api for API model cache refresh
}

/// Refresh request - discriminated union by source field
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum RefreshRequest {
  /// Bulk async refresh for all models - Request: {"source": "all"}
  All {},
  /// Single sync refresh for specific model - Request: {"source": "model", "repo": "...", "filename": "...", "snapshot": "..."}
  Model {
    /// Repository in format "user/repo"
    #[schema(example = "bartowski/Qwen2.5-3B-Instruct-GGUF")]
    repo: String,
    /// Filename of the GGUF model
    #[schema(example = "Qwen2.5-3B-Instruct-Q4_K_M.gguf")]
    filename: String,
    /// Snapshot/commit identifier
    #[schema(example = "8ba1c3c3ee94ba4b86ff92a749ae687dc41fce3f")]
    snapshot: String,
  },
}

/// Response for metadata refresh operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshResponse {
  /// Number of models queued ("all" for bulk refresh, "1" for single)
  pub num_queued: String,
  /// Model alias (only for single model refresh)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub alias: Option<String>,
}

/// Response for queue status operations
#[derive(Debug, Serialize, ToSchema)]
pub struct QueueStatusResponse {
  /// Queue status ("idle" or "processing")
  pub status: String,
}

/// Enum for different refresh response types
#[allow(clippy::large_enum_variant)]
pub enum RefreshResponseType {
  Sync(ModelAliasResponse),
  Async(RefreshResponse),
}

impl axum::response::IntoResponse for RefreshResponseType {
  fn into_response(self) -> axum::response::Response {
    match self {
      RefreshResponseType::Sync(response) => (StatusCode::OK, Json(response)).into_response(),
      RefreshResponseType::Async(response) => {
        (StatusCode::ACCEPTED, Json(response)).into_response()
      }
    }
  }
}

/// Paginated list of download requests
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedDownloadResponse {
  pub data: Vec<DownloadRequest>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// Paginated list of local model files
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedLocalModelResponse {
  pub data: Vec<LocalModelResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// Local model file response
#[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct LocalModelResponse {
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub size: Option<u64>,
  pub model_params: HashMap<String, Value>,
  /// Model metadata extracted from GGUF file (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<ModelMetadata>,
}

impl From<HubFile> for LocalModelResponse {
  fn from(model: HubFile) -> Self {
    LocalModelResponse {
      repo: model.repo.to_string(),
      filename: model.filename,
      snapshot: model.snapshot,
      size: model.size,
      model_params: HashMap::new(),
      metadata: None,
    }
  }
}

impl LocalModelResponse {
  /// Attach model metadata to this response
  pub fn with_metadata(mut self, metadata: Option<ModelMetadata>) -> Self {
    self.metadata = metadata;
    self
  }
}

/// Paginated list of user-defined model aliases
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedUserAliasResponse {
  pub data: Vec<UserAliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// Paginated list of all model aliases (user, model, and API)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponse {
  pub data: Vec<AliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// User-defined model alias response
#[allow(clippy::too_many_arguments)]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, derive_new::new, ToSchema)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    setter(into),
    build_fn(error = services::BuilderError)))]
pub struct UserAliasResponse {
  pub id: String,
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub source: String,

  pub model_params: HashMap<String, Value>,
  pub request_params: OAIRequestParams,
  pub context_params: Vec<String>,

  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,

  /// Model metadata extracted from GGUF file (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[cfg_attr(any(test, feature = "test-utils"), builder(default))]
  pub metadata: Option<ModelMetadata>,
}

impl From<UserAlias> for UserAliasResponse {
  fn from(alias: UserAlias) -> Self {
    UserAliasResponse {
      id: alias.id,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      alias: alias.alias,
      source: "user".to_string(), // UserAlias always has source "user"

      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params.into(),
      created_at: alias.created_at,
      updated_at: alias.updated_at,
      metadata: None,
    }
  }
}

impl UserAliasResponse {
  /// Attach model metadata to this response
  pub fn with_metadata(mut self, metadata: Option<ModelMetadata>) -> Self {
    self.metadata = metadata;
    self
  }
}

/// Response for auto-discovered model aliases
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct ModelAliasResponse {
  pub source: String,
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,

  /// Model metadata extracted from GGUF file (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<ModelMetadata>,
}

impl From<ModelAlias> for ModelAliasResponse {
  fn from(alias: ModelAlias) -> Self {
    Self {
      source: "model".to_string(),
      alias: alias.alias,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      metadata: None,
    }
  }
}

impl ModelAliasResponse {
  /// Attach model metadata to this response
  pub fn with_metadata(mut self, metadata: Option<ModelMetadata>) -> Self {
    self.metadata = metadata;
    self
  }
}

/// API response for API model aliases - hides internal cache fields
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct ApiAliasResponse {
  pub source: String,
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  /// Models available through this alias (merged from cache for forward_all)
  pub models: Vec<String>,
  pub prefix: Option<String>,
  pub forward_all_with_prefix: bool,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<ApiAlias> for ApiAliasResponse {
  fn from(alias: ApiAlias) -> Self {
    let models = alias.get_models().to_vec();
    Self {
      source: "api".to_string(),
      id: alias.id,
      api_format: alias.api_format,
      base_url: alias.base_url,
      models,
      prefix: alias.prefix,
      forward_all_with_prefix: alias.forward_all_with_prefix,
      created_at: alias.created_at,
      updated_at: alias.updated_at,
    }
  }
}

/// Response envelope for model aliases - hides internal implementation details
/// Uses untagged serialization - each variant has its own "source" field
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum AliasResponse {
  /// User-defined local model (source: "user")
  User(UserAliasResponse),
  /// Auto-discovered local model (source: "model")
  Model(ModelAliasResponse),
  /// Remote API model (source: "api")
  Api(ApiAliasResponse),
}

impl From<Alias> for AliasResponse {
  fn from(alias: Alias) -> Self {
    match alias {
      Alias::User(u) => AliasResponse::User(u.into()),
      Alias::Model(m) => AliasResponse::Model(m.into()),
      Alias::Api(a) => AliasResponse::Api(a.into()),
    }
  }
}

impl AliasResponse {
  /// Attach model metadata to this response (only applies to User and Model variants)
  pub fn with_metadata(self, metadata: Option<ModelMetadata>) -> Self {
    match self {
      AliasResponse::User(r) => AliasResponse::User(r.with_metadata(metadata)),
      AliasResponse::Model(r) => AliasResponse::Model(r.with_metadata(metadata)),
      AliasResponse::Api(r) => AliasResponse::Api(r), // API aliases don't have metadata
    }
  }
}
