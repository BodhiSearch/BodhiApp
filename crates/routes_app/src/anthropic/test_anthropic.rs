use crate::test_utils::RequestAuthContextExt;
use crate::{
  anthropic_messages_create_handler, anthropic_models_get_handler, anthropic_models_list_handler,
};
use anyhow_trace::anyhow_trace;
use axum::{
  extract::Request,
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  inference::{InferenceError, LlmEndpoint, MockInferenceService},
  test_utils::{anthropic_model, openai_model, AppServiceStubBuilder, TEST_TENANT_ID, TEST_USER_ID},
  ApiAliasBuilder, ApiFormat, ApiModel, AuthContext, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn ok_response() -> Result<axum::response::Response, InferenceError> {
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "application/json")
      .body(axum::body::Body::from(
        r#"{"id":"msg-123","content":[{"type":"text","text":"Hi"}]}"#,
      ))
      .unwrap(),
  )
}

async fn seed_anthropic_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-3-5-sonnet-20241022")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  Ok(api_alias)
}

// ============================================================================
// Local error envelope (Anthropic native format)
// ============================================================================
//
// The opaque proxy only validates that `model` is present in the request body
// (needed for alias routing). All other body validation is delegated to
// upstream Anthropic. Local errors that BodhiApp originates (missing model,
// alias not found, wrong format alias, invalid path param) all surface in the
// Anthropic-native envelope: `{"type":"error","error":{"type":"...","message":"..."}}`.

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_missing_model() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .json(json!({"max_tokens": 100, "messages": [{"role": "user", "content": "hi"}]}))?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!(
    "invalid_request_error",
    body["error"]["type"].as_str().unwrap()
  );
  assert!(body["error"]["message"].as_str().unwrap().contains("model"));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_model_not_found() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .json(json!({
          "model": "nonexistent-claude",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!("not_found_error", body["error"]["type"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_rejects_non_anthropic_alias() -> anyhow::Result<()> {
  // Seed an openai-format alias and verify the Anthropic endpoint rejects it
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .json(json!({
          "model": "gpt-4o",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!(
    "invalid_request_error",
    body["error"]["type"].as_str().unwrap()
  );
  let message = body["error"]["message"].as_str().unwrap();
  assert!(
    message.contains("anthropic"),
    "Error should mention anthropic format: {}",
    message
  );
  Ok(())
}

// ============================================================================
// Success paths — verify LlmEndpoint dispatch and client header forwarding
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_forwards_to_anthropic_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::AnthropicMessages && alias.id == "anthropic-alias"
    })
    .times(1)
    .return_once(|_, _, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .json(json!({
          "model": "claude-3-5-sonnet-20241022",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_forwards_anthropic_beta_header() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, _alias, _key, _params, headers| {
      if *endpoint != LlmEndpoint::AnthropicMessages {
        return false;
      }
      let Some(hdrs) = headers else { return false };
      hdrs
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("anthropic-beta") && v == "test-beta-flag")
    })
    .times(1)
    .return_once(|_, _, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .header("anthropic-beta", "test-beta-flag")
        .json(json!({
          "model": "claude-3-5-sonnet-20241022",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_non_anthropic_header_not_forwarded() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|_endpoint, _req, _alias, _key, _params, headers| {
      // Only anthropic-* headers should be forwarded; x-custom-thing is not.
      match headers {
        None => true,
        Some(hdrs) => !hdrs
          .iter()
          .any(|(k, _)| k.eq_ignore_ascii_case("x-custom-thing")),
      }
    })
    .times(1)
    .return_once(|_, _, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .header("x-custom-thing", "ignored")
        .json(json!({
          "model": "claude-3-5-sonnet-20241022",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_client_anthropic_version_forwarded() -> anyhow::Result<()> {
  // When the client sends anthropic-version, it MUST appear in client_headers so
  // the service can use it instead of injecting the default (avoiding duplicates).
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|_endpoint, _req, _alias, _key, _params, headers| {
      let Some(hdrs) = headers else { return false };
      hdrs
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("anthropic-version") && v == "2023-06-01")
    })
    .times(1)
    .return_once(|_, _, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .header("anthropic-version", "2023-06-01")
        .json(json!({
          "model": "claude-3-5-sonnet-20241022",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// Models list — DB aggregation
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_list_empty_when_no_anthropic_aliases() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/anthropic/v1/models", get(anthropic_models_list_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(0, body["data"].as_array().unwrap().len());
  assert_eq!(false, body["has_more"].as_bool().unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_list_aggregates_from_all_anthropic_aliases() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Seed two Anthropic aliases with overlapping models to exercise dedup.
  let alias_a = ApiAliasBuilder::test_default()
    .id("anthropic-a")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![
      anthropic_model("claude-3-5-sonnet-20241022"),
      anthropic_model("claude-3-opus-20240229"),
    ])
    .build_with_time(db_service.now())
    .unwrap();
  let alias_b = ApiAliasBuilder::test_default()
    .id("anthropic-b")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![
      anthropic_model("claude-3-5-sonnet-20241022"), // duplicate
      anthropic_model("claude-3-haiku-20240307"),
    ])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_a, None)
    .await?;
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_b, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/anthropic/v1/models", get(anthropic_models_list_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let data = body["data"].as_array().unwrap();
  assert_eq!(3, data.len()); // 3 unique models
  let ids: Vec<&str> = data.iter().map(|m| m["id"].as_str().unwrap()).collect();
  assert!(ids.contains(&"claude-3-5-sonnet-20241022"));
  assert!(ids.contains(&"claude-3-opus-20240229"));
  assert!(ids.contains(&"claude-3-haiku-20240307"));
  // All entries carry type: "model"
  for item in data {
    assert_eq!("model", item["type"].as_str().unwrap());
  }
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_list_excludes_non_anthropic_aliases() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // One Anthropic alias and one OpenAI alias owned by the same user.
  let anthropic_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-3-5-sonnet-20241022")])
    .build_with_time(db_service.now())
    .unwrap();
  let openai_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &anthropic_alias, None)
    .await?;
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &openai_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/anthropic/v1/models", get(anthropic_models_list_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let ids: Vec<&str> = body["data"]
    .as_array()
    .unwrap()
    .iter()
    .map(|m| m["id"].as_str().unwrap())
    .collect();
  assert!(ids.contains(&"claude-3-5-sonnet-20241022"));
  assert!(!ids.contains(&"gpt-4o"));
  Ok(())
}

// ============================================================================
// Single model GET — served from local cache
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_get_by_path_param_returns_cached_model() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_alias(&mut builder).await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/models/{model_id}",
      get(anthropic_models_get_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models/claude-3-5-sonnet-20241022")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("claude-3-5-sonnet-20241022", body["id"].as_str().unwrap());
  assert_eq!("model", body["type"].as_str().unwrap());
  assert_eq!(
    "claude-3-5-sonnet-20241022",
    body["display_name"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_get_invalid_model_id_slash() -> anyhow::Result<()> {
  // Model ids with characters outside [a-zA-Z0-9._-] are rejected by
  // validate_model_id (path-parameter safety, not body validation).
  // We use a percent-encoded space + bang so the router still dispatches to
  // the handler instead of a 404.
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/models/{model_id}",
      get(anthropic_models_get_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models/bad%20id%21")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!(
    "invalid_request_error",
    body["error"]["type"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_get_valid_id_with_dots_and_hyphens() -> anyhow::Result<()> {
  // Model ids with dots (like claude-3.5-sonnet) must be accepted.
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-3.5-sonnet")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/models/{model_id}",
      get(anthropic_models_get_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models/claude-3.5-sonnet")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("claude-3.5-sonnet", body["id"].as_str().unwrap());
  Ok(())
}
