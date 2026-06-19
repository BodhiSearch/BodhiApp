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

/// Facet filter query parameters for the All-Models list (`GET /bodhi/v1/models`).
///
/// All facets are server-side and applied before pagination so `total` and the page
/// reflect the filtered set. Multi-value facets accept a comma-separated list; an empty
/// or absent value means "no filter for this facet" (all rows pass).
#[derive(Debug, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct AliasFilterParams {
  /// Alias type facet (comma-separated): `local_file`, `model_alias`, `api_model`, `fallback`.
  #[serde(default)]
  #[schema(example = "local_file,model_alias")]
  pub r#type: Option<String>,

  /// API-format facet (comma-separated), API rows only: `openai`, `responses`, `anthropic`,
  /// `gemini`, `liberty`. `anthropic` matches both anthropic and anthropic_oauth aliases.
  #[serde(default)]
  #[schema(example = "openai,anthropic")]
  pub api_format: Option<String>,

  /// Minimum local-file size in bytes (inclusive). Applies to local rows with a known size;
  /// rows without a size (API/router) are not filtered out by size.
  #[serde(default)]
  pub size_min: Option<u64>,

  /// Maximum local-file size in bytes (inclusive). See `size_min`.
  #[serde(default)]
  pub size_max: Option<u64>,

  /// Capability facet (comma-separated), local rows only: `vision`, `tool_use`, `reasoning`.
  /// A row passes only if it has metadata with every requested capability set true.
  #[serde(default)]
  #[schema(example = "vision,tool_use")]
  pub capability: Option<String>,
}

impl AliasFilterParams {
  /// Parse a comma-separated facet value into lowercased, trimmed, non-empty tokens.
  fn tokens(value: &Option<String>) -> Vec<String> {
    value
      .as_deref()
      .map(|raw| {
        raw
          .split(',')
          .map(|t| t.trim().to_lowercase())
          .filter(|t| !t.is_empty())
          .collect()
      })
      .unwrap_or_default()
  }

  /// Requested alias-type tokens (e.g. `local_file`, `api_model`).
  pub fn type_tokens(&self) -> Vec<String> {
    Self::tokens(&self.r#type)
  }

  /// Requested api-format facet tokens (e.g. `openai`, `anthropic`).
  pub fn api_format_tokens(&self) -> Vec<String> {
    Self::tokens(&self.api_format)
  }

  /// Requested capability tokens (e.g. `vision`, `tool_use`).
  pub fn capability_tokens(&self) -> Vec<String> {
    Self::tokens(&self.capability)
  }

  /// True when no facet is active (the common, unfiltered path).
  pub fn is_empty(&self) -> bool {
    self.type_tokens().is_empty()
      && self.api_format_tokens().is_empty()
      && self.capability_tokens().is_empty()
      && self.size_min.is_none()
      && self.size_max.is_none()
  }

  /// True when a capability facet is requested (forces a whole-list metadata fetch).
  pub fn has_capability_filter(&self) -> bool {
    !self.capability_tokens().is_empty()
  }
}
