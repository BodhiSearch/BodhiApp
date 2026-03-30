mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use utils::{
  create_test_session_for_live_server, start_test_live_server,
  test_mcp_server::{TestMcpServer, TestPrompt, TestPromptArg, TestResource, TestTool},
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

/// Create an MCP instance via the REST API and return its ID.
async fn create_mcp_instance(
  client: &Client,
  base_url: &str,
  cookie: &str,
  server_id: &str,
) -> anyhow::Result<String> {
  let resp = client
    .post(format!("{}/bodhi/v1/mcps", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "name": "Test MCP Instance",
      "slug": "test-proxy",
      "mcp_server_id": server_id,
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Failed to create MCP instance"
  );
  let body: Value = resp.json().await?;
  Ok(body["id"].as_str().unwrap().to_string())
}

/// Create an MCP instance with a tools_filter via the REST API and return its ID.
async fn create_mcp_instance_with_filter(
  client: &Client,
  base_url: &str,
  cookie: &str,
  server_id: &str,
  slug: &str,
  tools_filter: Vec<&str>,
) -> anyhow::Result<String> {
  let resp = client
    .post(format!("{}/bodhi/v1/mcps", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "name": "Filtered MCP Instance",
      "slug": slug,
      "mcp_server_id": server_id,
      "enabled": true,
      "tools_filter": tools_filter
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Failed to create filtered MCP instance"
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
    .post(format!("{}/bodhi/v1/mcps/{}/mcp", base_url, mcp_id))
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
    .post(format!("{}/bodhi/v1/mcps/{}/mcp", base_url, mcp_id))
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
    .post(format!("{}/bodhi/v1/mcps/{}/mcp", base_url, mcp_id))
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
    .delete(format!("{}/bodhi/v1/mcps/{}/mcp", base_url, mcp_id))
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
// Test 6.1: Full lifecycle
// =============================================================================

/// Full proxy lifecycle: initialize -> notify -> list tools -> call tool -> delete session.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_full_lifecycle() -> anyhow::Result<()> {
  // 1. Start upstream TestMcpServer with echo + weather tools
  let upstream = TestMcpServer::start(
    TestMcpServer::builder()
      .tool(echo_tool())
      .tool(weather_tool())
      .build(),
  )
  .await?;

  // 2. Start Bodhi server
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  // 3. Create admin session, MCP server + instance
  let (admin_cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id =
    create_mcp_server(&client, &server.base_url, &admin_cookie, &upstream.url).await?;
  let mcp_id = create_mcp_instance(&client, &server.base_url, &admin_cookie, &server_id).await?;

  // 4. Initialize MCP session
  let (session_id, init_result) =
    mcp_initialize(&client, &server.base_url, &admin_cookie, &mcp_id).await?;

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

  // 5. Send notifications/initialized
  let notify_status = mcp_notify(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  // 6. List tools
  let tools_result = mcp_request(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  // 7. Call echo tool
  let call_result = mcp_request(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  // 8. Verify upstream received the call
  {
    let calls = upstream.calls_received.lock().await;
    assert_eq!(1, calls.len(), "Upstream should have received 1 tool call");
    assert_eq!("echo", calls[0].tool_name);
  }

  // 9. Delete session
  let delete_status = mcp_delete_session(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  // 10. Post with expired session should fail
  let resp = client
    .post(format!("{}/bodhi/v1/mcps/{}/mcp", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
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
  upstream.shutdown().await;
  server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test 6.2: Tools filter
// =============================================================================

/// Transparent proxy forwards all tools regardless of tools_filter setting.
/// tools_filter enforcement can be re-added later as JSON-level request/response inspection.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_tools_filter() -> anyhow::Result<()> {
  let upstream = TestMcpServer::start(
    TestMcpServer::builder()
      .tool(echo_tool())
      .tool(weather_tool())
      .build(),
  )
  .await?;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id =
    create_mcp_server(&client, &server.base_url, &admin_cookie, &upstream.url).await?;
  let mcp_id = create_mcp_instance_with_filter(
    &client,
    &server.base_url,
    &admin_cookie,
    &server_id,
    "filtered-proxy",
    vec!["echo"],
  )
  .await?;

  // Initialize session
  let (session_id, _) = mcp_initialize(&client, &server.base_url, &admin_cookie, &mcp_id).await?;

  // Send initialized notification
  mcp_notify(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // List tools - transparent proxy returns all upstream tools
  let tools_result = mcp_request(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  upstream.shutdown().await;
  server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test 6.3: Resources
// =============================================================================

/// Proxy should forward resource list and read requests to the upstream server.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_resources() -> anyhow::Result<()> {
  let upstream = TestMcpServer::start(
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
  )
  .await?;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id =
    create_mcp_server(&client, &server.base_url, &admin_cookie, &upstream.url).await?;
  let mcp_id = create_mcp_instance(&client, &server.base_url, &admin_cookie, &server_id).await?;

  let (session_id, _) = mcp_initialize(&client, &server.base_url, &admin_cookie, &mcp_id).await?;

  mcp_notify(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // List resources
  let resources_result = mcp_request(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  upstream.shutdown().await;
  server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test 6.4: Prompts
// =============================================================================

/// Proxy should forward prompt list and get requests to the upstream server.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_prompts() -> anyhow::Result<()> {
  let upstream = TestMcpServer::start(
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
  )
  .await?;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id =
    create_mcp_server(&client, &server.base_url, &admin_cookie, &upstream.url).await?;
  let mcp_id = create_mcp_instance(&client, &server.base_url, &admin_cookie, &server_id).await?;

  let (session_id, _) = mcp_initialize(&client, &server.base_url, &admin_cookie, &mcp_id).await?;

  mcp_notify(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
    &session_id,
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
  )
  .await?;

  // List prompts
  let prompts_result = mcp_request(
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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
    &client,
    &server.base_url,
    &admin_cookie,
    &mcp_id,
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

  upstream.shutdown().await;
  server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test 6.5: Disabled instance
// =============================================================================

/// Proxy should reject requests when the MCP instance is disabled.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_proxy_disabled_instance() -> anyhow::Result<()> {
  let upstream = TestMcpServer::start(TestMcpServer::builder().tool(echo_tool()).build()).await?;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  let server_id =
    create_mcp_server(&client, &server.base_url, &admin_cookie, &upstream.url).await?;
  let mcp_id = create_mcp_instance(&client, &server.base_url, &admin_cookie, &server_id).await?;

  // Disable the instance via PUT
  let resp = client
    .put(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
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
  let resp = client
    .post(format!("{}/bodhi/v1/mcps/{}/mcp", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .json(&json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2025-03-26",
        "capabilities": {},
        "clientInfo": {"name": "test-client", "version": "1.0"}
      }
    }))
    .send()
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

  upstream.shutdown().await;
  server.handle.shutdown().await?;
  Ok(())
}

// =============================================================================
// Test 6.6: Upstream down
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
  let mcp_id = create_mcp_instance(&client, &server.base_url, &admin_cookie, &server_id).await?;

  // Try to initialize - upstream is down, should fail during connection
  let resp = client
    .post(format!("{}/bodhi/v1/mcps/{}/mcp", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json, text/event-stream")
    .json(&json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2025-03-26",
        "capabilities": {},
        "clientInfo": {"name": "test-client", "version": "1.0"}
      }
    }))
    .send()
    .await?;

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
