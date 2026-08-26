use crate::{
  oai::test_live_utils::{get_weather_tool, parse_streaming_content, parse_streaming_tool_calls},
  test_utils::{build_live_test_router, create_authenticated_session, session_request_with_body},
};
use anyhow_trace::anyhow_trace;
use async_openai::types::chat::{
  ChatCompletionRequestDeveloperMessage, ChatCompletionRequestMessage,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
  CreateChatCompletionRequestArgs, CreateChatCompletionResponse, FinishReason,
};
use axum::body::Body;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::Value;
use server_core::test_utils::ResponseTestExt;
use tower::ServiceExt;

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_chat_completions_non_streamed() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let request = CreateChatCompletionRequestArgs::default()
    .model("ggml-org/Qwen3-1.7B-GGUF:Q8_0")
    .messages(vec![ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessageArgs::default()
        .content("What day comes after Monday?")
        .build()?,
    )])
    .build()?;
  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let result: CreateChatCompletionResponse = response.json().await?;

  assert!(
    !result.choices.is_empty(),
    "Expected at least one choice in response"
  );
  let choice = result.choices.first().unwrap();
  assert_eq!(
    Some(&FinishReason::Stop),
    choice.finish_reason.as_ref(),
    "Expected finish_reason to be Stop"
  );
  assert!(
    choice.message.content.is_some(),
    "Expected message content to be present"
  );
  assert!(
    !choice.message.content.as_ref().unwrap().is_empty(),
    "Expected non-empty message content"
  );
  assert_eq!(
    "ggml-org/Qwen3-1.7B-GGUF:Q8_0", result.model,
    "Expected model to be echoed back"
  );

  ctx.stop().await?;

  Ok(())
}

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_chat_completions_streamed() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let request = serde_json::json!({
    "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    "stream": true,
    "messages": [
      {
        "role": "user",
        "content": "What day comes after Monday?"
      }
    ]
  });
  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_text = response.text().await?;

  let (content, finish_reason) = parse_streaming_content(&response_text);

  assert!(
    content.contains("Tuesday"),
    "Expected content to contain 'Tuesday', got: {}",
    content
  );
  assert_eq!(
    "stop", finish_reason,
    "Expected finish_reason to be 'stop', got: {}",
    finish_reason
  );

  ctx.stop().await?;

  Ok(())
}

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_chat_completions_thinking_disabled() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let request = serde_json::json!({
    "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    "chat_template_kwargs": {
      "enable_thinking": false
    },
    "messages": [
      {
        "role": "user",
        "content": "What day comes after Monday?"
      }
    ]
  });
  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json: Value = response.json().await?;

  let message = &response_json["choices"][0]["message"];
  let content = message["content"].as_str().unwrap();
  assert!(
    content.contains("Tuesday"),
    "Expected content to contain 'Tuesday', got: {}",
    content
  );

  let reasoning_content = message.get("reasoning_content");
  assert!(
    reasoning_content.is_none() || reasoning_content.unwrap().is_null(),
    "Expected reasoning_content to be absent or null when thinking is disabled, got: {:?}",
    reasoning_content
  );

  assert_eq!(
    "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    response_json["model"].as_str().unwrap()
  );
  assert_eq!("stop", response_json["choices"][0]["finish_reason"]);

  ctx.stop().await?;

  Ok(())
}

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_chat_completions_reasoning_format_none() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let request = serde_json::json!({
    "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    "reasoning_format": "none",
    "messages": [
      {
        "role": "user",
        "content": "What day comes after Monday?"
      }
    ]
  });
  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json: Value = response.json().await?;

  let message = &response_json["choices"][0]["message"];
  let content = message["content"].as_str().unwrap();
  assert!(
    content.contains("Tuesday"),
    "Expected content to contain 'Tuesday', got: {}",
    content
  );

  let reasoning_content = message.get("reasoning_content");
  assert!(
    reasoning_content.is_none() || reasoning_content.unwrap().is_null(),
    "Expected reasoning_content to be absent or null with reasoning_format: none, got: {:?}",
    reasoning_content
  );

  assert_eq!(
    "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    response_json["model"].as_str().unwrap()
  );
  assert_eq!("stop", response_json["choices"][0]["finish_reason"]);

  ctx.stop().await?;

  Ok(())
}

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_chat_completions_thinking_enabled_default() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let request = serde_json::json!({
    "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    "messages": [
      {
        "role": "user",
        "content": "What day comes after Monday?"
      }
    ]
  });
  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json: Value = response.json().await?;

  let message = &response_json["choices"][0]["message"];
  let content = message["content"].as_str().unwrap();
  assert!(
    content.contains("Tuesday"),
    "Expected content to contain 'Tuesday', got: {}",
    content
  );

  let reasoning_content = message.get("reasoning_content");
  assert!(
    reasoning_content.is_some() && !reasoning_content.unwrap().is_null(),
    "Expected reasoning_content to be present with thinking enabled by default, got: {:?}",
    reasoning_content
  );

  let reasoning_text = reasoning_content.unwrap().as_str();
  assert!(
    reasoning_text.is_some() && !reasoning_text.unwrap().is_empty(),
    "Expected reasoning_content to be non-empty string, got: {:?}",
    reasoning_text
  );

  assert_eq!(
    "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
    response_json["model"].as_str().unwrap()
  );
  assert_eq!("stop", response_json["choices"][0]["finish_reason"]);

  ctx.stop().await?;

  Ok(())
}

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_tool_calling_non_streamed() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let messages: Vec<ChatCompletionRequestMessage> = vec![
    ChatCompletionRequestDeveloperMessage::from(
      "You are a model that can do tool calling with the following tools",
    )
    .into(),
    ChatCompletionRequestUserMessage::from("What's the temperature in London?").into(),
  ];

  let request = CreateChatCompletionRequestArgs::default()
    .model("ggml-org/Qwen3-1.7B-GGUF:Q8_0")
    .seed(42_i64)
    .stream(false)
    .tools(get_weather_tool())
    .messages(messages)
    .build()?;

  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json: Value = response.json().await?;

  let choice = &response_json["choices"][0];
  let finish_reason = choice["finish_reason"].as_str().unwrap();

  assert_eq!(
    "tool_calls", finish_reason,
    "Expected finish_reason to be 'tool_calls', got: {}",
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
  assert_eq!(
    "get_current_temperature", function_name,
    "Expected tool call to be get_current_temperature"
  );

  let arguments_str = first_tool_call["function"]["arguments"]
    .as_str()
    .expect("Expected arguments string");
  assert!(
    arguments_str.to_lowercase().contains("london"),
    "Expected arguments to contain 'London', got: {}",
    arguments_str
  );

  let tool_call_id = first_tool_call["id"].as_str();
  assert!(
    tool_call_id.is_some() && !tool_call_id.unwrap().is_empty(),
    "Expected tool call to have a non-empty ID"
  );

  assert_eq!("ggml-org/Qwen3-1.7B-GGUF:Q8_0", response_json["model"]);

  ctx.stop().await?;

  Ok(())
}

/// Requires the pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` model and its llama.cpp binary under `crates/llama_server_proc/bin/`.
#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
#[anyhow_trace]
#[serial_test::serial(live)]
#[timeout(std::time::Duration::from_secs(300))]
async fn test_live_tool_calling_streamed() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;

  let session_cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"])
      .await?;

  let messages: Vec<ChatCompletionRequestMessage> = vec![
    ChatCompletionRequestDeveloperMessage::from(
      "You are a model that can do tool calling with the following tools",
    )
    .into(),
    ChatCompletionRequestUserMessage::from("What's the temperature in London?").into(),
  ];

  let request = CreateChatCompletionRequestArgs::default()
    .model("ggml-org/Qwen3-1.7B-GGUF:Q8_0")
    .seed(42_i64)
    .stream(true)
    .tools(get_weather_tool())
    .messages(messages)
    .build()?;

  let request_body = serde_json::to_string(&request)?;

  // Triggers a real llama.cpp process
  let req = session_request_with_body(
    "POST",
    "/v1/chat/completions",
    &session_cookie,
    Body::from(request_body),
  );
  let response = router.oneshot(req).await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_text = response.text().await?;

  let (tool_calls, finish_reason) = parse_streaming_tool_calls(&response_text);

  assert_eq!(
    "tool_calls", finish_reason,
    "Expected finish_reason to be 'tool_calls', got: {}",
    finish_reason
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

  let arguments_str = first_tool_call["function"]["arguments"]
    .as_str()
    .expect("Expected arguments string");
  assert!(
    arguments_str.to_lowercase().contains("london"),
    "Expected arguments to contain 'London', got: {}",
    arguments_str
  );

  let tool_call_id = first_tool_call["id"].as_str();
  assert!(
    tool_call_id.is_some() && !tool_call_id.unwrap().is_empty(),
    "Expected tool call to have a non-empty ID"
  );

  ctx.stop().await?;

  Ok(())
}
