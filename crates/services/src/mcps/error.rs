use crate::db::{encryption::EncryptionError, DbError};
use crate::mcps::mcp_objs::McpInstanceNameError;
use errmeta::{AppError, ErrorType};
use mcp_client::McpClientError;

// ============================================================================
// McpServerError - Admin MCP server management operations
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum McpServerError {
  #[error("MCP server '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  McpServerNotFound(String),

  #[error("MCP server URL '{0}' already exists.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  UrlAlreadyExists(String),

  #[error("MCP server name is required.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NameRequired,

  #[error("MCP server URL is required.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  UrlRequired,

  #[error("MCP server URL is not valid: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  UrlInvalid(String),

  #[error("MCP server URL cannot exceed 2048 characters.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  UrlTooLong,

  #[error("MCP server name cannot exceed 100 characters.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NameTooLong,

  #[error("MCP server description cannot exceed 255 characters.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  DescriptionTooLong,

  #[error(transparent)]
  DbError(#[from] DbError),
}

// ============================================================================
// McpError - User MCP instance operations
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum McpError {
  #[error("MCP instance '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  McpNotFound(String),

  #[error("MCP server '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  McpServerNotFound(String),

  #[error("MCP server is disabled.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  McpDisabled,

  #[error("Tool '{0}' is not in the allowed tools filter.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolNotAllowed(String),

  #[error("Tool '{0}' not found in MCP server tools cache.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolNotFound(String),

  #[error("MCP slug '{0}' already exists.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  SlugExists(String),

  #[error("Invalid MCP slug: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSlug(String),

  #[error("Invalid MCP description: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidDescription(String),

  #[error("MCP name is required.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NameRequired,

  #[error("MCP instance name '{name}' exceeds the maximum length of {max_len} characters.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NameTooLong { name: String, max_len: usize },

  #[error(transparent)]
  Client(#[from] McpClientError),

  #[error("Encryption error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  EncryptionError(String),

  #[error("OAuth token not found for config '{0}'.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  OAuthTokenNotFound(String),

  #[error("OAuth token expired and no refresh token available for config '{0}'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  OAuthTokenExpired(String),

  #[error("OAuth token refresh failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  OAuthRefreshFailed(String),

  #[error("OAuth token exchange failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  OAuthTokenExchangeFailed(String),

  #[error("OAuth discovery failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  OAuthDiscoveryFailed(String),

  #[error(transparent)]
  DbError(#[from] DbError),
}

impl From<EncryptionError> for McpError {
  fn from(e: EncryptionError) -> Self {
    McpError::EncryptionError(e.to_string())
  }
}

impl From<McpInstanceNameError> for McpError {
  fn from(e: McpInstanceNameError) -> Self {
    match e {
      McpInstanceNameError::Empty => McpError::NameRequired,
      McpInstanceNameError::TooLong { name, max_len } => McpError::NameTooLong { name, max_len },
    }
  }
}
