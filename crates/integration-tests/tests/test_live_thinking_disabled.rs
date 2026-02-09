mod utils;

use crate::utils::{
  create_authenticated_session, create_session_cookie, get_oauth_tokens, live_server,
  TestServerHandle,
};
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::time::Duration;

#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_live_chat_completions_thinking_disabled(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    temp_cache_dir: _temp_cache_dir,
    host,
    port,
    handle,
    app_service,
  } = live_server?;

  let (access_token, refresh_token) = get_oauth_tokens(app_service.as_ref()).await?;
  let session_id =
    create_authenticated_session(&app_service, &access_token, &refresh_token).await?;
  let session_cookie = create_session_cookie(&session_id);

  let client = reqwest::Client::new();
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");

  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&serde_json::json!({
      "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
      "seed": 42,
      "chat_template_kwargs": {
        "enable_thinking": false
      },
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "Answer in one word. What day comes after Monday?"
        }
      ]
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  handle.shutdown().await?;

  // Verify response content
  let message = &response["choices"][0]["message"];
  let content = message["content"].as_str().unwrap();
  assert!(content.contains("Tuesday"));

  // Verify thinking is disabled: reasoning_content should be absent or null
  let reasoning_content = message.get("reasoning_content");
  assert!(
    reasoning_content.is_none() || reasoning_content.unwrap().is_null(),
    "Expected reasoning_content to be absent or null when thinking is disabled, got: {:?}",
    reasoning_content
  );

  assert_eq!("ggml-org/Qwen3-1.7B-GGUF:Q8_0", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);

  Ok(())
}

#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_live_chat_completions_reasoning_format_none(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    temp_cache_dir: _temp_cache_dir,
    host,
    port,
    handle,
    app_service,
  } = live_server?;

  let (access_token, refresh_token) = get_oauth_tokens(app_service.as_ref()).await?;
  let session_id =
    create_authenticated_session(&app_service, &access_token, &refresh_token).await?;
  let session_cookie = create_session_cookie(&session_id);

  let client = reqwest::Client::new();
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");

  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&serde_json::json!({
      "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
      "seed": 42,
      "reasoning_format": "none",
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "Answer in one word. What day comes after Monday?"
        }
      ]
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  handle.shutdown().await?;

  // Verify response content
  let message = &response["choices"][0]["message"];
  let content = message["content"].as_str().unwrap();
  assert!(content.contains("Tuesday"));

  // Verify reasoning_format: none leaves thoughts unparsed
  // reasoning_content should be absent or null (not extracted by parser)
  let reasoning_content = message.get("reasoning_content");
  assert!(
    reasoning_content.is_none() || reasoning_content.unwrap().is_null(),
    "Expected reasoning_content to be absent or null with reasoning_format: none, got: {:?}",
    reasoning_content
  );

  assert_eq!("ggml-org/Qwen3-1.7B-GGUF:Q8_0", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);

  Ok(())
}

#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_live_chat_completions_thinking_enabled_default(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    temp_cache_dir: _temp_cache_dir,
    host,
    port,
    handle,
    app_service,
  } = live_server?;

  let (access_token, refresh_token) = get_oauth_tokens(app_service.as_ref()).await?;
  let session_id =
    create_authenticated_session(&app_service, &access_token, &refresh_token).await?;
  let session_cookie = create_session_cookie(&session_id);

  let client = reqwest::Client::new();
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");

  // Request WITHOUT chat_template_kwargs.enable_thinking or reasoning_format
  // to verify default behavior produces reasoning_content
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&serde_json::json!({
      "model": "ggml-org/Qwen3-1.7B-GGUF:Q8_0",
      "seed": 42,
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "Answer in one word. What day comes after Monday?"
        }
      ]
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  handle.shutdown().await?;

  // Verify response content
  let message = &response["choices"][0]["message"];
  let content = message["content"].as_str().unwrap();
  assert!(content.contains("Tuesday"));

  // Verify thinking is enabled by default: reasoning_content should be present
  let reasoning_content = message.get("reasoning_content");
  assert!(
    reasoning_content.is_some() && !reasoning_content.unwrap().is_null(),
    "Expected reasoning_content to be present with thinking enabled by default, got: {:?}",
    reasoning_content
  );

  // Verify reasoning_content is non-empty string
  let reasoning_text = reasoning_content.unwrap().as_str();
  assert!(
    reasoning_text.is_some() && !reasoning_text.unwrap().is_empty(),
    "Expected reasoning_content to be non-empty string, got: {:?}",
    reasoning_text
  );

  assert_eq!("ggml-org/Qwen3-1.7B-GGUF:Q8_0", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);

  Ok(())
}
