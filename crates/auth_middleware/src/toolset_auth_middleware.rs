use crate::{
  KEY_HEADER_BODHIAPP_AZP, KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE,
  KEY_HEADER_BODHIAPP_TOOL_SCOPES, KEY_HEADER_BODHIAPP_USER_ID,
};
use axum::{
  extract::{Path, Request, State},
  middleware::Next,
  response::Response,
};
use objs::{ApiError, AppError, ErrorType, ToolsetScope};
use server_core::RouterState;
use services::ToolsetError;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetAuthError {
  #[error("User identification missing from request.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingUserId,

  #[error("Authentication required for toolset access.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,

  #[error("app_client_not_registered")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AppClientNotRegistered,

  #[error("missing_toolset_scope")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  MissingToolsetScope,

  #[error("missing_azp_header")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  MissingAzpHeader,

  #[error("Toolset not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolsetNotFound,

  #[error(transparent)]
  ToolsetError(#[from] ToolsetError),
}

/// Middleware for toolset execution endpoints
///
/// Authorization rules depend on auth type:
/// - Session (has ROLE header): Check toolset ownership + app-level type enabled + toolset available
/// - External OAuth (has SCOPE starting with "scope_user_"): Check toolset ownership + app-level type enabled + app-client registered + scope + toolset available
///
/// Note: API tokens (bodhiapp_*) are blocked at route level and won't reach this middleware.
pub async fn toolset_auth_middleware(
  State(state): State<Arc<dyn RouterState>>,
  Path((id, method)): Path<(String, String)>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(_impl(State(state), Path((id, method)), req, next).await?)
}

async fn _impl(
  State(state): State<Arc<dyn RouterState>>,
  Path((id, _method)): Path<(String, String)>,
  req: Request,
  next: Next,
) -> Result<Response, ToolsetAuthError> {
  let headers = req.headers();

  // Extract user_id
  let user_id = headers
    .get(KEY_HEADER_BODHIAPP_USER_ID)
    .and_then(|v| v.to_str().ok())
    .ok_or(ToolsetAuthError::MissingUserId)?;

  // Determine auth type:
  // - Session auth: has ROLE header (from session tokens)
  // - OAuth (external app): has SCOPE header starting with "scope_user_" (from OAuth tokens)
  //
  // Note: API tokens (scope_token_*) are blocked at route level and won't reach here.
  let is_session_auth = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE);

  let scope_header = headers
    .get(KEY_HEADER_BODHIAPP_SCOPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

  let is_oauth_auth = scope_header.starts_with("scope_user_") && !is_session_auth;

  if !is_session_auth && !is_oauth_auth {
    return Err(ToolsetAuthError::MissingAuth);
  }

  let tool_service = state.app_service().tool_service();

  // 1. Get toolset and verify ownership (returns None if not found OR not owned)
  let toolset = tool_service
    .get(user_id, &id)
    .await?
    .ok_or(ToolsetAuthError::ToolsetNotFound)?;

  let scope_uuid = &toolset.scope_uuid;

  // 2. Check app-level type enabled (both auth types)
  if !tool_service.is_type_enabled(scope_uuid).await? {
    return Err(ToolsetError::ToolsetAppDisabled.into());
  }

  // For OAuth (external apps), additional checks are required
  if is_oauth_auth {
    // 3. Check app-client registered for toolset scope_uuid
    let azp = headers
      .get(KEY_HEADER_BODHIAPP_AZP)
      .and_then(|v| v.to_str().ok())
      .ok_or(ToolsetAuthError::MissingAzpHeader)?;

    if !tool_service
      .is_app_client_registered_for_toolset(azp, scope_uuid)
      .await?
    {
      return Err(ToolsetAuthError::AppClientNotRegistered);
    }

    // 4. Check scope_toolset-{type} in token
    let toolset_scopes_header = headers
      .get(KEY_HEADER_BODHIAPP_TOOL_SCOPES)
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");

    // Get required scope from toolset
    let required_scope = ToolsetScope::from_str(&toolset.scope)
      .map_err(|_| ToolsetError::ToolsetNotFound(toolset.scope_uuid.clone()))?;

    // Check if required scope is present (space-separated)
    let has_scope = toolset_scopes_header
      .split_whitespace()
      .any(|s| s == required_scope.to_string());

    if !has_scope {
      return Err(ToolsetAuthError::MissingToolsetScope);
    }
  }

  // 5. Check toolset is available (has API key and is enabled)
  if !toolset.enabled || !toolset.has_api_key {
    return Err(ToolsetError::ToolsetNotConfigured.into());
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use super::*;
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
  use services::{test_utils::AppServiceStubBuilder, MockToolService};
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn test_handler() -> Response<Body> {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::empty())
      .unwrap()
  }

  fn test_router_with_tool_service(mock_tool_service: MockToolService) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .with_tool_service(Arc::new(mock_tool_service))
      .build()
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    Router::new()
      .route(
        "/toolsets/{id}/execute/{method}",
        post(test_handler).route_layer(from_fn_with_state(state.clone(), toolset_auth_middleware)),
      )
      .with_state(state)
  }

  #[fixture]
  fn test_instance() -> Toolset {
    let now = Utc::now();
    Toolset {
      id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
      name: "My Exa Search".to_string(),
      scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
      scope: "scope_toolset-builtin-exa-web-search".to_string(),
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

    // Setup expectations
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
        .withf(|scope_uuid| scope_uuid == "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
        .times(1)
        .returning(move |_| Ok(type_enabled));
    }

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", instance_id_for_uri))
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), expected_status);
  }

  // OAuth auth tests
  #[rstest]
  #[case::success(true, true, true, true, StatusCode::OK)]
  #[case::instance_not_found(false, false, false, false, StatusCode::NOT_FOUND)]
  #[case::type_disabled(true, false, false, false, StatusCode::BAD_REQUEST)]
  #[case::app_client_not_registered(true, true, false, false, StatusCode::FORBIDDEN)]
  #[case::missing_scope(true, true, true, false, StatusCode::FORBIDDEN)]
  #[tokio::test]
  async fn test_oauth_auth(
    test_instance: Toolset,
    #[case] get_returns_instance: bool,
    #[case] type_enabled: bool,
    #[case] client_registered: bool,
    #[case] has_scope: bool,
    #[case] expected_status: StatusCode,
  ) {
    let mut mock_tool_service = MockToolService::new();
    let instance_id = test_instance.id.clone();
    let instance_id_for_uri = test_instance.id.clone();
    let instance_clone = test_instance.clone();

    // Setup expectations
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
        .withf(|scope_uuid| scope_uuid == "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
        .times(1)
        .returning(move |_| Ok(type_enabled));

      if type_enabled {
        mock_tool_service
          .expect_is_app_client_registered_for_toolset()
          .withf(|app_client_id, scope_uuid| {
            app_client_id == "external-app" && scope_uuid == "4ff0e163-36fb-47d6-a5ef-26e396f067d6"
          })
          .times(1)
          .returning(move |_, _| Ok(client_registered));
      }
    }

    let app = test_router_with_tool_service(mock_tool_service);

    let mut builder = Request::builder()
      .method("POST")
      .uri(format!("/toolsets/{}/execute/search", instance_id_for_uri))
      .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
      .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
      .header(KEY_HEADER_BODHIAPP_AZP, "external-app");

    if has_scope {
      builder = builder.header(
        KEY_HEADER_BODHIAPP_TOOL_SCOPES,
        "scope_toolset-builtin-exa-web-search",
      );
    }

    let response = app
      .oneshot(builder.body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), expected_status);
  }

  // Error condition tests
  #[rstest]
  #[tokio::test]
  async fn test_missing_user_id(test_instance: Toolset) {
    let mock_tool_service = MockToolService::new();
    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", test_instance.id))
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  #[rstest]
  #[tokio::test]
  async fn test_missing_auth(test_instance: Toolset) {
    let mock_tool_service = MockToolService::new();
    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri(format!("/toolsets/{}/execute/search", test_instance.id))
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }
}
