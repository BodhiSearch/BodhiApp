use commands::CreateCommandError;
use objs::{AppError, ErrorType, ObjValidationError};
use services::AliasNotFoundError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ModelError {
  #[error("Failed to fetch metadata.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MetadataFetchFailed,
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
