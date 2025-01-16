use crate::KEY_USER_ROLES;
use axum::{
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use objs::{ApiError, AppError, ErrorType, Role, RoleError};
use server_core::RouterState;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiAuthError {
  #[error("forbidden")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  Forbidden,
  #[error("missing_role")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingRole,
  #[error("malformed_role")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MalformedRole(String),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidRole(#[from] RoleError),
}

pub async fn api_auth_middleware(
  required_role: Role,
  State(_state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // Get role from header
  let user_role = req
    .headers()
    .get(KEY_USER_ROLES)
    .ok_or(ApiAuthError::MissingRole)?
    .to_str()
    .map_err(|e| ApiAuthError::MalformedRole(e.to_string()))?;

  // Parse role from header
  let user_role = user_role.parse::<Role>().map_err(ApiAuthError::from)?;

  // Check if user has required role access
  if !user_role.has_access_to(&required_role) {
    return Err(ApiAuthError::Forbidden)?;
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use super::api_auth_middleware;
  use crate::KEY_USER_ROLES;
  use axum::{
    body::Body,
    http::{header::HeaderValue, Request, StatusCode},
    response::Response,
    routing::get,
    Router,
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService, Role};
  use rstest::rstest;
  use serde_json::Value;
  use server_core::{
    test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn test_handler() -> Response {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::empty())
      .unwrap()
  }

  fn test_router(required_role: Role) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .build()
      .unwrap();
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    Router::new()
      .route("/test", get(test_handler))
      .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        move |state, req, next| api_auth_middleware(required_role, state, req, next),
      ))
      .with_state(state)
  }

  #[rstest]
  #[case::user_accessing_user(Role::User, Role::User)]
  #[case::power_user_accessing_user(Role::PowerUser, Role::User)]
  #[case::manager_accessing_user(Role::Manager, Role::User)]
  #[case::admin_accessing_user(Role::Admin, Role::User)]
  #[case::power_user_accessing_power_user(Role::PowerUser, Role::PowerUser)]
  #[case::manager_accessing_power_user(Role::Manager, Role::PowerUser)]
  #[case::admin_accessing_power_user(Role::Admin, Role::PowerUser)]
  #[case::manager_accessing_manager(Role::Manager, Role::Manager)]
  #[case::admin_accessing_manager(Role::Admin, Role::Manager)]
  #[case::admin_accessing_admin(Role::Admin, Role::Admin)]
  #[tokio::test]
  async fn test_api_auth_middleware_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] required_role: Role,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_USER_ROLES, user_role.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(Role::User, Role::PowerUser)]
  #[case::user_accessing_manager(Role::User, Role::Manager)]
  #[case::power_user_accessing_manager(Role::PowerUser, Role::Manager)]
  #[case::user_accessing_admin(Role::User, Role::Admin)]
  #[case::power_user_accessing_admin(Role::PowerUser, Role::Admin)]
  #[case::manager_accessing_admin(Role::Manager, Role::Admin)]
  #[tokio::test]
  async fn test_api_auth_middleware_insufficient_role(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] required_role: Role,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_USER_ROLES, user_role.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      error,
      serde_json::json!({
        "error": {
          "message": "insufficient privileges to access this resource",
          "type": "forbidden_error",
          "code": "api_auth_error-forbidden"
        }
      })
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_missing_role(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User);
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      error,
      serde_json::json!({
        "error": {
          "message": "role information not found",
          "type": "authentication_error",
          "code": "api_auth_error-missing_role"
        }
      })
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_invalid_role(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User);
    let req = Request::builder()
      .uri("/test")
      .header(
        KEY_USER_ROLES,
        HeaderValue::from_bytes(b"some_invalid_role")?,
      )
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      serde_json::json! {{
        "error": {
          "message": "invalid role name: \u{2068}some_invalid_role\u{2069}",
          "type": "authentication_error",
          "code": "role_error-invalid_role_name"
        }
      }},
      error
    );
    Ok(())
  }
}
