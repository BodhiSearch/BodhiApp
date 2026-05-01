use crate::anthropic_messages_create_handler;
use crate::test_utils::{mock_ai_factory_returning, RequestAuthContextExt};
use anyhow_trace::anyhow_trace;
use axum::{extract::Request, routing::post, Router};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::models::llm_liberty_envelope::LlmLibertyEnvelope;
use services::{
  test_utils::{
    anthropic_model, test_llm_liberty_envelope, AppServiceStubBuilder, TEST_TENANT_ID, TEST_USER_ID,
  },
  AiApiClientFactoryError, ApiAliasBuilder, ApiFormat, AuthContext, LibertySource,
  MockAiApiClientFactory, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn ok_response() -> Result<axum::response::Response, AiApiClientFactoryError> {
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

async fn seed_anthropic_oauth_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-oauth-alias")
    .api_format(ApiFormat::AnthropicOAuth)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-sonnet-4-5-20250929")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  Ok(api_alias)
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_forwards_to_anthropic_oauth_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_oauth_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
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
          "model": "claude-sonnet-4-5-20250929",
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

// expires_at far in the future so ensure_fresh_credentials short-circuits without HTTP.
fn liberty_envelope(provider: &str, access_token: &str) -> LlmLibertyEnvelope {
  let mut env = test_llm_liberty_envelope();
  env.provider = provider.to_string();
  env.access_token = access_token.to_string();
  env.refresh_token = "refresh-test".into();
  env.expires_at = (chrono::Utc::now() + chrono::Duration::days(365)).timestamp();
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
async fn test_messages_create_forwards_to_llm_liberty_oauth_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let alias_id = "liberty-alias";
  let api_alias = ApiAliasBuilder::test_default()
    .id(alias_id)
    .api_format(ApiFormat::LlmLibertyOauth)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-haiku-4-5-20251001")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  db_service
    .create_llm_liberty_credentials(
      TEST_TENANT_ID,
      TEST_USER_ID,
      alias_id,
      &liberty_envelope("anthropic", "resolved-bearer-token"),
    )
    .await?;

  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai.expect_safe_http_client().returning(|| {
    services::SafeReqwest::builder()
      .allow_private_ips()
      .build()
      .unwrap()
  });
  mock_ai
    .expect_for_liberty()
    .withf(|source| match source {
      LibertySource::Resolved {
        creds, alias_id, ..
      } => creds.access_token == "resolved-bearer-token" && *alias_id == "liberty-alias",
      LibertySource::Envelope(_) => false,
    })
    .times(1)
    .returning(|_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_forward_request_with_method()
        .times(1)
        .returning(|_, _, _, _, _| {
          Ok(
            axum::response::Response::builder()
              .status(200)
              .header("content-type", "application/json")
              .body(axum::body::Body::from(
                r#"{"id":"msg-123","content":[{"type":"text","text":"Hi"}]}"#,
              ))
              .unwrap(),
          )
        });
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let app_service = builder
    .ai_api_client_factory(Arc::new(mock_ai))
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
          "model": "claude-haiku-4-5-20251001",
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
async fn test_messages_create_rejects_llm_liberty_non_anthropic_provider() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let alias_id = "liberty-codex-alias";
  let api_alias = ApiAliasBuilder::test_default()
    .id(alias_id)
    .api_format(ApiFormat::LlmLibertyOauth)
    .base_url("https://api.openai.com/v1")
    .models(vec![anthropic_model("claude-haiku-4-5-20251001")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  db_service
    .create_llm_liberty_credentials(
      TEST_TENANT_ID,
      TEST_USER_ID,
      alias_id,
      &liberty_envelope("google-gemini", "should-not-be-used"),
    )
    .await?;

  // Use the real factory so for_liberty returns LibertyProviderUnsupported
  // (BadRequest) for the openai-codex provider.
  let real_factory = services::DefaultAiApiClientFactory::new()?;
  let app_service = builder
    .ai_api_client_factory(Arc::new(real_factory))
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
          "model": "claude-haiku-4-5-20251001",
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
  let message = body["error"]["message"].as_str().unwrap_or("");
  assert!(
    message.contains("openai-codex") || message.to_lowercase().contains("provider"),
    "expected provider-mismatch message, got: {}",
    message
  );
  Ok(())
}

// OpenAI alias rejection path. AnthropicOAuth acceptance is covered by the test above.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_rejects_openai_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let openai_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![services::test_utils::openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &openai_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let err_response = app
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
  assert_eq!(StatusCode::BAD_REQUEST, err_response.status());
  let body = err_response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!(
    "invalid_request_error",
    body["error"]["type"].as_str().unwrap()
  );
  Ok(())
}
