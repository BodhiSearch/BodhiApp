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
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetAuthError {
  #[error("missing_user_id")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingUserId,

  #[error("missing_auth")]
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

  #[error(transparent)]
  ToolsetError(#[from] ToolsetError),
}

/// Middleware for toolset execution endpoints
///
/// Authorization rules depend on auth type:
/// - Session/First-party (has ROLE header): Check app-level + user config
/// - External OAuth (has SCOPE but no ROLE): Check app-level + app-client + scope + user config
pub async fn toolset_auth_middleware(
  State(state): State<Arc<dyn RouterState>>,
  Path(toolset_id): Path<String>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(_impl(State(state), Path(toolset_id), req, next).await?)
}

async fn _impl(
  State(state): State<Arc<dyn RouterState>>,
  Path(toolset_id): Path<String>,
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
  // - First-party token: has SCOPE header starting with "scope_token_" (from bodhiapp_ API keys)
  // - OAuth (external app): has SCOPE header starting with "scope_user_" (from OAuth tokens)
  let is_session_auth = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE);

  let scope_header = headers
    .get(KEY_HEADER_BODHIAPP_SCOPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

  let is_first_party_token = scope_header.starts_with("scope_token_");
  let is_oauth_auth = scope_header.starts_with("scope_user_") && !is_session_auth;

  if !is_session_auth && !is_first_party_token && !is_oauth_auth {
    return Err(ToolsetAuthError::MissingAuth);
  }

  let tool_service = state.app_service().tool_service();

  // 1. Check app-level enabled (both auth types)
  if !tool_service.is_toolset_enabled_for_app(&toolset_id).await? {
    return Err(ToolsetError::ToolsetAppDisabled.into());
  }

  // For OAuth (external apps), additional checks are required
  if is_oauth_auth {
    // 2. Check app-client registered for toolset
    let azp = headers
      .get(KEY_HEADER_BODHIAPP_AZP)
      .and_then(|v| v.to_str().ok())
      .ok_or(ToolsetAuthError::MissingAzpHeader)?;

    if !tool_service
      .is_app_client_registered_for_toolset(azp, &toolset_id)
      .await?
    {
      return Err(ToolsetAuthError::AppClientNotRegistered);
    }

    // 3. Check scope_toolset-* in token
    let toolset_scopes_header = headers
      .get(KEY_HEADER_BODHIAPP_TOOL_SCOPES)
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");

    // Get required scope for this toolset
    let required_scope = ToolsetScope::scope_for_toolset_id(&toolset_id)
      .ok_or_else(|| ToolsetError::ToolsetNotFound(toolset_id.clone()))?;

    // Check if required scope is present (space-separated)
    let has_scope = toolset_scopes_header
      .split_whitespace()
      .any(|s| s == required_scope.to_string());

    if !has_scope {
      return Err(ToolsetAuthError::MissingToolsetScope);
    }
  }

  // 4. Check user has toolset configured (API key required for execution)
  // For session auth, this also checks user_enabled
  // For OAuth auth, we only need to check API key exists
  let is_available = tool_service
    .is_toolset_available_for_user(user_id, &toolset_id)
    .await?;

  if !is_available {
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
  use objs::{
    test_utils::setup_l10n, FluentLocalizationService, ResourceRole, TokenScope, UserScope,
  };
  use rstest::rstest;
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
        "/toolsets/{toolset_id}/execute",
        post(test_handler).route_layer(from_fn_with_state(state.clone(), toolset_auth_middleware)),
      )
      .with_state(state)
  }

  #[rstest]
  #[tokio::test]
  async fn test_session_auth_toolset_available(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    // Session auth needs: app enabled + user available
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_toolset_available_for_user()
      .withf(|user_id, toolset_id| user_id == "user123" && toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(true));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[rstest]
  #[tokio::test]
  async fn test_session_auth_toolset_not_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_toolset_available_for_user()
      .withf(|user_id, toolset_id| user_id == "user123" && toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(false));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[rstest]
  #[tokio::test]
  async fn test_session_auth_app_disabled(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(false));
    // Should not reach is_toolset_available_for_user since app check fails

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[rstest]
  #[tokio::test]
  async fn test_first_party_token_auth_toolset_available(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    // First-party token (TokenScope) is treated same as session auth
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_toolset_available_for_user()
      .withf(|user_id, toolset_id| user_id == "user123" && toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(true));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, TokenScope::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[rstest]
  #[tokio::test]
  async fn test_first_party_token_auth_toolset_not_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_toolset_available_for_user()
      .withf(|user_id, toolset_id| user_id == "user123" && toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(false));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, TokenScope::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[rstest]
  #[tokio::test]
  async fn test_first_party_token_auth_app_disabled(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(false));
    // Should not reach is_toolset_available_for_user since app check fails

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, TokenScope::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_token_toolset_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    // OAuth needs: app enabled + app-client registered + scope + user available
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_app_client_registered_for_toolset()
      .withf(|app_client_id, toolset_id| {
        app_client_id == "external-app" && toolset_id == "builtin-exa-web-search"
      })
      .times(1)
      .returning(|_, _| Ok(true));
    mock_tool_service
      .expect_is_toolset_available_for_user()
      .withf(|user_id, toolset_id| user_id == "user123" && toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(true));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
          .header(KEY_HEADER_BODHIAPP_AZP, "external-app")
          .header(
            KEY_HEADER_BODHIAPP_TOOL_SCOPES,
            "scope_toolset-builtin-exa-web-search",
          )
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_token_toolset_not_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_app_client_registered_for_toolset()
      .withf(|app_client_id, toolset_id| {
        app_client_id == "external-app" && toolset_id == "builtin-exa-web-search"
      })
      .times(1)
      .returning(|_, _| Ok(true));
    mock_tool_service
      .expect_is_toolset_available_for_user()
      .withf(|user_id, toolset_id| user_id == "user123" && toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(false));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
          .header(KEY_HEADER_BODHIAPP_AZP, "external-app")
          .header(
            KEY_HEADER_BODHIAPP_TOOL_SCOPES,
            "scope_toolset-builtin-exa-web-search",
          )
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_token_app_client_not_registered(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_app_client_registered_for_toolset()
      .withf(|app_client_id, toolset_id| {
        app_client_id == "unregistered-app" && toolset_id == "builtin-exa-web-search"
      })
      .times(1)
      .returning(|_, _| Ok(false));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
          .header(KEY_HEADER_BODHIAPP_AZP, "unregistered-app")
          .header(
            KEY_HEADER_BODHIAPP_TOOL_SCOPES,
            "scope_toolset-builtin-exa-web-search",
          )
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_token_missing_toolset_scope(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_toolset_enabled_for_app()
      .withf(|toolset_id| toolset_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_| Ok(true));
    mock_tool_service
      .expect_is_app_client_registered_for_toolset()
      .withf(|app_client_id, toolset_id| {
        app_client_id == "external-app" && toolset_id == "builtin-exa-web-search"
      })
      .times(1)
      .returning(|_, _| Ok(true));
    // Missing the scope_toolset-* header means OAuth auth fails

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
          .header(KEY_HEADER_BODHIAPP_AZP, "external-app")
          // Missing KEY_HEADER_BODHIAPP_TOOL_SCOPES
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_missing_user_id(#[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>) {
    let mock_tool_service = MockToolService::new();
    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
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
  async fn test_missing_auth(#[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>) {
    let mock_tool_service = MockToolService::new();
    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }
}
