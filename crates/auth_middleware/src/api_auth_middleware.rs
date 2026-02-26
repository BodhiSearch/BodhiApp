use crate::AuthContext;
use axum::{
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use objs::{
  ApiError, AppError, ErrorType, ResourceRole, RoleError, TokenScope, TokenScopeError, UserScope,
  UserScopeError,
};
use server_core::RouterState;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiAuthError {
  #[error("Insufficient permissions for this resource.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  Forbidden,
  #[error("Authentication required. Provide an API key or log in.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidRole(#[from] RoleError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidScope(#[from] TokenScopeError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidUserScope(#[from] UserScopeError),
}

pub async fn api_auth_middleware(
  required_role: ResourceRole,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(_state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(
    authorize_request(
      required_role,
      required_token_scope,
      required_user_scope,
      req,
      next,
    )
    .await?,
  )
}

async fn authorize_request(
  required_role: ResourceRole,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  req: Request,
  next: Next,
) -> Result<Response, ApiAuthError> {
  let auth_context = req
    .extensions()
    .get::<AuthContext>()
    .ok_or(ApiAuthError::MissingAuth)?;

  match auth_context {
    AuthContext::Session {
      role: Some(role), ..
    } => {
      if !role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }
    AuthContext::Session { role: None, .. } => {
      return Err(ApiAuthError::MissingAuth);
    }
    AuthContext::ApiToken { role, .. } => {
      if let Some(required_token_scope) = required_token_scope {
        if !role.has_access_to(&required_token_scope) {
          return Err(ApiAuthError::Forbidden);
        }
      } else {
        return Err(ApiAuthError::MissingAuth);
      }
    }
    AuthContext::ExternalApp {
      role: Some(role), ..
    } => {
      if let Some(required_user_scope) = required_user_scope {
        if !role.has_access_to(&required_user_scope) {
          return Err(ApiAuthError::Forbidden);
        }
      } else {
        return Err(ApiAuthError::MissingAuth);
      }
    }
    AuthContext::ExternalApp { role: None, .. } => {
      return Err(ApiAuthError::MissingAuth);
    }
    AuthContext::Anonymous => {
      return Err(ApiAuthError::MissingAuth);
    }
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use crate::{api_auth_middleware, AuthContext};
  use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
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

  /// Helper middleware that injects AuthContext into request extensions
  async fn inject_auth_context(
    auth_context: AuthContext,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
  ) -> axum::response::Response {
    req.extensions_mut().insert(auth_context);
    next.run(req).await
  }

  async fn test_router_with_auth_context(
    required_role: ResourceRole,
    required_token_scope: Option<TokenScope>,
    auth_context: AuthContext,
  ) -> Router {
    let app_service = AppServiceStubBuilder::default().build().await.unwrap();
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let ctx = auth_context.clone();
    Router::new()
      .route("/test", get(test_handler))
      .route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          api_auth_middleware(required_role, required_token_scope, None, state, req, next)
        },
      ))
      .layer(axum::middleware::from_fn(move |req, next| {
        let ctx = ctx.clone();
        inject_auth_context(ctx, req, next)
      }))
      .with_state(state)
  }

  async fn test_router_user_scope_with_auth_context(
    required_role: ResourceRole,
    required_user_scope: Option<UserScope>,
    auth_context: AuthContext,
  ) -> Router {
    let app_service = AppServiceStubBuilder::default().build().await.unwrap();
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let ctx = auth_context.clone();
    Router::new()
      .route("/test", get(test_handler))
      .route_layer(from_fn_with_state(
        state.clone(),
        move |state, req, next| {
          api_auth_middleware(required_role, None, required_user_scope, state, req, next)
        },
      ))
      .layer(axum::middleware::from_fn(move |req, next| {
        let ctx = ctx.clone();
        inject_auth_context(ctx, req, next)
      }))
      .with_state(state)
  }

  #[rstest]
  #[case::user_accessing_user(ResourceRole::User, ResourceRole::User)]
  #[case::power_user_accessing_user(ResourceRole::PowerUser, ResourceRole::User)]
  #[case::power_user_accessing_power_user(ResourceRole::PowerUser, ResourceRole::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_role_success(
    #[case] user_role: ResourceRole,
    #[case] required_role: ResourceRole,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_session("user1", "user@test.com", user_role);
    let router = test_router_with_auth_context(required_role, None, ctx).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(ResourceRole::User, ResourceRole::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_role_insufficient(
    #[case] user_role: ResourceRole,
    #[case] required_role: ResourceRole,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_session("user1", "user@test.com", user_role);
    let router = test_router_with_auth_context(required_role, None, ctx).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_user(TokenScope::User, TokenScope::User)]
  #[case::power_user_accessing_user(TokenScope::PowerUser, TokenScope::User)]
  #[tokio::test]
  async fn test_api_auth_scope_success(
    #[case] user_scope: TokenScope,
    #[case] required_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_api_token("user1", user_scope);
    let router = test_router_with_auth_context(ResourceRole::User, Some(required_scope), ctx).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(TokenScope::User, TokenScope::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_scope_insufficient(
    #[case] user_scope: TokenScope,
    #[case] required_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_api_token("user1", user_scope);
    let router = test_router_with_auth_context(ResourceRole::User, Some(required_scope), ctx).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::scope_not_allowed(TokenScope::User)]
  #[case::scope_not_allowed_power_user(TokenScope::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_scope_not_allowed(#[case] scope: TokenScope) -> anyhow::Result<()> {
    let ctx = AuthContext::test_api_token("user1", scope);
    let router = test_router_with_auth_context(ResourceRole::User, None, ctx).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_missing_role() -> anyhow::Result<()> {
    let ctx = AuthContext::Anonymous;
    let router = test_router_with_auth_context(ResourceRole::User, None, ctx).await;
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

  // ===============================
  // UserScope Tests
  // ===============================

  #[rstest]
  #[case::user_accessing_user(UserScope::User, UserScope::User)]
  #[case::power_user_accessing_user(UserScope::PowerUser, UserScope::User)]
  #[case::power_user_accessing_power_user(UserScope::PowerUser, UserScope::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_user_scope_success(
    #[case] user_scope: UserScope,
    #[case] required_user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_external_app("user1", user_scope, "app1", None);
    let router =
      test_router_user_scope_with_auth_context(ResourceRole::User, Some(required_user_scope), ctx)
        .await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_accessing_power_user(UserScope::User, UserScope::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_user_scope_insufficient(
    #[case] user_scope: UserScope,
    #[case] required_user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_external_app("user1", user_scope, "app1", None);
    let router =
      test_router_user_scope_with_auth_context(ResourceRole::User, Some(required_user_scope), ctx)
        .await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::FORBIDDEN, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user_scope_not_allowed(UserScope::User)]
  #[case::power_user_scope_not_allowed(UserScope::PowerUser)]
  #[tokio::test]
  async fn test_api_auth_user_scope_not_allowed(
    #[case] user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let ctx = AuthContext::test_external_app("user1", user_scope, "app1", None);
    let router = test_router_user_scope_with_auth_context(ResourceRole::User, None, ctx).await;
    let req = Request::builder().uri("/test").body(Body::empty())?;

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_api_auth_middleware_user_scope_missing_auth() -> anyhow::Result<()> {
    let ctx = AuthContext::Anonymous;
    let router =
      test_router_user_scope_with_auth_context(ResourceRole::User, Some(UserScope::User), ctx)
        .await;
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
  async fn test_api_auth_external_app_no_role() -> anyhow::Result<()> {
    let ctx = AuthContext::test_external_app_no_role("user1", "app1", None);
    let router =
      test_router_user_scope_with_auth_context(ResourceRole::User, Some(UserScope::User), ctx)
        .await;
    let req = Request::builder().uri("/test").body(Body::empty())?;
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }
}
