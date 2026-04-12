use crate::embeddings_handler;
use crate::test_utils::RequestAuthContextExt;
use anyhow_trace::anyhow_trace;
use async_openai::types::embeddings::{
  CreateEmbeddingRequest, CreateEmbeddingResponse, EmbeddingInput,
};
use axum::{extract::Request, response::Response, routing::post, Router};
use mockall::predicate::{eq, function};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  inference::{InferenceError, LlmEndpoint, MockInferenceService},
  test_utils::{openai_model, AppServiceStubBuilder},
  Alias, AuthContext, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn embeddings_axum_response() -> Result<Response, InferenceError> {
  let response = json! {{
    "object": "list",
    "data": [
      {
        "object": "embedding",
        "index": 0,
        "embedding": vec![0.1, 0.2, 0.3, 0.4, 0.5]
      }
    ],
    "model": "testalias-exists:instruct",
    "usage": {
      "prompt_tokens": 8,
      "total_tokens": 8
    }
  }};
  let body = serde_json::to_string(&response).unwrap();
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "application/json")
      .body(axum::body::Body::from(body))
      .unwrap(),
  )
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_embeddings_handler_non_stream() -> anyhow::Result<()> {
  let request = CreateEmbeddingRequest {
    model: "testalias-exists:instruct".to_string(),
    input: EmbeddingInput::String("The quick brown fox jumps over the lazy dog".to_string()),
    encoding_format: None,
    user: None,
    dimensions: None,
  };
  let request_value = serde_json::to_value(&request)?;
  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_local()
    .with(
      eq(LlmEndpoint::Embeddings),
      eq(request_value),
      function(
        |alias: &Alias| matches!(alias, Alias::User(u) if u.alias == "testalias-exists:instruct"),
      ),
    )
    .times(1)
    .return_once(move |_, _, _| embeddings_axum_response());
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/embeddings", post(embeddings_handler))
    .with_state(router_state);
  let response = app
    .oneshot(
      Request::post("/v1/embeddings")
        .json(request)?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let result: CreateEmbeddingResponse = response.json().await?;
  assert_eq!("list", result.object);
  assert_eq!("testalias-exists:instruct", result.model);
  assert_eq!(1, result.data.len());
  assert_eq!(0, result.data[0].index);
  assert_eq!(vec![0.1, 0.2, 0.3, 0.4, 0.5], result.data[0].embedding);
  assert_eq!(8, result.usage.prompt_tokens);
  assert_eq!(8, result.usage.total_tokens);
  Ok(())
}

// ============================================================================
// Format rejection tests — openai_responses aliases must not be routed via
// embeddings endpoint
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_embeddings_rejects_responses_format_alias() -> anyhow::Result<()> {
  use services::test_utils::{TEST_TENANT_ID, TEST_USER_ID};
  use services::{ApiAliasBuilder, ApiFormat};

  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Seed an API alias with openai_responses format
  let api_alias = ApiAliasBuilder::test_default()
    .id("responses-alias")
    .api_format(ApiFormat::OpenAIResponses)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("text-embedding-ada-002")])
    .prefix("responses/".to_string())
    .build_with_time(db_service.now())
    .unwrap();

  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/embeddings", post(embeddings_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/embeddings")
        .json(json!({
          "model": "responses/text-embedding-ada-002",
          "input": "Hello"
        }))?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  let message = body["error"]["message"].as_str().unwrap();
  assert!(
    message.contains("openai_responses"),
    "Error message should mention the format: {}",
    message
  );
  Ok(())
}
