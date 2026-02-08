use crate::routes_oai::{oai_model_handler, oai_models_handler};
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
    .unwrap();
  create_router(Arc::new(service))
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_oai_models_handler(#[future] app: Router) -> anyhow::Result<()> {
  let response = app
    .oneshot(Request::builder().uri("/v1/models").body(Body::empty())?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "object": "list",
      "data": [
        {
          "id": "FakeFactory/fakemodel-gguf:Q4_0",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "MyFactory/testalias-gguf:Q8_0",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "TheBloke/Llama-2-7B-Chat-GGUF:Q8_0",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF:Q2_K",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "google/gemma-1.1-2b-it-GGUF:2b_it_v1p1",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "llama3:instruct",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "testalias-exists:instruct",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
        {
          "id": "tinyllama:instruct",
          "object": "model",
          "created": 0,
          "owned_by": "system"
        },
      ]
    }},
    response
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_oai_model_handler(#[future] app: Router) -> anyhow::Result<()> {
  let response = app
    .oneshot(
      Request::builder()
        .uri("/v1/models/llama3:instruct")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "id": "llama3:instruct",
      "object": "model",
      "created": 0,
      "owned_by": "system",
    }},
    response
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
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

#[tokio::test]
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
  let data = response["data"].as_array().unwrap();

  let model_ids: Vec<&str> = data.iter().map(|m| m["id"].as_str().unwrap()).collect();
  assert!(model_ids.contains(&"openai/gpt-4"));
  assert!(model_ids.contains(&"openai/gpt-3.5-turbo"));

  Ok(())
}

#[tokio::test]
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
  let data = response["data"].as_array().unwrap();

  let model_ids: Vec<&str> = data.iter().map(|m| m["id"].as_str().unwrap()).collect();
  assert!(model_ids.contains(&"gpt-4"));

  Ok(())
}

#[tokio::test]
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

#[tokio::test]
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
