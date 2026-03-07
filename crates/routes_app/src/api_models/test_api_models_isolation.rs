use crate::test_utils::{setup_env, RequestAuthContextExt};
use crate::{
  api_models_create, api_models_destroy, api_models_index, api_models_show, api_models_update,
  ENDPOINT_API_MODELS,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Method, Request},
  routing::get,
  Router,
};
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::AuthContext;
use services::{
  db::DbService,
  test_utils::{sea_context, AppServiceStubBuilder, SeaTestContext, TEST_TENANT_B_ID},
  ApiFormat, ApiKey, ApiKeyUpdate, ApiModelOutput, ApiModelRequest, AppService,
  PaginatedApiModelOutput, ResourceRole, Tenant,
};
use std::sync::Arc;
use tower::ServiceExt;

/// Returns (router, app_service, _ctx) -- caller must hold `_ctx` to keep the SQLite temp dir alive.
async fn isolation_router(
  db_type: &str,
) -> anyhow::Result<(Router, Arc<dyn AppService>, SeaTestContext)> {
  let ctx = sea_context(db_type).await;
  let db_svc: Arc<dyn DbService> = Arc::new(ctx.service.clone());
  let mut builder = AppServiceStubBuilder::default();
  builder
    .db_service(db_svc.clone())
    .with_tenant_service()
    .await;
  let app_service: Arc<dyn AppService> = Arc::new(builder.build().await?);

  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_default())
    .await?;
  app_service
    .db_service()
    .create_tenant_test(&Tenant::test_tenant_b())
    .await?;

  let router = Router::new()
    .route(
      ENDPOINT_API_MODELS,
      get(api_models_index).post(api_models_create),
    )
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      get(api_models_show)
        .put(api_models_update)
        .delete(api_models_destroy),
    )
    .with_state(app_service.clone());

  Ok((router, app_service, ctx))
}

fn create_form(base_url: &str) -> ApiModelRequest {
  ApiModelRequest {
    api_format: ApiFormat::OpenAI,
    base_url: base_url.to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test-key-12345".to_string()).unwrap()),
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_api_model_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create API model as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.openai.com/v1"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // Create API model as tenant B user A (same user, different tenant)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.anthropic.com/v1"))?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as tenant A -> only 1 model
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(ENDPOINT_API_MODELS)
        .body(Body::empty())?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedApiModelOutput>().await?;
  assert_eq!(1, list.data.len());

  // List as tenant B -> only 1 model
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(ENDPOINT_API_MODELS)
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedApiModelOutput>().await?;
  assert_eq!(1, list.data.len());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_api_model_show_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create API model as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.openai.com/v1"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let model = response.json::<ApiModelOutput>().await?;
  let model_id = model.id;

  // Show that model as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(format!("{}/{}", ENDPOINT_API_MODELS, model_id))
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_api_model_update_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create API model as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.openai.com/v1"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let model = response.json::<ApiModelOutput>().await?;
  let model_id = model.id;

  // Update that model as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("{}/{}", ENDPOINT_API_MODELS, model_id))
        .json(&create_form("https://api.updated.com/v1"))?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_api_model_delete_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

  // Create API model as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.openai.com/v1"))?
        .with_auth_context(auth_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let model = response.json::<ApiModelOutput>().await?;
  let model_id = model.id;

  // Delete that model as tenant B user A -> 404
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("{}/{}", ENDPOINT_API_MODELS, model_id))
        .body(Body::empty())?
        .with_auth_context(auth_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_api_model_list_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let (router, _app_service, _ctx) = isolation_router(db_type).await?;

  let auth_user_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
  let auth_user_b = AuthContext::test_session("user-b", "b@test.com", ResourceRole::Admin);

  // Create API model as tenant A user A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.openai.com/v1"))?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // Create API model as tenant A user B
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri(ENDPOINT_API_MODELS)
        .json(&create_form("https://api.anthropic.com/v1"))?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());

  // List as user A -> only 1 model
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(ENDPOINT_API_MODELS)
        .body(Body::empty())?
        .with_auth_context(auth_user_a.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedApiModelOutput>().await?;
  assert_eq!(1, list.data.len());

  // List as user B -> only 1 model
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri(ENDPOINT_API_MODELS)
        .body(Body::empty())?
        .with_auth_context(auth_user_b.clone()),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let list = response.json::<PaginatedApiModelOutput>().await?;
  assert_eq!(1, list.data.len());

  Ok(())
}
