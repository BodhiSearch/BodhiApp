use crate::{KEY_RESOURCE_ROLE, KEY_RESOURCE_SCOPE};
use axum::{
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use objs::{ApiError, AppError, ErrorType, Role, RoleError, TokenScope, TokenScopeError};
use server_core::RouterState;
use services::{SecretServiceError, SecretServiceExt};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiAuthError {
  #[error(transparent)]
  SecretService(#[from] SecretServiceError),
  #[error("forbidden")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  Forbidden,
  #[error("missing_auth")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,
  #[error("malformed_role")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MalformedRole(String),
  #[error("malformed_scope")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MalformedScope(String),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidRole(#[from] RoleError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidScope(#[from] TokenScopeError),
}

pub async fn api_auth_middleware(
  required_role: Role,
  required_scope: Option<TokenScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(_impl(required_role, required_scope, State(state), req, next).await?)
}

pub async fn _impl(
  required_role: Role,
  required_scope: Option<TokenScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiAuthError> {
  // Check if authorization is disabled
  let authz = &state.app_service().secret_service().authz()?;
  if !authz {
    return Ok(next.run(req).await);
  }

  // Get headers
  let role_header = req.headers().get(KEY_RESOURCE_ROLE);
  let scope_header = req.headers().get(KEY_RESOURCE_SCOPE);

  match (role_header, scope_header, required_scope) {
    // Role header present - validate against required role
    (Some(role_header), _, _) => {
      let user_role = role_header
        .to_str()
        .map_err(|e| ApiAuthError::MalformedRole(e.to_string()))?
        .parse::<Role>()?;

      if !user_role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }

    // No role header but scope allowed and present
    (None, Some(scope_header), Some(required_scope)) => {
      let user_scope = scope_header
        .to_str()
        .map_err(|e| ApiAuthError::MalformedScope(e.to_string()))?
        .parse::<TokenScope>()?;

      if !user_scope.has_access_to(&required_scope) {
        return Err(ApiAuthError::Forbidden);
      }
    }

    // No valid auth headers found
    _ => return Err(ApiAuthError::MissingAuth),
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use crate::{api_auth_middleware, KEY_RESOURCE_ROLE, KEY_RESOURCE_SCOPE};
  use axum::{
    body::Body,
    http::{HeaderValue, Request, Response, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService, Role, TokenScope};
  use rstest::rstest;
  use serde_json::Value;
  use server_core::{
    test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::test_utils::{AppServiceStubBuilder, SecretServiceStub};
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn test_handler() -> Response<Body> {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::empty())
      .unwrap()
  }

  fn test_router(required_role: Role, required_scope: Option<TokenScope>) -> Router {
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
      .route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          api_auth_middleware(required_role, required_scope, state, req, next)
        },
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
  async fn test_api_auth_role_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] required_role: Role,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, None);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_ROLE, user_role.to_string())
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
  async fn test_api_auth_role_insufficient(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] required_role: Role,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, None);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_ROLE, user_role.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_user(TokenScope::User, TokenScope::User)]
  #[case::power_user_accessing_user(TokenScope::PowerUser, TokenScope::User)]
  #[case::manager_accessing_user(TokenScope::Manager, TokenScope::User)]
  #[case::admin_accessing_user(TokenScope::Admin, TokenScope::User)]
  #[tokio::test]
  async fn test_api_auth_scope_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_scope: TokenScope,
    #[case] required_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User, Some(required_scope));
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(TokenScope::User, TokenScope::PowerUser)]
  #[case::user_accessing_manager(TokenScope::User, TokenScope::Manager)]
  #[case::power_user_accessing_manager(TokenScope::PowerUser, TokenScope::Manager)]
  #[tokio::test]
  async fn test_api_auth_scope_insufficient(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_scope: TokenScope,
    #[case] required_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User, Some(required_scope));
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::scope_not_allowed(TokenScope::User)]
  #[case::scope_not_allowed_power_user(TokenScope::PowerUser)]
  #[case::scope_not_allowed_manager(TokenScope::Manager)]
  #[case::scope_not_allowed_admin(TokenScope::Admin)]
  #[tokio::test]
  async fn test_api_auth_scope_not_allowed(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] scope: TokenScope,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User, None);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_SCOPE, scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_success_scope_insufficient(
    Role::Admin,  // user_role
    TokenScope::User,  // user_scope
    Role::User,  // required_role
    Some(TokenScope::Admin)  // required_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] user_scope: TokenScope,
    #[case] required_role: Role,
    #[case] required_scope: Option<TokenScope>,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, required_scope);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_ROLE, user_role.to_string())
      .header(KEY_RESOURCE_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_insufficient_scope_insufficient(
    Role::User,  // user_role
    TokenScope::User,  // user_scope
    Role::Admin,  // required_role
    Some(TokenScope::Admin)  // required_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_failure(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] user_scope: TokenScope,
    #[case] required_role: Role,
    #[case] required_scope: Option<TokenScope>,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, required_scope);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_ROLE, user_role.to_string())
      .header(KEY_RESOURCE_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_success_no_scope(
    Role::Admin,  // user_role
    TokenScope::User,  // user_scope
    Role::User,  // required_role
    None  // required_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_no_scope_required(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] user_role: Role,
    #[case] user_scope: TokenScope,
    #[case] required_role: Role,
    #[case] required_scope: Option<TokenScope>,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, required_scope);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_ROLE, user_role.to_string())
      .header(KEY_RESOURCE_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_missing_role(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User, None);
    let req = Request::builder().uri("/test").body(Body::empty())?;
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      serde_json::json!({
        "error": {
          "message": "missing authentication header",
          "type": "authentication_error",
          "code": "api_auth_error-missing_auth"
        }
      }),
      error
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_invalid_role(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let router = test_router(Role::User, None);
    let req = Request::builder()
      .uri("/test")
      .header(
        KEY_RESOURCE_ROLE,
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

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_skips_if_authz_disabled(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[values(Role::User, Role::PowerUser, Role::Manager, Role::Admin)] required_role: Role,
    #[values(Role::User, Role::PowerUser, Role::Manager, Role::Admin)] header_role: Role,
  ) -> anyhow::Result<()> {
    // Given
    let secret_service = SecretServiceStub::new()
      .with_authz_disabled()
      .with_app_status_ready();
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let router = Router::new()
      .route("/test", get(test_handler))
      .route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| api_auth_middleware(required_role, None, state, req, next),
      ))
      .with_state(state);
    let req = Request::builder()
      .uri("/test")
      .header(KEY_RESOURCE_ROLE, header_role.to_string())
      .body(Body::empty())?;

    // Then - Should pass through without checking auth
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());

    Ok(())
  }
}
