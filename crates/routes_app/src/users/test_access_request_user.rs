use crate::test_utils::{
  make_auth_no_role, make_auth_with_role_default_token, RequestAuthContextExt,
};
use crate::{
  users_request_access, users_request_status, ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_USER_REQUEST_STATUS,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::Request,
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::test_utils::ResponseTestExt;
use services::test_utils::temp_bodhi_home;
use services::AuthContext;
use services::ResourceRole;
use services::{
  test_utils::{test_db_service_with_temp_dir, AppServiceStubBuilder, TEST_TENANT_ID},
  AccessRepository,
};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

// ============================================================================
// users_request_access tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_access_success(
  #[values("session", "multi_tenant")] auth_variant: &str,
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(make_auth_no_role(
          auth_variant,
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
async fn test_user_request_access_already_has_role(
  #[values("session", "multi_tenant")] auth_variant: &str,
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(make_auth_with_role_default_token(
          auth_variant,
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
    "users_route_error-already_has_access",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_access_already_pending(
  #[values("session", "multi_tenant")] auth_variant: &str,
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;

  // Insert a pending request first
  db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "duplicate@example.com".to_string(),
      "dup-user-id".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(make_auth_no_role(
          auth_variant,
          "dup-user-id",
          "duplicate@example.com",
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::CONFLICT, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "users_route_error-already_pending",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// users_request_status tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_request_status_found(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "status@example.com".to_string(),
      "status-user-id".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
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

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
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
    "users_route_error-pending_request_not_found",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// Guest role tests (verifying Guest users can use these endpoints)
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_access_guest_role_succeeds(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(ENDPOINT_USER_REQUEST_ACCESS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "guest-user-id",
          "guest@example.com",
          ResourceRole::Guest,
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::CREATED, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_request_status_guest_role_not_found(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(ENDPOINT_USER_REQUEST_STATUS)
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "guest-user-id",
          "guest@example.com",
          ResourceRole::Guest,
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::NOT_FOUND, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "users_route_error-pending_request_not_found",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
