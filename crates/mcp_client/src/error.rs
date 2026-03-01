use errmeta::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum McpClientError {
  #[error("Failed to connect to MCP server at {url}: {reason}")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  ConnectionFailed { url: String, reason: String },

  #[error("MCP protocol error during {operation}: {reason}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ProtocolError { operation: String, reason: String },

  #[error("MCP tool execution failed for {tool}: {reason}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecutionFailed { tool: String, reason: String },

  #[error("Serialization error: {reason}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SerializationError { reason: String },
}
