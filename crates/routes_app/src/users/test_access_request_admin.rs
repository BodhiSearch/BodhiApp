use crate::test_utils::{make_auth_with_role, RequestAuthContextExt};
use crate::{
  users_access_request_approve, users_access_request_reject, users_access_requests_index,
  users_access_requests_pending, ENDPOINT_ACCESS_REQUESTS_ALL, ENDPOINT_ACCESS_REQUESTS_PENDING,
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
use serde_json::{json, Value};
use server_core::test_utils::ResponseTestExt;
use services::test_utils::temp_bodhi_home;
use services::AuthContext;
use services::ResourceRole;
use services::{
  test_utils::{
    build_token_with_exp, test_db_service_with_temp_dir, AppServiceStubBuilder, TEST_TENANT_ID,
  },
  DefaultSessionService, MockAuthService, SessionService,
  {AccessRepository, UserAccessRequestStatus},
};
use std::{collections::HashMap, fs::File, sync::Arc};
use tempfile::TempDir;
use time::OffsetDateTime;
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_clears_user_sessions(
  #[values("session", "multi_tenant")] auth_variant: &str,
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  // 1. Setup: Create real databases for both app and session
  let session_db = temp_bodhi_home.path().join("session.sqlite");

  // 2. Create services with real databases
  File::create(&session_db)?;

  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let session_service =
    Arc::new(DefaultSessionService::build_session_service(session_db.clone()).await);

  // 3. Create a pending access request for a user
  let user_id = "test-user-123";
  let username = "testuser@example.com";
  let access_request = db_service
    .insert_pending_request(TEST_TENANT_ID, username.to_string(), user_id.to_string())
    .await?;

  // 4. Simulate user having multiple active sessions
  // (as if they logged in from different devices/browsers)
  for i in 0..3 {
    let id = Id::default();
    let mut data = HashMap::new();
    data.insert(
      "user_id".to_string(),
      serde_json::Value::String(user_id.to_string()),
    );
    data.insert(
      "test-client:access_token".to_string(),
      serde_json::Value::String(format!("token_{}", i)),
    );
    data.insert(
      "active_client_id".to_string(),
      serde_json::Value::String("test-client".to_string()),
    );
    data.insert(
      "device".to_string(),
      serde_json::Value::String(format!("device_{}", i)),
    );

    let record = Record {
      id: id,
      data,
      expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
    };

    SessionStore::save(session_service.get_session_store(), &record).await?;
  }

  // 5. Verify sessions exist before approval
  let count_before = session_service.count_sessions_for_user(user_id).await?;

  assert_eq!(3, count_before, "User should have 3 active sessions");

  // 6. Setup mock auth service for role assignment
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_assign_user_role()
    .times(1)
    .withf(|_token, uid, role| uid == "test-user-123" && role == "resource_user")
    .return_once(|_, _, _| Ok(()));

  // 7. Setup app instance service with client registration info
  let db_arc: Arc<dyn services::DbService> = Arc::new(db_service);

  // 8. Build complete app service
  let mut builder = AppServiceStubBuilder::default();
  builder.db_service(db_arc);
  builder.with_tenant(services::Tenant::test_default()).await;
  let app_service = builder
    .with_default_session_service(session_service.clone())
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  // 9. Create router with approve endpoint
  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(users_access_request_approve),
    )
    .with_state(state.clone());

  // 10. Make HTTP request with required auth context (simulating authenticated admin)
  let request = Request::post(format!(
    "{}/{}/approve",
    ENDPOINT_ACCESS_REQUESTS_ALL, access_request.id
  ))
  .header("content-type", "application/json")
  .body(Body::from(serde_json::to_string(
    &json!({ "role": "resource_user" }),
  )?))?
  .with_auth_context(make_auth_with_role(
    auth_variant,
    "admin-user-id",
    "admin@example.com",
    ResourceRole::Manager,
    "dummy-admin-token",
  ));

  // Send request through the router
  let response = router.oneshot(request).await?;

  // Verify the handler succeeded
  assert_eq!(
    axum::http::StatusCode::OK,
    response.status(),
    "Handler should return OK status"
  );

  // 12. Verify all user sessions were cleared
  let count_after = session_service.count_sessions_for_user(user_id).await?;

  assert_eq!(
    0, count_after,
    "All user sessions should be cleared after role assignment"
  );

  // 13. Verify request status was updated
  let updated_request = state
    .db_service()
    .get_request_by_id(TEST_TENANT_ID, &access_request.id)
    .await?
    .unwrap();
  assert_eq!(UserAccessRequestStatus::Approved, updated_request.status);
  assert_eq!(
    Some("admin@example.com".to_string()),
    updated_request.reviewer
  );

  Ok(())
}

// ============================================================================
// users_access_requests_pending tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_pending_requests_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "user1@example.com".to_string(),
      "user-1".to_string(),
    )
    .await?;
  db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "user2@example.com".to_string(),
      "user-2".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_PENDING,
      get(users_access_requests_pending),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(format!(
        "{}?page=1&page_size=10",
        ENDPOINT_ACCESS_REQUESTS_PENDING
      ))
      .body(Body::empty())?
      .with_auth_context(AuthContext::test_session(
        "test-user",
        "testuser",
        ResourceRole::Admin,
      )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(2, body["total"].as_i64().unwrap());
  assert_eq!(2, body["requests"].as_array().unwrap().len());
  Ok(())
}

// ============================================================================
// users_access_requests_index tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_all_requests_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "user1@example.com".to_string(),
      "user-1".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_ALL,
      get(users_access_requests_index),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(format!(
        "{}?page=1&page_size=10",
        ENDPOINT_ACCESS_REQUESTS_ALL
      ))
      .body(Body::empty())?
      .with_auth_context(AuthContext::test_session(
        "test-user",
        "testuser",
        ResourceRole::Admin,
      )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(1, body["total"].as_i64().unwrap());
  Ok(())
}

// ============================================================================
// users_access_request_reject tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_reject_request_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let access_request = db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "toreject@example.com".to_string(),
      "reject-user-id".to_string(),
    )
    .await?;

  let (test_token, _) =
    build_token_with_exp((chrono::Utc::now() + chrono::Duration::hours(1)).timestamp())?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/reject", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(users_access_request_reject),
    )
    .with_state(state.clone());

  let response = router
    .oneshot(
      Request::post(format!(
        "{}/{}/reject",
        ENDPOINT_ACCESS_REQUESTS_ALL, access_request.id
      ))
      .body(Body::empty())?
      .with_auth_context(AuthContext::test_session_with_token(
        "test-user-id",
        "user@test.com",
        ResourceRole::Manager,
        &test_token,
      )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());

  // Verify the request was rejected
  let updated = state
    .db_service()
    .get_request_by_id(TEST_TENANT_ID, &access_request.id)
    .await?
    .unwrap();
  assert_eq!(UserAccessRequestStatus::Rejected, updated.status);
  Ok(())
}

// ============================================================================
// users_access_request_approve - insufficient privileges test
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_insufficient_privileges(
  #[values("session", "multi_tenant")] auth_variant: &str,
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let access_request = db_service
    .insert_pending_request(
      TEST_TENANT_ID,
      "priv@example.com".to_string(),
      "priv-user-id".to_string(),
    )
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(users_access_request_approve),
    )
    .with_state(state);

  // A user role trying to approve admin role should fail
  let response = router
    .oneshot(
      Request::post(format!(
        "{}/{}/approve",
        ENDPOINT_ACCESS_REQUESTS_ALL, access_request.id
      ))
      .header("content-type", "application/json")
      .body(Body::from(serde_json::to_string(
        &json!({ "role": "resource_admin" }),
      )?))?
      .with_auth_context(make_auth_with_role(
        auth_variant,
        "lowpriv-user-id",
        "lowpriv@example.com",
        ResourceRole::User,
        "dummy-token",
      )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "users_route_error-insufficient_privileges",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// users_access_request_approve - request not found test
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_not_found(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let state: Arc<dyn services::AppService> = Arc::new(app_service);

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(users_access_request_approve),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::post(format!("{}/99999/approve", ENDPOINT_ACCESS_REQUESTS_ALL))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(
          &json!({ "role": "resource_user" }),
        )?))?
        .with_auth_context(AuthContext::test_session_with_token(
          "admin-user-id",
          "admin@example.com",
          ResourceRole::Admin,
          "dummy-token",
        )),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::NOT_FOUND, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(
    "users_route_error-request_not_found",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
