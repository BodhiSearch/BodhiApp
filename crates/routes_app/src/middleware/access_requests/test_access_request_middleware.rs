use crate::middleware::access_requests::access_request_auth_middleware;
use axum::{
  body::Body,
  http::{Request, Response, StatusCode},
  middleware::from_fn_with_state,
  routing::post,
  Router,
};
use rstest::rstest;
use services::AppService;
use services::AuthContext;
use services::{
  test_utils::{AppServiceStubBuilder, MockDbService, TestDbService, TEST_TENANT_ID},
  {AccessRequestRepository, AppAccessRequestStatus},
};
use services::{ResourceRole, TokenScope, UserScope};
use std::sync::Arc;
use tower::ServiceExt;

async fn test_handler() -> Response<Body> {
  Response::builder()
    .status(StatusCode::OK)
    .body(Body::empty())
    .unwrap()
}

async fn inject_auth_context(
  auth_context: AuthContext,
  mut req: axum::extract::Request,
  next: axum::middleware::Next,
) -> axum::response::Response {
  req.extensions_mut().insert(auth_context);
  next.run(req).await
}

async fn test_mcp_router(auth_context: AuthContext) -> Router {
  let mock_db_service = MockDbService::new();

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(mock_db_service))
    .with_tenant_service()
    .await
    .build()
    .await
    .unwrap();

  let state: Arc<dyn AppService> = Arc::new(app_service);

  let ctx = auth_context.clone();
  Router::new()
    .route(
      "/mcps/{id}/tools/{tool_name}/execute",
      post(test_handler).route_layer(from_fn_with_state(
        state.clone(),
        access_request_auth_middleware,
      )),
    )
    .layer(axum::middleware::from_fn(move |req, next| {
      let ctx = ctx.clone();
      inject_auth_context(ctx, req, next)
    }))
    .with_state(state)
}

async fn test_mcp_router_with_db(
  db_service: Arc<TestDbService>,
  auth_context: AuthContext,
) -> Router {
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service)
    .build()
    .await
    .unwrap();

  let state: Arc<dyn AppService> = Arc::new(app_service);

  let ctx = auth_context.clone();
  Router::new()
    .route(
      "/mcps/{id}/tools/{tool_name}/execute",
      post(test_handler).route_layer(from_fn_with_state(
        state.clone(),
        access_request_auth_middleware,
      )),
    )
    .layer(axum::middleware::from_fn(move |req, next| {
      let ctx = ctx.clone();
      inject_auth_context(ctx, req, next)
    }))
    .with_state(state)
}

#[rstest]
#[tokio::test]
async fn test_mcp_session_auth_passes_through() {
  let ctx = AuthContext::test_session("user123", "user@test.com", ResourceRole::User);
  let app = test_mcp_router(ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/01ARZ3NDEKTSV4RRFFQ69G5FAV/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[rstest]
#[tokio::test]
async fn test_mcp_api_token_passes_through() {
  // API tokens bypass the access-request lifecycle check in this middleware — their
  // per-resource grants are enforced downstream by AccessPolicy. Guards the explicit
  // `AuthContext::ApiToken => Session` bypass arm against accidental removal (which
  // would otherwise 403 every API token reaching /apps/mcps/* with no failing test).
  let ctx = AuthContext::test_api_token("user123", TokenScope::User);
  let app = test_mcp_router(ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/01ARZ3NDEKTSV4RRFFQ69G5FAV/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(StatusCode::OK, response.status());
}

#[rstest]
#[tokio::test]
async fn test_mcp_multi_tenant_session_passes_through() {
  let ctx = AuthContext::test_multi_tenant_session_full(
    "user123",
    "user@test.com",
    "client1",
    "tenant1",
    ResourceRole::User,
    "test-token",
  );
  let app = test_mcp_router(ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/01ARZ3NDEKTSV4RRFFQ69G5FAV/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

/// The middleware now validates only the access-request **lifecycle** (exists +
/// Approved + app/user match). Per-instance authorization moved to AccessPolicy in
/// the handler, so an approved request passes the middleware regardless of which
/// MCP instance is addressed.
#[rstest]
#[case::approved(AppAccessRequestStatus::Approved, StatusCode::OK)]
#[case::denied(AppAccessRequestStatus::Denied, StatusCode::FORBIDDEN)]
// A revoked (inactive) grant must be rejected on every request.
#[case::revoked(AppAccessRequestStatus::Revoked, StatusCode::FORBIDDEN)]
#[tokio::test]
async fn test_mcp_oauth_lifecycle_validation(
  #[case] status: AppAccessRequestStatus,
  #[case] expected_status: StatusCode,
) {
  use services::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::AppAccessRequest {
    id: "ar-uuid".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "app1".to_string(),
    app_name: None,
    app_description: None,
    status,
    requested: r#"{"version":"1","mcp_servers":[{"url":"https://mcp.deepwiki.com/mcp"}]}"#
      .to_string(),
    approved: Some(
      r#"{"version":"1","mcps":[{"url":"https://mcp.deepwiki.com/mcp","status":"approved","instance":{"id":"01ARZ3NDEKTSV4RRFFQ69G5FAV"}}]}"#
        .to_string(),
    ),
    user_id: Some("user123".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    source_access_request_id: None,
    error_message: None,
    expires_at: now + chrono::Duration::hours(1),
    created_at: now,
    updated_at: now,
  };

  test_db.create(&access_request_row).await.unwrap();

  let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
  let app = test_mcp_router_with_db(Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/01ARZ3NDEKTSV4RRFFQ69G5FAV/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), expected_status);
}

#[rstest]
#[tokio::test]
async fn test_mcp_oauth_app_client_mismatch_forbidden() {
  use services::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::AppAccessRequest {
    id: "ar-uuid".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "a-different-app".to_string(),
    app_name: None,
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some("user123".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    source_access_request_id: None,
    error_message: None,
    expires_at: now + chrono::Duration::hours(1),
    created_at: now,
    updated_at: now,
  };

  test_db.create(&access_request_row).await.unwrap();

  // Bearer's app_client_id ("app1") differs from the record's ("a-different-app").
  let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
  let app = test_mcp_router_with_db(Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/01ARZ3NDEKTSV4RRFFQ69G5FAV/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
