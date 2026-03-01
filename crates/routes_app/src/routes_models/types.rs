use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
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
pub enum RefreshResponseType {
  Sync(crate::ModelAliasResponse),
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
