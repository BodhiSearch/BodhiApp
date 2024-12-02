mod utils;

use crate::utils::{live_server, TestServerHandle};
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::time::Duration;

#[rstest::rstest]
#[awt]
#[tokio::test]
#[timeout(Duration::from_secs(5 * 60))]
#[serial_test::serial(live)]
async fn test_live_chat_completions_stream(
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
      "stream": true,
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
    .await?
    .text()
    .await?;
  let streams = response
    .lines()
    .filter_map(|line| {
      if line.is_empty() || line == "data: [DONE]" {
        None
      } else if line.starts_with("data: ") {
        let value: Value = serde_json::from_str(line.strip_prefix("data: ").unwrap()).unwrap();
        Some(value)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();
  handle.shutdown().await?;
  let expected = if cfg!(target_os = "macos") {
    [" ", " T", "ues", "day"].as_slice()
  } else {
    [" ", " T", "ues", "day", "."].as_slice()
  };
  let actual = streams[0..streams.len() - 1]
    .iter()
    .map(|stream| stream["choices"][0]["delta"]["content"].as_str().unwrap())
    .collect::<Vec<_>>();
  assert_eq!(expected, actual);
  let expected: Value = serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#)?;
  let last = streams.last().unwrap()["choices"].clone();
  assert_eq!(expected, last);
  Ok(())
}
