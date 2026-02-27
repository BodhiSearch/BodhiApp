use crate::{
  approve_request_handler, list_all_requests_handler, list_pending_requests_handler,
  reject_request_handler, ENDPOINT_ACCESS_REQUESTS_ALL, ENDPOINT_ACCESS_REQUESTS_PENDING,
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
use serde_json::{json, Value};
use server_core::{
  test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  db::{AccessRepository, UserAccessRequestStatus},
  test_utils::{build_token_with_exp, test_db_service_with_temp_dir, AppServiceStubBuilder},
  DefaultSessionService, MockAuthService, SessionService,
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
async fn test_approve_request_clears_user_sessions(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
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
    .insert_pending_request(username.to_string(), user_id.to_string())
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
      "access_token".to_string(),
      serde_json::Value::String(format!("token_{}", i)),
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
  let db_arc: Arc<dyn services::db::DbService> = Arc::new(db_service);

  // 8. Build complete app service
  let mut builder = AppServiceStubBuilder::default();
  builder.db_service(db_arc);
  builder
    .with_app_instance(services::AppInstance {
      client_id: "test_client_id".to_string(),
      client_secret: "test_secret".to_string(),

      status: services::AppStatus::Ready,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    })
    .await;
  let app_service = builder
    .with_default_session_service(session_service.clone())
    .auth_service(Arc::new(mock_auth))
    .build()
    .await?;

  // 9. Create router with approve endpoint
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(approve_request_handler),
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
  .with_auth_context(AuthContext::test_session_with_token(
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
    .app_service()
    .db_service()
    .get_request_by_id(&access_request.id)
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
// list_pending_requests_handler tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_pending_requests_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  db_service
    .insert_pending_request("user1@example.com".to_string(), "user-1".to_string())
    .await?;
  db_service
    .insert_pending_request("user2@example.com".to_string(), "user-2".to_string())
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
      ENDPOINT_ACCESS_REQUESTS_PENDING,
      get(list_pending_requests_handler),
    )
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(format!(
        "{}?page=1&page_size=10",
        ENDPOINT_ACCESS_REQUESTS_PENDING
      ))
      .body(Body::empty())?,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(2, body["total"].as_i64().unwrap());
  assert_eq!(2, body["requests"].as_array().unwrap().len());
  Ok(())
}

// ============================================================================
// list_all_requests_handler tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_all_requests_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  db_service
    .insert_pending_request("user1@example.com".to_string(), "user-1".to_string())
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
    .route(ENDPOINT_ACCESS_REQUESTS_ALL, get(list_all_requests_handler))
    .with_state(state);

  let response = router
    .oneshot(
      Request::get(format!(
        "{}?page=1&page_size=10",
        ENDPOINT_ACCESS_REQUESTS_ALL
      ))
      .body(Body::empty())?,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  let body = response.json::<Value>().await?;
  assert_eq!(1, body["total"].as_i64().unwrap());
  Ok(())
}

// ============================================================================
// reject_request_handler tests
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_reject_request_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let access_request = db_service
    .insert_pending_request(
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

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/reject", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(reject_request_handler),
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
    .app_service()
    .db_service()
    .get_request_by_id(&access_request.id)
    .await?
    .unwrap();
  assert_eq!(UserAccessRequestStatus::Rejected, updated.status);
  Ok(())
}

// ============================================================================
// approve_request_handler - insufficient privileges test
// ============================================================================

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_insufficient_privileges(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  let access_request = db_service
    .insert_pending_request("priv@example.com".to_string(), "priv-user-id".to_string())
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
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(approve_request_handler),
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
      .with_auth_context(AuthContext::test_session_with_token(
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
    "access_request_error-insufficient_privileges",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// ============================================================================
// approve_request_handler - request not found test
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

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  let router = Router::new()
    .route(
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(approve_request_handler),
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
    "access_request_error-request_not_found",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
