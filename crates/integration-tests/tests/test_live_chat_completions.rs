mod utils;
use crate::utils::{live_server, TestServerHandle};
use pretty_assertions::assert_eq;
use serde_json::Value;

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
      "model": "llama68m",
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
  assert_eq!(200, response.status());
  // assert_eq!("", response.text().await?);
  let response = response.json::<Value>().await?;
  handle.shutdown().await?;
  assert_eq!(
    r#"Monday was the day of the weekend, and then last Sunday it was Sunday. 

Saturday: Saturday is Sunday
The day of the weekend is Saturday,
Monday: Saturday is Sunday
Monday: Saturday is Sunday
Monday: Saturday is Sunday
Sunday: Saturday is Sunday
Wednesday: Saturday is Sunday<|im_end|>"#,
    response["choices"][0]["message"]["content"]
  );
  assert_eq!("llama68m", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);
  Ok(())
}

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
      "model": "llama68m",
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
  let expected = [
    "M",
    "ond",
    "ay",
    " was",
    " the",
    " day",
    " of",
    " the",
    " week",
    "end",
    ",",
    " and",
    " then",
    " last",
    " Sunday",
    " it",
    " was",
    " Sunday",
    ".",
    " ",
    "\n",
    "\n",
    "S",
    "aturday",
    ":",
    " Saturday",
    " is",
    " Sunday",
    "\n",
    "The",
    " day",
    " of",
    " the",
    " week",
    "end",
    " is",
    " Saturday",
    ",",
    "\n",
    "M",
    "ond",
    "ay",
    ":",
    " Saturday",
    " is",
    " Sunday",
    "\n",
    "M",
    "ond",
    "ay",
    ":",
    " Saturday",
    " is",
    " Sunday",
    "\n",
    "M",
    "ond",
    "ay",
    ":",
    " Saturday",
    " is",
    " Sunday",
    "\n",
    "S",
    "und",
    "ay",
    ":",
    " Saturday",
    " is",
    " Sunday",
    "\n",
    "W",
    "ed",
    "nes",
    "day",
    ":",
    " Saturday",
    " is",
    " Sunday",
    "<",
    "|",
    "im",
    "_",
    "end",
    "|",
  ]
  .as_slice();
  let actual = streams[0..streams.len() - 2]
    .iter()
    .map(|stream| stream["choices"][0]["delta"]["content"].as_str().unwrap())
    .collect::<Vec<_>>();
  assert_eq!(expected, actual);
  let expected: Value = serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#)?;
  let last = streams.last().unwrap()["choices"].clone();
  assert_eq!(expected, last);
  Ok(())
}
