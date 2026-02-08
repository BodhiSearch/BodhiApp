use chrono::{DateTime, Utc};
use objs::{
  Alias, ApiAlias, ApiFormat, HubFile, ModelAlias, ModelMetadata, OAIRequestParams, UserAlias,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use services::db::{ApiToken, DownloadRequest};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedDownloadResponse {
  pub data: Vec<DownloadRequest>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedApiTokenResponse {
  pub data: Vec<ApiToken>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedUserAliasResponse {
  pub data: Vec<UserAliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponse {
  pub data: Vec<AliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedLocalModelResponse {
  pub data: Vec<LocalModelResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

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

#[allow(clippy::too_many_arguments)]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, derive_new::new, ToSchema)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    setter(into),
    build_fn(error = objs::BuilderError)))]
pub struct UserAliasResponse {
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub source: String,

  pub model_params: HashMap<String, Value>,
  pub request_params: OAIRequestParams,
  pub context_params: Vec<String>,

  /// Model metadata extracted from GGUF file (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[cfg_attr(any(test, feature = "test-utils"), builder(default))]
  pub metadata: Option<ModelMetadata>,
}

impl From<UserAlias> for UserAliasResponse {
  fn from(alias: UserAlias) -> Self {
    UserAliasResponse {
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      alias: alias.alias,
      source: "user".to_string(), // UserAlias always has source "user"

      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params,
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
    let models = alias.get_models().clone();
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
