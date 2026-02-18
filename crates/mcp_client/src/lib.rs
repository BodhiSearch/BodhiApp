use objs::McpTool;
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::ServiceExt;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Debug;

mod error;
pub use error::McpClientError;

/// Running MCP client type alias for rmcp
type RunningMcpClient = rmcp::service::RunningService<rmcp::RoleClient, ClientInfo>;

/// Trait for MCP client operations (connect, list tools, call tool).
/// Per-request connection pattern: each call creates a fresh connection.
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait McpClient: Debug + Send + Sync {
  /// Connect to an MCP server, fetch available tools, disconnect.
  async fn fetch_tools(&self, url: &str) -> Result<Vec<McpTool>, McpClientError>;

  /// Connect to an MCP server, call a tool, disconnect.
  async fn call_tool(
    &self,
    url: &str,
    tool_name: &str,
    args: Value,
  ) -> Result<Value, McpClientError>;
}

/// Default MCP client using rmcp with Streamable HTTP transport.
/// Creates a fresh connection per request (no connection pooling).
#[derive(Debug, Clone)]
pub struct DefaultMcpClient;

impl DefaultMcpClient {
  pub fn new() -> Self {
    Self
  }

  fn create_client_info() -> ClientInfo {
    ClientInfo {
      meta: None,
      protocol_version: Default::default(),
      capabilities: ClientCapabilities::default(),
      client_info: Implementation {
        name: "bodhi-mcp-client".to_string(),
        title: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        website_url: None,
        icons: None,
      },
    }
  }

  async fn connect(url: &str) -> Result<RunningMcpClient, McpClientError> {
    let http_client = reqwest::Client::new();
    let transport = StreamableHttpClientTransport::with_client(
      http_client,
      StreamableHttpClientTransportConfig::with_uri(url),
    );
    let client = Self::create_client_info()
      .serve(transport)
      .await
      .map_err(|e| McpClientError::ConnectionFailed {
        url: url.to_string(),
        reason: e.to_string(),
      })?;
    Ok(client)
  }
}

#[async_trait::async_trait]
impl McpClient for DefaultMcpClient {
  async fn fetch_tools(&self, url: &str) -> Result<Vec<McpTool>, McpClientError> {
    let client = Self::connect(url).await?;

    let tools_response =
      client
        .list_tools(Default::default())
        .await
        .map_err(|e| McpClientError::ProtocolError {
          operation: "list_tools".to_string(),
          reason: e.to_string(),
        })?;

    let tools: Vec<McpTool> = tools_response
      .tools
      .into_iter()
      .map(|tool| McpTool {
        name: tool.name.to_string(),
        description: tool.description.map(|d| d.to_string()),
        input_schema: serde_json::to_value(&tool.input_schema).ok(),
      })
      .collect();

    let _ = client.cancel().await;
    Ok(tools)
  }

  async fn call_tool(
    &self,
    url: &str,
    tool_name: &str,
    args: Value,
  ) -> Result<Value, McpClientError> {
    let client = Self::connect(url).await?;

    let result = client
      .call_tool(CallToolRequestParams {
        meta: None,
        name: Cow::Owned(tool_name.to_string()),
        arguments: args.as_object().cloned(),
        task: None,
      })
      .await
      .map_err(|e| McpClientError::ExecutionFailed {
        tool: tool_name.to_string(),
        reason: e.to_string(),
      })?;

    let content =
      serde_json::to_value(&result.content).map_err(|e| McpClientError::SerializationError {
        reason: e.to_string(),
      })?;

    let _ = client.cancel().await;
    Ok(content)
  }
}
