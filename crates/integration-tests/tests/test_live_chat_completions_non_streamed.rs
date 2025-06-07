mod utils;

use crate::utils::{live_server, TestServerHandle};
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
  } = live_server?;
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
      "model": "llama2:7b-chat",
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
  assert_eq!("llama2:7b-chat", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);
  Ok(())
}
