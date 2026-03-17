use crate::error::McpClientError;
use crate::mcp_objs::{McpAuthParams, McpTool};
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::ServiceExt;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Debug;

/// Running MCP client type alias for rmcp
type RunningMcpClient = rmcp::service::RunningService<rmcp::RoleClient, ClientInfo>;

/// Trait for MCP client operations (connect, list tools, call tool).
/// Per-request connection pattern: each call creates a fresh connection.
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait McpClient: Debug + Send + Sync {
  /// Connect to an MCP server, fetch available tools, disconnect.
  /// `auth_params`: optional auth parameters (headers + query params) to inject.
  async fn fetch_tools(
    &self,
    url: &str,
    auth_params: Option<McpAuthParams>,
  ) -> Result<Vec<McpTool>, McpClientError>;

  /// Connect to an MCP server, call a tool, disconnect.
  /// `auth_params`: optional auth parameters (headers + query params) to inject.
  async fn call_tool(
    &self,
    url: &str,
    tool_name: &str,
    args: Value,
    auth_params: Option<McpAuthParams>,
  ) -> Result<Value, McpClientError>;
}

/// Default MCP client using rmcp with Streamable HTTP transport.
/// Creates a fresh connection per request (no connection pooling).
#[derive(Debug, Clone)]
pub struct DefaultMcpClient;

impl Default for DefaultMcpClient {
  fn default() -> Self {
    Self::new()
  }
}

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
        description: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        website_url: None,
        icons: None,
      },
    }
  }

  /// Prepare auth parameters: build (final_url, headers) from the base URL and auth params.
  /// Headers are converted to a reqwest HeaderMap. Query params are appended to the URL.
  fn prepare_auth(
    url: &str,
    auth_params: Option<McpAuthParams>,
  ) -> Result<(String, reqwest::header::HeaderMap), McpClientError> {
    let auth_params = match auth_params {
      Some(p) if !p.headers.is_empty() || !p.query_params.is_empty() => p,
      _ => return Ok((url.to_string(), reqwest::header::HeaderMap::new())),
    };

    let final_url = if auth_params.query_params.is_empty() {
      url.to_string()
    } else {
      let mut parsed = url::Url::parse(url).map_err(|e| McpClientError::ConnectionFailed {
        url: url.to_string(),
        reason: format!("Invalid URL: {}", e),
      })?;
      {
        let mut pairs = parsed.query_pairs_mut();
        for (key, value) in &auth_params.query_params {
          pairs.append_pair(key, value);
        }
      }
      parsed.to_string()
    };

    let mut header_map = reqwest::header::HeaderMap::new();
    for (header_name, header_value) in &auth_params.headers {
      let name = reqwest::header::HeaderName::from_bytes(header_name.to_lowercase().as_bytes())
        .map_err(|e| McpClientError::ConnectionFailed {
          url: url.to_string(),
          reason: format!("Invalid header name '{}': {}", header_name, e),
        })?;
      let value = reqwest::header::HeaderValue::from_str(header_value).map_err(|e| {
        McpClientError::ConnectionFailed {
          url: url.to_string(),
          reason: format!("Invalid header value: {}", e),
        }
      })?;
      header_map.insert(name, value);
    }

    Ok((final_url, header_map))
  }

  async fn connect(
    url: &str,
    auth_params: Option<McpAuthParams>,
  ) -> Result<RunningMcpClient, McpClientError> {
    let (final_url, headers) = Self::prepare_auth(url, auth_params)?;

    let http_client = if headers.is_empty() {
      reqwest::Client::new()
    } else {
      reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| McpClientError::ConnectionFailed {
          url: url.to_string(),
          reason: e.to_string(),
        })?
    };

    let transport = StreamableHttpClientTransport::with_client(
      http_client,
      StreamableHttpClientTransportConfig::with_uri(&*final_url),
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
  async fn fetch_tools(
    &self,
    url: &str,
    auth_params: Option<McpAuthParams>,
  ) -> Result<Vec<McpTool>, McpClientError> {
    let client = Self::connect(url, auth_params).await?;

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
    auth_params: Option<McpAuthParams>,
  ) -> Result<Value, McpClientError> {
    let client = Self::connect(url, auth_params).await?;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mcp_auth_params_default() {
    let params = McpAuthParams::default();
    assert!(params.headers.is_empty());
    assert!(params.query_params.is_empty());
  }

  #[test]
  fn test_prepare_auth_none() {
    let (url, headers) = DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", None).unwrap();
    assert_eq!("http://localhost:8080/mcp", url);
    assert!(headers.is_empty());
  }

  #[test]
  fn test_prepare_auth_empty_params() {
    let params = McpAuthParams::default();
    let (url, headers) =
      DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", Some(params)).unwrap();
    assert_eq!("http://localhost:8080/mcp", url);
    assert!(headers.is_empty());
  }

  #[test]
  fn test_prepare_auth_headers_only() {
    let params = McpAuthParams {
      headers: vec![
        ("Authorization".to_string(), "Bearer token123".to_string()),
        ("X-API-Key".to_string(), "my-key".to_string()),
      ],
      query_params: vec![],
    };
    let (url, headers) =
      DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", Some(params)).unwrap();
    assert_eq!("http://localhost:8080/mcp", url);
    assert_eq!(2, headers.len());
    assert_eq!("Bearer token123", headers.get("authorization").unwrap());
    assert_eq!("my-key", headers.get("x-api-key").unwrap());
  }

  #[test]
  fn test_prepare_auth_query_params_only() {
    let params = McpAuthParams {
      headers: vec![],
      query_params: vec![("api_key".to_string(), "secret123".to_string())],
    };
    let (url, headers) =
      DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", Some(params)).unwrap();
    assert_eq!("http://localhost:8080/mcp?api_key=secret123", url);
    assert!(headers.is_empty());
  }

  #[test]
  fn test_prepare_auth_query_params_appends_to_existing() {
    let params = McpAuthParams {
      headers: vec![],
      query_params: vec![("api_key".to_string(), "secret123".to_string())],
    };
    let (url, headers) =
      DefaultMcpClient::prepare_auth("http://localhost:8080/mcp?existing=val", Some(params))
        .unwrap();
    assert_eq!(
      "http://localhost:8080/mcp?existing=val&api_key=secret123",
      url
    );
    assert!(headers.is_empty());
  }

  #[test]
  fn test_prepare_auth_mixed_headers_and_query() {
    let params = McpAuthParams {
      headers: vec![("Authorization".to_string(), "Bearer tok".to_string())],
      query_params: vec![
        ("api_key".to_string(), "key1".to_string()),
        ("session".to_string(), "sess2".to_string()),
      ],
    };
    let (url, headers) =
      DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", Some(params)).unwrap();
    assert_eq!("http://localhost:8080/mcp?api_key=key1&session=sess2", url);
    assert_eq!(1, headers.len());
    assert_eq!("Bearer tok", headers.get("authorization").unwrap());
  }

  #[test]
  fn test_prepare_auth_invalid_header_name() {
    let params = McpAuthParams {
      headers: vec![("Invalid Header\0".to_string(), "value".to_string())],
      query_params: vec![],
    };
    let result = DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", Some(params));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
      matches!(err, McpClientError::ConnectionFailed { .. }),
      "Expected ConnectionFailed, got: {:?}",
      err
    );
  }

  #[test]
  fn test_prepare_auth_invalid_url_for_query_params() {
    let params = McpAuthParams {
      headers: vec![],
      query_params: vec![("key".to_string(), "val".to_string())],
    };
    let result = DefaultMcpClient::prepare_auth("not-a-valid-url", Some(params));
    assert!(result.is_err());
  }

  #[test]
  fn test_prepare_auth_query_param_special_chars_encoded() {
    let params = McpAuthParams {
      headers: vec![],
      query_params: vec![(
        "key".to_string(),
        "value with spaces&special=chars".to_string(),
      )],
    };
    let (url, _) =
      DefaultMcpClient::prepare_auth("http://localhost:8080/mcp", Some(params)).unwrap();
    assert!(url.contains("key=value+with+spaces%26special%3Dchars"));
  }
}
