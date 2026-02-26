use super::*;
use crate::AuthContext;
use axum::{
  body::Body,
  http::{Request, Response, StatusCode},
  middleware::from_fn_with_state,
  routing::post,
  Router,
};
use objs::{ResourceRole, UserScope};
use rstest::{fixture, rstest};
use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{
  db::AccessRequestRepository,
  test_utils::{AppServiceStubBuilder, MockDbService, TestDbService},
};
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

#[fixture]
fn toolset_validator() -> Arc<dyn AccessRequestValidator> {
  Arc::new(ToolsetAccessRequestValidator)
}

async fn test_router(
  validator: Arc<dyn AccessRequestValidator>,
  auth_context: AuthContext,
) -> Router {
  let mock_db_service = MockDbService::new();

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(mock_db_service))
    .with_app_instance_service()
    .await
    .build()
    .await
    .unwrap();

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));

  let ctx = auth_context.clone();
  let v = validator.clone();
  Router::new()
    .route(
      "/toolsets/{id}/execute/{method}",
      post(test_handler).route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          let v = v.clone();
          access_request_auth_middleware(v, state, req, next)
        },
      )),
    )
    .layer(axum::middleware::from_fn(move |req, next| {
      let ctx = ctx.clone();
      inject_auth_context(ctx, req, next)
    }))
    .with_state(state)
}

async fn test_router_with_db(
  validator: Arc<dyn AccessRequestValidator>,
  db_service: Arc<TestDbService>,
  auth_context: AuthContext,
) -> Router {
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service)
    .build()
    .await
    .unwrap();

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));

  let ctx = auth_context.clone();
  let v = validator.clone();
  Router::new()
    .route(
      "/toolsets/{id}/execute/{method}",
      post(test_handler).route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          let v = v.clone();
          access_request_auth_middleware(v, state, req, next)
        },
      )),
    )
    .layer(axum::middleware::from_fn(move |req, next| {
      let ctx = ctx.clone();
      inject_auth_context(ctx, req, next)
    }))
    .with_state(state)
}

// Session auth: passthrough (no access request checks)
#[rstest]
#[tokio::test]
async fn test_session_auth_passes_through(toolset_validator: Arc<dyn AccessRequestValidator>) {
  let ctx = AuthContext::test_session("user123", "user@test.com", ResourceRole::User);
  let app = test_router(toolset_validator, ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[rstest]
#[tokio::test]
async fn test_missing_auth(toolset_validator: Arc<dyn AccessRequestValidator>) {
  let ctx = AuthContext::Anonymous;
  let app = test_router(toolset_validator, ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// OAuth access request validation tests
#[rstest]
#[case::oauth_approved_instance_in_list(
  "approved",
  Some(r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#.to_string()),
  StatusCode::OK,
)]
#[case::oauth_denied("denied", None, StatusCode::FORBIDDEN)]
#[case::oauth_draft("draft", None, StatusCode::FORBIDDEN)]
#[case::oauth_not_in_approved_list(
  "approved",
  Some(r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"different-toolset-id"}}]}"#.to_string()),
  StatusCode::FORBIDDEN,
)]
#[tokio::test]
async fn test_oauth_access_request_validation(
  toolset_validator: Arc<dyn AccessRequestValidator>,
  #[case] status: &str,
  #[case] approved: Option<String>,
  #[case] expected_status: StatusCode,
) {
  use objs::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::db::AppAccessRequestRow {
    id: "ar-uuid".to_string(),
    app_client_id: "app1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: status.to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved,
    user_id: Some("user123".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: (now + chrono::Duration::hours(1)).timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  test_db.create(&access_request_row).await.unwrap();

  let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
  let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), expected_status);
}

#[rstest]
#[tokio::test]
async fn test_oauth_app_client_mismatch(toolset_validator: Arc<dyn AccessRequestValidator>) {
  use objs::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::db::AppAccessRequestRow {
    id: "ar-uuid".to_string(),
    app_client_id: "app1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#
        .to_string(),
    ),
    user_id: Some("user123".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: (now + chrono::Duration::hours(1)).timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  test_db.create(&access_request_row).await.unwrap();

  let ctx = AuthContext::test_external_app("user123", UserScope::User, "app2", Some("ar-uuid"));
  let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[rstest]
#[tokio::test]
async fn test_oauth_user_mismatch(toolset_validator: Arc<dyn AccessRequestValidator>) {
  use objs::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::db::AppAccessRequestRow {
    id: "ar-uuid".to_string(),
    app_client_id: "app1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
    approved: Some(
      r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#
        .to_string(),
    ),
    user_id: Some("user1".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: (now + chrono::Duration::hours(1)).timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  test_db.create(&access_request_row).await.unwrap();

  let ctx = AuthContext::test_external_app("user2", UserScope::User, "app1", Some("ar-uuid"));
  let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[rstest]
#[tokio::test]
async fn test_oauth_auto_approved_no_toolsets(toolset_validator: Arc<dyn AccessRequestValidator>) {
  use objs::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::db::AppAccessRequestRow {
    id: "ar-uuid".to_string(),
    app_client_id: "app1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: None,
    user_id: Some("user123".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: (now + chrono::Duration::hours(1)).timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  test_db.create(&access_request_row).await.unwrap();

  let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
  let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[rstest]
#[tokio::test]
async fn test_oauth_access_request_not_found(toolset_validator: Arc<dyn AccessRequestValidator>) {
  use objs::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;

  let ctx = AuthContext::test_external_app(
    "user123",
    UserScope::User,
    "app1",
    Some("ar-uuid-nonexistent"),
  );
  let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ============================================================================
// MCP validator tests
// ============================================================================

#[fixture]
fn mcp_validator() -> Arc<dyn AccessRequestValidator> {
  Arc::new(McpAccessRequestValidator)
}

async fn test_mcp_router(
  validator: Arc<dyn AccessRequestValidator>,
  auth_context: AuthContext,
) -> Router {
  let mock_db_service = MockDbService::new();

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(mock_db_service))
    .with_app_instance_service()
    .await
    .build()
    .await
    .unwrap();

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));

  let ctx = auth_context.clone();
  let v = validator.clone();
  Router::new()
    .route(
      "/mcps/{id}/tools/{tool_name}/execute",
      post(test_handler).route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          let v = v.clone();
          access_request_auth_middleware(v, state, req, next)
        },
      )),
    )
    .layer(axum::middleware::from_fn(move |req, next| {
      let ctx = ctx.clone();
      inject_auth_context(ctx, req, next)
    }))
    .with_state(state)
}

async fn test_mcp_router_with_db(
  validator: Arc<dyn AccessRequestValidator>,
  db_service: Arc<TestDbService>,
  auth_context: AuthContext,
) -> Router {
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service)
    .build()
    .await
    .unwrap();

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));

  let ctx = auth_context.clone();
  let v = validator.clone();
  Router::new()
    .route(
      "/mcps/{id}/tools/{tool_name}/execute",
      post(test_handler).route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          let v = v.clone();
          access_request_auth_middleware(v, state, req, next)
        },
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
async fn test_mcp_session_auth_passes_through(mcp_validator: Arc<dyn AccessRequestValidator>) {
  let ctx = AuthContext::test_session("user123", "user@test.com", ResourceRole::User);
  let app = test_mcp_router(mcp_validator, ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/550e8400-e29b-41d4-a716-446655440000/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[rstest]
#[case::mcp_approved_instance_in_list(
  "approved",
  Some(r#"{"toolsets":[],"mcps":[{"url":"https://mcp.deepwiki.com/mcp","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#.to_string()),
  StatusCode::OK,
)]
#[case::mcp_denied("denied", None, StatusCode::FORBIDDEN)]
#[case::mcp_not_in_approved_list(
  "approved",
  Some(r#"{"toolsets":[],"mcps":[{"url":"https://mcp.deepwiki.com/mcp","status":"approved","instance":{"id":"different-mcp-id"}}]}"#.to_string()),
  StatusCode::FORBIDDEN,
)]
#[tokio::test]
async fn test_mcp_oauth_access_request_validation(
  mcp_validator: Arc<dyn AccessRequestValidator>,
  #[case] status: &str,
  #[case] approved: Option<String>,
  #[case] expected_status: StatusCode,
) {
  use objs::test_utils::temp_dir;
  use services::test_utils::test_db_service_with_temp_dir;

  let temp_dir = Arc::new(temp_dir());
  let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
  let now = test_db.now();

  let access_request_row = services::db::AppAccessRequestRow {
    id: "ar-uuid".to_string(),
    app_client_id: "app1".to_string(),
    app_name: None,
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: status.to_string(),
    requested: r#"{"mcp_servers":[{"url":"https://mcp.deepwiki.com/mcp"}]}"#.to_string(),
    approved,
    user_id: Some("user123".to_string()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: None,
    error_message: None,
    expires_at: (now + chrono::Duration::hours(1)).timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  test_db.create(&access_request_row).await.unwrap();

  let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
  let app = test_mcp_router_with_db(mcp_validator, Arc::new(test_db), ctx).await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/mcps/550e8400-e29b-41d4-a716-446655440000/tools/read/execute")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), expected_status);
}
