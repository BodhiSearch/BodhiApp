use axum::{http::StatusCode, Json};
use commands::CreateCommandError;
use objs::{AppError, ErrorType, ObjValidationError};
use serde::{Deserialize, Serialize};
use services::AliasNotFoundError;
use utoipa::ToSchema;
use validator::Validate;

// === From routes_models.rs ===

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ModelError {
  #[error("Failed to fetch metadata.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MetadataFetchFailed,
}

// === From routes_create.rs ===

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAliasRequest {
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: Option<String>,

  pub request_params: Option<objs::OAIRequestParams>,
  pub context_params: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateAliasRequest {
  pub repo: String,
  pub filename: String,
  pub snapshot: Option<String>,

  pub request_params: Option<objs::OAIRequestParams>,
  pub context_params: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum CreateAliasError {
  #[error(transparent)]
  AliasNotFound(#[from] AliasNotFoundError),
  #[error(transparent)]
  CreateCommand(#[from] CreateCommandError),
  #[error("Model alias in path '{path}' does not match alias in request '{request}'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AliasMismatch { path: String, request: String },
}

// === From routes_pull.rs ===

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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum PullError {
  #[error("File '{filename}' already exists in '{repo}'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  FileAlreadyExists {
    repo: String,
    filename: String,
    snapshot: String,
  },
  #[error(transparent)]
  PullCommand(#[from] commands::PullCommandError),
  #[error(transparent)]
  ObjValidation(#[from] ObjValidationError),
}

// === From routes_models_metadata.rs ===

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum MetadataError {
  #[error("Invalid repo format: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRepoFormat(String),

  #[error("Failed to list aliases.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ListAliasesFailed,

  #[error("Model alias not found for repo={repo}, filename={filename}, snapshot={snapshot}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  AliasNotFound {
    repo: String,
    filename: String,
    snapshot: String,
  },

  #[error("Failed to extract metadata: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ExtractionFailed(String),

  #[error("Failed to enqueue metadata refresh task.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  EnqueueFailed,
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
