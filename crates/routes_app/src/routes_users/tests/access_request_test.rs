use crate::{
  approve_request_handler, list_all_requests_handler, list_pending_requests_handler,
  reject_request_handler, request_status_handler, user_request_access_handler,
  ApproveUserAccessRequest, PaginatedUserAccessResponse, UserAccessStatusResponse,
  ENDPOINT_ACCESS_REQUESTS_ALL, ENDPOINT_ACCESS_REQUESTS_PENDING, ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_USER_REQUEST_STATUS,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::{
  body::Body,
  http::{Request, StatusCode},
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
  db::{AccessRepository, UserAccessRequest, UserAccessRequestStatus},
  test_utils::{
    build_token_with_exp, test_db_service_with_temp_dir, AppServiceStubBuilder, SecretServiceStub,
  },
  AppRegInfo, MockAuthService, SessionService, SqliteSessionService,
};
use std::{collections::HashMap, fs::File, sync::Arc};
use tempfile::TempDir;
use time::OffsetDateTime;
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

#[test]
fn test_user_access_status_response_from_user_access_request() {
  // Test DTO conversion
  let request = UserAccessRequest {
    id: 1,
    username: "test@example.com".to_string(),
    user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    reviewer: None,
    status: UserAccessRequestStatus::Pending,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  let response = UserAccessStatusResponse::from(request.clone());

  assert_eq!(request.username, response.username);
  assert_eq!(request.status, response.status);
  assert_eq!(request.created_at, response.created_at);
  assert_eq!(request.updated_at, response.updated_at);
}

#[test]
fn test_approve_user_access_request_serde() -> anyhow::Result<()> {
  // Test request deserialization
  let json = r#"{"role": "resource_user"}"#;
  let request: ApproveUserAccessRequest = serde_json::from_str(json)?;
  assert_eq!(ResourceRole::User, request.role);

  let json = r#"{"role": "resource_admin"}"#;
  let request: ApproveUserAccessRequest = serde_json::from_str(json)?;
  assert_eq!(ResourceRole::Admin, request.role);

  Ok(())
}

#[test]
fn test_paginated_user_access_response_serde() -> anyhow::Result<()> {
  // Test response serialization
  let response = PaginatedUserAccessResponse {
    requests: vec![],
    total: 0,
    page: 1,
    page_size: 20,
  };

  let json: serde_json::Value = serde_json::to_value(&response)?;
  assert_eq!(json!([]), json["requests"]);
  assert_eq!(0, json["total"]);
  assert_eq!(1, json["page"]);
  assert_eq!(20, json["page_size"]);

  Ok(())
}

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
    Arc::new(SqliteSessionService::build_session_service(session_db.clone()).await);

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

    SessionStore::save(&session_service.session_store, &record).await?;
  }

  // 5. Verify sessions exist before approval
  let count_before = session_service
    .session_store
    .count_sessions_for_user(user_id)
    .await?;

  assert_eq!(3, count_before, "User should have 3 active sessions");

  // 6. Setup mock auth service for role assignment
  let mut mock_auth = MockAuthService::default();
  mock_auth
    .expect_assign_user_role()
    .times(1)
    .withf(|_token, uid, role| uid == "test-user-123" && role == "resource_user")
    .return_once(|_, _, _| Ok(()));

  // 7. Setup secret service with app registration info
  let secret_service = SecretServiceStub::default().with_app_reg_info(&AppRegInfo {
    client_id: "test_client_id".to_string(),
    client_secret: "test_secret".to_string(),
    scope: "scope_test_client_id".to_string(),
  });

  // 8. Build complete app service
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_sqlite_session_service(session_service.clone())
    .auth_service(Arc::new(mock_auth))
    .secret_service(Arc::new(secret_service))
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
  let session_store = session_service.get_session_store();
  let count_after = session_store.count_sessions_for_user(user_id).await?;

  assert_eq!(
    0, count_after,
    "All user sessions should be cleared after role assignment"
  );

  // 13. Verify request status was updated
  let updated_request = state
    .app_service()
    .db_service()
    .get_request_by_id(access_request.id)
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
    .get_request_by_id(access_request.id)
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

// Auth tier: Manager (session-only) - access request management requires resource_manager or resource_admin role

#[anyhow_trace]
#[rstest]
#[case::list_pending("GET", "/bodhi/v1/access-requests/pending")]
#[case::list_all("GET", "/bodhi/v1/access-requests")]
#[case::approve("POST", "/bodhi/v1/access-requests/1/approve")]
#[case::reject("POST", "/bodhi/v1/access-requests/1/reject")]
#[tokio::test]
async fn test_access_request_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  use tower::ServiceExt;
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_access_request_endpoints_reject_insufficient_role(
  #[values("resource_user", "resource_power_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/access-requests/pending"),
    ("GET", "/bodhi/v1/access-requests"),
    ("POST", "/bodhi/v1/access-requests/1/approve"),
    ("POST", "/bodhi/v1/access-requests/1/reject")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
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
async fn test_access_request_endpoints_allow_manager_and_admin(
  #[values("resource_manager", "resource_admin")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/access-requests/pending"),
    ("GET", "/bodhi/v1/access-requests")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  // Both GET endpoints are safe: db_service returns empty list, no MockAuthService expectations
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// VIOLATION: POST endpoints for access request management cannot be added to allow test
// Reason: POST /access-requests/{id}/approve calls auth_service.assign_user_role() requiring MockAuthService
// These cannot work with build_test_router() without mock expectations.
