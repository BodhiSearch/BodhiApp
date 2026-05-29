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
use utils::{
  create_test_session_for_live_server, start_test_live_server, start_test_live_server_with_time,
};

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
  create_router_with_targets(
    client,
    base_url,
    cookie,
    alias,
    &[(target_alias, target_model)],
  )
  .await
}

/// Create a model-router whose enabled targets are `(alias_id, model)` pairs in
/// declared order.
async fn create_router_with_targets(
  client: &Client,
  base_url: &str,
  cookie: &str,
  alias: &str,
  targets: &[(&str, &str)],
) -> anyhow::Result<()> {
  create_router_with_strategy(
    client,
    base_url,
    cookie,
    alias,
    targets,
    json!({"strategy": "fallback"}),
  )
  .await
}

/// Like `create_router_with_targets` but with an explicit `strategy` JSON object,
/// so a test can exercise the persisted resilience knobs end-to-end.
async fn create_router_with_strategy(
  client: &Client,
  base_url: &str,
  cookie: &str,
  alias: &str,
  targets: &[(&str, &str)],
  strategy: Value,
) -> anyhow::Result<()> {
  let targets: Vec<Value> = targets
    .iter()
    .map(|(a, m)| json!({"alias": a, "model": m, "enabled": true}))
    .collect();
  let resp = client
    .post(format!("{}/bodhi/v1/models/router", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "alias": alias,
      "targets": targets,
      "strategy": strategy
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

/// Stub the provider's `GET /models` (create_openai_alias triggers a model fetch)
/// and a `POST /chat/completions` returning `status` with `body`. Returns the
/// chat mock so the caller can assert call counts.
async fn stub_chat_upstream(server: &mut MockServer, status: usize, body: &str) -> mockito::Mock {
  server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"object":"list","data":[{"id":"gpt-4","object":"model","created":1677610602,"owned_by":"openai"}]}"#)
    .create_async()
    .await;
  server
    .mock("POST", "/chat/completions")
    .with_status(status)
    .with_header("content-type", "application/json")
    .with_body(body)
    .create_async()
    .await
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

/// A retryable failure (503) on the primary target falls through to a working
/// secondary; the client gets the secondary's 200 and the headers report it
/// served after 2 attempts.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_model_router_falls_through_on_retryable() -> anyhow::Result<()> {
  let mut primary = MockServer::new_async().await;
  let primary_chat = stub_chat_upstream(&mut primary, 503, r#"{"error":"unavailable"}"#).await;
  let mut secondary = MockServer::new_async().await;
  let secondary_chat = stub_chat_upstream(
    &mut secondary,
    200,
    r#"{"id":"chatcmpl-2","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"served by secondary"},"finish_reason":"stop"}]}"#,
  )
  .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let primary_id = create_openai_alias(&client, &server.base_url, &cookie, &primary.url()).await?;
  let secondary_id =
    create_openai_alias(&client, &server.base_url, &cookie, &secondary.url()).await?;
  create_router_with_targets(
    &client,
    &server.base_url,
    &cookie,
    "my-stack",
    &[(&primary_id, "gpt-4"), (&secondary_id, "gpt-4")],
  )
  .await?;

  let resp = client
    .post(format!("{}/v1/chat/completions", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({"model": "my-stack", "messages": [{"role": "user", "content": "Hello"}]}))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!(
    secondary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
      .unwrap()
      .to_str()?
  );
  assert_eq!(
    "2",
    resp
      .headers()
      .get("x-bodhi-router-attempts")
      .unwrap()
      .to_str()?
  );
  let body: Value = resp.json().await?;
  assert_eq!(
    "served by secondary",
    body["choices"][0]["message"]["content"].as_str().unwrap()
  );

  primary_chat.assert_async().await;
  secondary_chat.assert_async().await;
  server.handle.shutdown().await?;
  Ok(())
}

/// Health memory across requests: once the primary fails, it is cooled and the
/// next request skips it (does NOT re-hit it) and is served by the secondary.
/// After the cooldown window elapses (advance the fake clock), the recovered
/// primary is tried again and serves — return-to-primary, no background probes.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_model_router_cools_then_recovers_to_primary() -> anyhow::Result<()> {
  // Primary: 503 for the first phase (cooldown), then 200 after "recovery".
  let mut primary = MockServer::new_async().await;
  // create_openai_alias triggers a model fetch on the primary.
  primary
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"object":"list","data":[{"id":"gpt-4","object":"model","created":1677610602,"owned_by":"openai"}]}"#)
    .create_async()
    .await;
  // Expect exactly one hit: request 1 fails here; request 2 must skip (cooled).
  let primary_down = primary
    .mock("POST", "/chat/completions")
    .with_status(503)
    .with_header("content-type", "application/json")
    .with_body(r#"{"error":"unavailable"}"#)
    .expect(1)
    .create_async()
    .await;

  let mut secondary = MockServer::new_async().await;
  let secondary_chat = stub_chat_upstream(
    &mut secondary,
    200,
    r#"{"id":"s","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"served by secondary"},"finish_reason":"stop"}]}"#,
  )
  .await;

  let (server, clock) = start_test_live_server_with_time().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let primary_id = create_openai_alias(&client, &server.base_url, &cookie, &primary.url()).await?;
  let secondary_id =
    create_openai_alias(&client, &server.base_url, &cookie, &secondary.url()).await?;
  create_router_with_targets(
    &client,
    &server.base_url,
    &cookie,
    "my-stack",
    &[(&primary_id, "gpt-4"), (&secondary_id, "gpt-4")],
  )
  .await?;

  async fn send(client: &reqwest::Client, base_url: &str, cookie: &str) -> reqwest::Response {
    client
      .post(format!("{}/v1/chat/completions", base_url))
      .header("Cookie", cookie)
      .json(&json!({"model": "my-stack", "messages": [{"role": "user", "content": "Hi"}]}))
      .send()
      .await
      .unwrap()
  }

  // Request 1: primary 503 (cooled), secondary serves.
  let resp = send(&client, &server.base_url, &cookie).await;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!(
    secondary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
      .unwrap()
      .to_str()?
  );

  // Request 2: primary is cooled → skipped (not re-hit); secondary serves with 1 attempt.
  let resp = send(&client, &server.base_url, &cookie).await;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!(
    secondary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
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
  // Primary's /chat/completions was hit exactly once (request 1), proving the skip.
  primary_down.assert_async().await;

  // Primary recovers upstream: drop the 503 mock, stub a 200.
  primary_down.remove_async().await;
  let primary_up = primary
    .mock("POST", "/chat/completions")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"p","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"served by primary"},"finish_reason":"stop"}]}"#)
    .create_async()
    .await;

  // Advance past the default 30s cooldown so the primary is eligible (half-open).
  clock.advance(chrono::Duration::seconds(31));

  // Request 3: primary tried first, succeeds → served by primary (return-to-primary).
  let resp = send(&client, &server.base_url, &cookie).await;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!(
    primary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
      .unwrap()
      .to_str()?
  );

  primary_up.assert_async().await;
  let _ = secondary_chat; // secondary served requests 1 & 2
  server.handle.shutdown().await?;
  Ok(())
}

/// The persisted `max_attempts` knob takes effect end-to-end: a router created via
/// the API with `max_attempts = 1` tries only the first target and returns its
/// (retryable) response verbatim, never reaching the secondary.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_model_router_max_attempts_config_takes_effect() -> anyhow::Result<()> {
  let mut primary = MockServer::new_async().await;
  let primary_chat = stub_chat_upstream(&mut primary, 503, r#"{"error":"unavailable"}"#).await;
  let mut secondary = MockServer::new_async().await;
  let secondary_chat = stub_chat_upstream(&mut secondary, 200, r#"{"id":"x"}"#).await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let primary_id = create_openai_alias(&client, &server.base_url, &cookie, &primary.url()).await?;
  let secondary_id =
    create_openai_alias(&client, &server.base_url, &cookie, &secondary.url()).await?;
  create_router_with_strategy(
    &client,
    &server.base_url,
    &cookie,
    "my-stack",
    &[(&primary_id, "gpt-4"), (&secondary_id, "gpt-4")],
    json!({"strategy": "fallback", "max_attempts": 1}),
  )
  .await?;

  let resp = client
    .post(format!("{}/v1/chat/completions", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({"model": "my-stack", "messages": [{"role": "user", "content": "Hi"}]}))
    .send()
    .await?;

  // Capped at 1 attempt: primary's 503 returned verbatim, secondary never tried.
  assert_eq!(StatusCode::SERVICE_UNAVAILABLE, resp.status());
  assert_eq!(
    primary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
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

  primary_chat.assert_async().await;
  secondary_chat.expect(0).assert_async().await;
  server.handle.shutdown().await?;
  Ok(())
}

/// A terminal failure (400) on the primary is returned verbatim and the
/// secondary is never tried.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_model_router_terminal_stops_immediately() -> anyhow::Result<()> {
  let mut primary = MockServer::new_async().await;
  let primary_chat = stub_chat_upstream(&mut primary, 400, r#"{"error":"bad request"}"#).await;
  let mut secondary = MockServer::new_async().await;
  let secondary_chat = stub_chat_upstream(&mut secondary, 200, r#"{"id":"x"}"#).await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let primary_id = create_openai_alias(&client, &server.base_url, &cookie, &primary.url()).await?;
  let secondary_id =
    create_openai_alias(&client, &server.base_url, &cookie, &secondary.url()).await?;
  create_router_with_targets(
    &client,
    &server.base_url,
    &cookie,
    "my-stack",
    &[(&primary_id, "gpt-4"), (&secondary_id, "gpt-4")],
  )
  .await?;

  let resp = client
    .post(format!("{}/v1/chat/completions", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({"model": "my-stack", "messages": [{"role": "user", "content": "Hello"}]}))
    .send()
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  assert_eq!(
    primary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
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

  primary_chat.assert_async().await;
  secondary_chat.expect(0).assert_async().await;
  server.handle.shutdown().await?;
  Ok(())
}

/// A streaming-capable success on the secondary streams through after the
/// primary fails with a retryable status (decision made before first byte).
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_model_router_streams_secondary_after_retryable() -> anyhow::Result<()> {
  let mut primary = MockServer::new_async().await;
  let primary_chat = stub_chat_upstream(&mut primary, 503, r#"{"error":"unavailable"}"#).await;
  let mut secondary = MockServer::new_async().await;
  // SSE stream body: two content deltas then [DONE].
  let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"hel\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n\
             data: [DONE]\n\n";
  secondary
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"object":"list","data":[{"id":"gpt-4","object":"model","created":1677610602,"owned_by":"openai"}]}"#)
    .create_async()
    .await;
  let secondary_chat = secondary
    .mock("POST", "/chat/completions")
    .with_status(200)
    .with_header("content-type", "text/event-stream")
    .with_body(sse)
    .create_async()
    .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let primary_id = create_openai_alias(&client, &server.base_url, &cookie, &primary.url()).await?;
  let secondary_id =
    create_openai_alias(&client, &server.base_url, &cookie, &secondary.url()).await?;
  create_router_with_targets(
    &client,
    &server.base_url,
    &cookie,
    "my-stack",
    &[(&primary_id, "gpt-4"), (&secondary_id, "gpt-4")],
  )
  .await?;

  let resp = client
    .post(format!("{}/v1/chat/completions", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({"model": "my-stack", "stream": true, "messages": [{"role": "user", "content": "Hi"}]}))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!(
    secondary_id,
    resp
      .headers()
      .get("x-bodhi-routed-alias")
      .unwrap()
      .to_str()?
  );
  let body = resp.text().await?;
  assert!(
    body.contains("hel"),
    "stream body missing first delta: {body}"
  );
  assert!(
    body.contains("lo"),
    "stream body missing second delta: {body}"
  );
  assert!(
    body.contains("[DONE]"),
    "stream body missing terminator: {body}"
  );

  primary_chat.assert_async().await;
  secondary_chat.assert_async().await;
  server.handle.shutdown().await?;
  Ok(())
}
