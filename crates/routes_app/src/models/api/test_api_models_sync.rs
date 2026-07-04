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
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::AuthContext;
use services::{
  test_utils::{
    anthropic_model, openai_model, test_db_service, AppServiceStubBuilder, TestDbService,
    TEST_TENANT_ID,
  },
  ApiAliasRepository,
};
use services::{
  ApiAliasBuilder, ApiAliasResponse, ApiFormat, ApiFormat::OpenAI, ApiKey, ApiKeyUpdate,
  ApiModelRequest, DefaultApiModelRequest, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

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

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_sync_models_handler_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut mock_ai = services::MockAiApiClientFactory::new();
  mock_ai.expect_for_alias().returning(|_, _| {
    let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
    client.expect_fetch_models().returning(|| {
      Ok(vec![
        openai_model("gpt-4"),
        openai_model("gpt-3.5-turbo"),
        openai_model("gpt-4-turbo"),
      ])
    });
    Ok(Box::new(client) as Box<dyn services::AiApiClient>)
  });

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .ai_api_client_factory(Arc::new(mock_ai))
      .build()
      .await?,
  );

  let create_form = ApiModelRequest::default_for(
    OpenAI,
    DefaultApiModelRequest {
      name: "Sync Forward-All Model".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test123".to_string())?),
      models: vec![],
      prefix: Some("fwd/".to_string()),
      forward_all_with_prefix: true,
      extra_headers: None,
      extra_body: None,
    },
  );

  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?
    .json::<ApiAliasResponse>()
    .await?;

  let sync_response = test_router(app_service)
    .oneshot(
      Request::post(format!(
        "/bodhi/v1/models/api/{}/sync-models",
        create_response.id
      ))
      .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, sync_response.status());

  let sync_body = sync_response.json::<ApiAliasResponse>().await?;
  assert_eq!(create_response.id, sync_body.id);
  assert_eq!(OpenAI, sync_body.api_format);
  // Models should be returned without prefix - UI applies prefix
  let model_ids: Vec<&str> = sync_body.models.iter().map(|m| m.model.id()).collect();
  assert_eq!(vec!["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"], model_ids);
  assert_eq!(Some("fwd/".to_string()), sync_body.prefix);
  assert_eq!(true, sync_body.forward_all_with_prefix);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();

  let test_id = Uuid::new_v4().to_string();

  let api_alias = ApiAliasBuilder::test_default()
    .id(test_id.clone())
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4")])
    .build_with_time(now)?;

  db_service
    .create_api_model_alias(
      TEST_TENANT_ID,
      "",
      &api_alias,
      Some("sk-test123".to_string()),
    )
    .await?;

  let retrieved = db_service
    .get_api_model_alias(TEST_TENANT_ID, "", &test_id)
    .await?;
  let retrieved = retrieved.ok_or_else(|| anyhow::anyhow!("expected api model alias to exist"))?;
  assert_eq!(retrieved.id, test_id);

  // round-trip through encryption: the stored key must decrypt back to the original
  let api_key = db_service
    .get_api_key_for_alias(TEST_TENANT_ID, "", &test_id)
    .await?;
  assert_eq!(api_key, Some("sk-test123".to_string()));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_api_model(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();

  let api_alias = ApiAliasBuilder::test_default()
    .id("to-delete")
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4")])
    .build_with_time(now)?;

  db_service
    .create_api_model_alias(TEST_TENANT_ID, "", &api_alias, Some("sk-test".to_string()))
    .await?;

  assert!(db_service
    .get_api_model_alias(TEST_TENANT_ID, "", "to-delete")
    .await?
    .is_some());

  db_service
    .delete_api_model_alias(TEST_TENANT_ID, "", "to-delete")
    .await?;

  assert!(db_service
    .get_api_model_alias(TEST_TENANT_ID, "", "to-delete")
    .await?
    .is_none());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_sync_models_rejects_non_forward_all(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Mock AI returns models for the create validation call; sync should not reach it
  let mut mock_ai = services::MockAiApiClientFactory::new();
  mock_ai.expect_for_alias().returning(|_, _| {
    let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
    client
      .expect_fetch_models()
      .returning(|| Ok(vec![openai_model("gpt-4")]));
    Ok(Box::new(client) as Box<dyn services::AiApiClient>)
  });

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .ai_api_client_factory(Arc::new(mock_ai))
      .build()
      .await?,
  );

  let create_form = ApiModelRequest::default_for(
    OpenAI,
    DefaultApiModelRequest {
      name: "Sync Non-Forward-All Model".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test123".to_string())?),
      models: vec!["gpt-4".to_string()],
      prefix: None,
      forward_all_with_prefix: false,
      extra_headers: None,
      extra_body: None,
    },
  );

  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?
    .json::<ApiAliasResponse>()
    .await?;

  // sync is only valid for forward_all aliases; a non-forward_all alias must be rejected
  let sync_response = test_router(app_service)
    .oneshot(
      Request::post(format!(
        "/bodhi/v1/models/api/{}/sync-models",
        create_response.id
      ))
      .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, sync_response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_sync_models_anthropic_oauth_passes_extra_headers(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let extra_headers = json!({"anthropic-beta": "claude-code-20250219,oauth-2025-04-20"});
  let extra_body = json!({"system": [{"type": "text", "text": "You are Claude Code..."}]});

  let expected_extra_headers = extra_headers.clone();
  let expected_extra_body = extra_body.clone();

  // for_alias must receive the alias's extra_headers/extra_body intact
  let mut mock_ai = services::MockAiApiClientFactory::new();
  mock_ai
    .expect_for_alias()
    .withf(move |alias, _| {
      let services::Alias::Api(api_alias) = alias else {
        return false;
      };
      api_alias.api_format == ApiFormat::AnthropicOAuth
        && api_alias.extra_headers.as_ref() == Some(&expected_extra_headers)
        && api_alias.extra_body.as_ref() == Some(&expected_extra_body)
    })
    .returning(|_, _| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client.expect_fetch_models().returning(|| {
        Ok(vec![
          anthropic_model("claude-sonnet-4-5-20250929"),
          anthropic_model("claude-haiku-4-5-20251001"),
        ])
      });
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .ai_api_client_factory(Arc::new(mock_ai))
      .build()
      .await?,
  );

  let create_form = ApiModelRequest::default_for(
    ApiFormat::AnthropicOAuth,
    DefaultApiModelRequest {
      name: "Sync Anthropic OAuth Model".to_string(),
      base_url: "https://api.anthropic.com/v1".to_string(),
      api_key: ApiKeyUpdate::Set(ApiKey::some("sk-ant-oat01-token".to_string())?),
      models: vec![],
      prefix: Some("oauth/".to_string()),
      forward_all_with_prefix: true,
      extra_headers: Some(extra_headers),
      extra_body: Some(extra_body),
    },
  );

  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?
    .json::<ApiAliasResponse>()
    .await?;

  // the mock's withf is the real assertion: it fails the test if extra fields aren't forwarded
  let sync_response = test_router(app_service)
    .oneshot(
      Request::post(format!(
        "/bodhi/v1/models/api/{}/sync-models",
        create_response.id
      ))
      .body(axum::body::Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, sync_response.status());

  let sync_body = sync_response.json::<ApiAliasResponse>().await?;
  assert_eq!(ApiFormat::AnthropicOAuth, sync_body.api_format);
  let model_ids: Vec<&str> = sync_body.models.iter().map(|m| m.model.id()).collect();
  assert!(model_ids.contains(&"claude-sonnet-4-5-20250929"));
  assert!(model_ids.contains(&"claude-haiku-4-5-20251001"));

  Ok(())
}
