use objs::{HubFile, OAIRequestParams, UserAlias};
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

#[derive(Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
  pub data: Vec<T>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedDownloadResponse {
  pub data: Vec<DownloadRequest>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

impl From<PaginatedResponse<DownloadRequest>> for PaginatedDownloadResponse {
  fn from(paginated: PaginatedResponse<DownloadRequest>) -> Self {
    PaginatedDownloadResponse {
      data: paginated.data,
      total: paginated.total,
      page: paginated.page,
      page_size: paginated.page_size,
    }
  }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedApiTokenResponse {
  pub data: Vec<ApiToken>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

impl From<PaginatedResponse<ApiToken>> for PaginatedApiTokenResponse {
  fn from(paginated: PaginatedResponse<ApiToken>) -> Self {
    PaginatedApiTokenResponse {
      data: paginated.data,
      total: paginated.total,
      page: paginated.page,
      page_size: paginated.page_size,
    }
  }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponse {
  pub data: Vec<AliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

impl From<PaginatedResponse<AliasResponse>> for PaginatedAliasResponse {
  fn from(paginated: PaginatedResponse<AliasResponse>) -> Self {
    PaginatedAliasResponse {
      data: paginated.data,
      total: paginated.total,
      page: paginated.page,
      page_size: paginated.page_size,
    }
  }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedLocalModelResponse {
  pub data: Vec<LocalModelResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

impl From<PaginatedResponse<LocalModelResponse>> for PaginatedLocalModelResponse {
  fn from(paginated: PaginatedResponse<LocalModelResponse>) -> Self {
    PaginatedLocalModelResponse {
      data: paginated.data,
      total: paginated.total,
      page: paginated.page,
      page_size: paginated.page_size,
    }
  }
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
      source: alias.source.to_string(),

      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params,
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

impl From<PaginatedResponse<UnifiedModelResponse>> for PaginatedUnifiedModelResponse {
  fn from(paginated: PaginatedResponse<UnifiedModelResponse>) -> Self {
    PaginatedUnifiedModelResponse {
      data: paginated.data,
      total: paginated.total,
      page: paginated.page,
      page_size: paginated.page_size,
    }
  }
}
