mod utils;

use crate::utils::{
  create_authenticated_session, create_session_cookie, get_oauth_tokens, live_server,
  TestServerHandle,
};
use async_openai::types::chat::{
  ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
  ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestDeveloperMessage,
  ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs,
  ChatCompletionRequestUserMessage, FunctionCall,
};
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use serde_json::{json, Value};
use std::time::Duration;

/// Tests end-to-end agentic chat with Exa web search toolset.
/// This test validates the complete flow:
/// 1. Enable Exa toolset at app level (admin operation)
/// 2. Configure Exa toolset at user level with API key
/// 3. User query triggers tool call to Exa search
/// 4. Backend executes actual Exa API call
/// 5. LLM receives tool result and generates final response
#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(10 * 60))]
#[serial_test::serial(live)]
async fn test_live_agentic_chat_with_exa_toolset(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    temp_cache_dir: _temp_cache_dir,
    host,
    port,
    handle,
    app_service,
  } = live_server?;

  // Get OAuth tokens using client credentials flow
  let (access_token, refresh_token) = get_oauth_tokens(app_service.as_ref()).await?;

  // Create authenticated session
  let session_id =
    create_authenticated_session(&app_service, &access_token, &refresh_token).await?;
  let session_cookie = create_session_cookie(&session_id);

  let client = reqwest::Client::new();
  let base_url = format!("http://{host}:{port}");

  // Step 1: Enable Exa toolset at app level (admin operation)
  println!("Step 1: Enabling Exa toolset at app level...");
  let enable_response = client
    .put(format!(
      "{}/bodhi/v1/toolset_types/builtin-exa-web-search/app-config",
      base_url
    ))
    .header("Cookie", session_cookie.to_string())
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    enable_response.status(),
    "Failed to enable Exa toolset at app level"
  );

  // Step 2: Configure Exa toolset at user level with API key
  println!("Step 2: Configuring Exa toolset with user API key...");
  let exa_api_key = std::env::var("INTEG_TEST_EXA_API_KEY")
    .expect("INTEG_TEST_EXA_API_KEY environment variable must be set for this test");

  let config_response = client
    .post(format!("{}/bodhi/v1/toolsets", base_url))
    .header("Cookie", session_cookie.to_string())
    .json(&json!({
      "toolset_type": "builtin-exa-web-search",
      "name": "builtin-exa-web-search",
      "description": "Exa web search toolset",
      "enabled": true,
      "api_key": exa_api_key
    }))
    .send()
    .await?;

  assert_eq!(
    StatusCode::CREATED,
    config_response.status(),
    "Failed to configure Exa toolset"
  );

  // Step 3: Get available toolsets and verify Exa is enabled with all tools
  println!("Step 3: Fetching available toolsets...");
  let toolsets_response = client
    .get(format!("{}/bodhi/v1/toolsets", base_url))
    .header("Cookie", session_cookie.to_string())
    .send()
    .await?;

  assert_eq!(StatusCode::OK, toolsets_response.status());
  let toolsets_json = toolsets_response.json::<Value>().await?;

  let exa_toolset = toolsets_json["toolsets"]
    .as_array()
    .and_then(|toolsets| {
      toolsets
        .iter()
        .find(|t| t["toolset_type"] == "builtin-exa-web-search")
    })
    .expect("Exa toolset not found in available toolsets");

  assert_eq!(true, exa_toolset["app_enabled"]);
  assert_eq!(true, exa_toolset["enabled"]);
  assert_eq!(true, exa_toolset["has_api_key"]);
  let toolset_uuid = exa_toolset["id"].as_str().expect("Expected toolset UUID");

  let tools = exa_toolset["tools"]
    .as_array()
    .expect("Expected tools array in Exa toolset");
  assert_eq!(4, tools.len(), "Expected 4 Exa tools");

  // Build tools array with qualified names for chat completion
  let chat_tools: Vec<Value> = tools
    .iter()
    .map(|tool| {
      json!({
        "type": "function",
        "function": {
          "name": format!("toolset__builtin-exa-web-search__{}", tool["function"]["name"].as_str().unwrap()),
          "description": tool["function"]["description"],
          "parameters": tool["function"]["parameters"]
        }
      })
    })
    .collect();

  // Step 4: Send chat completion request that should trigger Exa search
  println!("Step 4: Sending chat completion request with Exa tools...");
  let messages: Vec<ChatCompletionRequestMessage> = vec![
    ChatCompletionRequestDeveloperMessage::from(
      "You are an AI assistant with access to web search tools. Use the search tool to find current information when needed.",
    )
    .into(),
    ChatCompletionRequestUserMessage::from("What is the latest news about AI from San Francisco?").into(),
  ];

  let request = json!({
    "model": "qwen3:1.7b-instruct",
    "seed": 42,
    "stream": false,
    "tools": chat_tools,
    "messages": messages
  });

  let chat_endpoint = format!("{}/v1/chat/completions", base_url);
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&request)
    .send()
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<Value>().await?;

  // Step 5: Verify model made a tool call
  println!("Step 5: Verifying model made a tool call...");
  let choice = &response_json["choices"][0];
  let finish_reason = choice["finish_reason"]
    .as_str()
    .expect("Expected finish_reason");

  assert_eq!(
    "tool_calls", finish_reason,
    "Expected model to make a tool call, got: {}",
    finish_reason
  );

  let tool_calls = choice["message"]["tool_calls"]
    .as_array()
    .expect("Expected tool_calls array");
  assert!(!tool_calls.is_empty(), "Expected at least one tool call");

  let first_tool_call = &tool_calls[0];
  let function_name = first_tool_call["function"]["name"]
    .as_str()
    .expect("Expected function name");

  assert!(
    function_name.starts_with("toolset__builtin-exa-web-search__"),
    "Expected qualified tool name, got: {}",
    function_name
  );

  // Parse the method name from the qualified function name
  let method = function_name
    .strip_prefix("toolset__builtin-exa-web-search__")
    .expect("Expected tool name with prefix");

  let tool_call_id = first_tool_call["id"]
    .as_str()
    .expect("Expected tool call id");

  let arguments = first_tool_call["function"]["arguments"]
    .as_str()
    .expect("Expected arguments string");

  println!(
    "Model called tool: {} with arguments: {}",
    function_name, arguments
  );

  // Step 6: Execute the actual tool call via backend
  println!("Step 6: Executing tool call via backend...");
  let execute_response = client
    .post(format!(
      "{}/bodhi/v1/toolsets/{}/execute/{}",
      base_url, toolset_uuid, method
    ))
    .header("Cookie", session_cookie.to_string())
    .json(&json!({
      "tool_call_id": tool_call_id,
      "params": serde_json::from_str::<Value>(arguments)?
    }))
    .send()
    .await?;

  assert_eq!(
    StatusCode::OK,
    execute_response.status(),
    "Failed to execute tool call"
  );

  let execute_json = execute_response.json::<Value>().await?;
  assert_eq!(tool_call_id, execute_json["tool_call_id"]);
  assert!(
    execute_json["result"].is_object(),
    "Expected result object in tool execution response"
  );

  println!("Tool execution result: {}", execute_json["result"]);

  // Step 7: Send tool result back to model for final response
  println!("Step 7: Sending tool result back to model...");
  let typed_tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_calls
    .iter()
    .map(|tc| {
      ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
        id: tc["id"].as_str().unwrap().to_string(),
        function: FunctionCall {
          name: tc["function"]["name"].as_str().unwrap().to_string(),
          arguments: tc["function"]["arguments"].as_str().unwrap().to_string(),
        },
      })
    })
    .collect();

  let assistant_msg = ChatCompletionRequestAssistantMessageArgs::default()
    .tool_calls(typed_tool_calls)
    .build()?;

  let tool_result_content = serde_json::to_string(&execute_json["result"])?;
  let tool_msg = ChatCompletionRequestToolMessageArgs::default()
    .content(tool_result_content.clone())
    .tool_call_id(tool_call_id.to_string())
    .build()?;

  let mut final_messages = messages.clone();
  final_messages.push(assistant_msg.into());
  final_messages.push(tool_msg.into());

  let final_request = json!({
    "model": "qwen3:1.7b-instruct",
    "seed": 42,
    "stream": false,
    "tools": chat_tools,
    "messages": final_messages
  });

  let final_response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&final_request)
    .send()
    .await?;

  assert_eq!(StatusCode::OK, final_response.status());
  let final_json = final_response.json::<Value>().await?;

  // Step 8: Verify final response
  println!("Step 8: Verifying final response...");
  let final_choice = &final_json["choices"][0];
  let final_finish_reason = final_choice["finish_reason"]
    .as_str()
    .expect("Expected finish_reason in final response");

  assert_eq!(
    "stop", final_finish_reason,
    "Expected final response to have finish_reason 'stop', got: {}",
    final_finish_reason
  );

  let final_content = final_choice["message"]["content"]
    .as_str()
    .expect("Expected content in final response");

  println!("Final response: {}", final_content);

  // Verify the response contains relevant content
  // The model should incorporate information from the tool result
  assert!(
    !final_content.is_empty(),
    "Expected non-empty final response"
  );

  // The response should mention AI or San Francisco or news/information
  let content_lower = final_content.to_lowercase();
  assert!(
    content_lower.contains("ai")
      || content_lower.contains("san francisco")
      || content_lower.contains("news")
      || content_lower.contains("information")
      || content_lower.contains("latest"),
    "Expected final response to contain relevant content about AI news, got: {}",
    final_content
  );

  handle.shutdown().await?;

  println!("Test completed successfully!");
  Ok(())
}
