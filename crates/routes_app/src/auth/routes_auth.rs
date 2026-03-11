use crate::middleware::{
  access_token_key, generate_random_string, refresh_token_key, SESSION_KEY_ACTIVE_CLIENT_ID,
  SESSION_KEY_USER_ID,
};
use crate::shared::{utils::extract_request_host, AuthScope};
use crate::{ApiError, OpenAIApiError};
use crate::{
  AuthCallbackRequest, AuthInitiateRequest, AuthRouteError, RedirectResponse, API_TAG_AUTH,
  ENDPOINT_LOGOUT,
};
use crate::{ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE};
use axum::{
  http::{
    header::{HeaderMap, CACHE_CONTROL},
    StatusCode,
  },
  Json,
};
use base64::{engine::general_purpose, Engine as _};
use oauth2::url::Url;
use oauth2::{AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl};
use services::{extract_claims, AppStatus, Claims, CHAT_PATH};
use sha2::{Digest, Sha256};
use tower_sessions::Session;

/// Start OAuth flow - returns location for OAuth provider or home
#[utoipa::path(
    post,
    path = ENDPOINT_AUTH_INITIATE,
    tag = API_TAG_AUTH,
    operation_id = "initiateOAuthFlow",
    summary = "Initiate OAuth Authentication",
    description = "Initiates OAuth authentication flow. Returns OAuth authorization URL for unauthenticated users or home page URL for already authenticated users.",
    request_body(content = AuthInitiateRequest, description = "OAuth initiate parameters"),
    responses(
        (status = 201, description = "User not authenticated, OAuth authorization URL provided", body = RedirectResponse,
         example = json!({
             "location": "https://auth.example.com/auth?client_id=bodhi&redirect_uri=..."
         })),
        (status = 200, description = "User already authenticated, home page URL provided", body = RedirectResponse,
         example = json!({
             "location": "https://app.example.com/dashboard"
         })),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn auth_initiate(
  auth_scope: AuthScope,
  headers: HeaderMap,
  session: Session,
  Json(request): Json<AuthInitiateRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
  let auth_context = auth_scope.auth_context();
  let settings = auth_scope.settings();

  // Early return if user is already authenticated with the SAME resource tenant.
  // Dashboard-only MultiTenantSession (token: None) must still initiate tenant OAuth.
  // When requesting OAuth for a different tenant (invite flow), proceed even if authenticated.
  if auth_context.is_authenticated()
    && auth_context.token().is_some()
    && auth_context.client_id() == Some(request.client_id.as_str())
  {
    return Ok((
      StatusCode::OK,
      [(CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
      Json(RedirectResponse {
        location: settings.frontend_default_url().await,
      }),
    ));
  }

  // User not authenticated, generate auth URL
  let tenant_svc = auth_scope.tenants();
  let instance = tenant_svc
    .get_tenant_by_client_id(&request.client_id)
    .await?
    .ok_or(AuthRouteError::TenantNotFound)?;
  // Store client_id in session for callback retrieval
  session
    .insert("auth_client_id", &request.client_id)
    .await
    .map_err(AuthRouteError::from)?;
  // Determine callback URL based on whether public host is explicitly configured
  let callback_url = if settings.get_public_host_explicit().await.is_some() {
    // Explicit configuration (including RunPod) - use configured callback URL
    settings.login_callback_url().await
  } else {
    // Local/network installation mode - use request host or fallback
    if let Some(request_host) = extract_request_host(&headers) {
      // Use the actual request host for the callback URL
      format!(
        "{}://{}:{}{}",
        settings.public_scheme().await,
        request_host,
        settings.public_port().await,
        services::LOGIN_CALLBACK_PATH
      )
    } else {
      // Fallback to configured URL if host extraction fails
      settings.login_callback_url().await
    }
  };
  let client_id = instance.client_id;

  // Generate simple random state for CSRF protection
  let state = generate_random_string(32);
  session
    .insert("oauth_state", &state)
    .await
    .map_err(AuthRouteError::from)?;

  // Generate PKCE parameters
  let (code_verifier, code_challenge) = generate_pkce();
  session
    .insert("pkce_verifier", &code_verifier)
    .await
    .map_err(AuthRouteError::from)?;

  // Store callback URL in session
  session
    .insert("callback_url", &callback_url)
    .await
    .map_err(AuthRouteError::from)?;

  let scope = ["openid", "email", "profile", "roles"].join("%20");
  let login_url = format!(
    "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope={}",
    settings.login_url().await, client_id, callback_url, state, code_challenge, scope
  );

  Ok((
    StatusCode::CREATED,
    [(CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
    Json(RedirectResponse {
      location: login_url,
    }),
  ))
}

/// Complete OAuth flow with authorization code
#[utoipa::path(
    post,
    path = ENDPOINT_AUTH_CALLBACK,
    tag = API_TAG_AUTH,
    operation_id = "completeOAuthFlow",
    summary = "Complete OAuth Authentication",
    description = "Completes the OAuth authentication flow by exchanging authorization code for tokens and establishing user session.",
    request_body(
        content = AuthCallbackRequest,
        description = "OAuth callback parameters from authorization server",
        example = json!({
            "code": "auth_code_123",
            "state": "random_state_456"
        })
    ),
    responses(
        (status = 200, description = "OAuth flow completed successfully, user authenticated", body = RedirectResponse,
         example = json!({
             "location": "https://app.example.com/dashboard"
         })),
        (status = 422, description = "OAuth error, invalid request parameters, or state mismatch", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "State parameter mismatch",
                 "type": "invalid_request_error",
                 "code": "oauth_state_mismatch"
             }
         })),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn auth_callback(
  auth_scope: AuthScope,
  session: Session,
  Json(request): Json<AuthCallbackRequest>,
) -> Result<Json<RedirectResponse>, ApiError> {
  let settings = auth_scope.settings();
  let tenant_svc = auth_scope.tenants();
  let auth_flow = auth_scope.auth_flow();

  // Handle OAuth errors from the auth server
  if let Some(error) = &request.error {
    let error_message = if let Some(error_description) = &request.error_description {
      format!("{}: {}", error, error_description)
    } else {
      error.clone()
    };
    return Err(AuthRouteError::OAuthError(error_message))?;
  }
  // Validate state parameter for CSRF protection
  let stored_state = session
    .get::<String>("oauth_state")
    .await
    .map_err(AuthRouteError::from)?
    .ok_or(AuthRouteError::SessionInfoNotFound)?;

  let received_state = request.state.as_ref().ok_or(AuthRouteError::MissingState)?;

  if stored_state != *received_state {
    return Err(AuthRouteError::StateDigestMismatch)?;
  }

  // Check for required authorization code
  let code = request.code.as_ref().ok_or(AuthRouteError::MissingCode)?;

  // Get PKCE verifier
  let pkce_verifier = session
    .get::<String>("pkce_verifier")
    .await
    .map_err(AuthRouteError::from)?
    .ok_or(AuthRouteError::SessionInfoNotFound)?;

  // Get callback URL from session
  let callback_url = session
    .get::<String>("callback_url")
    .await
    .map_err(AuthRouteError::from)?
    .ok_or(AuthRouteError::SessionInfoNotFound)?;

  let auth_client_id = session
    .get::<String>("auth_client_id")
    .await
    .map_err(AuthRouteError::from)?
    .ok_or(AuthRouteError::SessionInfoNotFound)?;
  let instance = tenant_svc
    .get_tenant_by_client_id(&auth_client_id)
    .await?
    .ok_or(AuthRouteError::TenantNotFound)?;

  // Use the specific tenant's status — app_status_or_default() checks standalone app only
  // and returns Setup in multi-tenant mode where no standalone app exists.
  let app_status = instance.status.clone();
  if app_status == AppStatus::Setup {
    return Err(AuthRouteError::AppStatusInvalid(app_status))?;
  }

  // Exchange code for tokens
  let token_response = auth_flow
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(instance.client_id.clone()),
      ClientSecret::new(instance.client_secret.clone()),
      RedirectUrl::new(callback_url.clone()).map_err(AuthRouteError::from)?,
      PkceCodeVerifier::new(pkce_verifier),
    )
    .await?;

  // Clean up OAuth state and PKCE parameters
  session
    .remove::<String>("oauth_state")
    .await
    .map_err(AuthRouteError::from)?;
  session
    .remove::<String>("pkce_verifier")
    .await
    .map_err(AuthRouteError::from)?;

  let status_resource_admin = app_status == AppStatus::ResourceAdmin;
  let mut access_token = token_response.0.secret().to_string();
  let mut refresh_token = token_response.1.secret().to_string();

  // Extract claims from JWT token to get user information
  let claims = extract_claims::<Claims>(&access_token)?;
  let user_id = claims.sub.clone();

  if status_resource_admin {
    auth_flow
      .make_resource_admin(&instance.client_id, &instance.client_secret, &user_id)
      .await?;
    tenant_svc.set_tenant_ready(&instance.id, &user_id).await?;
    let (new_access_token, new_refresh_token) = auth_flow
      .refresh_token(&instance.client_id, &instance.client_secret, &refresh_token)
      .await?;
    access_token = new_access_token;
    refresh_token = new_refresh_token.ok_or(AuthRouteError::SessionInfoNotFound)?;
  }

  // Store user_id and tokens in session (namespaced by client_id)
  session
    .insert(SESSION_KEY_USER_ID, &user_id)
    .await
    .map_err(AuthRouteError::from)?;
  session
    .insert(SESSION_KEY_ACTIVE_CLIENT_ID, &instance.client_id)
    .await
    .map_err(AuthRouteError::from)?;
  session
    .insert(&access_token_key(&instance.client_id), access_token)
    .await
    .map_err(AuthRouteError::from)?;
  session
    .insert(&refresh_token_key(&instance.client_id), refresh_token)
    .await
    .map_err(AuthRouteError::from)?;

  // Determine redirect URL using callback URL host
  let ui_setup_resume = if let Ok(parsed_url) = Url::parse(&callback_url) {
    let mut new_url = parsed_url.clone();
    new_url.set_path(CHAT_PATH);
    new_url.set_query(None);
    new_url.to_string()
  } else {
    settings.frontend_default_url().await
  };

  // Clean up callback URL and auth_client_id from session
  session
    .remove::<String>("callback_url")
    .await
    .map_err(AuthRouteError::from)?;
  session
    .remove::<String>("auth_client_id")
    .await
    .map_err(AuthRouteError::from)?;

  // Return successful redirect
  Ok(Json(RedirectResponse {
    location: ui_setup_resume,
  }))
}

pub fn generate_pkce() -> (String, String) {
  let code_verifier = generate_random_string(43);
  let code_challenge =
    general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
  (code_verifier, code_challenge)
}

/// Logout the current user by destroying their session
#[utoipa::path(
    post,
    path = ENDPOINT_LOGOUT,
    tag = API_TAG_AUTH,
    operation_id = "logoutUser",
    summary = "Logout User",
    description = "Logs out the current user by destroying their session and returns redirect URL to login page",
    responses(
        (status = 200, description = "User logged out successfully", body = RedirectResponse,
         example = json!({
             "location": "https://app.example.com/login"
         })),
    )
)]
pub async fn auth_logout(
  auth_scope: AuthScope,
  session: Session,
) -> Result<Json<RedirectResponse>, ApiError> {
  let settings = auth_scope.settings();
  session
    .delete()
    .await
    .map_err(AuthRouteError::SessionDelete)?;
  let ui_login = format!("{}/ui/login", settings.public_server_url().await);
  Ok(Json(RedirectResponse { location: ui_login }))
}

#[cfg(test)]
#[path = "test_login_initiate.rs"]
mod test_login_initiate;

#[cfg(test)]
#[path = "test_login_callback.rs"]
mod test_login_callback;

#[cfg(test)]
#[path = "test_login_logout.rs"]
mod test_login_logout;

#[cfg(test)]
#[path = "test_login_resource_admin.rs"]
mod test_login_resource_admin;
