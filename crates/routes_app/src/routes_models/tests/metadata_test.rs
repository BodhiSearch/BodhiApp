use crate::{refresh_metadata_handler, RefreshResponse};
use axum::{body::Body, http::Request, http::StatusCode, routing::post, Router};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState};
use services::{
  test_utils::{app_service_stub_builder, AppServiceStubBuilder},
  MockQueueProducer,
};
use std::sync::Arc;
use tower::ServiceExt;

fn test_metadata_router(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .route("/api/models/refresh", post(refresh_metadata_handler))
    .with_state(state)
}

// ============================================================================
// refresh_metadata_handler tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
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
    .build()
    .unwrap();

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(
      Request::post("/api/models/refresh")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"source":"all"}"#))
        .unwrap(),
    )
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
    .build()
    .unwrap();

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let response = test_metadata_router(state)
    .oneshot(
      Request::post("/api/models/refresh")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"source":"all"}"#))
        .unwrap(),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let body = response.json::<Value>().await?;
  assert!(body["error"]["message"]
    .as_str()
    .unwrap()
    .contains("enqueue"));

  Ok(())
}
