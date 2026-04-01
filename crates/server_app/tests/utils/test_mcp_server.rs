#![allow(dead_code)]

use std::{sync::Arc, time::Duration};

use axum::{
  extract::Request,
  middleware::{self, Next},
  response::{IntoResponse, Response},
};
use rmcp::{
  model::{
    AnnotateAble, CallToolRequestParams, CallToolResult, CompleteRequestParams, CompleteResult,
    Content, GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    Prompt, PromptArgument, PromptMessage, PromptMessageRole, RawResource,
    ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents, ServerCapabilities,
    ServerInfo, Tool,
  },
  service::{RequestContext, RoleServer},
  transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
  },
  ErrorData as McpError, ServerHandler,
};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TestTool {
  pub name: String,
  pub description: String,
  pub input_schema: Value,
  pub response: Value,
}

#[derive(Clone, Debug)]
pub struct TestResource {
  pub uri: String,
  pub name: String,
  pub mime_type: String,
  pub content: String,
}

#[derive(Clone, Debug)]
pub struct TestPrompt {
  pub name: String,
  pub description: String,
  pub arguments: Vec<TestPromptArg>,
  pub template: String,
}

#[derive(Clone, Debug)]
pub struct TestPromptArg {
  pub name: String,
  pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ReceivedToolCall {
  pub tool_name: String,
  pub arguments: Option<serde_json::Map<String, Value>>,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TestMcpServerConfig {
  pub port: u16,
  pub tools: Vec<TestTool>,
  pub resources: Vec<TestResource>,
  pub prompts: Vec<TestPrompt>,
  /// If set, the server requires this header (key, value) for authentication.
  /// Requests missing the header or providing the wrong value receive 401.
  pub require_auth: Option<(String, String)>,
}

pub struct TestMcpServerBuilder {
  port: u16,
  tools: Vec<TestTool>,
  resources: Vec<TestResource>,
  prompts: Vec<TestPrompt>,
  require_auth: Option<(String, String)>,
}

impl TestMcpServerBuilder {
  pub fn port(mut self, port: u16) -> Self {
    self.port = port;
    self
  }

  pub fn tool(mut self, tool: TestTool) -> Self {
    self.tools.push(tool);
    self
  }

  pub fn resource(mut self, resource: TestResource) -> Self {
    self.resources.push(resource);
    self
  }

  pub fn prompt(mut self, prompt: TestPrompt) -> Self {
    self.prompts.push(prompt);
    self
  }

  pub fn require_auth(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.require_auth = Some((key.into(), value.into()));
    self
  }

  pub fn build(self) -> TestMcpServerConfig {
    TestMcpServerConfig {
      port: self.port,
      tools: self.tools,
      resources: self.resources,
      prompts: self.prompts,
      require_auth: self.require_auth,
    }
  }
}

// ---------------------------------------------------------------------------
// TestMcpServer -- the running server handle
// ---------------------------------------------------------------------------

pub struct TestMcpServer {
  pub url: String,
  pub port: u16,
  shutdown: CancellationToken,
  pub calls_received: Arc<Mutex<Vec<ReceivedToolCall>>>,
}

impl TestMcpServer {
  pub fn builder() -> TestMcpServerBuilder {
    TestMcpServerBuilder {
      port: 0,
      tools: Vec::new(),
      resources: Vec::new(),
      prompts: Vec::new(),
      require_auth: None,
    }
  }

  pub async fn start(config: TestMcpServerConfig) -> anyhow::Result<Self> {
    let cancellation_token = CancellationToken::new();
    let calls_received = Arc::new(Mutex::new(Vec::new()));

    let handler_config = Arc::new(config.clone());
    let calls = calls_received.clone();

    let service: StreamableHttpService<TestMcpServerHandler, LocalSessionManager> =
      StreamableHttpService::new(
        move || {
          Ok(TestMcpServerHandler::new(
            handler_config.clone(),
            calls.clone(),
          ))
        },
        Default::default(),
        StreamableHttpServerConfig {
          stateful_mode: true,
          sse_keep_alive: Some(Duration::from_secs(15)),
          sse_retry: Some(Duration::from_secs(3)),
          cancellation_token: cancellation_token.child_token(),
        },
      );

    let router = if let Some((ref header_key, ref header_value)) = config.require_auth {
      let expected_key = header_key.clone();
      let expected_value = header_value.clone();
      axum::Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn(move |req: Request, next: Next| {
          let expected_key = expected_key.clone();
          let expected_value = expected_value.clone();
          async move {
            let actual = req
              .headers()
              .get(&expected_key)
              .and_then(|v| v.to_str().ok())
              .map(|s| s.to_string());
            if actual.as_deref() != Some(&expected_value) {
              return Response::builder()
                .status(401)
                .body(axum::body::Body::from("Unauthorized"))
                .unwrap()
                .into_response();
            }
            next.run(req).await
          }
        }))
    } else {
      axum::Router::new().nest_service("/mcp", service)
    };

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.port)).await?;
    let port = listener.local_addr()?.port();

    let token = cancellation_token.clone();
    tokio::spawn(async move {
      axum::serve(listener, router)
        .with_graceful_shutdown(async move { token.cancelled_owned().await })
        .await
        .ok();
    });

    Ok(TestMcpServer {
      url: format!("http://127.0.0.1:{}/mcp", port),
      port,
      shutdown: cancellation_token,
      calls_received,
    })
  }

  pub async fn shutdown(self) {
    self.shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(100)).await;
  }
}

// ---------------------------------------------------------------------------
// TestMcpServerHandler -- implements rmcp::ServerHandler
// ---------------------------------------------------------------------------

struct TestMcpServerHandler {
  config: Arc<TestMcpServerConfig>,
  calls_received: Arc<Mutex<Vec<ReceivedToolCall>>>,
}

impl TestMcpServerHandler {
  fn new(
    config: Arc<TestMcpServerConfig>,
    calls_received: Arc<Mutex<Vec<ReceivedToolCall>>>,
  ) -> Self {
    Self {
      config,
      calls_received,
    }
  }
}

/// Build ServerCapabilities from the config, enabling only the capabilities
/// that have items configured. We construct the raw struct directly because
/// the const-generic builder pattern does not support conditional chaining
/// via if/else.
fn build_capabilities(config: &TestMcpServerConfig) -> ServerCapabilities {
  use rmcp::model::{PromptsCapability, ResourcesCapability, ToolsCapability};

  ServerCapabilities {
    tools: if config.tools.is_empty() {
      None
    } else {
      Some(ToolsCapability::default())
    },
    resources: if config.resources.is_empty() {
      None
    } else {
      Some(ResourcesCapability::default())
    },
    prompts: if config.prompts.is_empty() {
      None
    } else {
      Some(PromptsCapability::default())
    },
    ..Default::default()
  }
}

impl ServerHandler for TestMcpServerHandler {
  fn get_info(&self) -> ServerInfo {
    ServerInfo {
      capabilities: build_capabilities(&self.config),
      server_info: Implementation {
        name: "test-mcp-server".into(),
        title: None,
        version: "0.1.0".into(),
        description: None,
        icons: None,
        website_url: None,
      },
      instructions: Some("A configurable test MCP server".into()),
      ..Default::default()
    }
  }

  fn list_tools(
    &self,
    _request: Option<PaginatedRequestParams>,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
    let tools: Vec<Tool> = self
      .config
      .tools
      .iter()
      .map(|t| {
        let schema: serde_json::Map<String, Value> =
          serde_json::from_value(t.input_schema.clone()).unwrap_or_default();
        Tool::new(t.name.clone(), t.description.clone(), Arc::new(schema))
      })
      .collect();
    std::future::ready(Ok(ListToolsResult::with_all_items(tools)))
  }

  fn call_tool(
    &self,
    request: CallToolRequestParams,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
    let config = self.config.clone();
    let calls = self.calls_received.clone();
    async move {
      let tool_name = request.name.to_string();
      let arguments = request.arguments.clone();

      // Record the call
      calls.lock().await.push(ReceivedToolCall {
        tool_name: tool_name.clone(),
        arguments,
      });

      // Find matching tool
      let tool = config.tools.iter().find(|t| t.name == tool_name);
      match tool {
        Some(t) => {
          let content = parse_response_to_content(&t.response);
          Ok(CallToolResult::success(content))
        }
        None => Err(McpError::invalid_params(
          format!("Unknown tool: {}", tool_name),
          None,
        )),
      }
    }
  }

  fn list_resources(
    &self,
    _request: Option<PaginatedRequestParams>,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
    let resources: Vec<Resource> = self
      .config
      .resources
      .iter()
      .map(|r| {
        let mut raw = RawResource::new(r.uri.clone(), r.name.clone());
        raw.mime_type = Some(r.mime_type.clone());
        raw.no_annotation()
      })
      .collect();
    std::future::ready(Ok(ListResourcesResult::with_all_items(resources)))
  }

  fn read_resource(
    &self,
    request: ReadResourceRequestParams,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
    let resource = self
      .config
      .resources
      .iter()
      .find(|r| r.uri == request.uri)
      .cloned();
    std::future::ready(match resource {
      Some(r) => Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
          uri: r.uri,
          mime_type: Some(r.mime_type),
          text: r.content,
          meta: None,
        }],
      }),
      None => Err(McpError::resource_not_found(
        format!("Resource not found: {}", request.uri),
        None,
      )),
    })
  }

  fn list_resource_templates(
    &self,
    _request: Option<PaginatedRequestParams>,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_
  {
    std::future::ready(Ok(ListResourceTemplatesResult::default()))
  }

  fn list_prompts(
    &self,
    _request: Option<PaginatedRequestParams>,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<ListPromptsResult, McpError>> + Send + '_ {
    let prompts: Vec<Prompt> = self
      .config
      .prompts
      .iter()
      .map(|p| {
        let arguments: Vec<PromptArgument> = p
          .arguments
          .iter()
          .map(|a| PromptArgument {
            name: a.name.clone(),
            title: None,
            description: None,
            required: Some(a.required),
          })
          .collect();
        Prompt::new(
          p.name.clone(),
          Some(p.description.clone()),
          if arguments.is_empty() {
            None
          } else {
            Some(arguments)
          },
        )
      })
      .collect();
    std::future::ready(Ok(ListPromptsResult::with_all_items(prompts)))
  }

  fn get_prompt(
    &self,
    request: GetPromptRequestParams,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<GetPromptResult, McpError>> + Send + '_ {
    let prompt = self
      .config
      .prompts
      .iter()
      .find(|p| p.name == request.name)
      .cloned();
    std::future::ready(match prompt {
      Some(p) => {
        // Substitute arguments in template
        let mut text = p.template.clone();
        if let Some(args) = &request.arguments {
          for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
              Value::String(s) => s.clone(),
              other => other.to_string(),
            };
            text = text.replace(&placeholder, &replacement);
          }
        }
        Ok(GetPromptResult {
          description: Some(p.description.clone()),
          messages: vec![PromptMessage::new_text(PromptMessageRole::User, text)],
        })
      }
      None => Err(McpError::invalid_params(
        format!("Unknown prompt: {}", request.name),
        None,
      )),
    })
  }

  fn complete(
    &self,
    _request: CompleteRequestParams,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<CompleteResult, McpError>> + Send + '_ {
    std::future::ready(Ok(CompleteResult::default()))
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a JSON value into MCP content items.
/// Expects either an array of `{"type": "text", "text": "..."}` objects
/// or a single string value (converted to text content).
fn parse_response_to_content(value: &Value) -> Vec<Content> {
  match value {
    Value::Array(items) => items
      .iter()
      .filter_map(|item| {
        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
          Some(Content::text(text.to_string()))
        } else {
          None
        }
      })
      .collect(),
    Value::String(s) => vec![Content::text(s.clone())],
    other => vec![Content::text(other.to_string())],
  }
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use rmcp::{
    model::{CallToolRequestParams, PromptMessageContent},
    service::ServiceExt,
    transport::StreamableHttpClientTransport,
  };
  use serde_json::json;

  #[tokio::test]
  async fn test_mcp_server_lifecycle() -> anyhow::Result<()> {
    let config = TestMcpServer::builder()
      .tool(TestTool {
        name: "echo".into(),
        description: "Echoes input".into(),
        input_schema: json!({
          "type": "object",
          "properties": {
            "message": {"type": "string"}
          }
        }),
        response: json!([{"type": "text", "text": "echoed: hello"}]),
      })
      .resource(TestResource {
        uri: "file://test".into(),
        name: "TestFile".into(),
        mime_type: "text/plain".into(),
        content: "# Hello World".into(),
      })
      .prompt(TestPrompt {
        name: "greeting".into(),
        description: "Greets user".into(),
        arguments: vec![TestPromptArg {
          name: "name".into(),
          required: true,
        }],
        template: "Hello {name}!".into(),
      })
      .build();

    let server = TestMcpServer::start(config).await?;

    // Connect with rmcp client
    let transport = StreamableHttpClientTransport::from_uri(server.url.as_str());
    let client = rmcp::model::ClientInfo::default().serve(transport).await?;

    // -- Test list_tools --
    let tools_result = client.list_tools(None).await?;
    assert_eq!(1, tools_result.tools.len());
    assert_eq!("echo", tools_result.tools[0].name.as_ref());

    // -- Test call_tool --
    let call_result = client
      .call_tool(CallToolRequestParams {
        meta: None,
        name: "echo".into(),
        arguments: Some(serde_json::Map::from_iter([(
          "message".to_string(),
          json!("hello"),
        )])),
        task: None,
      })
      .await?;
    assert_eq!(1, call_result.content.len());
    let text = call_result.content[0]
      .raw
      .as_text()
      .expect("Expected text content");
    assert_eq!("echoed: hello", text.text);

    // -- Test list_resources --
    let resources_result = client.list_resources(None).await?;
    assert_eq!(1, resources_result.resources.len());
    assert_eq!("file://test", resources_result.resources[0].uri);

    // -- Test read_resource --
    let read_result = client
      .read_resource(ReadResourceRequestParams {
        meta: None,
        uri: "file://test".into(),
      })
      .await?;
    assert_eq!(1, read_result.contents.len());
    match &read_result.contents[0] {
      ResourceContents::TextResourceContents { text, .. } => {
        assert_eq!("# Hello World", text);
      }
      other => panic!("Expected TextResourceContents, got {:?}", other),
    }

    // -- Test list_prompts --
    let prompts_result = client.list_prompts(None).await?;
    assert_eq!(1, prompts_result.prompts.len());
    assert_eq!("greeting", prompts_result.prompts[0].name);

    // -- Test get_prompt --
    let prompt_result = client
      .get_prompt(GetPromptRequestParams {
        meta: None,
        name: "greeting".into(),
        arguments: Some(serde_json::Map::from_iter([(
          "name".to_string(),
          json!("World"),
        )])),
      })
      .await?;
    assert_eq!(1, prompt_result.messages.len());
    match &prompt_result.messages[0].content {
      PromptMessageContent::Text { text } => {
        assert_eq!("Hello World!", text);
      }
      other => panic!("Expected Text content, got {:?}", other),
    }

    // -- Verify recorded tool calls --
    let calls = server.calls_received.lock().await;
    assert_eq!(1, calls.len());
    assert_eq!("echo", calls[0].tool_name);
    assert!(calls[0].arguments.is_some());
    let args = calls[0].arguments.as_ref().unwrap();
    assert_eq!(json!("hello"), args["message"]);
    drop(calls);

    // Shutdown
    server.shutdown().await;
    Ok(())
  }

  #[tokio::test]
  async fn test_mcp_server_require_auth() -> anyhow::Result<()> {
    let config = TestMcpServer::builder()
      .tool(TestTool {
        name: "echo".into(),
        description: "Echoes input".into(),
        input_schema: json!({
          "type": "object",
          "properties": {
            "message": {"type": "string"}
          }
        }),
        response: json!([{"type": "text", "text": "echoed: hello"}]),
      })
      .require_auth("X-Api-Key", "test-secret")
      .build();

    let server = TestMcpServer::start(config).await?;

    // Without auth header: should get 401
    let http = reqwest::Client::new();
    let resp = http
      .post(format!("{}", server.url))
      .header("Content-Type", "application/json")
      .header("Accept", "application/json, text/event-stream")
      .json(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "protocolVersion": "2025-03-26",
          "capabilities": {},
          "clientInfo": {"name": "test", "version": "1.0"}
        }
      }))
      .send()
      .await?;
    assert_eq!(
      401,
      resp.status().as_u16(),
      "Missing auth should return 401"
    );

    // With wrong auth header: should get 401
    let resp = http
      .post(format!("{}", server.url))
      .header("Content-Type", "application/json")
      .header("Accept", "application/json, text/event-stream")
      .header("X-Api-Key", "wrong-secret")
      .json(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
          "protocolVersion": "2025-03-26",
          "capabilities": {},
          "clientInfo": {"name": "test", "version": "1.0"}
        }
      }))
      .send()
      .await?;
    assert_eq!(401, resp.status().as_u16(), "Wrong auth should return 401");

    // With correct auth header: should succeed
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Api-Key", "test-secret".parse().unwrap());
    let http_client = reqwest::Client::builder()
      .default_headers(headers)
      .build()?;
    let transport = StreamableHttpClientTransport::with_client(
      http_client,
      rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig::with_uri(
        server.url.as_str(),
      ),
    );
    let client = rmcp::model::ClientInfo::default().serve(transport).await?;

    let tools_result = client.list_tools(None).await?;
    assert_eq!(1, tools_result.tools.len());
    assert_eq!("echo", tools_result.tools[0].name.as_ref());

    server.shutdown().await;
    Ok(())
  }
}
