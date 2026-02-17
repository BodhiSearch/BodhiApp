use crate::AuthContext;
use axum::{
  body::Body,
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use objs::{ApiError, AppError, ErrorType};
use server_core::RouterState;
use services::ToolsetError;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetAuthError {
  #[error("Authentication required for toolset access.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,

  #[error("Toolset not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolsetNotFound,

  #[error("Access request {access_request_id} not found.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestNotFound { access_request_id: String },

  #[error("Access request {access_request_id} has status '{status}'. Only approved requests can access toolsets.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestNotApproved {
    access_request_id: String,
    status: String,
  },

  #[error("Access request {access_request_id} is invalid: {reason}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestInvalid {
    access_request_id: String,
    reason: String,
  },

  #[error("Toolset {toolset_id} is not included in your approved tools for this app.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  ToolsetNotApproved { toolset_id: String },

  #[error("Access request app client ID mismatch: expected {expected}, found {found}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AppClientMismatch { expected: String, found: String },

  #[error("Access request user ID mismatch: expected {expected}, found {found}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  UserMismatch { expected: String, found: String },

  #[error("Invalid approved JSON in access request: {error}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidApprovedJson { error: String },

  #[error(transparent)]
  ToolsetError(#[from] ToolsetError),
}

/// Middleware for toolset execution endpoints
///
/// Authorization rules:
/// - Session (has role): Check toolset ownership + app-level type enabled + toolset available
/// - OAuth (ExternalApp with access_request_id): Validate access request + instance authorization + toolset configuration
///
/// Note:
/// - API tokens (bodhiapp_*) are blocked at route level and won't reach this middleware.
pub async fn toolset_auth_middleware(
  State(state): State<Arc<dyn RouterState>>,
  req: Request<Body>,
  next: Next,
) -> Result<Response, ApiError> {
  let auth_context = req
    .extensions()
    .get::<AuthContext>()
    .ok_or(ToolsetAuthError::MissingAuth)?
    .clone();

  // Extract user_id - required for all flows
  let user_id = auth_context
    .user_id()
    .ok_or(ToolsetAuthError::MissingAuth)?
    .to_string();

  // Extract toolset ID from path
  let id = req
    .uri()
    .path()
    .split('/')
    .find(|seg| seg.len() == 36 && seg.contains('-'))
    .ok_or(ToolsetAuthError::ToolsetNotFound)?
    .to_string();

  let tool_service = state.app_service().tool_service();
  let db_service = state.app_service().db_service();

  // Determine auth flow type from AuthContext variant
  let (is_session, is_oauth) = match &auth_context {
    AuthContext::Session { .. } => (true, false),
    AuthContext::ExternalApp {
      access_request_id, ..
    } => (false, access_request_id.is_some()),
    _ => (false, false),
  };

  if !is_session && !is_oauth {
    return Err(ToolsetAuthError::MissingAuth.into());
  }

  // BOTH FLOWS: Verify toolset exists and get type
  let toolset = tool_service
    .get(&user_id, &id)
    .await?
    .ok_or(ToolsetAuthError::ToolsetNotFound)?;

  // OAUTH FLOW: Access request validation
  if let AuthContext::ExternalApp {
    access_request_id: Some(ref ar_id),
    ref azp,
    ..
  } = auth_context
  {
    // Fetch access request
    let access_request =
      db_service
        .get(ar_id)
        .await?
        .ok_or(ToolsetAuthError::AccessRequestNotFound {
          access_request_id: ar_id.clone(),
        })?;

    // Validate status
    if access_request.status != "approved" {
      return Err(
        ToolsetAuthError::AccessRequestNotApproved {
          access_request_id: ar_id.clone(),
          status: access_request.status,
        }
        .into(),
      );
    }

    // Validate app_client_id matches token azp
    if access_request.app_client_id != azp.as_str() {
      return Err(
        ToolsetAuthError::AppClientMismatch {
          expected: access_request.app_client_id,
          found: azp.clone(),
        }
        .into(),
      );
    }

    // Validate user_id matches (must be present for approved requests)
    let ar_user_id =
      access_request
        .user_id
        .as_ref()
        .ok_or(ToolsetAuthError::AccessRequestInvalid {
          access_request_id: ar_id.clone(),
          reason: "Missing user_id in approved access request".to_string(),
        })?;

    if ar_user_id != &user_id {
      return Err(
        ToolsetAuthError::UserMismatch {
          expected: ar_user_id.clone(),
          found: user_id.clone(),
        }
        .into(),
      );
    }

    // Validate toolset instance in approved list
    if let Some(approved_json) = &access_request.approved {
      let approvals: serde_json::Value =
        serde_json::from_str(approved_json).map_err(|e| ToolsetAuthError::InvalidApprovedJson {
          error: e.to_string(),
        })?;

      let toolset_types = approvals
        .get("toolset_types")
        .and_then(|v| v.as_array())
        .ok_or(ToolsetAuthError::InvalidApprovedJson {
          error: "Missing toolset_types array".to_string(),
        })?;

      let instance_approved = toolset_types.iter().any(|approval| {
        approval.get("status").and_then(|s| s.as_str()) == Some("approved")
          && approval.get("instance_id").and_then(|i| i.as_str()) == Some(&id)
      });

      if !instance_approved {
        return Err(ToolsetAuthError::ToolsetNotApproved { toolset_id: id }.into());
      }
    } else {
      // approved is NULL - auto-approved request with no toolsets
      return Err(ToolsetAuthError::ToolsetNotApproved { toolset_id: id }.into());
    }
  }

  // BOTH FLOWS: Verify app-level type enabled
  if !tool_service.is_type_enabled(&toolset.toolset_type).await? {
    return Err(ToolsetError::ToolsetAppDisabled.into());
  }

  // BOTH FLOWS: Verify instance configured
  if !toolset.enabled {
    return Err(ToolsetError::ToolsetNotConfigured.into());
  }

  if !toolset.has_api_key {
    return Err(ToolsetError::ToolsetNotConfigured.into());
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::AuthContext;
  use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::from_fn_with_state,
    routing::post,
    Router,
  };
  use chrono::Utc;
  use objs::{ResourceRole, Toolset, UserScope};
  use rstest::{fixture, rstest};
  use server_core::{DefaultRouterState, MockSharedContext};
  use services::{
    db::AccessRequestRepository,
    test_utils::{AppServiceStubBuilder, MockDbService, TestDbService},
    MockToolService,
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn test_handler() -> Response<Body> {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::empty())
      .unwrap()
  }

  /// Helper middleware that injects AuthContext into request extensions
  async fn inject_auth_context(
    auth_context: AuthContext,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
  ) -> axum::response::Response {
    req.extensions_mut().insert(auth_context);
    next.run(req).await
  }

  async fn test_router_with_tool_service(
    mock_tool_service: MockToolService,
    auth_context: AuthContext,
  ) -> Router {
    let mock_db_service = MockDbService::new();

    let app_service = AppServiceStubBuilder::default()
      .with_tool_service(Arc::new(mock_tool_service))
      .db_service(Arc::new(mock_db_service))
      .build()
      .await
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let ctx = auth_context.clone();
    Router::new()
      .route(
        "/toolsets/{id}/execute/{method}",
        post(test_handler).route_layer(from_fn_with_state(state.clone(), toolset_auth_middleware)),
      )
      .layer(axum::middleware::from_fn(move |req, next| {
        let ctx = ctx.clone();
        inject_auth_context(ctx, req, next)
      }))
      .with_state(state)
  }

  #[fixture]
  fn test_instance() -> Toolset {
    let now = Utc::now();
    Toolset {
      id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
      slug: "my-exa-search".to_string(),
      toolset_type: "builtin-exa-search".to_string(),
      description: Some("Test instance".to_string()),
      enabled: true,
      has_api_key: true,
      created_at: now,
      updated_at: now,
    }
  }

  // Session auth tests
  #[rstest]
  #[case::success(true, true, true, true, StatusCode::OK)]
  #[case::instance_not_found(false, false, false, false, StatusCode::NOT_FOUND)]
  #[case::type_disabled(true, false, true, true, StatusCode::BAD_REQUEST)]
  #[case::instance_disabled(true, true, false, true, StatusCode::BAD_REQUEST)]
  #[case::instance_no_api_key(true, true, true, false, StatusCode::BAD_REQUEST)]
  #[tokio::test]
  async fn test_session_auth(
    test_instance: Toolset,
    #[case] get_returns_instance: bool,
    #[case] type_enabled: bool,
    #[case] instance_enabled: bool,
    #[case] instance_has_api_key: bool,
    #[case] expected_status: StatusCode,
  ) {
    let mut mock_tool_service = MockToolService::new();
    let instance_id = test_instance.id.clone();
    let instance_id_for_uri = test_instance.id.clone();
    let mut instance_clone = test_instance.clone();
    instance_clone.enabled = instance_enabled;
    instance_clone.has_api_key = instance_has_api_key;

    mock_tool_service
      .expect_get()
      .withf(move |user_id, id| user_id == "user123" && id == &instance_id)
      .times(1)
      .returning(move |_, _| {
        if get_returns_instance {
          Ok(Some(instance_clone.clone()))
        } else {
          Ok(None)
        }
      });

    if get_returns_instance {
      mock_tool_service
        .expect_is_type_enabled()
        .withf(|tool_type| tool_type == "builtin-exa-search")
        .times(1)
        .returning(move |_| Ok(type_enabled));
    }

    let ctx = AuthContext::test_session("user123", "user@test.com", ResourceRole::User);
    let app = test_router_with_tool_service(mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id_for_uri))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), expected_status);
  }

  #[rstest]
  #[tokio::test]
  async fn test_missing_auth(test_instance: Toolset) {
    let mock_tool_service = MockToolService::new();
    let ctx = AuthContext::Anonymous;
    let app = test_router_with_tool_service(mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", test_instance.id))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  // OAuth access request validation tests
  async fn test_router_with_db_and_tool_service(
    db_service: Arc<TestDbService>,
    mock_tool_service: MockToolService,
    auth_context: AuthContext,
  ) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .with_tool_service(Arc::new(mock_tool_service))
      .db_service(db_service)
      .build()
      .await
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let ctx = auth_context.clone();
    Router::new()
      .route(
        "/toolsets/{id}/execute/{method}",
        post(test_handler).route_layer(from_fn_with_state(state.clone(), toolset_auth_middleware)),
      )
      .layer(axum::middleware::from_fn(move |req, next| {
        let ctx = ctx.clone();
        inject_auth_context(ctx, req, next)
      }))
      .with_state(state)
  }

  #[rstest]
  #[case::oauth_approved_instance_in_list("approved", Some(r#"{"toolset_types":[{"toolset_type":"builtin-exa-search","status":"approved","instance_id":"550e8400-e29b-41d4-a716-446655440000"}]}"#.to_string()), StatusCode::OK, false)]
  #[case::oauth_denied("denied", None, StatusCode::FORBIDDEN, false)]
  #[case::oauth_draft("draft", None, StatusCode::FORBIDDEN, false)]
  #[case::oauth_expired("approved", Some(r#"{"toolset_types":[{"toolset_type":"builtin-exa-search","status":"approved","instance_id":"550e8400-e29b-41d4-a716-446655440000"}]}"#.to_string()), StatusCode::OK, true)]
  #[case::oauth_not_in_approved_list("approved", Some(r#"{"toolset_types":[{"toolset_type":"builtin-exa-search","status":"approved","instance_id":"different-toolset-id"}]}"#.to_string()), StatusCode::FORBIDDEN, false)]
  #[tokio::test]
  async fn test_oauth_access_request_validation(
    test_instance: Toolset,
    #[case] status: &str,
    #[case] approved: Option<String>,
    #[case] expected_status: StatusCode,
    #[case] is_expired: bool,
  ) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    // Adjust expires_at for expired case
    let expires_at = if is_expired {
      (now - chrono::Duration::hours(1)).timestamp()
    } else {
      (now + chrono::Duration::hours(1)).timestamp()
    };

    // Create access request record
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
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at,
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    // Setup mock tool service
    let instance_id = test_instance.id.clone();
    let mut mock_tool_service = MockToolService::new();
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_get()
      .withf(move |user_id, id| user_id == "user123" && id == &instance_clone.id)
      .times(1)
      .returning(move |_, _| Ok(Some(test_instance.clone())));

    // Only expect is_type_enabled for cases that pass OAuth validation
    if status == "approved" && expected_status == StatusCode::OK {
      mock_tool_service
        .expect_is_type_enabled()
        .withf(|tool_type| tool_type == "builtin-exa-search")
        .times(1)
        .returning(|_| Ok(true));
    }

    let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
    let app = test_router_with_db_and_tool_service(Arc::new(test_db), mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), expected_status);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_app_client_mismatch(test_instance: Toolset) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    // Create access request with app_client_id = "app1"
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
        r#"{"toolset_types":[{"toolset_type":"builtin-exa-search","status":"approved","instance_id":"550e8400-e29b-41d4-a716-446655440000"}]}"#
          .to_string(),
      ),
      user_id: Some("user123".to_string()),
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    // Setup mock tool service
    let instance_id = test_instance.id.clone();
    let mut mock_tool_service = MockToolService::new();
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_get()
      .withf(move |user_id, id| user_id == "user123" && id == &instance_clone.id)
      .times(1)
      .returning(move |_, _| Ok(Some(test_instance.clone())));

    // Send request with azp = "app2" (mismatch)
    let ctx = AuthContext::test_external_app("user123", UserScope::User, "app2", Some("ar-uuid"));
    let app = test_router_with_db_and_tool_service(Arc::new(test_db), mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_user_mismatch(test_instance: Toolset) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    // Create access request with user_id = "user1"
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
        r#"{"toolset_types":[{"toolset_type":"builtin-exa-search","status":"approved","instance_id":"550e8400-e29b-41d4-a716-446655440000"}]}"#
          .to_string(),
      ),
      user_id: Some("user1".to_string()),
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    // Setup mock tool service
    let instance_id = test_instance.id.clone();
    let mut mock_tool_service = MockToolService::new();
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_get()
      .withf(move |user_id, id| user_id == "user2" && id == &instance_clone.id)
      .times(1)
      .returning(move |_, _| Ok(Some(test_instance.clone())));

    // Send request with user_id = "user2" (mismatch)
    let ctx = AuthContext::test_external_app("user2", UserScope::User, "app1", Some("ar-uuid"));
    let app = test_router_with_db_and_tool_service(Arc::new(test_db), mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_auto_approved_no_toolsets(test_instance: Toolset) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    // Create access request with approved = NULL (auto-approved with no tools)
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
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    // Setup mock tool service
    let instance_id = test_instance.id.clone();
    let mut mock_tool_service = MockToolService::new();
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_get()
      .withf(move |user_id, id| user_id == "user123" && id == &instance_clone.id)
      .times(1)
      .returning(move |_, _| Ok(Some(test_instance.clone())));

    let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
    let app = test_router_with_db_and_tool_service(Arc::new(test_db), mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_access_request_not_found(test_instance: Toolset) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;

    // Don't create any access request record

    // Setup mock tool service
    let instance_id = test_instance.id.clone();
    let mut mock_tool_service = MockToolService::new();
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_get()
      .withf(move |user_id, id| user_id == "user123" && id == &instance_clone.id)
      .times(1)
      .returning(move |_, _| Ok(Some(test_instance.clone())));

    let ctx = AuthContext::test_external_app(
      "user123",
      UserScope::User,
      "app1",
      Some("ar-uuid-nonexistent"),
    );
    let app = test_router_with_db_and_tool_service(Arc::new(test_db), mock_tool_service, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }
}
