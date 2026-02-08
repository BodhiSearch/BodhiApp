use crate::{
  approve_request_handler, ApproveUserAccessRequest, PaginatedUserAccessResponse,
  UserAccessStatusResponse, ENDPOINT_ACCESS_REQUESTS_ALL,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USERNAME,
  KEY_HEADER_BODHIAPP_USER_ID,
};
use axum::{body::Body, http::Request, routing::post, Router};
use objs::test_utils::temp_bodhi_home;
use objs::ResourceRole;
use rstest::rstest;
use serde_json::json;
use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{
  db::{DbService, UserAccessRequest, UserAccessRequestStatus},
  test_utils::{test_db_service_with_temp_dir, AppServiceStubBuilder, SecretServiceStub},
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

  assert_eq!(response.username, request.username);
  assert_eq!(response.status, request.status);
  assert_eq!(response.created_at, request.created_at);
  assert_eq!(response.updated_at, request.updated_at);
}

#[test]
fn test_approve_user_access_request_serde() {
  // Test request deserialization
  let json = r#"{"role": "resource_user"}"#;
  let request: ApproveUserAccessRequest = serde_json::from_str(json).unwrap();
  assert_eq!(request.role, ResourceRole::User);

  let json = r#"{"role": "resource_admin"}"#;
  let request: ApproveUserAccessRequest = serde_json::from_str(json).unwrap();
  assert_eq!(request.role, ResourceRole::Admin);
}

#[test]
fn test_paginated_user_access_response_serde() {
  // Test response serialization
  let response = PaginatedUserAccessResponse {
    requests: vec![],
    total: 0,
    page: 1,
    page_size: 20,
  };

  let json = serde_json::to_string(&response).unwrap();
  assert!(json.contains("\"requests\":[]"));
  assert!(json.contains("\"total\":0"));
  assert!(json.contains("\"page\":1"));
  assert!(json.contains("\"page_size\":20"));
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_clears_user_sessions(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
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
      id: id.clone(),
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
  });

  // 8. Build complete app service
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_sqlite_session_service(session_service.clone())
    .auth_service(Arc::new(mock_auth))
    .secret_service(Arc::new(secret_service))
    .build()?;

  // 9. Create router with approve endpoint
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));

  // 9. Create router with approve endpoint (no auth middleware needed)
  let router = Router::new()
    .route(
      &format!("{}/{{id}}/approve", ENDPOINT_ACCESS_REQUESTS_ALL),
      post(approve_request_handler),
    )
    .with_state(state.clone());

  // 10. Make HTTP request with required headers (simulating authenticated admin)
  let request = Request::post(&format!(
    "{}/{}/approve",
    ENDPOINT_ACCESS_REQUESTS_ALL, access_request.id
  ))
  .header(KEY_HEADER_BODHIAPP_ROLE, "resource_manager")
  .header(KEY_HEADER_BODHIAPP_TOKEN, "dummy-admin-token")
  .header(KEY_HEADER_BODHIAPP_USERNAME, "admin@example.com")
  .header(KEY_HEADER_BODHIAPP_USER_ID, "admin-user-id")
  .header("content-type", "application/json")
  .body(Body::from(serde_json::to_string(
    &json!({ "role": "resource_user" }),
  )?))
  .unwrap();

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
