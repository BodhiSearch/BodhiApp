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
async fn test_live_chat_completions_non_streamed(
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

  // First, verify the model is available
  let models_endpoint = format!("http://{host}:{port}/v1/models");
  let models_response = client
    .get(&models_endpoint)
    .header("Cookie", session_cookie.to_string())
    .send()
    .await?;
  assert_eq!(StatusCode::OK, models_response.status());
  let models_json = models_response.json::<Value>().await?;
  let models = models_json["data"]
    .as_array()
    .expect("Expected 'data' to be an array");

  let qwen_model = models
    .iter()
    .find(|m| m["id"] == "qwen3:1.7b-instruct")
    .unwrap_or_else(|| {
      panic!(
        "Expected to find qwen3:1.7b-instruct model in /v1/models response. Actual response: {}",
        serde_json::to_string_pretty(&models_json).unwrap()
      )
    });
  assert_eq!("qwen3:1.7b-instruct", qwen_model["id"]);

  // Now test chat completions
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Sec-Fetch-Site", "same-origin")
    .header("Cookie", session_cookie.to_string())
    .json(&serde_json::json!({
      "model": "qwen3:1.7b-instruct",
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

  let content = response["choices"][0]["message"]["content"]
    .as_str()
    .unwrap();
  assert!(content.contains("Tuesday"));
  assert_eq!("qwen3:1.7b-instruct", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);
  Ok(())
}
