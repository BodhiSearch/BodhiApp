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

  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .header("Cookie", session_cookie.to_string())
    .json(&serde_json::json!({
      "model": "phi4:mini-instruct",
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
  assert_eq!("phi4:mini-instruct", response["model"]);
  assert_eq!("stop", response["choices"][0]["finish_reason"]);
  Ok(())
}
