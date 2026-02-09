use crate::routes_oai::{oai_model_handler, oai_models_handler};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  Router,
};
use chrono::Utc;
use objs::{ApiAlias, ApiFormat};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use serde_json::{json, Value};
use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
use services::{test_utils::AppServiceStubBuilder, AppService};
use std::sync::Arc;
use tower::ServiceExt;

fn create_router(service: Arc<dyn services::AppService>) -> Router {
  let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), service);
  Router::new()
    .route("/v1/models", axum::routing::get(oai_models_handler))
    .route("/v1/models/{id}", axum::routing::get(oai_model_handler))
    .with_state(Arc::new(router_state))
}

#[fixture]
async fn app() -> Router {
  let service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .build()
    .expect("failed to build app service");
  create_router(Arc::new(service))
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_models_handler_list_all(#[future] app: Router) -> anyhow::Result<()> {
  let response = app
    .oneshot(Request::builder().uri("/v1/models").body(Body::empty())?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!("list", response["object"].as_str().unwrap());
  let data = response["data"].as_array().unwrap();
  assert_eq!(8, data.len());
  let model_ids: Vec<&str> = data
    .iter()
    .map(|m| m["id"].as_str().unwrap())
    .collect();
  // Verify all expected models are present (sorted alphabetically)
  let expected_ids = vec![
    "FakeFactory/fakemodel-gguf:Q4_0",
    "MyFactory/testalias-gguf:Q8_0",
    "TheBloke/Llama-2-7B-Chat-GGUF:Q8_0",
    "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF:Q2_K",
    "google/gemma-1.1-2b-it-GGUF:2b_it_v1p1",
    "llama3:instruct",
    "testalias-exists:instruct",
    "tinyllama:instruct",
  ];
  assert_eq!(expected_ids, model_ids);
  // Model aliases have created=0 (from filesystem), user aliases have non-zero timestamps
  for model in data {
    assert_eq!("model", model["object"].as_str().unwrap());
    assert_eq!("system", model["owned_by"].as_str().unwrap());
  }
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_model_handler_found(#[future] app: Router) -> anyhow::Result<()> {
  let response = app
    .oneshot(
      Request::builder()
        .uri("/v1/models/llama3:instruct")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!("llama3:instruct", response["id"].as_str().unwrap());
  assert_eq!("model", response["object"].as_str().unwrap());
  assert_eq!("system", response["owned_by"].as_str().unwrap());
  // User aliases now have non-zero created timestamp from DB
  assert!(response["created"].as_u64().unwrap() > 0);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_model_handler_not_found(#[future] app: Router) -> anyhow::Result<()> {
  let response = app
    .oneshot(
      Request::builder()
        .uri("/v1/models/non_existent_model")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_models_handler_api_alias_with_prefix() -> anyhow::Result<()> {
  let service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .build()?;
  let db_service = service.db_service();

  let api_alias = ApiAlias::new(
    "openai-gpt4",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    Some("openai/".to_string()),
    false,
    Utc::now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  let app = create_router(Arc::new(service));
  let response = app
    .oneshot(Request::builder().uri("/v1/models").body(Body::empty())?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  let data = response["data"]
    .as_array()
    .expect("expected data to be an array");

  let model_ids: Vec<&str> = data
    .iter()
    .map(|m| m["id"].as_str().expect("expected id to be a string"))
    .collect();
  assert!(model_ids.contains(&"openai/gpt-4"));
  assert!(model_ids.contains(&"openai/gpt-3.5-turbo"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_models_handler_api_alias_without_prefix() -> anyhow::Result<()> {
  let service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .build()?;
  let db_service = service.db_service();

  let api_alias = ApiAlias::new(
    "openai-gpt4",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    Utc::now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  let app = create_router(Arc::new(service));
  let response = app
    .oneshot(Request::builder().uri("/v1/models").body(Body::empty())?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  let data = response["data"]
    .as_array()
    .expect("expected data to be an array");

  let model_ids: Vec<&str> = data
    .iter()
    .map(|m| m["id"].as_str().expect("expected id to be a string"))
    .collect();
  assert!(model_ids.contains(&"gpt-4"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_model_handler_api_alias_with_prefix() -> anyhow::Result<()> {
  let service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .build()?;
  let db_service = service.db_service();

  let api_alias = ApiAlias::new(
    "openai-gpt4",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string()],
    Some("openai/".to_string()),
    false,
    Utc::now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  let app = create_router(Arc::new(service));
  let response = app
    .oneshot(
      Request::builder()
        .uri("/v1/models/openai%2Fgpt-4")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "id": "openai/gpt-4",
      "object": "model",
      "created": api_alias.created_at.timestamp() as u32,
      "owned_by": "https://api.openai.com/v1",
    }},
    response
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_oai_model_handler_api_alias_without_prefix() -> anyhow::Result<()> {
  let service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .build()?;
  let db_service = service.db_service();

  let api_alias = ApiAlias::new(
    "openai-gpt4",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    Utc::now(),
  );
  db_service
    .create_api_model_alias(&api_alias, Some("test-key".to_string()))
    .await?;

  let app = create_router(Arc::new(service));
  let response = app
    .oneshot(
      Request::builder()
        .uri("/v1/models/gpt-4")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "id": "gpt-4",
      "object": "model",
      "created": api_alias.created_at.timestamp() as u32,
      "owned_by": "https://api.openai.com/v1",
    }},
    response
  );

  Ok(())
}
