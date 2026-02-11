mod utils;

use crate::utils::{
  create_authenticated_session, create_session_cookie, get_oauth_tokens, live_server,
  tool_call::{get_weather_tool, parse_streaming_content, parse_streaming_tool_calls},
  TestServerHandle,
};
use async_openai::types::chat::{
  ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
  ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestDeveloperMessage,
  ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs,
  ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs, FunctionCall,
};
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use std::time::Duration;

/// Tests multi-turn streaming tool calling with Qwen3 model.
/// Verifies the complete flow: user query -> tool call -> tool response -> final answer (streamed).
#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_live_tool_calling_multi_turn_streamed(
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

  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();

  // Turn 1: Initial request with developer role for tool calling
  let mut messages: Vec<ChatCompletionRequestMessage> = vec![
    ChatCompletionRequestDeveloperMessage::from(
      "You are a model that can do tool calling with the following tools",
    )
    .into(),
    ChatCompletionRequestUserMessage::from("What's the temperature in London?").into(),
  ];

  // Build Turn 1 request using async_openai types
  let request = CreateChatCompletionRequestArgs::default()
    .model("ggml-org/Qwen3-1.7B-GGUF:Q8_0")
    .seed(42_i64)
    .stream(true)
    .tools(get_weather_tool())
    .messages(messages.clone())
    .build()
    .unwrap();

  // Serialize to JSON for sending
  let request_json = serde_json::to_value(&request).unwrap();

  // Send Turn 1 streaming request, expect tool call
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&request_json)
    .send()
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_text = response.text().await?;

  // Parse Turn 1 streaming response
  let (tool_calls, finish_reason) = parse_streaming_tool_calls(&response_text);

  assert_eq!(
    "tool_calls", finish_reason,
    "Turn 1: Expected finish_reason to be 'tool_calls'"
  );

  assert!(!tool_calls.is_empty(), "Expected at least one tool call");

  let first_tool_call = &tool_calls[0];
  let function_name = first_tool_call["function"]["name"]
    .as_str()
    .expect("Expected function name");
  assert_eq!(
    "get_current_temperature", function_name,
    "Expected tool call to be get_current_temperature"
  );

  let tool_call_id = first_tool_call["id"]
    .as_str()
    .expect("Expected tool call id");

  // Turn 2: Convert tool_calls from JSON to typed structs
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

  // Build assistant message with tool_calls
  let assistant_msg = ChatCompletionRequestAssistantMessageArgs::default()
    .tool_calls(typed_tool_calls)
    .build()
    .unwrap();

  // Build tool response message
  let tool_msg = ChatCompletionRequestToolMessageArgs::default()
    .content("{\"temperature\": 15, \"unit\": \"celsius\"}")
    .tool_call_id(tool_call_id.to_string())
    .build()
    .unwrap();

  messages.push(assistant_msg.into());
  messages.push(tool_msg.into());

  // Build Turn 2 request using async_openai types
  let request = CreateChatCompletionRequestArgs::default()
    .model("ggml-org/Qwen3-1.7B-GGUF:Q8_0")
    .seed(42_i64)
    .stream(true)
    .tools(get_weather_tool())
    .messages(messages)
    .build()
    .unwrap();

  // Serialize to JSON for sending
  let request_json = serde_json::to_value(&request).unwrap();

  // Send Turn 2 streaming request, expect final answer
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&request_json)
    .send()
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_text = response.text().await?;

  handle.shutdown().await?;

  // Parse Turn 2 streaming response
  let (content, finish_reason) = parse_streaming_content(&response_text);

  assert_eq!(
    "stop", finish_reason,
    "Turn 2: Expected finish_reason to be 'stop', got: {}",
    finish_reason
  );

  // The final response should mention temperature or weather-related content
  let content_lower = content.to_lowercase();
  assert!(
    content_lower.contains("15")
      || content_lower.contains("temperature")
      || content_lower.contains("celsius")
      || content_lower.contains("london"),
    "Expected final response to contain temperature information, got: {}",
    content
  );

  Ok(())
}
