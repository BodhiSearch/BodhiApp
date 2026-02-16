use crate::{KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE};
use axum::{
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use objs::{
  ApiError, AppError, ErrorType, ResourceRole, ResourceScope, ResourceScopeError, RoleError,
  TokenScope, TokenScopeError, UserScope, UserScopeError,
};
use server_core::RouterState;
use services::SecretServiceError;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiAuthError {
  #[error(transparent)]
  SecretService(#[from] SecretServiceError),
  #[error("Insufficient permissions for this resource.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  Forbidden,
  #[error("Authentication required. Provide an API key or log in.")]
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
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidUserScope(#[from] UserScopeError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidResourceScope(#[from] ResourceScopeError),
}

pub async fn api_auth_middleware(
  required_role: ResourceRole,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(
    _impl(
      required_role,
      required_token_scope,
      required_user_scope,
      State(state),
      req,
      next,
    )
    .await?,
  )
}

pub async fn _impl(
  required_role: ResourceRole,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(_state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiAuthError> {
  // Get headers
  let role_header = req.headers().get(KEY_HEADER_BODHIAPP_ROLE);
  let scope_header = req.headers().get(KEY_HEADER_BODHIAPP_SCOPE);

  match (role_header, scope_header) {
    // Role header present - validate against required role (takes precedence)
    (Some(role_header), _) => {
      let user_role = role_header
        .to_str()
        .map_err(|e| ApiAuthError::MalformedRole(e.to_string()))?
        .parse::<ResourceRole>()?;

      if !user_role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }

    // No role header but scope header present
    (None, Some(scope_header)) => {
      let scope_str = scope_header
        .to_str()
        .map_err(|e| ApiAuthError::MalformedScope(e.to_string()))?;

      let resource_scope = ResourceScope::try_parse(scope_str)?;

      match resource_scope {
        ResourceScope::Token(token_scope) => {
          if let Some(required_token_scope) = required_token_scope {
            if !token_scope.has_access_to(&required_token_scope) {
              return Err(ApiAuthError::Forbidden);
            }
          } else {
            return Err(ApiAuthError::MissingAuth);
          }
        }
        ResourceScope::User(user_scope) => {
          if let Some(required_user_scope) = required_user_scope {
            if !user_scope.has_access_to(&required_user_scope) {
              return Err(ApiAuthError::Forbidden);
            }
          } else {
            return Err(ApiAuthError::MissingAuth);
          }
        }
      }
    }

    // No valid auth headers found
    _ => return Err(ApiAuthError::MissingAuth),
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use crate::{api_auth_middleware, KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE};
  use axum::{
    body::Body,
    http::{HeaderValue, Request, Response, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
  };
  use objs::{ResourceRole, TokenScope, UserScope};
  use rstest::rstest;
  use serde_json::Value;
  use server_core::{
    test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn test_handler() -> Response<Body> {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::empty())
      .unwrap()
  }

  async fn test_router(
    required_role: ResourceRole,
    required_token_scope: Option<TokenScope>,
  ) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .build()
      .await
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
          api_auth_middleware(required_role, required_token_scope, None, state, req, next)
        },
      ))
      .with_state(state)
  }

  async fn test_router_user_scope(
    required_role: ResourceRole,
    required_user_scope: Option<UserScope>,
  ) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .build()
      .await
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
          api_auth_middleware(required_role, None, required_user_scope, state, req, next)
        },
      ))
      .with_state(state)
  }

  #[rstest]
  #[case::user_accessing_user(ResourceRole::User, ResourceRole::User)]
  #[case::power_user_accessing_user(ResourceRole::PowerUser, ResourceRole::User)]
  #[case::manager_accessing_user(ResourceRole::Manager, ResourceRole::User)]
  #[case::admin_accessing_user(ResourceRole::Admin, ResourceRole::User)]
  #[case::power_user_accessing_power_user(ResourceRole::PowerUser, ResourceRole::PowerUser)]
  #[case::manager_accessing_power_user(ResourceRole::Manager, ResourceRole::PowerUser)]
  #[case::admin_accessing_power_user(ResourceRole::Admin, ResourceRole::PowerUser)]
  #[case::manager_accessing_manager(ResourceRole::Manager, ResourceRole::Manager)]
  #[case::admin_accessing_manager(ResourceRole::Admin, ResourceRole::Manager)]
  #[case::admin_accessing_admin(ResourceRole::Admin, ResourceRole::Admin)]
  #[tokio::test]
  async fn test_api_auth_role_success(
    #[case] user_role: ResourceRole,
    #[case] required_role: ResourceRole,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, None).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(ResourceRole::User, ResourceRole::PowerUser)]
  #[case::user_accessing_manager(ResourceRole::User, ResourceRole::Manager)]
  #[case::power_user_accessing_manager(ResourceRole::PowerUser, ResourceRole::Manager)]
  #[case::user_accessing_admin(ResourceRole::User, ResourceRole::Admin)]
  #[case::power_user_accessing_admin(ResourceRole::PowerUser, ResourceRole::Admin)]
  #[case::manager_accessing_admin(ResourceRole::Manager, ResourceRole::Admin)]
  #[tokio::test]
  async fn test_api_auth_role_insufficient(
    #[case] user_role: ResourceRole,
    #[case] required_role: ResourceRole,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, None).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
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
    #[case] user_scope: TokenScope,
    #[case] required_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let router = test_router(ResourceRole::User, Some(required_scope)).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
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
    #[case] user_scope: TokenScope,
    #[case] required_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let router = test_router(ResourceRole::User, Some(required_scope)).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
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
  async fn test_api_auth_scope_not_allowed(#[case] scope: TokenScope) -> anyhow::Result<()> {
    let router = test_router(ResourceRole::User, None).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_success_scope_insufficient(
    ResourceRole::Admin,  // user_role
    TokenScope::User,  // user_scope
    ResourceRole::User,  // required_role
    Some(TokenScope::Admin)  // required_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_success(
    #[case] user_role: ResourceRole,
    #[case] user_scope: TokenScope,
    #[case] required_role: ResourceRole,
    #[case] required_scope: Option<TokenScope>,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, required_scope).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_insufficient_scope_insufficient(
    ResourceRole::User,  // user_role
    TokenScope::User,  // user_scope
    ResourceRole::Admin,  // required_role
    Some(TokenScope::Admin)  // required_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_failure(
    #[case] user_role: ResourceRole,
    #[case] user_scope: TokenScope,
    #[case] required_role: ResourceRole,
    #[case] required_scope: Option<TokenScope>,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, required_scope).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_success_no_scope(
    ResourceRole::Admin,  // user_role
    TokenScope::User,  // user_scope
    ResourceRole::User,  // required_role
    None  // required_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_no_scope_required(
    #[case] user_role: ResourceRole,
    #[case] user_scope: TokenScope,
    #[case] required_role: ResourceRole,
    #[case] required_scope: Option<TokenScope>,
  ) -> anyhow::Result<()> {
    let router = test_router(required_role, required_scope).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_missing_role() -> anyhow::Result<()> {
    let router = test_router(ResourceRole::User, None).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      serde_json::json!({
        "error": {
          "message": "Authentication required. Provide an API key or log in.",
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
  async fn test_api_auth_middleware_invalid_role() -> anyhow::Result<()> {
    let router = test_router(ResourceRole::User, None).await;
    let req = Request::builder()
      .uri("/test")
      .header(
        KEY_HEADER_BODHIAPP_ROLE,
        HeaderValue::from_bytes(b"some_invalid_role")?,
      )
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      serde_json::json! {{
        "error": {
          "message": "invalid_role_name",
          "type": "authentication_error",
          "code": "role_error-invalid_role_name",
          "param": {
            "var_0": "some_invalid_role"
          }
        }
      }},
      error
    );
    Ok(())
  }

  // ===============================
  // UserScope Tests
  // ===============================

  #[rstest]
  #[case::user_accessing_user(UserScope::User, UserScope::User)]
  #[case::power_user_accessing_user(UserScope::PowerUser, UserScope::User)]
  #[case::manager_accessing_user(UserScope::Manager, UserScope::User)]
  #[case::admin_accessing_user(UserScope::Admin, UserScope::User)]
  #[case::power_user_accessing_power_user(UserScope::PowerUser, UserScope::PowerUser)]
  #[case::manager_accessing_power_user(UserScope::Manager, UserScope::PowerUser)]
  #[case::admin_accessing_power_user(UserScope::Admin, UserScope::PowerUser)]
  #[case::manager_accessing_manager(UserScope::Manager, UserScope::Manager)]
  #[case::admin_accessing_manager(UserScope::Admin, UserScope::Manager)]
  #[case::admin_accessing_admin(UserScope::Admin, UserScope::Admin)]
  #[tokio::test]
  async fn test_api_auth_user_scope_success(
    #[case] user_scope: UserScope,
    #[case] required_user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let router = test_router_user_scope(ResourceRole::User, Some(required_user_scope)).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(UserScope::User, UserScope::PowerUser)]
  #[case::user_accessing_manager(UserScope::User, UserScope::Manager)]
  #[case::power_user_accessing_manager(UserScope::PowerUser, UserScope::Manager)]
  #[case::user_accessing_admin(UserScope::User, UserScope::Admin)]
  #[case::power_user_accessing_admin(UserScope::PowerUser, UserScope::Admin)]
  #[case::manager_accessing_admin(UserScope::Manager, UserScope::Admin)]
  #[tokio::test]
  async fn test_api_auth_user_scope_insufficient(
    #[case] user_scope: UserScope,
    #[case] required_user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let router = test_router_user_scope(ResourceRole::User, Some(required_user_scope)).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_scope_not_allowed(UserScope::User)]
  #[case::power_user_scope_not_allowed(UserScope::PowerUser)]
  #[case::manager_scope_not_allowed(UserScope::Manager)]
  #[case::admin_scope_not_allowed(UserScope::Admin)]
  #[tokio::test]
  async fn test_api_auth_user_scope_not_allowed(
    #[case] user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let router = test_router_user_scope(ResourceRole::User, None).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_success_user_scope_insufficient(
    ResourceRole::Admin,  // user_role
    UserScope::User,  // user_scope
    ResourceRole::User,  // required_role
    Some(UserScope::Admin)  // required_user_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_over_user_scope_success(
    #[case] user_role: ResourceRole,
    #[case] user_scope: UserScope,
    #[case] required_role: ResourceRole,
    #[case] required_user_scope: Option<UserScope>,
  ) -> anyhow::Result<()> {
    let router = test_router_user_scope(required_role, required_user_scope).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_insufficient_user_scope_insufficient(
    ResourceRole::User,  // user_role
    UserScope::User,  // user_scope
    ResourceRole::Admin,  // required_role
    Some(UserScope::Admin)  // required_user_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_over_user_scope_failure(
    #[case] user_role: ResourceRole,
    #[case] user_scope: UserScope,
    #[case] required_role: ResourceRole,
    #[case] required_user_scope: Option<UserScope>,
  ) -> anyhow::Result<()> {
    let router = test_router_user_scope(required_role, required_user_scope).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::role_success_no_user_scope(
    ResourceRole::Admin,  // user_role
    UserScope::User,  // user_scope
    ResourceRole::User,  // required_role
    None  // required_user_scope
  )]
  #[tokio::test]
  async fn test_api_auth_role_precedence_no_user_scope_required(
    #[case] user_role: ResourceRole,
    #[case] user_scope: UserScope,
    #[case] required_role: ResourceRole,
    #[case] required_user_scope: Option<UserScope>,
  ) -> anyhow::Result<()> {
    let router = test_router_user_scope(required_role, required_user_scope).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_ROLE, user_role.to_string())
      .header(KEY_HEADER_BODHIAPP_SCOPE, user_scope.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_user_scope_missing_auth() -> anyhow::Result<()> {
    let router = test_router_user_scope(ResourceRole::User, Some(UserScope::User)).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      serde_json::json!({
        "error": {
          "message": "Authentication required. Provide an API key or log in.",
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
  async fn test_api_auth_middleware_invalid_user_scope() -> anyhow::Result<()> {
    let router = test_router_user_scope(ResourceRole::User, Some(UserScope::User)).await;
    let req = Request::builder()
      .uri("/test")
      .header(
        KEY_HEADER_BODHIAPP_SCOPE,
        HeaderValue::from_bytes(b"invalid_user_scope")?,
      )
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let error = response.json::<Value>().await?;
    assert_eq!(
      serde_json::json!({
        "error": {
          "message": "invalid resource scope: invalid_user_scope",
          "type": "authentication_error",
          "code": "resource_scope_error-invalid_scope",
          "param": {
            "var_0": "invalid_user_scope"
          }
        }
      }),
      error
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_token_scope_in_user_scope_context() -> anyhow::Result<()> {
    // Test sending a TokenScope value when UserScope is expected
    let router = test_router_user_scope(ResourceRole::User, Some(UserScope::User)).await;
    let req = Request::builder()
      .uri("/test")
      .header(KEY_HEADER_BODHIAPP_SCOPE, TokenScope::Admin.to_string())
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }
}
