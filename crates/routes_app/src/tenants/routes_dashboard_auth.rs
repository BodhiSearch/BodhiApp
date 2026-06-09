use crate::auth::generate_pkce;
use crate::middleware::generate_random_string;
use crate::shared::AuthScope;
use crate::{AuthCallbackRequest, BodhiErrorResponse, RedirectResponse};
use crate::{
  DashboardAuthRouteError, DASHBOARD_ACCESS_TOKEN_KEY, DASHBOARD_REFRESH_TOKEN_KEY,
  ENDPOINT_DASHBOARD_AUTH_CALLBACK, ENDPOINT_DASHBOARD_AUTH_INITIATE,
};
use axum::{
  http::{header::HeaderMap, StatusCode},
  Json,
};
use oauth2::{AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl};
use services::extract_claims;
use services::Claims;
use tower_sessions::Session;
use tracing::warn;

/// Start dashboard OAuth flow - returns location for OAuth provider
#[utoipa::path(
    post,
    path = ENDPOINT_DASHBOARD_AUTH_INITIATE,
    tag = "auth",
    operation_id = "initiateDashboardOAuthFlow",
    summary = "Initiate Dashboard OAuth Authentication",
    description = "Initiates the dashboard OAuth authentication flow using the multi-tenant client. Only available in multi-tenant mode.",
    request_body = (),
    responses(
        (status = 201, description = "OAuth authorization URL provided for dashboard login", body = RedirectResponse),
        (status = 200, description = "User already has a valid dashboard token", body = RedirectResponse),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn dashboard_auth_initiate(
  auth_scope: AuthScope,
  _headers: HeaderMap,
  session: Session,
) -> Result<impl axum::response::IntoResponse, BodhiErrorResponse> {
  if !auth_scope.auth_context().is_multi_tenant() {
    return Err(DashboardAuthRouteError::NotMultiTenant)?;
  }

  let settings = auth_scope.settings();

  // secret is only needed at callback time
  let client_id = settings.multitenant_client_id().await?;

  if let Some(existing_token) = session
    .get::<String>(DASHBOARD_ACCESS_TOKEN_KEY)
    .await
    .map_err(DashboardAuthRouteError::from)?
  {
    if extract_claims::<Claims>(&existing_token).is_ok() {
      return Ok((
        StatusCode::OK,
        Json(RedirectResponse {
          location: "/ui/login".to_string(),
        }),
      ));
    }
  }

  let (code_verifier, code_challenge) = generate_pkce();

  // random state for CSRF protection
  let state = generate_random_string(32);

  session
    .insert("dashboard_oauth_state", &state)
    .await
    .map_err(DashboardAuthRouteError::from)?;
  session
    .insert("dashboard_pkce_verifier", &code_verifier)
    .await
    .map_err(DashboardAuthRouteError::from)?;

  let callback_url = settings.dashboard_callback_url().await;
  session
    .insert("dashboard_callback_url", &callback_url)
    .await
    .map_err(DashboardAuthRouteError::from)?;

  let scope = ["openid", "email", "profile"].join("%20");
  let login_url = format!(
    "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope={}",
    settings.login_url().await, client_id, callback_url, state, code_challenge, scope
  );

  Ok((
    StatusCode::CREATED,
    Json(RedirectResponse {
      location: login_url,
    }),
  ))
}

/// Complete dashboard OAuth flow with authorization code
#[utoipa::path(
    post,
    path = ENDPOINT_DASHBOARD_AUTH_CALLBACK,
    tag = "auth",
    operation_id = "completeDashboardOAuthFlow",
    summary = "Complete Dashboard OAuth Authentication",
    description = "Completes the dashboard OAuth authentication flow by exchanging authorization code for tokens.",
    request_body(
        content = AuthCallbackRequest,
        description = "OAuth callback parameters from authorization server"
    ),
    responses(
        (status = 200, description = "Dashboard OAuth flow completed successfully", body = RedirectResponse),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn dashboard_auth_callback(
  auth_scope: AuthScope,
  session: Session,
  Json(request): Json<AuthCallbackRequest>,
) -> Result<Json<RedirectResponse>, BodhiErrorResponse> {
  if !auth_scope.auth_context().is_multi_tenant() {
    return Err(DashboardAuthRouteError::NotMultiTenant)?;
  }

  let settings = auth_scope.settings();
  let auth_flow = auth_scope.auth_flow();

  if let Some(error) = &request.error {
    let error_message = if let Some(error_description) = &request.error_description {
      format!("{}: {}", error, error_description)
    } else {
      error.clone()
    };
    return Err(DashboardAuthRouteError::OAuthError(error_message))?;
  }

  // validate state parameter for CSRF protection
  let stored_state = session
    .get::<String>("dashboard_oauth_state")
    .await
    .map_err(DashboardAuthRouteError::from)?
    .ok_or(DashboardAuthRouteError::SessionInfoNotFound)?;

  let received_state = request
    .state
    .as_ref()
    .ok_or(DashboardAuthRouteError::MissingState)?;

  if stored_state != *received_state {
    return Err(DashboardAuthRouteError::StateDigestMismatch)?;
  }

  let code = request
    .code
    .as_ref()
    .ok_or(DashboardAuthRouteError::MissingCode)?;

  let pkce_verifier = session
    .get::<String>("dashboard_pkce_verifier")
    .await
    .map_err(DashboardAuthRouteError::from)?
    .ok_or(DashboardAuthRouteError::SessionInfoNotFound)?;

  let callback_url = session
    .get::<String>("dashboard_callback_url")
    .await
    .map_err(DashboardAuthRouteError::from)?
    .ok_or(DashboardAuthRouteError::SessionInfoNotFound)?;

  let client_id = settings.multitenant_client_id().await?;
  let client_secret = settings.multitenant_client_secret().await?;

  let token_response = auth_flow
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(client_id),
      ClientSecret::new(client_secret),
      RedirectUrl::new(callback_url).map_err(DashboardAuthRouteError::from)?,
      PkceCodeVerifier::new(pkce_verifier),
    )
    .await?;

  session
    .remove::<String>("dashboard_oauth_state")
    .await
    .map_err(DashboardAuthRouteError::from)?;
  session
    .remove::<String>("dashboard_pkce_verifier")
    .await
    .map_err(DashboardAuthRouteError::from)?;
  session
    .remove::<String>("dashboard_callback_url")
    .await
    .map_err(DashboardAuthRouteError::from)?;

  // Rotate session ID to prevent session fixation attacks (AUTH-VULN-07)
  if let Err(e) = session.cycle_id().await {
    warn!(
      "Failed to rotate session ID after dashboard OAuth callback: {}",
      e
    );
  }

  let access_token = token_response.0.secret().to_string();
  let refresh_token = token_response.1.secret().to_string();

  session
    .insert(DASHBOARD_ACCESS_TOKEN_KEY, &access_token)
    .await
    .map_err(DashboardAuthRouteError::from)?;
  session
    .insert(DASHBOARD_REFRESH_TOKEN_KEY, &refresh_token)
    .await
    .map_err(DashboardAuthRouteError::from)?;

  Ok(Json(RedirectResponse {
    location: "/ui/login".to_string(),
  }))
}

#[cfg(test)]
#[path = "test_dashboard_auth.rs"]
mod test_dashboard_auth;
