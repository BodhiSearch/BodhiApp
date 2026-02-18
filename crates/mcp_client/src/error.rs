#[derive(Debug, thiserror::Error)]
pub enum McpClientError {
  #[error("Failed to connect to MCP server at {url}: {reason}")]
  ConnectionFailed { url: String, reason: String },

  #[error("MCP protocol error during {operation}: {reason}")]
  ProtocolError { operation: String, reason: String },

  #[error("MCP tool execution failed for {tool}: {reason}")]
  ExecutionFailed { tool: String, reason: String },

  #[error("Serialization error: {reason}")]
  SerializationError { reason: String },
}
