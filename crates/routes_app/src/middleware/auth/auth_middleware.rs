use crate::middleware::auth::AuthError;
use crate::middleware::token_service::DefaultTokenService;
use crate::middleware::MiddlewareError;
use axum::{
  extract::{Request, State},
  http::{header::HOST, HeaderMap},
  middleware::Next,
  response::Response,
};

use services::{
  access_token_key, refresh_token_key, AppService, AuthContext, DeploymentMode,
  SESSION_KEY_ACTIVE_CLIENT_ID, SESSION_KEY_USER_ID,
};
use services::{extract_claims, Claims, TenantError, UserIdClaims};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::debug;

pub const KEY_PREFIX_HEADER_BODHIAPP: &str = "X-BodhiApp-";

const SEC_FETCH_SITE_HEADER: &str = "sec-fetch-site";

/// Clears authentication data from the session.
/// Removes the active client's namespaced tokens and the active_client_id marker.
async fn clear_session_auth_data(session: &Session) {
  if let Ok(Some(client_id)) = session.get::<String>(SESSION_KEY_ACTIVE_CLIENT_ID).await {
    if let Err(e) = session
      .remove::<String>(&access_token_key(&client_id))
      .await
    {
      debug!(?e, "Failed to clear access token from session");
    }
    if let Err(e) = session
      .remove::<String>(&refresh_token_key(&client_id))
      .await
    {
      debug!(?e, "Failed to clear refresh token from session");
    }
  }
  if let Err(e) = session.remove::<String>(SESSION_KEY_ACTIVE_CLIENT_ID).await {
    debug!(?e, "Failed to clear active_client_id from session");
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

/// Try to read and validate/refresh the dashboard token from the session.
/// Returns `Some(valid_dashboard_token)` on success, `None` if no dashboard token or refresh fails.
async fn try_resolve_dashboard_token(
  session: &Session,
  token_service: &DefaultTokenService,
) -> Option<String> {
  let dashboard_token = session
    .get::<String>(services::DASHBOARD_ACCESS_TOKEN_KEY)
    .await
    .ok()
    .flatten()?;

  match token_service
    .get_valid_dashboard_token(session.clone(), dashboard_token)
    .await
  {
    Ok(token) => Some(token),
    Err(e) => {
      debug!(?e, "Dashboard token validation/refresh failed");
      None
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
  let deployment = app_service.setting_service().deployment_mode().await;
  let is_multi_tenant = deployment == DeploymentMode::MultiTenant;
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    tenant_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
    app_service.time_service(),
  );

  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    let header = header
      .to_str()
      .map_err(|err| AuthError::InvalidToken(err.to_string()))?;
    let auth_context = token_service.validate_bearer_token(header).await?;
    req.extensions_mut().insert(auth_context);
    Ok(next.run(req).await)
  } else if is_same_origin(req.headers()) {
    // Two-step lookup: read active_client_id, then namespaced access token
    let active_client_id = session
      .get::<String>(SESSION_KEY_ACTIVE_CLIENT_ID)
      .await
      .map_err(AuthError::from)?;
    let access_token = if let Some(ref cid) = active_client_id {
      session
        .get::<String>(&access_token_key(cid))
        .await
        .map_err(AuthError::from)?
    } else {
      None
    };
    if let Some(access_token) = access_token {
      debug!("auth_middleware: found access token in session, validating");
      // Resolve tenant from JWT azp claim
      let claims = extract_claims::<Claims>(&access_token)?;
      let tenant = tenant_service
        .get_tenant_by_client_id(&claims.azp)
        .await?
        .ok_or(TenantError::NotFound)?;

      let (access_token, role) = token_service
        .get_valid_session_token(session.clone(), access_token, &tenant)
        .await?;
      let user_claims = extract_claims::<UserIdClaims>(&access_token)?;

      let auth_context = if is_multi_tenant {
        // Multi-tenant: also resolve dashboard token
        let dashboard_token = try_resolve_dashboard_token(&session, &token_service)
          .await
          .ok_or(AuthError::RefreshTokenNotFound)?;
        AuthContext::MultiTenantSession {
          client_id: Some(tenant.client_id),
          tenant_id: Some(tenant.id),
          user_id: user_claims.sub.clone(),
          username: user_claims.preferred_username,
          role,
          token: Some(access_token),
          dashboard_token,
        }
      } else {
        AuthContext::Session {
          client_id: tenant.client_id,
          tenant_id: tenant.id,
          user_id: user_claims.sub.clone(),
          username: user_claims.preferred_username,
          role,
          token: access_token,
        }
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
  let deployment = app_service.setting_service().deployment_mode().await;
  let is_multi_tenant = deployment == DeploymentMode::MultiTenant;
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    tenant_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
    app_service.time_service(),
  );

  let anon = || AuthContext::Anonymous {
    deployment: deployment.clone(),
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
    // Two-step lookup: read active_client_id, then namespaced access token
    let active_client_id = session
      .get::<String>(SESSION_KEY_ACTIVE_CLIENT_ID)
      .await
      .unwrap_or(None);
    let access_token = if let Some(ref cid) = active_client_id {
      session
        .get::<String>(&access_token_key(cid))
        .await
        .unwrap_or(None)
    } else {
      None
    };
    match access_token {
      Some(access_token) => {
        let result: Result<AuthContext, AuthError> = async {
          let claims = extract_claims::<Claims>(&access_token)?;
          let tenant = tenant_service
            .get_tenant_by_client_id(&claims.azp)
            .await?
            .ok_or(TenantError::NotFound)?;
          let (validated_token, role) = token_service
            .get_valid_session_token(session.clone(), access_token, &tenant)
            .await?;
          let user_claims = extract_claims::<UserIdClaims>(&validated_token)?;

          if is_multi_tenant {
            let dashboard_token = try_resolve_dashboard_token(&session, &token_service)
              .await
              .ok_or(AuthError::RefreshTokenNotFound)?;
            Ok(AuthContext::MultiTenantSession {
              client_id: Some(tenant.client_id),
              tenant_id: Some(tenant.id),
              user_id: user_claims.sub.clone(),
              username: user_claims.preferred_username,
              role,
              token: Some(validated_token),
              dashboard_token,
            })
          } else {
            Ok(AuthContext::Session {
              client_id: tenant.client_id,
              tenant_id: tenant.id,
              user_id: user_claims.sub.clone(),
              username: user_claims.preferred_username,
              role,
              token: validated_token,
            })
          }
        }
        .await;

        match result {
          Ok(auth_context) => {
            req.extensions_mut().insert(auth_context);
          }
          Err(err) => {
            if should_clear_session(&err) {
              clear_session_auth_data(&session).await;
            }
            req.extensions_mut().insert(anon());
          }
        }
      }
      None if is_multi_tenant => {
        // No resource token, but might have dashboard token (dashboard-only session)
        match try_resolve_dashboard_token(&session, &token_service).await {
          Some(dashboard_token) => {
            // Extract user info from dashboard JWT
            match extract_claims::<UserIdClaims>(&dashboard_token) {
              Ok(user_claims) => {
                req
                  .extensions_mut()
                  .insert(AuthContext::MultiTenantSession {
                    client_id: None,
                    tenant_id: None,
                    user_id: user_claims.sub.clone(),
                    username: user_claims.preferred_username,
                    role: services::ResourceRole::Guest,
                    token: None,
                    dashboard_token,
                  });
              }
              Err(_) => {
                req.extensions_mut().insert(anon());
              }
            }
          }
          None => {
            req.extensions_mut().insert(anon());
          }
        }
      }
      None => {
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

#[cfg(test)]
#[path = "test_auth_middleware_isolation.rs"]
mod test_auth_middleware_isolation;
