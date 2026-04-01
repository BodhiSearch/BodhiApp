mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use utils::{
  create_test_session_for_live_server, start_test_live_server,
  test_mcp_server::{
    TestMcpServer, TestMcpServerConfig, TestPrompt, TestPromptArg, TestResource, TestTool,
  },
  TestLiveServer,
};

// =============================================================================
// SSE parsing helper
// =============================================================================

/// Parse SSE response text and extract JSON-RPC payloads from `data:` lines.
///
/// Handles the SSE format returned by the MCP Streamable HTTP protocol:
/// ```text
/// event: message
/// data: {"jsonrpc":"2.0","id":1,"result":{...}}
/// ```
///
/// Also handles priming events (empty data lines, retry, id lines) by skipping them.
fn parse_sse_jsonrpc(sse_text: &str) -> Vec<Value> {
  let mut results = Vec::new();
  for line in sse_text.lines() {
    if let Some(data) = line.strip_prefix("data: ") {
      let trimmed = data.trim();
      if trimmed.is_empty() {
        continue;
      }
      if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        results.push(value);
      }
    }
  }
  results
}

// =============================================================================
// MCP protocol helpers
// =============================================================================

/// Create an MCP server entry via the REST API and return its ID.
async fn create_mcp_server(
  client: &Client,
  base_url: &str,
  cookie: &str,
  upstream_url: &str,
) -> anyhow::Result<String> {
  let resp = client
    .post(format!("{}/bodhi/v1/mcps/servers", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "url": upstream_url,
      "name": "Test MCP Server",
      "description": "Test MCP server for proxy tests",
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Failed to create MCP server"
  );
  let body: Value = resp.json().await?;
  Ok(body["id"].as_str().unwrap().to_string())
}

/// Send an MCP initialize request and return (session_id, response_json).
///
/// This is the first POST to the proxy endpoint, without a Mcp-Session-Id header.
/// The response is SSE with the Mcp-Session-Id returned in a response header.
async fn mcp_initialize(
  client: &Client,
  base_url: &str,
  cookie: &str,
  mcp_id: &str,
) -> anyhow::Result<(String, Value)> {
  let resp = client
    .post(format!("{}/bodhi/v1/apps/mcps/{}/mcp", base_url, mcp_id))
    .header("Cookie", cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .json(&json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2025-03-26",
        "capabilities": {},
        "clientInfo": {
          "name": "test-client",
          "version": "1.0"
        }
      }
    }))
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "Initialize should return 200"
  );

  let session_id = resp
    .headers()
    .get("mcp-session-id")
    .expect("Response should contain Mcp-Session-Id header")
    .to_str()?
    .to_string();

  let sse_text = resp.text().await?;
  let messages = parse_sse_jsonrpc(&sse_text);
  assert!(
    !messages.is_empty(),
    "Initialize should return at least one JSON-RPC message in SSE, got: {}",
    sse_text
  );

  Ok((session_id, messages[0].clone()))
}

/// Send a raw MCP initialize request and return the raw response (for error cases).
async fn mcp_initialize_raw(
  client: &Client,
  base_url: &str,
  cookie: &str,
  mcp_id: &str,
) -> anyhow::Result<reqwest::Response> {
  let resp = client
    .post(format!("{}/bodhi/v1/apps/mcps/{}/mcp", base_url, mcp_id))
    .header("Cookie", cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .json(&json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2025-03-26",
        "capabilities": {},
        "clientInfo": {
          "name": "test-client",
          "version": "1.0"
        }
      }
    }))
    .send()
    .await?;

  Ok(resp)
}

/// Send an MCP request with session ID and return the response JSON.
async fn mcp_request(
  client: &Client,
  base_url: &str,
  cookie: &str,
  mcp_id: &str,
  session_id: &str,
  body: Value,
) -> anyhow::Result<Value> {
  let resp = client
    .post(format!("{}/bodhi/v1/apps/mcps/{}/mcp", base_url, mcp_id))
    .header("Cookie", cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .header("Mcp-Session-Id", session_id)
    .json(&body)
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "MCP request should return 200, body: {}",
    body
  );

  let sse_text = resp.text().await?;
  let messages = parse_sse_jsonrpc(&sse_text);
  assert!(
    !messages.is_empty(),
    "MCP request should return at least one JSON-RPC message, got: {}",
    sse_text
  );

  Ok(messages[0].clone())
}

/// Send an MCP notification (no `id` field) and return the status code.
/// Notifications should return 202 Accepted.
async fn mcp_notify(
  client: &Client,
  base_url: &str,
  cookie: &str,
  mcp_id: &str,
  session_id: &str,
  body: Value,
) -> anyhow::Result<StatusCode> {
  let resp = client
    .post(format!("{}/bodhi/v1/apps/mcps/{}/mcp", base_url, mcp_id))
    .header("Cookie", cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .header("Mcp-Session-Id", session_id)
    .json(&body)
    .send()
    .await?;

  Ok(resp.status())
}

/// Delete an MCP session via the DELETE method.
async fn mcp_delete_session(
  client: &Client,
  base_url: &str,
  cookie: &str,
  mcp_id: &str,
  session_id: &str,
) -> anyhow::Result<StatusCode> {
  let resp = client
    .delete(format!("{}/bodhi/v1/apps/mcps/{}/mcp", base_url, mcp_id))
    .header("Cookie", cookie)
    .header("Mcp-Session-Id", session_id)
    .send()
    .await?;

  Ok(resp.status())
}

// =============================================================================
// Test helper: standard echo + weather tools
// =============================================================================

fn echo_tool() -> TestTool {
  TestTool {
    name: "echo".into(),
    description: "Echoes the input message".into(),
    input_schema: json!({
      "type": "object",
      "properties": {
        "message": {"type": "string"}
      },
      "required": ["message"]
    }),
    response: json!([{"type": "text", "text": "echoed: hello"}]),
  }
}

fn weather_tool() -> TestTool {
  TestTool {
    name: "weather".into(),
    description: "Gets weather for a city".into(),
    input_schema: json!({
      "type": "object",
      "properties": {
        "city": {"type": "string"}
      },
      "required": ["city"]
    }),
    response: json!([{"type": "text", "text": "Sunny, 72F"}]),
  }
}

// =============================================================================
// Shared proxy test setup
// =============================================================================

/// Holds all handles needed by proxy integration tests.
struct ProxyTestSetup {
  upstream: TestMcpServer,
  server: TestLiveServer,
  client: Client,
  cookie: String,
  mcp_id: String,
}

impl ProxyTestSetup {
  /// Convenience: the Bodhi server's base URL.
  fn base_url(&self) -> &str {
    &self.server.base_url
  }
}

/// Standard proxy test setup: start upstream, start Bodhi, create session + MCP server + instance.
///
/// The `instance_json` controls the MCP instance payload. For a simple public instance use
/// `json!({"name": "Test MCP Instance", "slug": "test-proxy", "enabled": true})`.
async fn setup_proxy_test(
  upstream_config: TestMcpServerConfig,
  instance_json: Value,
) -> anyhow::Result<ProxyTestSetup> {
  let upstream = TestMcpServer::start(upstream_config).await?;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id = create_mcp_server(&client, &server.base_url, &cookie, &upstream.url).await?;

  // Build instance payload, injecting mcp_server_id
  let mut payload = instance_json;
  payload["mcp_server_id"] = json!(server_id);

  let resp = client
    .post(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &cookie)
    .json(&payload)
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Failed to create MCP instance"
  );
  let body: Value = resp.json().await?;
  let mcp_id = body["id"].as_str().unwrap().to_string();

  Ok(ProxyTestSetup {
    upstream,
    server,
    client,
    cookie,
    mcp_id,
  })
}

/// Default public instance JSON (no auth).
fn default_instance_json() -> Value {
  json!({
    "name": "Test MCP Instance",
    "slug": "test-proxy",
    "enabled": true
  })
}

// =============================================================================
// Test: Full lifecycle
// =============================================================================

/// Full proxy lifecycle: initialize -> notify -> list tools -> call tool -> delete session.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_full_lifecycle() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder()
      .tool(echo_tool())
      .tool(weather_tool())
      .build(),
    default_instance_json(),
  )
  .await?;

  // Initialize MCP session
  let (session_id, init_result) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  assert!(
    !session_id.is_empty(),
    "Should receive a non-empty Mcp-Session-Id"
  );
  assert_eq!("2.0", init_result["jsonrpc"]);
  assert!(
    init_result["result"]["serverInfo"]["name"]
      .as_str()
      .is_some(),
    "InitializeResult should have serverInfo.name"
  );

  // Send notifications/initialized
  let notify_status = mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "method": "notifications/initialized"
    }),
  )
  .await?;
  assert_eq!(
    StatusCode::ACCEPTED,
    notify_status,
    "Notification should return 202"
  );

  // List tools
  let tools_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "id": 2,
      "method": "tools/list"
    }),
  )
  .await?;

  let tools = tools_result["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools array");
  assert_eq!(2, tools.len(), "Should have echo and weather tools");
  let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
  assert!(tool_names.contains(&"echo"), "Should contain echo tool");
  assert!(
    tool_names.contains(&"weather"),
    "Should contain weather tool"
  );

  // Call echo tool
  let call_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "tools/call",
      "params": {
        "name": "echo",
        "arguments": {
          "message": "hello"
        }
      }
    }),
  )
  .await?;

  let content = call_result["result"]["content"]
    .as_array()
    .expect("tools/call should return content array");
  assert!(
    !content.is_empty(),
    "Tool call result should have content items"
  );
  assert_eq!(
    "echoed: hello",
    content[0]["text"].as_str().unwrap(),
    "Echo tool should return the echoed message"
  );

  // Verify upstream received the call
  {
    let calls = setup.upstream.calls_received.lock().await;
    assert_eq!(1, calls.len(), "Upstream should have received 1 tool call");
    assert_eq!("echo", calls[0].tool_name);
  }

  // Delete session
  let delete_status = mcp_delete_session(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
  )
  .await?;
  assert!(
    delete_status == StatusCode::OK
      || delete_status == StatusCode::ACCEPTED
      || delete_status == StatusCode::NO_CONTENT,
    "DELETE session should succeed, got: {}",
    delete_status
  );

  // Post with expired session should fail
  let resp = setup
    .client
    .post(format!(
      "{}/bodhi/v1/apps/mcps/{}/mcp",
      setup.base_url(),
      setup.mcp_id
    ))
    .header("Cookie", &setup.cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .header("Mcp-Session-Id", &session_id)
    .json(&json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "tools/list"
    }))
    .send()
    .await?;
  assert!(
    resp.status() == StatusCode::NOT_FOUND || resp.status().is_client_error(),
    "Request with deleted session should fail, got: {}",
    resp.status()
  );

  // Cleanup
  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: All tools forwarded
// =============================================================================

/// Transparent proxy forwards all upstream tools.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_all_tools_forwarded() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder()
      .tool(echo_tool())
      .tool(weather_tool())
      .build(),
    default_instance_json(),
  )
  .await?;

  // Initialize session
  let (session_id, _) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  // Send initialized notification
  mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // List tools - transparent proxy returns all upstream tools
  let tools_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
  )
  .await?;

  let tools = tools_result["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools array");
  assert_eq!(
    2,
    tools.len(),
    "Transparent proxy returns all upstream tools"
  );
  let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
  assert!(tool_names.contains(&"echo"));
  assert!(tool_names.contains(&"weather"));

  // Call echo - should succeed
  let call_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "tools/call",
      "params": {"name": "echo", "arguments": {"message": "test"}}
    }),
  )
  .await?;
  assert!(
    call_result["result"]["content"].is_array(),
    "Echo call should succeed"
  );

  // Call weather - transparent proxy forwards all tools
  let weather_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "tools/call",
      "params": {"name": "weather", "arguments": {"city": "London"}}
    }),
  )
  .await?;
  assert!(
    weather_result["result"]["content"].is_array(),
    "Weather call should succeed through transparent proxy"
  );

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Resources
// =============================================================================

/// Proxy should forward resource list and read requests to the upstream server.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_resources() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder()
      .resource(TestResource {
        uri: "file://readme".into(),
        name: "README".into(),
        mime_type: "text/plain".into(),
        content: "# Hello from README".into(),
      })
      .resource(TestResource {
        uri: "file://config".into(),
        name: "Config".into(),
        mime_type: "application/json".into(),
        content: r#"{"key": "value"}"#.into(),
      })
      .build(),
    default_instance_json(),
  )
  .await?;

  let (session_id, _) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // List resources
  let resources_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "id": 2, "method": "resources/list"}),
  )
  .await?;

  let resources = resources_result["result"]["resources"]
    .as_array()
    .expect("resources/list should return resources array");
  assert_eq!(2, resources.len(), "Should have 2 resources");
  let resource_uris: Vec<&str> = resources
    .iter()
    .map(|r| r["uri"].as_str().unwrap())
    .collect();
  assert!(resource_uris.contains(&"file://readme"));
  assert!(resource_uris.contains(&"file://config"));

  // Read resource
  let read_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "resources/read",
      "params": {"uri": "file://readme"}
    }),
  )
  .await?;

  let contents = read_result["result"]["contents"]
    .as_array()
    .expect("resources/read should return contents array");
  assert!(
    !contents.is_empty(),
    "Should have at least one content item"
  );
  assert_eq!(
    "# Hello from README",
    contents[0]["text"].as_str().unwrap(),
    "Resource content should match"
  );

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Prompts
// =============================================================================

/// Proxy should forward prompt list and get requests to the upstream server.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_prompts() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder()
      .prompt(TestPrompt {
        name: "greeting".into(),
        description: "Greets a user".into(),
        arguments: vec![TestPromptArg {
          name: "name".into(),
          required: true,
        }],
        template: "Hello {name}, welcome!".into(),
      })
      .prompt(TestPrompt {
        name: "farewell".into(),
        description: "Says goodbye".into(),
        arguments: vec![],
        template: "Goodbye!".into(),
      })
      .build(),
    default_instance_json(),
  )
  .await?;

  let (session_id, _) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // List prompts
  let prompts_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "id": 2, "method": "prompts/list"}),
  )
  .await?;

  let prompts = prompts_result["result"]["prompts"]
    .as_array()
    .expect("prompts/list should return prompts array");
  assert_eq!(2, prompts.len(), "Should have 2 prompts");
  let prompt_names: Vec<&str> = prompts
    .iter()
    .map(|p| p["name"].as_str().unwrap())
    .collect();
  assert!(prompt_names.contains(&"greeting"));
  assert!(prompt_names.contains(&"farewell"));

  // Get prompt with arguments
  let get_result = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "prompts/get",
      "params": {
        "name": "greeting",
        "arguments": {
          "name": "World"
        }
      }
    }),
  )
  .await?;

  let messages = get_result["result"]["messages"]
    .as_array()
    .expect("prompts/get should return messages array");
  assert!(!messages.is_empty(), "Should have at least one message");

  // The template should be rendered with the argument
  let text = messages[0]["content"]["text"]
    .as_str()
    .expect("Message content should have text");
  assert_eq!(
    "Hello World, welcome!", text,
    "Prompt template should be rendered with arguments"
  );

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Disabled instance
// =============================================================================

/// Proxy should reject requests when the MCP instance is disabled.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_disabled_instance() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder().tool(echo_tool()).build(),
    default_instance_json(),
  )
  .await?;

  // Disable the instance via PUT
  let resp = setup
    .client
    .put(format!(
      "{}/bodhi/v1/mcps/{}",
      setup.base_url(),
      setup.mcp_id
    ))
    .header("Cookie", &setup.cookie)
    .json(&json!({
      "name": "Test MCP Instance",
      "slug": "test-proxy",
      "enabled": false
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "Should be able to disable MCP instance"
  );
  let updated: Value = resp.json().await?;
  assert_eq!(false, updated["enabled"].as_bool().unwrap());

  // Try to initialize - should fail
  let resp = mcp_initialize_raw(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  // The proxy may return a direct error or SSE with error
  let status = resp.status();
  let body_text = resp.text().await?;

  if status == StatusCode::OK {
    // SSE response: parse for JSON-RPC error
    let messages = parse_sse_jsonrpc(&body_text);
    assert!(!messages.is_empty(), "Should have at least one SSE message");
    assert!(
      messages[0]["error"].is_object(),
      "Initialize on disabled instance should return JSON-RPC error, got: {}",
      messages[0]
    );
  } else {
    // Direct HTTP error is also acceptable
    assert!(
      status.is_client_error() || status.is_server_error(),
      "Initialize on disabled instance should fail, got status: {}, body: {}",
      status,
      body_text
    );
  }

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Upstream down
// =============================================================================

/// Proxy should return an error when the upstream MCP server is unreachable.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_upstream_down() -> anyhow::Result<()> {
  // Start upstream, get its URL, then shut it down
  let upstream = TestMcpServer::start(TestMcpServer::builder().tool(echo_tool()).build()).await?;
  let upstream_url = upstream.url.clone();
  upstream.shutdown().await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id =
    create_mcp_server(&client, &server.base_url, &admin_cookie, &upstream_url).await?;

  let resp = client
    .post(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "name": "Test MCP Instance",
      "slug": "test-proxy",
      "mcp_server_id": server_id,
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::CREATED, resp.status());
  let body: Value = resp.json().await?;
  let mcp_id = body["id"].as_str().unwrap().to_string();

  // Try to initialize - upstream is down, should fail during connection
  let resp = mcp_initialize_raw(&client, &server.base_url, &admin_cookie, &mcp_id).await?;

  let status = resp.status();
  let body_text = resp.text().await?;

  if status == StatusCode::OK {
    // SSE response: should contain a JSON-RPC error about connection failure
    let messages = parse_sse_jsonrpc(&body_text);
    assert!(!messages.is_empty(), "Should have at least one SSE message");
    assert!(
      messages[0]["error"].is_object(),
      "Initialize with downed upstream should return JSON-RPC error, got: {}",
      messages[0]
    );
  } else {
    // Direct HTTP error is also acceptable
    assert!(
      status.is_client_error() || status.is_server_error(),
      "Initialize with downed upstream should fail, got status: {}, body: {}",
      status,
      body_text
    );
  }

  server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Wrong auth headers
// =============================================================================

/// Proxy should forward upstream 401 when MCP instance has wrong credentials.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_wrong_auth_headers() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder()
      .tool(echo_tool())
      .require_auth("X-Api-Key", "correct-secret")
      .build(),
    json!({
      "name": "Test MCP Instance",
      "slug": "test-proxy",
      "enabled": true,
      "auth_type": "header",
      "credentials": [
        {
          "param_type": "header",
          "param_key": "X-Api-Key",
          "value": "wrong-secret"
        }
      ]
    }),
  )
  .await?;

  // Try to initialize - upstream should reject with 401
  let resp = mcp_initialize_raw(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  let status = resp.status();
  let body_text = resp.text().await?;

  // The proxy forwards the upstream 401 status or wraps it in an error
  assert!(
    status == StatusCode::UNAUTHORIZED || status.is_client_error() || status.is_server_error() || {
      // SSE with JSON-RPC error is also acceptable
      let messages = parse_sse_jsonrpc(&body_text);
      !messages.is_empty() && messages[0]["error"].is_object()
    },
    "Wrong auth should fail, got status: {}, body: {}",
    status,
    body_text
  );

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Missing auth headers
// =============================================================================

/// Proxy should forward upstream 401 when MCP instance has no auth configured
/// but upstream requires it.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_missing_auth_headers() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder()
      .tool(echo_tool())
      .require_auth("X-Api-Key", "correct-secret")
      .build(),
    // Public auth (no credentials) — upstream will reject
    default_instance_json(),
  )
  .await?;

  // Try to initialize - upstream should reject with 401
  let resp = mcp_initialize_raw(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  let status = resp.status();
  let body_text = resp.text().await?;

  // The proxy forwards the upstream 401 status or wraps it in an error
  assert!(
    status == StatusCode::UNAUTHORIZED || status.is_client_error() || status.is_server_error() || {
      // SSE with JSON-RPC error is also acceptable
      let messages = parse_sse_jsonrpc(&body_text);
      !messages.is_empty() && messages[0]["error"].is_object()
    },
    "Missing auth should fail, got status: {}, body: {}",
    status,
    body_text
  );

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: GET SSE stream
// =============================================================================

/// After initializing an MCP session, a GET request with Mcp-Session-Id and
/// Accept: text/event-stream should open an SSE stream (status 200).
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_get_sse_stream() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder().tool(echo_tool()).build(),
    default_instance_json(),
  )
  .await?;

  // Initialize session
  let (session_id, _) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  // Send initialized notification
  mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // Open GET SSE stream
  let resp = setup
    .client
    .get(format!(
      "{}/bodhi/v1/apps/mcps/{}/mcp",
      setup.base_url(),
      setup.mcp_id
    ))
    .header("Cookie", &setup.cookie)
    .header("Accept", "text/event-stream")
    .header("Mcp-Session-Id", &session_id)
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "GET SSE stream should return 200"
  );

  let content_type = resp
    .headers()
    .get("content-type")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
  assert!(
    content_type.contains("text/event-stream"),
    "Content-Type should include text/event-stream, got: {}",
    content_type
  );

  // Don't try to read the full stream (it's long-lived).
  // The connection opening successfully is the assertion.

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test: Concurrent sessions
// =============================================================================

/// Two clients initializing separate sessions to the same MCP should get
/// distinct session IDs and both should be able to call tools independently.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_concurrent_sessions() -> anyhow::Result<()> {
  let setup = setup_proxy_test(
    TestMcpServer::builder().tool(echo_tool()).build(),
    default_instance_json(),
  )
  .await?;

  // Client A: initialize session
  let (session_id_a, _) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  // Client B: initialize session
  let (session_id_b, _) = mcp_initialize(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
  )
  .await?;

  // Sessions should be distinct
  assert_ne!(
    session_id_a, session_id_b,
    "Two clients should get different session IDs"
  );

  // Send initialized notifications for both
  mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id_a,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  mcp_notify(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id_b,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // Client A: call echo tool
  let result_a = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id_a,
    json!({
      "jsonrpc": "2.0",
      "id": 2,
      "method": "tools/call",
      "params": {"name": "echo", "arguments": {"message": "from_a"}}
    }),
  )
  .await?;
  assert!(
    result_a["result"]["content"].is_array(),
    "Client A echo should succeed"
  );

  // Client B: call echo tool
  let result_b = mcp_request(
    &setup.client,
    setup.base_url(),
    &setup.cookie,
    &setup.mcp_id,
    &session_id_b,
    json!({
      "jsonrpc": "2.0",
      "id": 2,
      "method": "tools/call",
      "params": {"name": "echo", "arguments": {"message": "from_b"}}
    }),
  )
  .await?;
  assert!(
    result_b["result"]["content"].is_array(),
    "Client B echo should succeed"
  );

  // Verify upstream received both calls
  {
    let calls = setup.upstream.calls_received.lock().await;
    assert_eq!(2, calls.len(), "Upstream should have received 2 tool calls");
    let call_messages: Vec<&str> = calls
      .iter()
      .filter_map(|c| {
        c.arguments
          .as_ref()
          .and_then(|a| a.get("message"))
          .and_then(|v| v.as_str())
      })
      .collect();
    assert!(
      call_messages.contains(&"from_a"),
      "Upstream should have received call from client A"
    );
    assert!(
      call_messages.contains(&"from_b"),
      "Upstream should have received call from client B"
    );
  }

  setup.upstream.shutdown().await;
  setup.server.handle.shutdown().await?;
  Ok(())
}
