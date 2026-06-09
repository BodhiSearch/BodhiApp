use crate::{
  api_models_create, api_models_destroy, api_models_fetch_models, api_models_show, api_models_sync,
  api_models_test, api_models_update, ENDPOINT_MODELS_API,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::test_utils::{
  anthropic_model, test_db_service, test_llm_liberty_envelope, AppServiceStubBuilder, TestDbService,
};
use services::{
  ApiAliasResponse, ApiFormat, ApiModelRequest, AuthContext, LibertySource,
  LlmLibertyApiModelRequest, LlmLibertyEnvelope, LlmLibertyEnvelopeUpdate, MockAiApiClientFactory,
  ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route(ENDPOINT_MODELS_API, post(api_models_create))
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_API, "{id}"),
      get(api_models_show),
    )
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_API, "{id}"),
      put(api_models_update),
    )
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_API, "{id}"),
      delete(api_models_destroy),
    )
    .route(
      &format!("{}/test", ENDPOINT_MODELS_API),
      post(api_models_test),
    )
    .route(
      &format!("{}/fetch-models", ENDPOINT_MODELS_API),
      post(api_models_fetch_models),
    )
    .route(
      &format!("{}/{{id}}/sync-models", ENDPOINT_MODELS_API),
      post(api_models_sync),
    )
    .layer(axum::Extension(AuthContext::test_session(
      "test-user",
      "testuser",
      ResourceRole::PowerUser,
    )))
    .with_state(app_service)
}

fn valid_envelope() -> LlmLibertyEnvelope {
  let mut env = test_llm_liberty_envelope();
  env.access_token = "access-test".into();
  env.refresh_token = "refresh-test".into();
  env.expires_at = 9999999999;
  env.oauth.client_id = "client-id-public".into();
  env.api.base_url = "https://api.anthropic.com/v1".into();
  env.api.chat_url = "https://api.anthropic.com/v1/messages".into();
  env.api.models_url = Some("https://api.anthropic.com/v1/models".into());
  env
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn create_201_with_valid_envelope(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai
    .expect_for_liberty()
    .withf(|source| matches!(source, LibertySource::Envelope(_)))
    .returning(|_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_fetch_models()
        .returning(|| Ok(vec![anthropic_model("claude-haiku-4-5-20251001")]));
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Anthropic".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(valid_envelope()),
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(req)?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  let body = response.json::<ApiAliasResponse>().await?;
  assert_eq!("Liberty Anthropic", body.name);
  assert_eq!(ApiFormat::LlmLibertyOauth, body.api_format);
  let summary = body.llm_liberty.expect("llm_liberty summary");
  assert_eq!("anthropic", summary.provider);
  assert_eq!("1.0.0", summary.envelope_version);
  assert!(summary.has_refresh_token);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn create_400_when_envelope_version_unsupported(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mock_ai = MockAiApiClientFactory::new();
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let mut env = valid_envelope();
  env.version = "2.0.0".into();
  let req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Bad Version".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(env),
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(req)?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body: Value = response.json().await?;
  let message = body["error"]["message"].as_str().unwrap_or("");
  assert!(
    message.contains("2.0.0") || message.to_lowercase().contains("unsupported"),
    "expected version-related error message, got: {}",
    message
  );
  Ok(())
}

/// Each case: a mutator that clears one required leaf, plus the substring
/// that must appear in the BadRequest message.
#[rstest]
#[case::auth_in("auth.in", |env: &mut LlmLibertyEnvelope| env.auth.location = String::new())]
#[case::auth_key("auth.key", |env: &mut LlmLibertyEnvelope| env.auth.key = String::new())]
#[case::auth_scheme("auth.scheme", |env: &mut LlmLibertyEnvelope| env.auth.scheme = String::new())]
#[case::oauth_token_url("oauth.token_url", |env: &mut LlmLibertyEnvelope| env.oauth.token_url = String::new())]
#[case::oauth_client_id("oauth.client_id", |env: &mut LlmLibertyEnvelope| env.oauth.client_id = String::new())]
#[case::api_chat_url("api.chat_url", |env: &mut LlmLibertyEnvelope| env.api.chat_url = String::new())]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn create_400_when_required_envelope_field_missing(
  #[case] field_substr: &str,
  #[case] mutator: fn(&mut LlmLibertyEnvelope),
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mock_ai = MockAiApiClientFactory::new();
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let mut env = valid_envelope();
  mutator(&mut env);
  let req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Missing Field".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(env),
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(req)?)
    .await?;
  assert_eq!(
    StatusCode::BAD_REQUEST,
    response.status(),
    "field={}",
    field_substr
  );

  let body: Value = response.json().await?;
  let message = body["error"]["message"].as_str().unwrap_or("");
  assert!(
    message.contains(field_substr),
    "expected '{}' in message, got: {}",
    field_substr,
    message
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn create_400_when_envelope_provider_unsupported(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mock_ai = MockAiApiClientFactory::new();
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let mut env = valid_envelope();
  env.provider = "google-gemini".into();
  let req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Bad Provider".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(env),
    models: vec!["gemini-model".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(req)?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body: Value = response.json().await?;
  let message = body["error"]["message"].as_str().unwrap_or("");
  assert!(
    message.to_lowercase().contains("provider") || message.contains("google-gemini"),
    "expected provider-related error message, got: {}",
    message
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn update_replaces_credentials_when_envelope_set(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai
    .expect_for_liberty()
    .withf(|source| matches!(source, LibertySource::Envelope(_)))
    .returning(|_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_fetch_models()
        .returning(|| Ok(vec![anthropic_model("claude-haiku-4-5-20251001")]));
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let db_arc = Arc::new(db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_arc.clone())
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;
  let app_service: Arc<dyn services::AppService> = Arc::new(app_service);

  let create_req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Created".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(valid_envelope()),
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });
  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_req)?)
    .await?;
  assert_eq!(StatusCode::CREATED, create_response.status());
  let created = create_response.json::<ApiAliasResponse>().await?;
  let alias_id = created.id;

  let mut new_env = valid_envelope();
  new_env.access_token = "access-rotated".into();
  new_env.refresh_token = "refresh-rotated".into();
  let update_req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Rotated".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(new_env),
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });
  let update_response = test_router(app_service.clone())
    .oneshot(Request::put(&format!("{}/{}", ENDPOINT_MODELS_API, alias_id)).json(update_req)?)
    .await?;
  assert_eq!(StatusCode::OK, update_response.status());

  // read creds straight from the DB to confirm the rotation persisted
  use services::LlmLibertyCredentialsRepository;
  let creds = LlmLibertyCredentialsRepository::get_llm_liberty_credentials(
    db_arc.as_ref(),
    services::test_utils::TEST_TENANT_ID,
    "test-user",
    &alias_id,
  )
  .await?
  .expect("creds row");
  assert_eq!("access-rotated", creds.access_token);
  assert_eq!("refresh-rotated", creds.refresh_token);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn update_keeps_credentials_when_envelope_keep(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai
    .expect_for_liberty()
    .withf(|source| matches!(source, LibertySource::Envelope(_)))
    .returning(|_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_fetch_models()
        .returning(|| Ok(vec![anthropic_model("claude-haiku-4-5-20251001")]));
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });
  mock_ai
    .expect_for_liberty()
    .withf(|source| matches!(source, LibertySource::Resolved { .. }))
    .returning(|_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_fetch_models()
        .returning(|| Ok(vec![anthropic_model("claude-haiku-4-5-20251001")]));
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let db_arc = Arc::new(db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_arc.clone())
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;
  let app_service: Arc<dyn services::AppService> = Arc::new(app_service);

  let create_req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Created".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Set(valid_envelope()),
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: None,
    forward_all_with_prefix: false,
  });
  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_req)?)
    .await?;
  assert_eq!(StatusCode::CREATED, create_response.status());
  let created = create_response.json::<ApiAliasResponse>().await?;
  let alias_id = created.id;

  // envelope=Keep is an alias-scope-only change; credentials must stay untouched.
  let update_req = ApiModelRequest::LlmLibertyOauth(LlmLibertyApiModelRequest {
    name: "Liberty Kept".to_string(),
    envelope: LlmLibertyEnvelopeUpdate::Keep,
    models: vec!["claude-haiku-4-5-20251001".into()],
    prefix: Some("liberty-".into()),
    forward_all_with_prefix: false,
  });
  let update_response = test_router(app_service.clone())
    .oneshot(Request::put(&format!("{}/{}", ENDPOINT_MODELS_API, alias_id)).json(update_req)?)
    .await?;
  assert_eq!(StatusCode::OK, update_response.status());

  use services::LlmLibertyCredentialsRepository;
  let creds = LlmLibertyCredentialsRepository::get_llm_liberty_credentials(
    db_arc.as_ref(),
    services::test_utils::TEST_TENANT_ID,
    "test-user",
    &alias_id,
  )
  .await?
  .expect("creds row");
  assert_eq!("access-test", creds.access_token);
  assert_eq!("refresh-test", creds.refresh_token);

  let updated_response = test_router(app_service.clone())
    .oneshot(Request::get(&format!("{}/{}", ENDPOINT_MODELS_API, alias_id)).body(Body::empty())?)
    .await?;
  assert_eq!(StatusCode::OK, updated_response.status());
  let body = updated_response.json::<ApiAliasResponse>().await?;
  assert_eq!(Some("liberty-".to_string()), body.prefix);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_endpoint_400_when_id_and_envelope_both_present(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mock_ai = MockAiApiClientFactory::new();
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let payload = json!({
    "api_format": "llm_liberty_oauth",
    "id": "some-alias",
    "envelope": valid_envelope(),
    "model": "claude-haiku-4-5-20251001",
    "prompt": "hi"
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(&format!("{}/test", ENDPOINT_MODELS_API)).json(payload)?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_endpoint_400_when_neither_id_nor_envelope(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mock_ai = MockAiApiClientFactory::new();
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let payload = json!({
    "api_format": "llm_liberty_oauth",
    "model": "claude-haiku-4-5-20251001",
    "prompt": "hi"
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(&format!("{}/test", ENDPOINT_MODELS_API)).json(payload)?)
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  Ok(())
}
