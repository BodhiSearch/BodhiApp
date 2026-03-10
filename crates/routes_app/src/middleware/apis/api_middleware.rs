use crate::middleware::apis::ApiAuthError;
use crate::middleware::MiddlewareError;
use axum::{
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use services::AppService;
use services::AuthContext;
use services::{ResourceRole, TokenScope, UserScope};
use std::sync::Arc;

pub async fn api_auth_middleware(
  required_role: ResourceRole,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(_app_service): State<Arc<dyn AppService>>,
  req: Request,
  next: Next,
) -> Result<Response, MiddlewareError> {
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
    }
    | AuthContext::MultiTenantSession {
      role: Some(role), ..
    } => {
      if !role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }
    AuthContext::Session { role: None, .. }
    | AuthContext::MultiTenantSession { role: None, .. } => {
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
    AuthContext::Anonymous { .. } => {
      return Err(ApiAuthError::MissingAuth);
    }
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
#[path = "test_api_middleware.rs"]
mod test_api_middleware;
