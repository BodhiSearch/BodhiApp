mod utils;

use crate::utils::{
  create_authenticated_session, create_session_cookie, get_oauth_tokens, live_server,
  TestServerHandle,
};
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::{cmp::max, time::Duration};

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
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let response = response.text().await?;
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
  let actual = streams[0..max(streams.len() - 1, 0)]
    .iter()
    .map(|stream| {
      stream["choices"][0]["delta"]["content"]
        .as_str()
        .unwrap_or_default()
    })
    .collect::<Vec<_>>();

  // Check that the response contains "Tuesday" in some form
  let full_content = actual.join("");
  assert!(full_content.contains("Tuesday") || full_content.contains("Tues"));
  let expected: Value = serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#)?;
  let last = streams.last().unwrap()["choices"].clone();
  assert_eq!(expected, last);
  Ok(())
}
