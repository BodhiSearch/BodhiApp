use crate::{app_status_or_default, AuthContext, DefaultTokenService};
use axum::{
  extract::{Request, State},
  http::{header::HOST, HeaderMap},
  middleware::Next,
  response::Response,
};

use objs::{ApiError, AppError, ErrorType, RoleError, TokenScopeError, UserScopeError};
use server_core::RouterState;
use services::{
  db::DbError, extract_claims, AppInstanceError, AppStatus, AuthServiceError, TokenError,
  UserIdClaims,
};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::debug;

pub const SESSION_KEY_ACCESS_TOKEN: &str = "access_token";
pub const SESSION_KEY_REFRESH_TOKEN: &str = "refresh_token";
pub const SESSION_KEY_USER_ID: &str = "user_id";

pub const KEY_PREFIX_HEADER_BODHIAPP: &str = "X-BodhiApp-";

const SEC_FETCH_SITE_HEADER: &str = "sec-fetch-site";

/// Clears authentication data from the session.
async fn clear_session_auth_data(session: &Session) {
  if let Err(e) = session.remove::<String>(SESSION_KEY_ACCESS_TOKEN).await {
    debug!(?e, "Failed to clear access token from session");
  }
  if let Err(e) = session.remove::<String>(SESSION_KEY_REFRESH_TOKEN).await {
    debug!(?e, "Failed to clear refresh token from session");
  }
  if let Err(e) = session.remove::<String>(SESSION_KEY_USER_ID).await {
    debug!(?e, "Failed to clear user_id from session");
  }
}

/// Determines whether session data should be cleared for the given error.
fn should_clear_session(err: &AuthError) -> bool {
  matches!(
    err,
    AuthError::RefreshTokenNotFound
      | AuthError::Token(_)
      | AuthError::AuthService(_)
      | AuthError::InvalidToken(_)
  )
}

/// Returns true if the request originates from the same site ("same-origin").
fn is_same_origin(headers: &HeaderMap) -> bool {
  let host = headers.get(HOST).and_then(|v| v.to_str().ok());
  let sec_fetch_site = headers
    .get(SEC_FETCH_SITE_HEADER)
    .and_then(|v| v.to_str().ok());
  evaluate_same_origin(host, sec_fetch_site)
}

fn evaluate_same_origin(host: Option<&str>, sec_fetch_site: Option<&str>) -> bool {
  if let Some(host) = host {
    if host.starts_with("localhost:") {
      let result = matches!(sec_fetch_site, Some("same-origin"));
      debug!("is_same_origin: result: {}", result);
      return result;
    } else {
      debug!("is_same_origin: host is not localhost: {}", host);
    }
  }
  debug!("is_same_origin: host is None");
  true
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthError {
  #[error(transparent)]
  Token(#[from] TokenError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  Role(#[from] RoleError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenScope(#[from] TokenScopeError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  UserScope(#[from] UserScopeError),
  #[error("User has no valid access roles.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingRoles,
  #[error("Access denied.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidAccess,
  #[error("API token is inactive.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenInactive,
  #[error("API token not found.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenNotFound,
  #[error(transparent)]
  AuthService(#[from] AuthServiceError),
  #[error(transparent)]
  AppInstance(#[from] AppInstanceError),
  #[error(transparent)]
  DbError(#[from] DbError),
  #[error("Session expired. Please log out and log in again.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  RefreshTokenNotFound,
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "auth_error-tower_sessions", args_delegate = false)]
  TowerSession(#[from] tower_sessions::session::Error),
  #[error("Invalid token: {0}.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidToken(String),
  #[error("Application is not ready. Current status: {0}.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppStatusInvalid(AppStatus),
}

pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);

  let app_service = state.app_service();
  let app_instance_service = app_service.app_instance_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    app_instance_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
  );

  if app_status_or_default(&app_instance_service).await == AppStatus::Setup {
    return Err(AuthError::AppStatusInvalid(AppStatus::Setup).into());
  }

  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    let header = header
      .to_str()
      .map_err(|err| AuthError::InvalidToken(err.to_string()))?;
    let auth_context = token_service.validate_bearer_token(header).await?;
    req.extensions_mut().insert(auth_context);
    Ok(next.run(req).await)
  } else if is_same_origin(req.headers()) {
    // Check for token in session
    if let Some(access_token) = session
      .get::<String>(SESSION_KEY_ACCESS_TOKEN)
      .await
      .map_err(AuthError::from)?
    {
      debug!("auth_middleware: found access token in session, validating");
      let (access_token, role) = token_service
        .get_valid_session_token(session, access_token)
        .await?;
      let role = role.ok_or(AuthError::MissingRoles)?;
      let claims = extract_claims::<UserIdClaims>(&access_token)?;
      let user_id = claims.sub.clone();
      debug!(
        "auth_middleware: session token validated successfully for user: {}",
        user_id
      );

      let auth_context = AuthContext::Session {
        user_id,
        username: claims.preferred_username,
        role: Some(role),
        token: access_token,
      };

      req.extensions_mut().insert(auth_context);
      Ok(next.run(req).await)
    } else {
      debug!("auth_middleware: no access token in session, returning InvalidAccess");
      Err(AuthError::InvalidAccess)?
    }
  } else {
    debug!("auth_middleware: request is not same-origin, returning InvalidAccess");
    Err(AuthError::InvalidAccess)?
  }
}

pub async fn optional_auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);
  let app_service = state.app_service();
  let app_instance_service = app_service.app_instance_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    app_instance_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
  );

  // Check app status
  if app_status_or_default(&app_instance_service).await == AppStatus::Setup {
    req.extensions_mut().insert(AuthContext::Anonymous);
    return Ok(next.run(req).await);
  }

  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    debug!("optional_auth_middleware: validating bearer token");
    if let Ok(header) = header.to_str() {
      match token_service.validate_bearer_token(header).await {
        Ok(auth_context) => {
          debug!("optional_auth_middleware: bearer token validated successfully");
          req.extensions_mut().insert(auth_context);
        }
        Err(err) => {
          debug!(
            ?err,
            "optional_auth_middleware: bearer token validation failed"
          );
          req.extensions_mut().insert(AuthContext::Anonymous);
        }
      }
    } else {
      debug!("optional_auth_middleware: Authorization header is not valid UTF-8");
      req.extensions_mut().insert(AuthContext::Anonymous);
    }
  } else if is_same_origin(req.headers()) {
    // session token
    match session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await {
      Ok(Some(access_token)) => {
        match token_service
          .get_valid_session_token(session.clone(), access_token.clone())
          .await
        {
          Ok((validated_token, role)) => {
            debug!("optional_auth_middleware: session token validated successfully");
            let claims = extract_claims::<UserIdClaims>(&validated_token)?;
            let auth_context = AuthContext::Session {
              user_id: claims.sub.clone(),
              username: claims.preferred_username,
              role,
              token: validated_token,
            };
            req.extensions_mut().insert(auth_context);
          }
          Err(err) => {
            debug!(
              ?err,
              "optional_auth_middleware: token validation/refresh failed"
            );
            if should_clear_session(&err) {
              clear_session_auth_data(&session).await;
            }
            req.extensions_mut().insert(AuthContext::Anonymous);
          }
        }
      }
      Ok(None) => {
        req.extensions_mut().insert(AuthContext::Anonymous);
      }
      Err(err) => {
        debug!(?err, "optional_auth_middleware: error reading session");
        req.extensions_mut().insert(AuthContext::Anonymous);
      }
    }
  } else {
    req.extensions_mut().insert(AuthContext::Anonymous);
  }
  // Continue with the request
  Ok(next.run(req).await)
}

fn remove_app_headers(req: &mut axum::http::Request<axum::body::Body>) {
  // Remove internal headers to prevent injection attacks
  let headers_to_remove: Vec<_> = req
    .headers()
    .keys()
    .filter(|key| {
      key
        .as_str()
        .to_lowercase()
        .starts_with(&KEY_PREFIX_HEADER_BODHIAPP.to_lowercase())
    })
    .cloned()
    .collect();
  for key in headers_to_remove {
    req.headers_mut().remove(key);
  }
}
