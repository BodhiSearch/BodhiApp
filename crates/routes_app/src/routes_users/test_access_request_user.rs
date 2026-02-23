use crate::{
  request_status_handler, user_request_access_handler, ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_USER_REQUEST_STATUS,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::{
  body::Body,
  http::Request,
  routing::{get, post},
  Router,
};
use objs::test_utils::temp_bodhi_home;
use objs::ResourceRole;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  db::AccessRepository,
  test_utils::{test_db_service_with_temp_dir, AppServiceStubBuilder},
};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

// ============================================================================
// user_request_access_handler tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_access_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      ENDPOINT_USER_REQUEST_ACCESS,
      post(user_request_access_handler),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_no_role(
          "new-user-id-123",
          "newuser@example.com",
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::CREATED, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_access_already_has_role(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      ENDPOINT_USER_REQUEST_ACCESS,
      post(user_request_access_handler),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "existing-user-id",
          "existing@example.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(
    axum::http::StatusCode::UNPROCESSABLE_ENTITY,
    response.status()
  );
  let body = response.json::<Value>().await?;
  assert_eq!(
    "access_request_error-already_has_access",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_access_already_pending(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;

  // Insert a pending request first
  db_service
    .insert_pending_request(
      "duplicate@example.com".to_string(),
      "dup-user-id".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      ENDPOINT_USER_REQUEST_ACCESS,
      post(user_request_access_handler),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_no_role(
          "dup-user-id",
          "duplicate@example.com",
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::CONFLICT, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "access_request_error-already_pending",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// request_status_handler tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_request_status_found(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  db_service
    .insert_pending_request(
      "status@example.com".to_string(),
      "status-user-id".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_STATUS, get(request_status_handler))
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(ENDPOINT_USER_REQUEST_STATUS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "status-user-id",
          "status@example.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!("status@example.com", body["username"].as_str().unwrap());
  assert_eq!("pending", body["status"].as_str().unwrap());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_request_status_not_found(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_STATUS, get(request_status_handler))
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(ENDPOINT_USER_REQUEST_STATUS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "no-such-user",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::NOT_FOUND, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "access_request_error-pending_request_not_found",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
