use crate::middleware::auth::AuthError;
use crate::middleware::token_service::DefaultTokenService;
use crate::middleware::MiddlewareError;
use axum::{
  extract::{Request, State},
  http::{header::HOST, HeaderMap},
  middleware::Next,
  response::Response,
};

use services::AppService;
use services::AuthContext;
use services::{extract_claims, AppStatus, TenantError, UserIdClaims};
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

fn evaluate_same_origin(_host: Option<&str>, sec_fetch_site: Option<&str>) -> bool {
  match sec_fetch_site {
    Some("same-origin") | Some("same-site") => {
      debug!("is_same_origin: sec-fetch-site indicates same origin/site");
      true
    }
    Some("cross-site") | Some("none") => {
      debug!("is_same_origin: sec-fetch-site indicates cross-site request, rejecting");
      false
    }
    Some(other) => {
      debug!(
        "is_same_origin: unknown sec-fetch-site value '{}', rejecting",
        other
      );
      false
    }
    None => {
      // Non-browser clients (curl, Postman, API clients) don't send Sec-Fetch-Site
      debug!("is_same_origin: no sec-fetch-site header (non-browser client), allowing");
      true
    }
  }
}

pub async fn auth_middleware(
  session: Session,
  State(app_service): State<Arc<dyn AppService>>,
  mut req: Request,
  next: Next,
) -> Result<Response, MiddlewareError> {
  remove_app_headers(&mut req);
  let tenant_service = app_service.tenant_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    tenant_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
    app_service.time_service(),
  );

  // Single get_standalone_app() call: extract both status and client_id
  let tenant = tenant_service.get_standalone_app().await?;
  let status = tenant
    .as_ref()
    .map(|t| t.status.clone())
    .unwrap_or_default();
  if status == AppStatus::Setup {
    return Err(AuthError::AppStatusInvalid(AppStatus::Setup).into());
  }
  let tenant = tenant.ok_or(TenantError::NotFound)?;
  let instance_client_id = tenant.client_id.clone();
  let instance_tenant_id = tenant.id.clone();

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
        client_id: instance_client_id,
        tenant_id: instance_tenant_id,
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
  State(app_service): State<Arc<dyn AppService>>,
  mut req: Request,
  next: Next,
) -> Result<Response, MiddlewareError> {
  remove_app_headers(&mut req);
  let tenant_service = app_service.tenant_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    tenant_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
    app_service.time_service(),
  );

  // Single get_standalone_app() call: extract both status and client_id
  let tenant = tenant_service.get_standalone_app().await.ok().flatten();
  let status = tenant
    .as_ref()
    .map(|t| t.status.clone())
    .unwrap_or_default();
  if status == AppStatus::Setup {
    req.extensions_mut().insert(AuthContext::Anonymous {
      client_id: None,
      tenant_id: None,
    });
    return Ok(next.run(req).await);
  }

  let (instance_client_id, instance_tenant_id): (Option<String>, Option<String>) = tenant
    .map(|t| (Some(t.client_id), Some(t.id)))
    .unwrap_or((None, None));

  // Fall back to Anonymous when instance lookup fails
  let anon = || AuthContext::Anonymous {
    client_id: instance_client_id.clone(),
    tenant_id: instance_tenant_id.clone(),
  };

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
          req.extensions_mut().insert(anon());
        }
      }
    } else {
      debug!("optional_auth_middleware: Authorization header is not valid UTF-8");
      req.extensions_mut().insert(anon());
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
            if let Some(client_id) = instance_client_id.clone() {
              let claims = extract_claims::<UserIdClaims>(&validated_token)?;
              let auth_context = AuthContext::Session {
                client_id,
                tenant_id: instance_tenant_id.clone().ok_or(AuthError::InvalidAccess)?,
                user_id: claims.sub.clone(),
                username: claims.preferred_username,
                role,
                token: validated_token,
              };
              req.extensions_mut().insert(auth_context);
            } else {
              debug!("optional_auth_middleware: no instance client_id, falling back to Anonymous");
              req.extensions_mut().insert(anon());
            }
          }
          Err(err) => {
            debug!(
              ?err,
              "optional_auth_middleware: token validation/refresh failed"
            );
            if should_clear_session(&err) {
              clear_session_auth_data(&session).await;
            }
            req.extensions_mut().insert(anon());
          }
        }
      }
      Ok(None) => {
        req.extensions_mut().insert(anon());
      }
      Err(err) => {
        debug!(?err, "optional_auth_middleware: error reading session");
        req.extensions_mut().insert(anon());
      }
    }
  } else {
    req.extensions_mut().insert(anon());
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

#[cfg(test)]
#[path = "test_auth_middleware.rs"]
mod test_auth_middleware;
