use crate::{KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_USER_ID};
use axum::{
  extract::{Path, Request, State},
  middleware::Next,
  response::Response,
};
use objs::{ApiError, AppError, ErrorType};
use server_core::RouterState;
use services::ToolError;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolAuthError {
  #[error("missing_user_id")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingUserId,

  #[error("missing_auth")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,

  #[error(transparent)]
  ToolError(#[from] ToolError),
}

/// Middleware for tool execution endpoints
///
/// Authorization rules:
/// - Check that user has the tool configured (enabled + API key set)
/// - For all auth types: session, first-party tokens, and OAuth tokens
///
/// Note: OAuth-specific tool scope validation is deferred to future enhancement
/// when auth_middleware is extended to preserve full JWT scope strings.
pub async fn tool_auth_middleware(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(_impl(State(state), Path(tool_id), req, next).await?)
}

async fn _impl(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  req: Request,
  next: Next,
) -> Result<Response, ToolAuthError> {
  let headers = req.headers();

  // Extract user_id
  let user_id = headers
    .get(KEY_HEADER_BODHIAPP_USER_ID)
    .and_then(|v| v.to_str().ok())
    .ok_or(ToolAuthError::MissingUserId)?;

  // Verify authentication exists (either role or scope header)
  let has_auth = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE)
    || headers.contains_key(KEY_HEADER_BODHIAPP_SCOPE);

  if !has_auth {
    return Err(ToolAuthError::MissingAuth);
  }

  // Check if tool is configured and available for user
  let is_available = state
    .app_service()
    .tool_service()
    .is_tool_available_for_user(user_id, &tool_id)
    .await?;

  if !is_available {
    return Err(ToolError::ToolNotConfigured.into());
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
        "/tools/{tool_id}/execute",
        post(test_handler).route_layer(from_fn_with_state(state.clone(), tool_auth_middleware)),
      )
      .with_state(state)
  }

  #[rstest]
  #[tokio::test]
  async fn test_session_auth_tool_available(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_available_for_user()
      .withf(|user_id, tool_id| user_id == "user123" && tool_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(true));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/tools/builtin-exa-web-search/execute")
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
  async fn test_session_auth_tool_not_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_available_for_user()
      .withf(|user_id, tool_id| user_id == "user123" && tool_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(false));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/tools/builtin-exa-web-search/execute")
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
  async fn test_first_party_token_auth_tool_available(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_available_for_user()
      .withf(|user_id, tool_id| user_id == "user123" && tool_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(true));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/tools/builtin-exa-web-search/execute")
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
  async fn test_oauth_token_tool_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_available_for_user()
      .withf(|user_id, tool_id| user_id == "user123" && tool_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(true));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/tools/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_token_tool_not_configured(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) {
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_available_for_user()
      .withf(|user_id, tool_id| user_id == "user123" && tool_id == "builtin-exa-web-search")
      .times(1)
      .returning(|_, _| Ok(false));

    let app = test_router_with_tool_service(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/tools/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, UserScope::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
          .uri("/tools/builtin-exa-web-search/execute")
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
          .uri("/tools/builtin-exa-web-search/execute")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }
}
