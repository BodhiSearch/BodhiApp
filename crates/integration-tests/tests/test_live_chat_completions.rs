mod utils;
use crate::utils::{live_server, TestServerHandle};
use serde_json::Value;

#[cfg(not(ci))]
#[rstest::rstest]
#[awt]
#[serial_test::serial(live)]
#[tokio::test]
async fn test_live_chat_completions(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle { host, port, handle } = live_server?;
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
      "model": "tinyllama:instruct",
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
    .await?
    .json::<Value>()
    .await?;
  handle.shutdown().await?;
  assert_eq!(
    "One word: Tuesday.",
    response["choices"][0]["message"]["content"]
  );
  let expected: Value = serde_json::from_str(
    r#"[{"finish_reason":"stop","index":0,"message":{"content":"One word: Tuesday.","role":"assistant"}}]"#,
  )?;
  assert_eq!(expected, response["choices"]);
  assert_eq!("tinyllama:instruct", response["model"]);
  Ok(())
}

#[cfg(not(ci))]
#[rstest::rstest]
#[awt]
#[serial_test::serial(live)]
#[tokio::test]
async fn test_live_chat_completions_stream(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle { host, port, handle } = live_server?;
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
      "model": "tinyllama:instruct",
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
      if line.is_empty() {
        None
      } else if line == "data: [DONE]" {
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
  for (index, content) in ["One", " word", ":", " T", "ues", "day", "."]
    .iter()
    .enumerate()
  {
    // TODO: have index 0, 1, 2 ... from llama.cpp
    let expected: Value = serde_json::from_str(&format!(
      r#"[{{"delta":{{"content":"{}"}},"finish_reason":null,"index":0}}]"#,
      content
    ))?;
    assert_eq!(expected, streams.get(index).unwrap()["choices"]);
  }
  let expected: Value = serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#)?;
  assert_eq!(expected, streams.get(7).unwrap()["choices"]);
  Ok(())
}
