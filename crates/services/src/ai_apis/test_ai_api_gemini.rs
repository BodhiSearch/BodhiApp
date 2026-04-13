use super::{AiApiService, DefaultAiApiService};
use crate::models::{ApiAlias, ApiFormat, ApiModel};
use crate::test_utils::{fixed_dt, gemini_model};
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use mockito::{Matcher, Server};
use rstest::rstest;
use serde_json::json;

const GEMINI_MODELS_SAMPLE: &str = include_str!("test_data/gemini_models_upstream_sample.json");

fn make_gemini_alias(url: &str) -> ApiAlias {
  ApiAlias::new(
    "gemini-api",
    ApiFormat::Gemini,
    url,
    vec![ApiModel::Gemini(gemini_model("gemini-2.5-flash"))],
    None,
    false,
    fixed_dt(),
    None,
    None,
  )
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_gemini_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let expected_body = json!({
    "contents": [{"role": "user", "parts": [{"text": "Hello"}]}]
  });

  let _mock = server
    .mock("POST", "/models/gemini-2.5-flash:generateContent")
    .match_header("x-goog-api-key", "test-key")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &expected_body,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{"candidates": [{"content": {"role": "model", "parts": [{"text": "Tuesday"}]}, "finishReason": "STOP"}]}"#,
    )
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gemini-2.5-flash",
      "Hello",
      &ApiFormat::Gemini,
      None,
      None,
    )
    .await?;
  assert_eq!("Tuesday", result);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_gemini_passes_through_embedding_only() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("GET", "/models")
    .match_header("x-goog-api-key", "test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(GEMINI_MODELS_SAMPLE)
    .create_async()
    .await;

  let models = service
    .fetch_models(
      Some("test-key".to_string()),
      &url,
      &ApiFormat::Gemini,
      None,
      None,
    )
    .await?;

  assert_eq!(3, models.len());
  let model_ids: Vec<&str> = models.iter().map(|m| m.id()).collect();
  assert!(
    model_ids.contains(&"gemini-embedding-001"),
    "embedding-only model should be present"
  );
  assert!(model_ids.contains(&"gemini-2.5-flash"));
  assert!(model_ids.contains(&"gemini-2.5-pro"));

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_gemini_preserves_display_name() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("GET", "/models")
    .match_header("x-goog-api-key", "test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(GEMINI_MODELS_SAMPLE)
    .create_async()
    .await;

  let models = service
    .fetch_models(
      Some("test-key".to_string()),
      &url,
      &ApiFormat::Gemini,
      None,
      None,
    )
    .await?;

  let flash = models
    .iter()
    .find(|m| m.id() == "gemini-2.5-flash")
    .expect("gemini-2.5-flash should be present");
  match flash {
    ApiModel::Gemini(m) => {
      assert_eq!(Some("Gemini 2.5 Flash".to_string()), m.display_name);
    }
    other => panic!("expected ApiModel::Gemini, got {:?}", other),
  }

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_gemini_forwards_query_params() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let api_alias = make_gemini_alias(&url);

  let body = json!({
    "contents": [{"role": "user", "parts": [{"text": "Hello"}]}]
  });

  let _mock = server
    .mock("POST", "/models/gemini-2.5-flash:generateContent")
    .match_header("x-goog-api-key", "test-key")
    .match_query(Matcher::UrlEncoded("alt".into(), "sse".into()))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"candidates": []}"#)
    .create_async()
    .await;

  let response = service
    .forward_request_with_method(
      &Method::POST,
      "/models/gemini-2.5-flash:generateContent",
      &api_alias,
      Some("test-key".to_string()),
      Some(body),
      Some(vec![("alt".to_string(), "sse".to_string())]),
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_gemini_passes_through() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let api_alias = make_gemini_alias(&url);

  let body = json!({
    "contents": [{"role": "user", "parts": [{"text": "Hello"}]}]
  });

  let _mock = server
    .mock("POST", "/models/gemini-2.5-flash:generateContent")
    .match_header("x-goog-api-key", "test-key")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(&body)?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{"candidates": [{"content": {"role": "model", "parts": [{"text": "Hello back!"}]}, "finishReason": "STOP"}]}"#,
    )
    .create_async()
    .await;

  let response = service
    .forward_request_with_method(
      &Method::POST,
      "/models/gemini-2.5-flash:generateContent",
      &api_alias,
      Some("test-key".to_string()),
      Some(body),
      None,
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());

  Ok(())
}
