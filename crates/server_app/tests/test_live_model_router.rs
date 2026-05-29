//! End-to-end live test for the model-router (composite alias) pass-through routing.
//! Uses a mockito upstream as an OpenAI-compatible provider and verifies that a chat
//! request addressed to a model-router is forwarded to its first enabled target and
//! that the observability headers identify that target.

mod utils;

use anyhow_trace::anyhow_trace;
use mockito::Server as MockServer;
use pretty_assertions::assert_eq;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use utils::{create_test_session_for_live_server, start_test_live_server};

async fn create_openai_alias(
  client: &Client,
  base_url: &str,
  cookie: &str,
  upstream_url: &str,
) -> anyhow::Result<String> {
  let resp = client
    .post(format!("{}/bodhi/v1/models/api", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "api_format": "openai",
      "base_url": upstream_url,
      "api_key": {"action": "set", "value": "sk-test-key"},
      "models": ["gpt-4"],
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "failed to create OpenAI alias: {}",
    resp.text().await?
  );
  let body: Value = resp.json().await?;
  Ok(body["id"].as_str().unwrap().to_string())
}

async fn create_router(
  client: &Client,
  base_url: &str,
  cookie: &str,
  alias: &str,
  target_alias: &str,
  target_model: &str,
) -> anyhow::Result<()> {
  let resp = client
    .post(format!("{}/bodhi/v1/models/router", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "alias": alias,
      "targets": [{"alias": target_alias, "model": target_model, "enabled": true}],
      "strategy": {"strategy": "fallback"}
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "failed to create model-router: {}",
    resp.text().await?
  );
  Ok(())
}

/// A chat request to a model-router forwards to its first enabled target and returns
/// the upstream response plus observability headers identifying that target.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_model_router_pass_through_chat_completion() -> anyhow::Result<()> {
  let mut mock_server = MockServer::new_async().await;
  // create_openai_alias triggers a provider model fetch.
  mock_server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"object":"list","data":[{"id":"gpt-4","object":"model","created":1677610602,"owned_by":"openai"}]}"#)
    .create_async()
    .await;
  let chat_mock = mock_server
    .mock("POST", "/chat/completions")
    .match_header("authorization", "Bearer sk-test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{"id":"chatcmpl-1","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"hello from upstream"},"finish_reason":"stop"}]}"#,
    )
    .create_async()
    .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let alias_id =
    create_openai_alias(&client, &server.base_url, &cookie, &mock_server.url()).await?;
  create_router(
    &client,
    &server.base_url,
    &cookie,
    "my-stack",
    &alias_id,
    "gpt-4",
  )
  .await?;

  let resp = client
    .post(format!("{}/v1/chat/completions", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({
      "model": "my-stack",
      "messages": [{"role": "user", "content": "Hello"}]
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  // Observability headers identify the served target.
  assert_eq!(
    alias_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
      .unwrap()
      .to_str()?
  );
  assert_eq!(
    "gpt-4",
    resp
      .headers()
      .get("x-bodhi-routed-model")
      .unwrap()
      .to_str()?
  );
  assert_eq!(
    "fallback",
    resp
      .headers()
      .get("x-bodhi-router-strategy")
      .unwrap()
      .to_str()?
  );
  assert_eq!(
    "1",
    resp
      .headers()
      .get("x-bodhi-router-attempts")
      .unwrap()
      .to_str()?
  );
  let body: Value = resp.json().await?;
  assert_eq!(
    "hello from upstream",
    body["choices"][0]["message"]["content"].as_str().unwrap()
  );

  chat_mock.assert_async().await;
  server.handle.shutdown().await?;
  Ok(())
}
