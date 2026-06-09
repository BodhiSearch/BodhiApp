use crate::{
  model_router_create, model_router_destroy, model_router_show, model_router_update,
  ENDPOINT_MODELS_ROUTER,
};
use anyhow_trace::anyhow_trace;
use axum::{
  http::{Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::test_utils::{openai_model, test_db_service, AppServiceStubBuilder, TestDbService};
use services::{
  ApiAlias, ApiAliasRepository, ApiFormat, AuthContext, LocalDataService, MockHubService,
  ModelRouterResponse, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route(ENDPOINT_MODELS_ROUTER, post(model_router_create))
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_ROUTER, "{id}"),
      get(model_router_show),
    )
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_ROUTER, "{id}"),
      put(model_router_update),
    )
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_ROUTER, "{id}"),
      delete(model_router_destroy),
    )
    .layer(axum::Extension(AuthContext::test_session(
      "test-user",
      "testuser",
      ResourceRole::User,
    )))
    .with_state(app_service)
}

async fn app_with_api_alias(
  db_service: TestDbService,
) -> anyhow::Result<Arc<dyn services::AppService>> {
  let now = db_service.now();
  let db = Arc::new(db_service);
  let api = ApiAlias::new(
    "oai",
    "OpenAI Alias",
    ApiFormat::OpenAI,
    "https://api.openai.com/v1",
    vec![openai_model("gpt-4")],
    Some("oai/".to_string()),
    false,
    now,
    None,
    None,
  );
  db.create_api_model_alias(
    "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "test-user",
    &api,
    Some("sk".into()),
  )
  .await?;
  let mut hub = MockHubService::new();
  hub.expect_list_model_aliases().returning(|| Ok(vec![]));
  let data = Arc::new(LocalDataService::new(Arc::new(hub), db.clone()));
  let app = AppServiceStubBuilder::default()
    .db_service(db)
    .data_service(data)
    .build()
    .await?;
  Ok(Arc::new(app))
}

fn create_body(alias: &str, target_alias: &str, target_model: &str) -> serde_json::Value {
  serde_json::json!({
    "alias": alias,
    "targets": [{"alias": target_alias, "model": target_model, "enabled": true}],
    "strategy": {"strategy": "fallback"}
  })
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_model_router_create_show_delete(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let app = app_with_api_alias(db_service).await?;

  // create
  let response = test_router(app.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_ROUTER).json(create_body(
      "my-stack",
      "oai",
      "oai/gpt-4",
    ))?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let created = response.json::<ModelRouterResponse>().await?;
  assert_eq!("my-stack", created.alias);
  assert_eq!("model_router", created.source);
  let id = created.id.clone();

  // show
  let response = test_router(app.clone())
    .oneshot(
      Request::get(format!("{ENDPOINT_MODELS_ROUTER}/{id}")).body(axum::body::Body::empty())?,
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());

  // delete
  let response = test_router(app.clone())
    .oneshot(
      Request::delete(format!("{ENDPOINT_MODELS_ROUTER}/{id}")).body(axum::body::Body::empty())?,
    )
    .await?;
  assert_eq!(StatusCode::NO_CONTENT, response.status());

  // show after delete -> 404
  let response = test_router(app)
    .oneshot(
      Request::get(format!("{ENDPOINT_MODELS_ROUTER}/{id}")).body(axum::body::Body::empty())?,
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_model_router_create_missing_reference_rejected(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let app = app_with_api_alias(db_service).await?;
  let response = test_router(app)
    .oneshot(Request::post(ENDPOINT_MODELS_ROUTER).json(create_body("r1", "absent", "absent/x"))?)
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_model_router_create_empty_alias_rejected(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let app = app_with_api_alias(db_service).await?;
  let response = test_router(app)
    .oneshot(
      Request::post(ENDPOINT_MODELS_ROUTER).json(serde_json::json!({
        "alias": "",
        "targets": [],
        "strategy": {"strategy": "fallback"}
      }))?,
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  Ok(())
}
