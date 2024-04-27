mod utils;
use anyhow::{Context, Result};
use async_openai::types::CreateChatCompletionResponse;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use utils::test_server;

use crate::utils::TestServerHandle;

#[rstest]
#[tokio::test]
pub async fn test_server_chat(
  #[future] test_server: Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    host,
    port,
    shutdown,
    join,
    bodhi_home,
  } = test_server.await.context("initializing server")?;
  let chat_endpoint = format!("http://{}:{}/v1/chat/completions", host, port);
  let response = reqwest::Client::new()
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&json!({
      "model": "tinyllama-15m-q8_0",
      "seed": 42,
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "What day comes after Monday?"
        }
      ]
    }))
    .send()
    .await
    .context("querying chat endpoint")?;
  assert_eq!(response.status(), StatusCode::OK);
  let response = response.json::<CreateChatCompletionResponse>().await?;
  assert_eq!(response.choices.len(), 1);
  assert_eq!(
    response
      .choices
      .first()
      .unwrap()
      .message
      .content
      .as_ref()
      .unwrap(),
    "The day that comes after Monday is Tuesday."
  );
  shutdown
    .send(())
    .map_err(|_| anyhow::anyhow!("error sending shutdown signal"))
    .context("sending shutdown signal to server")?;
  let result = join.await.context("waiting for server to stop")?;
  assert!(result.is_ok());
  drop(bodhi_home);
  Ok(())
}
