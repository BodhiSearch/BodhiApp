use serde::Deserialize;
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
