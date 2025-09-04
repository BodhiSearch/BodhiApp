use objs::{Alias, HubFile, OAIRequestParams, UserAlias};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use services::db::{ApiToken, DownloadRequest};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

/// Query parameters for pagination and sorting
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationSortParams {
  /// Page number (1-based)
  #[serde(default = "default_page")]
  pub page: usize,

  /// Number of items per page (max 100)
  #[serde(default = "default_page_size")]
  pub page_size: usize,

  /// Field to sort by (repo, filename, size, updated_at, snapshot)
  #[serde(default)]
  pub sort: Option<String>,

  /// Sort order (asc or desc)
  #[serde(default = "default_sort_order")]
  pub sort_order: String,
}

fn default_page() -> usize {
  1
}

fn default_page_size() -> usize {
  30
}

fn default_sort_order() -> String {
  "asc".to_string()
}

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
}

impl From<HubFile> for LocalModelResponse {
  fn from(model: HubFile) -> Self {
    LocalModelResponse {
      repo: model.repo.to_string(),
      filename: model.filename,
      snapshot: model.snapshot,
      size: model.size,
      model_params: HashMap::new(),
    }
  }
}

#[allow(clippy::too_many_arguments)]
#[derive(Serialize, Deserialize, Debug, PartialEq, derive_new::new, ToSchema)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    setter(into),
    build_fn(error = objs::BuilderError)))]
pub struct AliasResponse {
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub source: String,

  pub model_params: HashMap<String, Value>,
  pub request_params: OAIRequestParams,
  pub context_params: Vec<String>,
}

impl From<UserAlias> for AliasResponse {
  fn from(alias: UserAlias) -> Self {
    AliasResponse {
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      alias: alias.alias,
      source: "user".to_string(), // UserAlias always has source "user"

      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params,
    }
  }
}

impl From<Alias> for AliasResponse {
  fn from(alias: Alias) -> Self {
    match alias {
      Alias::User(user_alias) => AliasResponse::from(user_alias),
      Alias::Model(model_alias) => AliasResponse {
        repo: model_alias.repo.to_string(),
        filename: model_alias.filename,
        snapshot: model_alias.snapshot,
        alias: model_alias.alias,
        source: "model".to_string(), // ModelAlias has source "model"

        model_params: HashMap::new(),
        request_params: Default::default(), // ModelAlias doesn't have request params
        context_params: Vec::new(),         // ModelAlias doesn't have context params
      },
      Alias::Api(api_alias) => AliasResponse {
        repo: "".to_string(),     // API aliases don't have repos
        filename: "".to_string(), // API aliases don't have filenames
        snapshot: "".to_string(), // API aliases don't have snapshots
        alias: api_alias.id,
        source: "api".to_string(), // ApiAlias has source "api"

        model_params: HashMap::new(),
        request_params: Default::default(), // API aliases don't have request params
        context_params: Vec::new(),         // API aliases don't have context params
      },
    }
  }
}

/// Unified model response that can represent both local aliases and API models
#[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
#[serde(tag = "model_type")]
pub enum UnifiedModelResponse {
  #[serde(rename = "local")]
  Local(AliasResponse),
  #[serde(rename = "api")]
  Api(crate::api_models_dto::ApiModelResponse),
}

impl From<AliasResponse> for UnifiedModelResponse {
  fn from(alias: AliasResponse) -> Self {
    UnifiedModelResponse::Local(alias)
  }
}

impl From<crate::api_models_dto::ApiModelResponse> for UnifiedModelResponse {
  fn from(api_model: crate::api_models_dto::ApiModelResponse) -> Self {
    UnifiedModelResponse::Api(api_model)
  }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedUnifiedModelResponse {
  pub data: Vec<UnifiedModelResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}
