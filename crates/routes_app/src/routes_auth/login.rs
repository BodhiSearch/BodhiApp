use crate::shared::utils::extract_request_host;
use crate::{AuthCallbackRequest, LoginError, RedirectResponse, ENDPOINT_LOGOUT};
use crate::{ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE};
use auth_middleware::{
  app_status_or_default, generate_random_string, AuthContext, SESSION_KEY_ACCESS_TOKEN,
  SESSION_KEY_REFRESH_TOKEN, SESSION_KEY_USER_ID,
};
use axum::{
  extract::State,
  http::{
    header::{HeaderMap, CACHE_CONTROL},
    StatusCode,
  },
  Extension, Json,
};
use base64::{engine::general_purpose, Engine as _};
use oauth2::url::Url;
use oauth2::{AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl};
use objs::{ApiError, OpenAIApiError, API_TAG_AUTH};
use server_core::RouterState;
use services::{extract_claims, AppStatus, Claims, SecretServiceExt, CHAT_PATH};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

/// Start OAuth flow - returns location for OAuth provider or home
#[utoipa::path(
    post,
    path = ENDPOINT_AUTH_INITIATE,
    tag = API_TAG_AUTH,
    operation_id = "initiateOAuthFlow",
    summary = "Initiate OAuth Authentication",
    description = "Initiates OAuth authentication flow. Returns OAuth authorization URL for unauthenticated users or home page URL for already authenticated users.",
    request_body = (),
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
pub async fn auth_initiate_handler(
  Extension(auth_context): Extension<AuthContext>,
  headers: HeaderMap,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
  let app_service = state.app_service();
  let setting_service = app_service.setting_service();

  // Early return if user is already authenticated
  if auth_context.is_authenticated() {
    return Ok((
      StatusCode::OK,
      [(CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
      Json(RedirectResponse {
        location: setting_service.frontend_default_url(),
      }),
    ));
  }

  // User not authenticated, generate auth URL
  let secret_service = app_service.secret_service();
  let app_reg_info = secret_service
    .app_reg_info()?
    .ok_or(LoginError::AppRegInfoNotFound)?;
  // Determine callback URL based on whether public host is explicitly configured
  let callback_url = if setting_service.get_public_host_explicit().is_some() {
    // Explicit configuration (including RunPod) - use configured callback URL
    setting_service.login_callback_url()
  } else {
    // Local/network installation mode - use request host or fallback
    if let Some(request_host) = extract_request_host(&headers) {
      // Use the actual request host for the callback URL
      format!(
        "{}://{}:{}{}",
        setting_service.public_scheme(),
        request_host,
        setting_service.public_port(),
        services::LOGIN_CALLBACK_PATH
      )
    } else {
      // Fallback to configured URL if host extraction fails
      setting_service.login_callback_url()
    }
  };
  let client_id = app_reg_info.client_id;

  // Generate simple random state for CSRF protection
  let state = generate_random_string(32);
  session
    .insert("oauth_state", &state)
    .await
    .map_err(LoginError::from)?;

  // Generate PKCE parameters
  let (code_verifier, code_challenge) = generate_pkce();
  session
    .insert("pkce_verifier", &code_verifier)
    .await
    .map_err(LoginError::from)?;

  // Store callback URL in session
  session
    .insert("callback_url", &callback_url)
    .await
    .map_err(LoginError::from)?;

  let scope = ["openid", "email", "profile", "roles"].join("%20");
  let login_url = format!(
    "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope={}",
    setting_service.login_url(), client_id, callback_url, state, code_challenge, scope
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
pub async fn auth_callback_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<AuthCallbackRequest>,
) -> Result<Json<RedirectResponse>, ApiError> {
  let app_service = state.app_service();
  let setting_service = app_service.setting_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();

  let app_status = app_status_or_default(&secret_service);
  if app_status == AppStatus::Setup {
    return Err(LoginError::AppStatusInvalid(app_status))?;
  }

  // Handle OAuth errors from the auth server
  if let Some(error) = &request.error {
    let error_message = if let Some(error_description) = &request.error_description {
      format!("{}: {}", error, error_description)
    } else {
      error.clone()
    };
    return Err(LoginError::OAuthError(error_message))?;
  }
  // Validate state parameter for CSRF protection
  let stored_state = session
    .get::<String>("oauth_state")
    .await
    .map_err(LoginError::from)?
    .ok_or(LoginError::SessionInfoNotFound)?;

  let received_state = request.state.as_ref().ok_or(LoginError::MissingState)?;

  if stored_state != *received_state {
    return Err(LoginError::StateDigestMismatch)?;
  }

  // Check for required authorization code
  let code = request.code.as_ref().ok_or(LoginError::MissingCode)?;

  // Get PKCE verifier
  let pkce_verifier = session
    .get::<String>("pkce_verifier")
    .await
    .map_err(LoginError::from)?
    .ok_or(LoginError::SessionInfoNotFound)?;

  // Get callback URL from session
  let callback_url = session
    .get::<String>("callback_url")
    .await
    .map_err(LoginError::from)?
    .ok_or(LoginError::SessionInfoNotFound)?;

  let app_reg_info = secret_service
    .app_reg_info()?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  // Exchange code for tokens
  let token_response = auth_service
    .exchange_auth_code(
      AuthorizationCode::new(code.to_string()),
      ClientId::new(app_reg_info.client_id.clone()),
      ClientSecret::new(app_reg_info.client_secret.clone()),
      RedirectUrl::new(callback_url.clone()).map_err(LoginError::from)?,
      PkceCodeVerifier::new(pkce_verifier),
    )
    .await?;

  // Clean up OAuth state and PKCE parameters
  session
    .remove::<String>("oauth_state")
    .await
    .map_err(LoginError::from)?;
  session
    .remove::<String>("pkce_verifier")
    .await
    .map_err(LoginError::from)?;

  let status_resource_admin = app_status == AppStatus::ResourceAdmin;
  let mut access_token = token_response.0.secret().to_string();
  let mut refresh_token = token_response.1.secret().to_string();

  // Extract claims from JWT token to get user information
  let claims = extract_claims::<Claims>(&access_token)?;
  let user_id = claims.sub.clone();

  if status_resource_admin {
    auth_service
      .make_resource_admin(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        &user_id,
      )
      .await?;
    secret_service.set_app_status(&AppStatus::Ready)?;
    let (new_access_token, new_refresh_token) = auth_service
      .refresh_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        &refresh_token,
      )
      .await?;
    access_token = new_access_token;
    refresh_token =
      new_refresh_token.expect("refresh token is missing when refreshing an existing token");
  }

  // Store user_id and tokens in session
  session
    .insert(SESSION_KEY_USER_ID, &user_id)
    .await
    .map_err(LoginError::from)?;
  session
    .insert(SESSION_KEY_ACCESS_TOKEN, access_token)
    .await
    .map_err(LoginError::from)?;
  session
    .insert(SESSION_KEY_REFRESH_TOKEN, refresh_token)
    .await
    .map_err(LoginError::from)?;

  // Determine redirect URL using callback URL host
  let ui_setup_resume = if let Ok(parsed_url) = Url::parse(&callback_url) {
    let mut new_url = parsed_url.clone();
    new_url.set_path(CHAT_PATH);
    new_url.set_query(None);
    new_url.to_string()
  } else {
    setting_service.frontend_default_url()
  };

  // Clean up callback URL from session
  session
    .remove::<String>("callback_url")
    .await
    .map_err(LoginError::from)?;

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
pub async fn logout_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<RedirectResponse>, ApiError> {
  let setting_service = state.app_service().setting_service();
  session.delete().await.map_err(LoginError::SessionDelete)?;
  let ui_login = format!("{}/ui/login", setting_service.public_server_url());
  Ok(Json(RedirectResponse { location: ui_login }))
}
