use crate::{
  create_api_model_handler, delete_api_model_handler, fetch_models_handler, get_api_model_handler,
  list_api_models_handler, sync_models_handler, test_api_model_handler, update_api_model_handler,
  ApiKey, ApiModelResponse, CreateApiModelRequest, ENDPOINT_API_MODELS,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use mockall::predicate;
use objs::ApiAliasBuilder;
use objs::ApiFormat::OpenAI;
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  db::ModelRepository,
  test_utils::{test_db_service, AppServiceStubBuilder, TestDbService},
};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
  Router::new()
    .route(ENDPOINT_API_MODELS, get(list_api_models_handler))
    .route(ENDPOINT_API_MODELS, post(create_api_model_handler))
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      get(get_api_model_handler),
    )
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      put(update_api_model_handler),
    )
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      delete(delete_api_model_handler),
    )
    .route(
      &format!("{}/test", ENDPOINT_API_MODELS),
      post(test_api_model_handler),
    )
    .route(
      &format!("{}/fetch-models", ENDPOINT_API_MODELS),
      post(fetch_models_handler),
    )
    .route(
      &format!("{}/{{id}}/sync-models", ENDPOINT_API_MODELS),
      post(sync_models_handler),
    )
    .with_state(Arc::new(router_state))
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
  // Set up mock AI API service with expectations
  let mut mock_ai = services::MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .with(
      predicate::eq(Some("sk-test123".to_string())),
      predicate::eq("https://api.openai.com/v1"),
    )
    .returning(|_, _| {
      Ok(vec![
        "gpt-4".to_string(),
        "gpt-3.5-turbo".to_string(),
        "gpt-4-turbo".to_string(),
      ])
    });

  // Create app service with clean database and mock AI service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .ai_api_service(Arc::new(mock_ai))
      .with_secret_service()
      .build()
      .await?,
  );

  // First create an API model
  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123".to_string())?,
    models: vec![],
    prefix: Some("fwd/".to_string()),
    forward_all_with_prefix: true,
  };

  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?
    .json::<ApiModelResponse>()
    .await?;

  // Sync models
  let sync_response = test_router(app_service)
    .oneshot(
      Request::post(format!(
        "/bodhi/v1/api-models/{}/sync-models",
        create_response.id
      ))
      .body(Body::empty())?,
    )
    .await?;

  // Verify response status is 200 OK
  assert_eq!(StatusCode::OK, sync_response.status());

  // Verify response contains the API model with cached models (unprefixed)
  let sync_body = sync_response.json::<ApiModelResponse>().await?;
  assert_eq!(create_response.id, sync_body.id);
  assert_eq!(OpenAI, sync_body.api_format);
  // Models should be returned without prefix - UI applies prefix
  assert_eq!(
    vec!["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"],
    sync_body.models
  );
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

  // Generate a unique ID for the test
  let test_id = Uuid::new_v4().to_string();

  // Create API model via database
  let api_alias = ApiAliasBuilder::test_default()
    .id(test_id.clone())
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec!["gpt-4".to_string()])
    .build_with_time(now)?;

  db_service
    .create_api_model_alias(&api_alias, Some("sk-test123".to_string()))
    .await?;

  // Verify it was created
  let retrieved = db_service.get_api_model_alias(&test_id).await?;
  let retrieved = retrieved.ok_or_else(|| anyhow::anyhow!("expected api model alias to exist"))?;
  assert_eq!(retrieved.id, test_id);

  // Verify API key is encrypted but retrievable
  let api_key = db_service.get_api_key_for_alias(&test_id).await?;
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

  // Create API model
  let api_alias = ApiAliasBuilder::test_default()
    .id("to-delete")
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec!["gpt-4".to_string()])
    .build_with_time(now)?;

  db_service
    .create_api_model_alias(&api_alias, Some("sk-test".to_string()))
    .await?;

  // Verify it exists
  assert!(db_service.get_api_model_alias("to-delete").await?.is_some());

  // Delete it
  db_service.delete_api_model_alias("to-delete").await?;

  // Verify it's gone
  assert!(db_service.get_api_model_alias("to-delete").await?.is_none());

  Ok(())
}
