mod utils;
use crate::utils::TestServerHandle;
use anyhow::{Context, Result};
use async_openai::types::CreateChatCompletionStreamResponse;
use mousse::Parser;
use rstest::rstest;
use serde_json::json;
use tokio_stream::StreamExt;
use utils::test_server;

#[rstest]
#[tokio::test]
pub async fn test_server_chat_stream(
  #[future] test_server: Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    host,
    port,
    shutdown,
    join,
  } = test_server.await.context("initializing server")?;
  let chat_endpoint = format!("http://{}:{}/v1/chat/completions", host, port);
  let mut response = reqwest::Client::new()
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&json!({
      "model": "tinyllama-15m-q8_0",
      "seed": 42,
      "stream": true,
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "List down all the days of the week."
        }
      ]
    }))
    .send()
    .await
    .context("querying chat endpoint")?
    .bytes_stream();
  let mut response_str = String::new();
  while let Some(item) = response.next().await {
    let bytes = item.context("error parsing bytes_stream")?;
    let text = std::str::from_utf8(&bytes)?;
    response_str.push_str(text);
  }
  shutdown
    .send(())
    .map_err(|_| anyhow::anyhow!("error sending shutdown signal"))
    .context("sending shutdown signal to server")?;
  let result = join.await.context("waiting for server to stop")?;
  assert!(result.is_ok());
  let mut events = Parser::new(&response_str);
  let mut responses = Vec::<CreateChatCompletionStreamResponse>::new();
  while let Some(event) = events.next_event() {
    if let Some(data) = event.data {
      let data = data.as_ref();
      let delta: CreateChatCompletionStreamResponse =
        serde_json::from_str(data).context(format!("error parsing {data}"))?;
      responses.push(delta);
    }
  }
  assert_eq!(66, responses.len());
  let acc = responses.into_iter().fold(String::new(), |mut str, val| {
    let binding = String::from("");
    let delta = val.choices[0].delta.content.as_ref().unwrap_or(&binding);
    str.push_str(delta);
    str
  });
  let expected = r#"Sure! Here are the 7 days of the week:

1. Monday
2. Tuesday
3. Wednesday
4. Thursday
5. Friday
6. Saturday
7. Sunday

I hope that helps! Let me know if you have any other questions."#;
  assert_eq!(expected, acc);
  Ok(())
}
