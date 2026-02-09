use crate::{queue_status_handler, refresh_metadata_handler, RefreshResponse};
use anyhow_trace::anyhow_trace;
use axum::{http::Request, http::StatusCode, routing::get, routing::post, Router};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  test_utils::{app_service_stub_builder, AppServiceStubBuilder},
  MockQueueProducer,
};
use std::sync::Arc;
use tower::ServiceExt;

fn test_metadata_router(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .route("/api/models/refresh", post(refresh_metadata_handler))
    .route("/api/queue", get(queue_status_handler))
    .with_state(state)
}

// ============================================================================
// refresh_metadata_handler tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_metadata_no_params_returns_202_accepted(
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  // Configure mock to succeed on enqueue
  let mut mock_queue = MockQueueProducer::new();
  mock_queue
    .expect_enqueue()
    .returning(|_| Box::pin(async { Ok(()) }));
  mock_queue
    .expect_queue_status()
    .returning(|| "idle".to_string());

  let app_service = app_service_stub_builder
    .queue_producer(Arc::new(mock_queue))
    .build()?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(Request::post("/api/models/refresh").json_str(r#"{"source":"all"}"#)?)
    .await?;

  assert_eq!(StatusCode::ACCEPTED, response.status());

  let body = response.json::<RefreshResponse>().await?;
  assert_eq!("all", body.num_queued);
  assert!(body.alias.is_none());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_metadata_enqueue_failure_returns_400(
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  // Configure mock to fail on enqueue
  let mut mock_queue = MockQueueProducer::new();
  mock_queue
    .expect_enqueue()
    .returning(|_| Box::pin(async { Err("Queue full".into()) }));
  mock_queue
    .expect_queue_status()
    .returning(|| "idle".to_string());

  let app_service = app_service_stub_builder
    .queue_producer(Arc::new(mock_queue))
    .build()?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(Request::post("/api/models/refresh").json_str(r#"{"source":"all"}"#)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "metadata_error-enqueue_failed",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// refresh_metadata_handler - sync model path tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_metadata_model_invalid_repo_format(
  #[future] app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  let app_service = app_service_stub_builder.build()?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(
      Request::post("/api/models/refresh").json_str(
        r#"{"source":"model","repo":"invalid-repo-no-slash","filename":"test.gguf","snapshot":"abc123"}"#,
      )?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "metadata_error-invalid_repo_format",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_metadata_model_alias_not_found(
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  let app_service = app_service_stub_builder.with_data_service().await.build()?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(
      Request::post("/api/models/refresh").json_str(
        r#"{"source":"model","repo":"nonexistent/model","filename":"nonexistent.gguf","snapshot":"abc123"}"#,
      )?,
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!(
    "metadata_error-alias_not_found",
    body["error"]["code"].as_str().unwrap()
  );

  Ok(())
}

// ============================================================================
// queue_status_handler tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_queue_status_handler_returns_idle(
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  let mut mock_queue = MockQueueProducer::new();
  mock_queue
    .expect_enqueue()
    .returning(|_| Box::pin(async { Ok(()) }));
  mock_queue
    .expect_queue_status()
    .returning(|| "idle".to_string());

  let app_service = app_service_stub_builder
    .queue_producer(Arc::new(mock_queue))
    .build()?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(Request::get("/api/queue").body(axum::body::Body::empty())?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());

  let body = response.json::<Value>().await?;
  assert_eq!("idle", body["status"].as_str().unwrap());

  Ok(())
}

// Auth tier tests (merged from tests/routes_models_metadata_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::refresh_metadata("POST", "/bodhi/v1/models/refresh")]
#[case::queue_status("GET", "/bodhi/v1/queue")]
#[tokio::test]
async fn test_metadata_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_metadata_endpoints_reject_insufficient_role(
  #[values("resource_user")] role: &str,
  #[values(
    ("POST", "/bodhi/v1/models/refresh"),
    ("GET", "/bodhi/v1/queue")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_metadata_endpoints_allow_power_user_and_above(
  #[values("resource_power_user", "resource_manager", "resource_admin")] role: &str,
  #[values(("GET", "/bodhi/v1/queue"))]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  // GET /bodhi/v1/queue returns 200 OK with StubQueue returning "idle"
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
