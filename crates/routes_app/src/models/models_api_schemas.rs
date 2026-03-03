// NOTE: Most response types have been moved to services::models::model_objs.
// This file retains only presentation types specific to routes_app
// (LocalModelResponse, QueueStatusResponse, RefreshResponseType).

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use services::{HubFile, ModelAliasResponse, ModelMetadata, RefreshResponse};
use std::collections::HashMap;
use utoipa::ToSchema;

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
