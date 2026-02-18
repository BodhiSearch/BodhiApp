use crate::db::DbError;
use mcp_client::McpClientError;
use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum McpError {
  #[error("MCP instance '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  McpNotFound(String),

  #[error("MCP server URL is not in the allowlist.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  McpUrlNotAllowed,

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

  #[error("Failed to connect to MCP server: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ConnectionFailed(String),

  #[error("MCP tool execution failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecutionFailed(String),

  #[error(transparent)]
  DbError(#[from] DbError),
}

impl From<McpClientError> for McpError {
  fn from(err: McpClientError) -> Self {
    match err {
      McpClientError::ConnectionFailed { url, reason } => {
        McpError::ConnectionFailed(format!("{}: {}", url, reason))
      }
      McpClientError::ExecutionFailed { tool, reason } => {
        McpError::ExecutionFailed(format!("{}: {}", tool, reason))
      }
      McpClientError::ProtocolError { operation, reason } => {
        McpError::ExecutionFailed(format!("{}: {}", operation, reason))
      }
      McpClientError::SerializationError { reason } => McpError::ExecutionFailed(reason),
    }
  }
}
