use std::collections::HashMap;

use objs::HubFile;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
pub struct PaginatedResponse<T> {
  pub data: Vec<T>,
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
