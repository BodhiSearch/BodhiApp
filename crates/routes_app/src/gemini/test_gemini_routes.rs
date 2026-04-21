use crate::test_utils::RequestAuthContextExt;
use crate::{gemini_action_handler, gemini_models_get, gemini_models_list, ENDPOINT_GEMINI_MODEL};
use anyhow_trace::anyhow_trace;
use axum::{body::Body, extract::Request, routing::get, Router};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use server_core::test_utils::ResponseTestExt;
use services::{
  inference::{InferenceError, LlmEndpoint, MockInferenceService},
  test_utils::{gemini_model, openai_model, AppServiceStubBuilder, TEST_TENANT_ID, TEST_USER_ID},
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
        r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"Hello"}]}}]}"#,
      ))
      .unwrap(),
  )
}

async fn seed_gemini_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  seed_gemini_alias_with_prefix(builder, None).await
}

async fn seed_gemini_alias_with_prefix(
  builder: &mut AppServiceStubBuilder,
  prefix: Option<String>,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let mut alias_builder = ApiAliasBuilder::test_default();
  let model = gemini_model("gemini-2.5-flash");
  alias_builder
    .id("gemini-alias")
    .api_format(ApiFormat::Gemini)
    .base_url("https://generativelanguage.googleapis.com/v1beta")
    .models(vec![ApiModel::Gemini(model)]);
  if let Some(p) = prefix {
    alias_builder.prefix(p);
  }
  let api_alias = alias_builder.build_with_time(db_service.now()).unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  Ok(api_alias)
}

// ============================================================================
// gemini_models_list
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_list_empty_when_no_gemini_aliases() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1beta/models", get(gemini_models_list))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(0, body["models"].as_array().unwrap().len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_list_returns_gemini_models_with_name_prefix() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1beta/models", get(gemini_models_list))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let models = body["models"].as_array().unwrap();
  assert_eq!(1, models.len());
  assert_eq!(
    "models/gemini-2.5-flash",
    models[0]["name"].as_str().unwrap()
  );
  assert!(models[0].get("baseModelId").is_none());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_list_applies_alias_prefix() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias_with_prefix(&mut builder, Some("google/".to_string())).await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1beta/models", get(gemini_models_list))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let models = body["models"].as_array().unwrap();
  assert_eq!(1, models.len());
  // alias prefix "google/" applied: name = "models/google/{id}"
  assert_eq!(
    "models/google/gemini-2.5-flash",
    models[0]["name"].as_str().unwrap()
  );
  assert!(models[0].get("baseModelId").is_none());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_list_deduplicates_across_aliases() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Two aliases both containing gemini-2.5-flash — should be deduplicated.
  let alias_a = ApiAliasBuilder::test_default()
    .id("gemini-a")
    .api_format(ApiFormat::Gemini)
    .base_url("https://generativelanguage.googleapis.com/v1beta")
    .models(vec![ApiModel::Gemini(gemini_model("gemini-2.5-flash"))])
    .build_with_time(db_service.now())
    .unwrap();
  let alias_b = ApiAliasBuilder::test_default()
    .id("gemini-b")
    .api_format(ApiFormat::Gemini)
    .base_url("https://generativelanguage.googleapis.com/v1beta")
    .models(vec![
      ApiModel::Gemini(gemini_model("gemini-2.5-flash")), // duplicate
      ApiModel::Gemini(gemini_model("gemini-1.5-pro")),
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
    .route("/v1beta/models", get(gemini_models_list))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let models = body["models"].as_array().unwrap();
  assert_eq!(2, models.len()); // deduped
  let names: Vec<&str> = models.iter().map(|m| m["name"].as_str().unwrap()).collect();
  assert!(names.contains(&"models/gemini-2.5-flash"));
  assert!(names.contains(&"models/gemini-1.5-pro"));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_list_excludes_non_gemini_aliases() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // OpenAI alias — must not appear in Gemini models list.
  let openai_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &openai_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1beta/models", get(gemini_models_list))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(0, body["models"].as_array().unwrap().len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_list_returns_prefixed_name() -> anyhow::Result<()> {
  // Stored model name is bare; the Gemini listing route prepends the alias prefix
  // to the returned `name` so SDK clients see `models/gmn/gemini-2.5-flash`.
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let m = gemini_model("gemini-2.5-flash");
  let api_alias = ApiAliasBuilder::test_default()
    .id("gmn-alias")
    .api_format(ApiFormat::Gemini)
    .base_url("https://generativelanguage.googleapis.com/v1beta")
    .models(vec![ApiModel::Gemini(m)])
    .prefix("gmn/".to_string())
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1beta/models", get(gemini_models_list))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let models = body["models"].as_array().unwrap();
  assert_eq!(1, models.len());
  assert_eq!(
    "models/gmn/gemini-2.5-flash",
    models[0]["name"].as_str().unwrap()
  );
  assert!(models[0].get("baseModelId").is_none());
  Ok(())
}

// ============================================================================
// gemini_models_get
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_get_found() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models/gemini-2.5-flash")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("models/gemini-2.5-flash", body["name"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_get_not_found() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1beta/models/nonexistent-model")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(404, body["error"]["code"].as_i64().unwrap());
  assert_eq!("NOT_FOUND", body["error"]["status"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_models_get_invalid_model_id() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  // percent-encoded space + bang avoids router 404; handler rejects with 400
  let response = app
    .oneshot(
      Request::get("/v1beta/models/bad%20id%21")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "INVALID_ARGUMENT",
    body["error"]["status"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// gemini_generate_content
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_generate_content_forwards_to_gemini_endpoint() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::GeminiGenerateContent("gemini-2.5-flash".to_string())
        && alias.id == "gemini-alias"
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:generateContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
async fn test_action_handler_returns_not_found_for_unknown_alias() -> anyhow::Result<()> {
  // Exercises the From<BodhiErrorResponse> for GeminiApiError path: AliasNotFound → 404 NOT_FOUND
  // with grpc status "NOT_FOUND" in the response body.
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/nonexistent-model:generateContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(404, body["error"]["code"].as_i64().unwrap());
  assert_eq!("NOT_FOUND", body["error"]["status"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_generate_content_rejects_non_gemini_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Seed an OpenAI alias under a name and try to use it via the Gemini endpoint.
  let openai_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &openai_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gpt-4o:generateContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "INVALID_ARGUMENT",
    body["error"]["status"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_generate_content_invalid_model_id_rejected() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  // percent-encoded space in model id — handler rejects with 400
  let response = app
    .oneshot(
      Request::post("/v1beta/models/bad%20id:generateContent")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"contents":[]}"#))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "INVALID_ARGUMENT",
    body["error"]["status"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_generate_content_unsupported_action_rejected() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:countTokens")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"contents":[]}"#))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "INVALID_ARGUMENT",
    body["error"]["status"].as_str().unwrap()
  );
  let msg = body["error"]["message"].as_str().unwrap();
  assert!(
    msg.contains("countTokens") || msg.contains("Unsupported"),
    "Expected unsupported action error, got: {}",
    msg
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_generate_content_strips_alias_prefix() -> anyhow::Result<()> {
  // Alias has prefix "google/"; request model is "google/gemini-2.5-flash"
  // => forwarded stripped model must be "gemini-2.5-flash"
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias_with_prefix(&mut builder, Some("google/".to_string())).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, _alias, _key, _params, _headers| {
      // Stripped model id must be "gemini-2.5-flash" (without "google/" prefix)
      *endpoint == LlmEndpoint::GeminiGenerateContent("gemini-2.5-flash".to_string())
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/google%2Fgemini-2.5-flash:generateContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
async fn test_action_handler_accepts_literal_slash_in_prefixed_alias() -> anyhow::Result<()> {
  // Regression: pi-ai's @google/genai SDK does NOT URL-encode `/` in the model
  // segment, so a prefixed alias hits `/v1beta/models/gem/gemini-flash-latest:…`.
  // The wildcard route `/v1beta/models/{*model_path}` must match this literal
  // multi-segment path; the previous `{id}` single-segment route 404'd.
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias_with_prefix(&mut builder, Some("gem/".to_string())).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, _alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::GeminiStreamGenerateContent("gemini-2.5-flash".to_string())
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  // Literal slash between prefix and bare model id (no %2F encoding).
  let response = app
    .oneshot(
      Request::post("/v1beta/models/gem/gemini-2.5-flash:streamGenerateContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
async fn test_action_handler_forwards_alt_sse_query_param() -> anyhow::Result<()> {
  // Regression: the @google/genai SDK appends `?alt=sse` to request SSE-formatted
  // streaming responses. Without forwarding this query param to upstream Gemini,
  // we get a JSON array (`[{...},{...}]`) instead of SSE chunks
  // (`data: {...}\r\n\r\n`), and the SDK fails with "incomplete json segment".
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|_endpoint, _req, _alias, _key, params, _headers| {
      params
        .as_ref()
        .is_some_and(|p| p.iter().any(|(k, v)| k == "alt" && v == "sse"))
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
async fn test_action_handler_forwards_x_goog_headers() -> anyhow::Result<()> {
  // Gemini SDK sends `x-goog-api-client` and `x-goog-request-params` as telemetry
  // headers; these must reach upstream. Non-`x-goog-*` headers must be dropped.
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|_endpoint, _req, _alias, _key, _params, headers| {
      let Some(h) = headers.as_ref() else {
        return false;
      };
      let has_api_client = h
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("x-goog-api-client") && v == "genai-js/1.0");
      let has_request_params = h
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("x-goog-request-params") && v == "model=gemini");
      let no_content_type = !h
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
      has_api_client && has_request_params && no_content_type
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:generateContent")
        .header("content-type", "application/json")
        .header("x-goog-api-client", "genai-js/1.0")
        .header("x-goog-request-params", "model=gemini")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
// stream and embed actions via single action handler
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_stream_generate_content_forwards_correct_endpoint() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::GeminiStreamGenerateContent("gemini-2.5-flash".to_string())
        && alias.id == "gemini-alias"
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:streamGenerateContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
async fn test_embed_content_forwards_correct_endpoint() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::GeminiEmbedContent("gemini-2.5-flash".to_string())
        && alias.id == "gemini-alias"
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:embedContent")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"content":{"parts":[{"text":"hello world"}]}}"#,
        ))?
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
async fn test_batch_embed_contents_forwards_correct_endpoint() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::GeminiBatchEmbedContents("gemini-2.5-flash".to_string())
        && alias.id == "gemini-alias"
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:batchEmbedContents")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"requests":[{"model":"models/gemini-2.5-flash","content":{"parts":[{"text":"hello world"}]}}]}"#,
        ))?
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
// Axum path routing: verify GET/POST disambiguation
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_axum_get_and_post_routes_do_not_conflict() -> anyhow::Result<()> {
  // Regression test: GET /v1beta/models/{model_id} must not intercept
  //                  POST /v1beta/models/{model_action}
  // Axum's method-based routing correctly separates GET from POST.
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .times(1)
    .return_once(|_, _, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  // Single path pattern for both GET (model lookup) and POST (action dispatch)
  let app = Router::new()
    .route(
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  // POST to action URL should NOT go to the GET handler
  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:generateContent")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"contents":[]}"#))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  // Should reach gemini_action_handler, not gemini_models_get
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_gemini_action_forwards_alt_sse_query() -> anyhow::Result<()> {
  // Verify ?alt=sse reaches upstream via forwarded_params.
  let mut builder = AppServiceStubBuilder::default();
  seed_gemini_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|_endpoint, _req, _alias, _key, params, _headers| {
      params
        .as_ref()
        .is_some_and(|p| p.iter().any(|(k, v)| k == "alt" && v == "sse"))
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
      ENDPOINT_GEMINI_MODEL,
      get(gemini_models_get).post(gemini_action_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1beta/models/gemini-2.5-flash:generateContent?alt=sse")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"contents":[{"role":"user","parts":[{"text":"Hello"}]}]}"#,
        ))?
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
