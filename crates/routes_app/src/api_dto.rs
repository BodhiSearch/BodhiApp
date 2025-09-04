use objs::{Alias, HubFile, OAIRequestParams, UserAlias};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use services::db::{ApiToken, DownloadRequest};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

/// Query parameters for pagination and sorting
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationSortParams {
  /// Page number (1-based indexing)
  #[serde(default = "default_page")]
  #[schema(minimum = 1, example = 1)]
  pub page: usize,

  /// Number of items to return per page (maximum 100)
  #[serde(default = "default_page_size")]
  #[schema(minimum = 1, maximum = 100, example = 30)]
  pub page_size: usize,

  /// Field to sort by. Common values: repo, filename, size, updated_at, snapshot, created_at
  #[serde(default)]
  #[schema(example = "updated_at")]
  pub sort: Option<String>,

  /// Sort order: 'asc' for ascending, 'desc' for descending
  #[serde(default = "default_sort_order")]
  #[schema(pattern = "^(asc|desc)$", example = "desc")]
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
pub struct PaginatedUserAliasResponse {
  pub data: Vec<UserAliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponse {
  pub data: Vec<Alias>,
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
pub struct UserAliasResponse {
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub source: String,

  pub model_params: HashMap<String, Value>,
  pub request_params: OAIRequestParams,
  pub context_params: Vec<String>,
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
    }
  }
}
