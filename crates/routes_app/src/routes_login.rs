use crate::utils::extract_request_host;
use crate::{LoginError, RedirectResponse, ENDPOINT_APPS_REQUEST_ACCESS, ENDPOINT_LOGOUT};
use crate::{ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE};
use auth_middleware::{
  app_status_or_default, generate_random_string, KEY_HEADER_BODHIAPP_TOKEN,
  SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN, SESSION_KEY_USER_ID,
};
use axum::{
  extract::State,
  http::{
    header::{HeaderMap, CACHE_CONTROL},
    StatusCode,
  },
  Json,
};
use base64::{engine::general_purpose, Engine as _};
use oauth2::url::Url;
use oauth2::{AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl};
use objs::{ApiError, AppError, BadRequestError, ErrorType, OpenAIApiError, API_TAG_AUTH};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  db::AppClientToolsetConfigRow, extract_claims, AppAccessRequest, AppAccessResponse,
  AppClientToolset, AppStatus, Claims, SecretServiceExt, CHAT_PATH, DOWNLOAD_MODELS_PATH,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;
use utoipa::ToSchema;

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
  headers: HeaderMap,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
  let app_service = state.app_service();
  let setting_service = app_service.setting_service();

  // Early return if user is already authenticated
  if headers.get(KEY_HEADER_BODHIAPP_TOKEN).is_some() {
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

  let received_state = request
    .state
    .as_ref()
    .ok_or_else(|| BadRequestError::new("missing state parameter".to_string()))?;

  if stored_state != *received_state {
    return Err(BadRequestError::new(
      "state parameter in callback does not match with the one sent in login request".to_string(),
    ))?;
  }

  // Check for required authorization code
  let code = request
    .code
    .as_ref()
    .ok_or_else(|| BadRequestError::new("missing code parameter".to_string()))?;

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
  let ui_setup_resume = if status_resource_admin {
    // Extract host from callback URL to construct download-models URL
    if let Ok(parsed_url) = Url::parse(&callback_url) {
      let mut new_url = parsed_url.clone();
      new_url.set_path(DOWNLOAD_MODELS_PATH);
      new_url.set_query(None);
      new_url.to_string()
    } else {
      // Fallback to configured URL if parsing fails
      format!(
        "{}{}",
        setting_service.public_server_url(),
        DOWNLOAD_MODELS_PATH
      )
    }
  } else {
    // Extract host from callback URL to construct frontend URL
    if let Ok(parsed_url) = Url::parse(&callback_url) {
      let mut new_url = parsed_url.clone();
      new_url.set_path(CHAT_PATH);
      new_url.set_query(None);
      new_url.to_string()
    } else {
      // Fallback to configured URL if parsing fails
      setting_service.frontend_default_url()
    }
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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LogoutError {
  #[error("Failed to delete session: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, code = "logout_error-session_delete_error", args_delegate = false)]
  SessionDelete(#[from] tower_sessions::session::Error),
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
  session.delete().await.map_err(LogoutError::from)?;
  let ui_login = format!("{}/ui/login", setting_service.public_server_url());
  Ok(Json(RedirectResponse { location: ui_login }))
}
/// Request access for an app client to this resource server
#[utoipa::path(
    post,
    path = ENDPOINT_APPS_REQUEST_ACCESS,
    tag = API_TAG_AUTH,
    operation_id = "requestAccess",
    summary = "Request Resource Access",
    description = "Requests access permissions for an application client to access this resource server's protected resources. Supports caching via optional version parameter.",
    request_body(
        content = AppAccessRequest,
        description = "Application client requesting access",
        example = json!({
            "app_client_id": "my_app_client_123",
            "toolset_scope_ids": ["uuid-for-toolset-scope-1"],
            "version": "v1.0.0"
        })
    ),
    responses(
        (status = 200, description = "Access granted successfully", body = AppAccessResponse,
         example = json!({
             "scope": "scope_resource_bodhi-server",
             "toolsets": [{"id": "builtin-exa-web-search", "scope": "scope_toolset-builtin-exa-web-search"}],
             "app_client_config_version": "v1.0.0"
         })),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn request_access_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<AppAccessRequest>,
) -> Result<Json<AppAccessResponse>, ApiError> {
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();
  let db_service = app_service.db_service();

  // Get app registration info
  let app_reg_info = secret_service
    .app_reg_info()?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  // Check cache if version is provided
  if let Ok(Some(cached)) = db_service
    .get_app_client_toolset_config(&request.app_client_id)
    .await
  {
    // Require both versions to be Some and matching for cache hit
    if let (Some(ref version), Some(ref cached_version)) =
      (&request.version, &cached.config_version)
    {
      if cached_version == version {
        // Version matches - check if all requested scope_ids are already added to resource-client
        let toolsets: Vec<AppClientToolset> =
          serde_json::from_str(&cached.toolsets_json).unwrap_or_default();

        let all_scopes_added = if let Some(ref requested_scope_ids) = request.toolset_scope_ids {
          // Check if all requested scope_ids have added_to_resource_client=true
          requested_scope_ids.iter().all(|requested_id| {
            toolsets
              .iter()
              .any(|t| &t.scope_id == requested_id && t.added_to_resource_client == Some(true))
          })
        } else {
          // No toolset_scope_ids requested - cache hit
          true
        };

        if all_scopes_added {
          // Full cache hit - return cached data
          return Ok(Json(AppAccessResponse {
            scope: cached.resource_scope,
            toolsets,
            app_client_config_version: cached.config_version.clone(),
          }));
        }
      }
    }
  }

  // Cache miss or scope_ids not yet added - call auth server with toolset_scope_ids
  let auth_response = auth_service
    .request_access(
      &app_reg_info.client_id,
      &app_reg_info.client_secret,
      &request.app_client_id,
      request.toolset_scope_ids.clone(),
    )
    .await?;

  // Log if version mismatch or None cases
  match (&request.version, &auth_response.app_client_config_version) {
    (Some(req_ver), Some(resp_ver)) if req_ver != resp_ver => {
      tracing::warn!(
        "App client {} sent version {} but auth server returned version {}",
        request.app_client_id,
        req_ver,
        resp_ver
      );
    }
    (Some(_), None) | (None, Some(_)) | (None, None) => {
      tracing::warn!(
        "App client {} does not have config versioning configured properly",
        request.app_client_id
      );
    }
    _ => {} // versions match, no log needed
  }

  // Store/update cache
  let toolsets_json =
    serde_json::to_string(&auth_response.toolsets).unwrap_or_else(|_| "[]".to_string());
  let now = db_service.now().timestamp();
  let config_row = AppClientToolsetConfigRow {
    id: 0, // Will be set by DB
    app_client_id: request.app_client_id.clone(),
    config_version: auth_response.app_client_config_version.clone(),
    toolsets_json,
    resource_scope: auth_response.scope.clone(),
    created_at: now,
    updated_at: now,
  };

  if let Err(e) = db_service
    .upsert_app_client_toolset_config(&config_row)
    .await
  {
    tracing::warn!(
      "Failed to cache app client toolset config for {}: {}",
      request.app_client_id,
      e
    );
    // Don't fail the request if caching fails
  }

  Ok(Json(AppAccessResponse {
    scope: auth_response.scope,
    toolsets: auth_response.toolsets,
    app_client_config_version: auth_response.app_client_config_version,
  }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({
    "code": "auth_code_123",
    "state": "random_state_456"
}))]
pub struct AuthCallbackRequest {
  /// OAuth authorization code from successful authentication (required for success flow)
  #[schema(example = "auth_code_123")]
  pub code: Option<String>,
  /// OAuth state parameter for CSRF protection (must match initiated request)
  #[schema(example = "random_state_456")]
  pub state: Option<String>,
  /// OAuth error code if authentication failed (e.g., "access_denied")
  #[schema(example = "access_denied")]
  pub error: Option<String>,
  /// Human-readable OAuth error description if authentication failed
  #[schema(example = "The user denied the request")]
  pub error_description: Option<String>,
  /// Additional OAuth 2.1 parameters sent by the authorization server
  #[serde(flatten)]
  #[schema(additional_properties = true)]
  pub additional_params: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
  use crate::{
    auth_callback_handler, auth_initiate_handler, generate_pkce, logout_handler,
    request_access_handler, RedirectResponse,
  };
  use anyhow_trace::anyhow_trace;
  use auth_middleware::{generate_random_string, inject_optional_auth_info};
  use axum::body::to_bytes;
  use axum::{
    http::{status::StatusCode, Request},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
  };
  use axum_test::TestServer;
  use chrono::Utc;
  use mockito::{Matcher, Server};
  use oauth2::{AccessToken, PkceCodeVerifier, RefreshToken};
  use objs::test_utils::temp_bodhi_home;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::{
    db::AppClientToolsetConfigRow,
    test_utils::{
      build_token, expired_token, test_auth_service, token, AppServiceStub, AppServiceStubBuilder,
      SecretServiceStub, SessionTestExt, SettingServiceStub,
    },
    AppAccessResponse, AppRegInfo, AppService, AuthServiceError, MockAuthService, SecretServiceExt,
    SqliteSessionService, BODHI_AUTH_REALM, BODHI_AUTH_URL,
  };
  use services::{AppStatus, BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
  use std::{collections::HashMap, sync::Arc};
  use tempfile::TempDir;
  use time::{Duration, OffsetDateTime};
  use tower::ServiceExt;
  use tower_sessions::{
    session::{Id, Record},
    Session, SessionStore,
  };
  use url::Url;
  use uuid::Uuid;

  #[rstest]
  #[case(
        SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
            }),
    )]
  #[tokio::test]
  async fn test_auth_initiate_handler(
    #[case] secret_service: SecretServiceStub,
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let callback_url = "http://localhost:3000/ui/auth/callback";
    let login_url = "http://test-id.getbodhi.app/realms/test-realm/protocol/openid-connect/auth";

    let setting_service = SettingServiceStub::with_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "localhost".to_string()),
      (BODHI_PORT.to_string(), "3000".to_string()),
      (
        BODHI_AUTH_URL.to_string(),
        "http://test-id.getbodhi.app".to_string(),
      ),
      (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
    ]));
    let dbfile = temp_bodhi_home.path().join("test.db");
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .setting_service(Arc::new(setting_service))
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let resp = router
      .oneshot(Request::post("/auth/initiate").json(json! {{}})?)
      .await?;

    let status = resp.status();
    assert_eq!(status, StatusCode::CREATED);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
    let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
    assert!(body.location.starts_with(login_url));

    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!("code", query_params.get("response_type").unwrap());
    assert_eq!("test_client_id", query_params.get("client_id").unwrap());
    assert_eq!(callback_url, query_params.get("redirect_uri").unwrap());
    assert!(query_params.contains_key("state"));
    assert!(query_params.contains_key("code_challenge"));
    assert_eq!("S256", query_params.get("code_challenge_method").unwrap());
    assert_eq!(
      "openid email profile roles",
      query_params.get("scope").unwrap()
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_initiate_handler_loopback_host_detection(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    });

    // Configure with default 0.0.0.0 host (loopback)
    let setting_service = SettingServiceStub::with_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (BODHI_PORT.to_string(), "1135".to_string()),
      (
        BODHI_AUTH_URL.to_string(),
        "http://test-id.getbodhi.app".to_string(),
      ),
      (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
    ]));

    let dbfile = temp_bodhi_home.path().join("test.db");
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .setting_service(Arc::new(setting_service))
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    // Request with localhost:1135 Host header
    let resp = router
      .oneshot(
        Request::post("/auth/initiate")
          .header("Host", "localhost:1135")
          .json(json! {{}})?,
      )
      .await?;

    assert_eq!(StatusCode::CREATED, resp.status());
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
    let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

    // Should use localhost from Host header instead of configured 0.0.0.0
    let callback_url = query_params.get("redirect_uri").unwrap();
    assert_eq!("http://localhost:1135/ui/auth/callback", callback_url);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_initiate_handler_network_host_usage(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_reg_info(&AppRegInfo {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
    });

    // Configure with default settings (no explicit public host)
    let setting_service = SettingServiceStub::with_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (BODHI_PORT.to_string(), "1135".to_string()),
      (
        BODHI_AUTH_URL.to_string(),
        "http://test-id.getbodhi.app".to_string(),
      ),
      (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
    ]));

    let dbfile = temp_bodhi_home.path().join("test.db");
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .setting_service(Arc::new(setting_service))
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    // Request with network host header
    let resp = router
      .oneshot(
        Request::post("/auth/initiate")
          .header("Host", "192.168.1.100:1135")
          .json(json! {{}})?,
      )
      .await?;

    assert_eq!(StatusCode::CREATED, resp.status());
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
    let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

    // Should now use the request host for network installation support
    let callback_url = query_params.get("redirect_uri").unwrap();
    assert_eq!("http://192.168.1.100:1135/ui/auth/callback", callback_url);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_initiate_handler_logged_in_redirects_to_home(
    temp_bodhi_home: TempDir,
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let (status, body) = auth_initiate_handler_with_token_response(temp_bodhi_home, token).await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
      body
        .location
        .starts_with("http://frontend.localhost:3000/ui/chat"),
      "{} does not start with http://frontend.localhost:3000/ui/chat",
      body.location
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_initiate_handler_for_expired_token_redirects_to_login(
    temp_bodhi_home: TempDir,
    expired_token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = expired_token;
    let (status, body) = auth_initiate_handler_with_token_response(temp_bodhi_home, token).await?;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body
      .location
      .starts_with("http://id.localhost/realms/test-realm/protocol/openid-connect/auth"));
    Ok(())
  }

  async fn auth_initiate_handler_with_token_response(
    temp_bodhi_home: TempDir,
    token: String,
  ) -> anyhow::Result<(StatusCode, RedirectResponse)> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = SqliteSessionService::build_session_service(dbfile).await;
    let record = set_token_in_session(&session_service, &token).await?;
    let app_service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_bodhi_home)
      .setting_service(Arc::new(SettingServiceStub::default().append_settings(
        HashMap::from([
          (BODHI_SCHEME.to_string(), "http".to_string()),
          (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
          (BODHI_PORT.to_string(), "3000".to_string()),
        ]),
      )))
      .with_sqlite_session_service(Arc::new(session_service))
      .with_secret_service()
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .route_layer(from_fn_with_state(state.clone(), inject_optional_auth_info))
      .with_state(state)
      .layer(app_service.session_service().session_layer());
    let resp = router
      .oneshot(
        Request::post("/auth/initiate")
          .header("Cookie", format!("bodhiapp_session_id={}", record.id))
          .header("Sec-Fetch-Site", "same-origin")
          .json(json! {{}})?,
      )
      .await?;
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
    let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
    Ok((status, body))
  }

  async fn set_token_in_session(
    session_service: &SqliteSessionService,
    token: &str,
  ) -> Result<Record, anyhow::Error> {
    let id = Id::default();
    let mut record = Record {
      id,
      data: maplit::hashmap! {
        "access_token".to_string() => Value::String(token.to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + time::Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;
    Ok(record)
  }

  #[rstest]
  fn test_generate_pkce() {
    let (generated_verifier, challenge) = generate_pkce();
    assert_eq!(43, generated_verifier.len());
    assert_eq!(43, challenge.len());
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_auth_callback_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    // Create a token with the correct scope that matches what auth_initiate_handler uses
    let claims = json! {{
      "exp": (Utc::now() + chrono::Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": "test_issuer".to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": "Bearer",
      "azp": "test_client_id",
      "session_state": Uuid::new_v4().to_string(),
      "scope": "email openid profile roles", // Sorted scope that matches auth_initiate_handler
      "sid": Uuid::new_v4().to_string(),
      "email_verified": true,
      "name": "Test User",
      "preferred_username": "testuser@email.com",
      "given_name": "Test",
      "family_name": "User",
      "email": "testuser@email.com"
    }};
    let (token, _) = build_token(claims).unwrap();
    let dbfile = temp_bodhi_home.path().join("test.db");
    let mut mock_auth_service = MockAuthService::default();
    let token_clone = token.clone();
    mock_auth_service
      .expect_exchange_auth_code()
      .times(1)
      .return_once(move |_, _, _, _, _| {
        Ok((
          AccessToken::new(token_clone.clone()),
          RefreshToken::new("test_refresh_token".to_string()),
        ))
      });

    let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
      (BODHI_PORT.to_string(), "3000".to_string()),
    ]));

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .setting_service(Arc::new(setting_service))
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .build()?;

    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .route("/auth/callback", post(auth_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.save_cookies();

    // Perform login request
    let login_resp = client.post("/auth/initiate").await;
    login_resp.assert_status(StatusCode::CREATED);
    let body: RedirectResponse = login_resp.json();
    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

    // Extract state and code_challenge from the login response
    let state = query_params.get("state").unwrap();

    // Perform callback request
    let resp = client
      .post("/auth/callback")
      .json(&json! {{
        "code": "test_code",
        "state": state,
      }})
      .await;
    resp.assert_status(StatusCode::OK);
    let callback_body: RedirectResponse = resp.json();
    assert_eq!(
      "http://frontend.localhost:3000/ui/chat",
      callback_body.location
    );
    let session_id = resp.cookie("bodhiapp_session_id");
    let access_token = session_service
      .get_session_value(session_id.value(), "access_token")
      .await
      .unwrap();
    let access_token = access_token.as_str().unwrap();
    assert_eq!(token, access_token);
    let refresh_token = session_service
      .get_session_value(session_id.value(), "refresh_token")
      .await
      .unwrap();
    let refresh_token = refresh_token.as_str().unwrap();
    assert_eq!("test_refresh_token", refresh_token);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_auth_callback_handler_state_not_in_session(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Ready);
    let secret_service = Arc::new(secret_service);
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .secret_service(secret_service)
      .build_session_service(temp_bodhi_home.path().join("test.db"))
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/callback", post(auth_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let resp = router
      .oneshot(Request::post("/auth/callback").json(json! {{
        "code": "test_code",
        "state": "test_state",
      }})?)
      .await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
    let json = resp.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "Login session not found. Are cookies enabled?",
          "code": "login_error-session_info_not_found",
          "type": "internal_server_error"
        }
      }},
      json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_auth_callback_handler_with_loopback_callback_url(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    // Create a token with the correct scope that matches what auth_initiate_handler uses
    let claims = json! {{
      "exp": (Utc::now() + chrono::Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": "test_issuer".to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": "Bearer",
      "azp": "test_client_id",
      "session_state": Uuid::new_v4().to_string(),
      "scope": "email openid profile roles", // Sorted scope that matches auth_initiate_handler
      "sid": Uuid::new_v4().to_string(),
      "email_verified": true,
      "name": "Test User",
      "preferred_username": "testuser@email.com",
      "given_name": "Test",
      "family_name": "User",
      "email": "testuser@email.com"
    }};
    let (token, _) = build_token(claims).unwrap();
    let dbfile = temp_bodhi_home.path().join("test.db");
    let mut mock_auth_service = MockAuthService::default();
    let token_clone = token.clone();
    mock_auth_service
      .expect_exchange_auth_code()
      .times(1)
      .return_once(move |_, _, _, _, _| {
        Ok((
          AccessToken::new(token_clone.clone()),
          RefreshToken::new("test_refresh_token".to_string()),
        ))
      });

    // Configure with 0.0.0.0 (loopback)
    let setting_service = SettingServiceStub::default().append_settings(HashMap::from([
      (BODHI_SCHEME.to_string(), "http".to_string()),
      (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (BODHI_PORT.to_string(), "1135".to_string()),
    ]));

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .setting_service(Arc::new(setting_service))
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .build()?;

    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .route("/auth/callback", post(auth_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.save_cookies();

    // Perform login request with Host header
    let login_resp = client
      .post("/auth/initiate")
      .add_header("Host", "localhost:1135")
      .await;
    login_resp.assert_status(StatusCode::CREATED);
    let body: RedirectResponse = login_resp.json();
    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

    // Verify callback URL uses localhost from Host header
    let callback_url = query_params.get("redirect_uri").unwrap();
    assert_eq!("http://localhost:1135/ui/auth/callback", callback_url);

    // Extract state for the callback request
    let state = query_params.get("state").unwrap();

    // Perform callback request
    let resp = client
      .post("/auth/callback")
      .json(&json! {{
        "code": "test_code",
        "state": state,
      }})
      .await;
    resp.assert_status(StatusCode::OK);
    let callback_body: RedirectResponse = resp.json();

    // Final redirect should use localhost from the callback URL
    assert_eq!("http://localhost:1135/ui/chat", callback_body.location);

    // Verify session contains access token
    let session_id = resp.cookie("bodhiapp_session_id");
    let access_token = session_service
      .get_session_value(session_id.value(), "access_token")
      .await
      .unwrap();
    let access_token = access_token.as_str().unwrap();
    assert_eq!(token, access_token);

    Ok(())
  }

  #[rstest]
  #[case(
    "modified-",
    true,
    "state parameter in callback does not match with the one sent in login request"
  )]
  // #[case("", false, "missing code parameter")]
  #[tokio::test]
  async fn test_auth_callback_handler_missing_params(
    temp_bodhi_home: TempDir,

    #[case] state_prefix: String,
    #[case] code_present: bool,
    #[case] expected_error: &str,
  ) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .route("/auth/callback", post(auth_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.save_cookies();

    let login_resp = client.post("/auth/initiate").await;
    login_resp.assert_status(StatusCode::CREATED);
    let body: RedirectResponse = login_resp.json();
    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let state = format!("{}{}", state_prefix, query_params.get("state").unwrap());
    let code = if code_present {
      Some("test_code".to_string())
    } else {
      None
    };

    let resp = client
      .post("/auth/callback")
      .json(&json! {{
        "code": code,
        "state": state,
      }})
      .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
    let error = resp.json::<Value>();
    let expected_message = format!("Invalid request: {}.", expected_error);
    assert_eq!(
      json! {{
        "error": {
          "message": expected_message,
          "code": "bad_request_error",
          "type": "invalid_request_error",
          "param": {
            "reason": expected_error
          }
        }
      }},
      error
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_callback_handler_auth_service_error(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let mut mock_auth_service = MockAuthService::new();
    mock_auth_service
      .expect_exchange_auth_code()
      .times(1)
      .return_once(|_, _, _, _, _| {
        Err(AuthServiceError::AuthServiceApiError(
          "network error".to_string(),
        ))
      });
    let app_service: AppServiceStub = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .with_sqlite_session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router = Router::new()
      .route("/auth/initiate", post(auth_initiate_handler))
      .route("/auth/callback", post(auth_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.save_cookies();

    // Simulate login to set up session
    let login_resp = client.post("/auth/initiate").await;
    login_resp.assert_status(StatusCode::CREATED);
    let body: RedirectResponse = login_resp.json();
    let url = Url::parse(&body.location)?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let state = query_params.get("state").unwrap().to_string();
    let code = "test_code".to_string();
    let resp = client
      .post("/auth/callback")
      .json(&json! {{
        "code": code,
        "state": state,
      }})
      .await;

    resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    let error = resp.json::<Value>();
    assert_eq!(
      json! {{
        "error": {
          "message": "Authentication service error: network error.",
          "code": "auth_service_error-auth_service_api_error",
          "type": "internal_server_error",
          "param": {
            "var_0": "network error"
          }
        }
      }},
      error
    );
    Ok(())
  }

  pub async fn create_test_session_handler(session: Session) -> impl IntoResponse {
    session.insert("test", "test").await.unwrap();
    (StatusCode::CREATED, Json(json!({})))
  }

  #[rstest]
  #[tokio::test]
  async fn test_logout_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service =
      Arc::new(SqliteSessionService::build_session_service(dbfile.clone()).await);
    let app_service: Arc<dyn AppService> = Arc::new(
      AppServiceStubBuilder::default()
        .with_sqlite_session_service(session_service.clone())
        .build()?,
    );

    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/app/logout", post(logout_handler))
      .route("/test/session/new", post(create_test_session_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let mut client = TestServer::new(router)?;
    client.save_cookies();

    let resp = client.post("/test/session/new").await;
    resp.assert_status(StatusCode::CREATED);
    let cookie = resp.cookie("bodhiapp_session_id");
    let session_id = cookie.value_trimmed();

    let record = session_service.get_session_record(session_id).await;
    assert!(record.is_some());

    let resp = client.post("/app/logout").await;
    resp.assert_status(StatusCode::OK);
    let body: RedirectResponse = resp.json();
    assert_eq!("http://localhost:1135/ui/login", body.location);
    let record = session_service.get_session_record(session_id).await;
    assert!(record.is_none());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_callback_handler_resource_admin(
    temp_bodhi_home: TempDir,
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let mut server = Server::new_async().await;
    let auth_server_url = server.url();
    let id = Id::default();
    let state = generate_random_string(32); // Use simple random state like the actual implementation
    let app_service =
      setup_app_service_resource_admin(&temp_bodhi_home, &id, &auth_server_url, &state).await?;
    setup_auth_server_mocks_resource_admin(&mut server, &token).await;
    let result = execute_auth_callback(&id, app_service.clone(), &state).await?;
    assert_login_callback_result_resource_admin(result, app_service).await?;
    Ok(())
  }

  async fn setup_app_service_resource_admin(
    temp_bodhi_home: &TempDir,
    id: &Id,
    auth_server_url: &str,
    state: &str,
  ) -> anyhow::Result<Arc<AppServiceStub>> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = SqliteSessionService::build_session_service(dbfile).await;
    let mut record = Record {
      id: *id,
      data: maplit::hashmap! {
        "oauth_state".to_string() => Value::String(state.to_string()),
        "pkce_verifier".to_string() => Value::String("test_pkce_verifier".to_string()),
        "callback_url".to_string() => Value::String(format!("http://frontend.localhost:3000/ui/auth/callback")),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;
    let session_service = Arc::new(session_service);
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::ResourceAdmin);
    let secret_service = Arc::new(secret_service);
    let auth_service = Arc::new(test_auth_service(auth_server_url));
    let setting_service = Arc::new(
      SettingServiceStub::default().append_settings(HashMap::from([
        (BODHI_SCHEME.to_string(), "http".to_string()),
        (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
        (BODHI_PORT.to_string(), "3000".to_string()),
        (BODHI_AUTH_URL.to_string(), auth_server_url.to_string()),
      ])),
    );
    let app_service = AppServiceStubBuilder::default()
      .secret_service(secret_service)
      .auth_service(auth_service)
      .setting_service(setting_service)
      .with_sqlite_session_service(session_service)
      .build()?;
    Ok(Arc::new(app_service))
  }

  async fn setup_auth_server_mocks_resource_admin(server: &mut Server, token: &str) {
    // Mock token endpoint for code exchange
    let code_verifier = PkceCodeVerifier::new("test_pkce_verifier".to_string());
    let code_secret = code_verifier.secret();
    server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
        Matcher::UrlEncoded("code".into(), "test_code".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
        Matcher::UrlEncoded(
          "redirect_uri".into(),
          "http://frontend.localhost:3000/ui/auth/callback".into(),
        ),
        Matcher::UrlEncoded("code_verifier".into(), code_secret.into()),
      ]))
      .with_status(200)
      .with_body(
        json!({
          "access_token": token,
          "refresh_token": "initial_refresh_token",
          "token_type": "Bearer",
          "expires_in": 300,
        })
        .to_string(),
      )
      .create_async()
      .await;

    // Mock token endpoint for client credentials
    server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_body(
        json!({
          "access_token": "client_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
        })
        .to_string(),
      )
      .create_async()
      .await;

    // Mock make-resource-admin endpoint
    server
      .mock(
        "POST",
        "/realms/test-realm/bodhi/resources/make-resource-admin",
      )
      .match_header("Authorization", "Bearer client_access_token")
      .match_body(Matcher::Regex(r#"\{"user_id":"[^"]+"\}"#.to_string()))
      .with_status(200)
      .with_body("{}")
      .create_async()
      .await;

    // Mock token refresh endpoint
    server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
        Matcher::UrlEncoded("refresh_token".into(), "initial_refresh_token".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_body(
        json!({
          "access_token": "new_access_token",
          "refresh_token": "new_refresh_token",
          "token_type": "Bearer",
          "expires_in": 300,
        })
        .to_string(),
      )
      .create_async()
      .await;
  }

  async fn execute_auth_callback(
    id: &Id,
    app_service: Arc<AppServiceStub>,
    request_state: &str,
  ) -> Result<Response, anyhow::Error> {
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));
    let router: Router = Router::new()
      .route("/auth/callback", post(auth_callback_handler))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let request = Request::post("/auth/callback")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .json(json! {{
        "code": "test_code",
        "state": request_state,
      }})
      .unwrap();
    let response = router.oneshot(request).await?;
    assert_eq!(StatusCode::OK, response.status());
    Ok(response)
  }

  async fn assert_login_callback_result_resource_admin(
    response: Response,
    app_service: Arc<AppServiceStub>,
  ) -> anyhow::Result<()> {
    let body_bytes = to_bytes(response.into_body(), usize::MAX).await?;
    let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
    assert_eq!(
      "http://frontend.localhost:3000/ui/setup/download-models",
      body.location
    );
    let secret_service = app_service.secret_service();
    let updated_status = secret_service.app_status().unwrap();
    assert_eq!(AppStatus::Ready, updated_status);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_request_access_handler_success(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let app_client_id = "test_app_client_id";
    let expected_scope = "scope_resource_test-resource-server";

    // Mock auth server
    let mut server = Server::new_async().await;
    let url = server.url();

    // Mock token endpoint for client credentials
    let token_mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_body(
        json!({
            "access_token": "test_access_token",
            "token_type": "Bearer",
            "expires_in": 300,
        })
        .to_string(),
      )
      .create();

    // Mock request-access endpoint
    let access_mock = server
      .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
      .match_header("Authorization", "Bearer test_access_token")
      .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
      .with_status(200)
      .with_body(
        json!({
          "scope": expected_scope,
          "toolsets": [{
            "id": "builtin-exa-web-search",
            "scope": "scope_toolset-builtin-exa-web-search",
            "scope_id": "test-scope-uuid-123",
            "added_to_resource_client": true
          }],
          "app_client_config_version": "v1.0.0"
        })
        .to_string(),
      )
      .create();

    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let auth_service = Arc::new(test_auth_service(&url));
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(auth_service)
      .with_db_service()
      .await
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/request-access", post(request_access_handler))
      .with_state(state);

    let resp = router
      .oneshot(Request::post("/auth/request-access").json(json! {{
        "app_client_id": app_client_id
      }})?)
      .await?;

    assert_eq!(StatusCode::OK, resp.status());
    let body: AppAccessResponse = resp.json().await?;
    assert_eq!(expected_scope, body.scope);
    assert_eq!(Some("v1.0.0".to_string()), body.app_client_config_version);

    token_mock.assert();
    access_mock.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_request_access_handler_none_version_cache_miss(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let app_client_id = "test_app_client_id";
    let expected_scope = "scope_resource_test-resource-server";

    // Mock auth server
    let mut server = Server::new_async().await;
    let url = server.url();

    // Mock token endpoint
    let token_mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_body(
        json!({
            "access_token": "test_access_token",
            "token_type": "Bearer",
            "expires_in": 300,
        })
        .to_string(),
      )
      .create();

    // Mock request-access endpoint
    let access_mock = server
      .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
      .match_header("Authorization", "Bearer test_access_token")
      .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
      .with_status(200)
      .with_body(
        json!({
          "scope": expected_scope,
          "toolsets": [{
            "id": "builtin-exa-web-search",
            "scope": "scope_toolset-builtin-exa-web-search",
            "scope_id": "test-scope-uuid-123",
            "added_to_resource_client": true
          }],
          "app_client_config_version": "v2.0.0"
        })
        .to_string(),
      )
      .create();

    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let auth_service = Arc::new(test_auth_service(&url));
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(auth_service)
      .with_db_service()
      .await
      .build_session_service(dbfile.clone())
      .await
      .build()?;

    // Pre-populate cache with None version
    let db_service = app_service.db_service();
    let now = db_service.now().timestamp();
    db_service
      .upsert_app_client_toolset_config(&AppClientToolsetConfigRow {
        id: 0,
        app_client_id: app_client_id.to_string(),
        config_version: None,
        toolsets_json: "[]".to_string(),
        resource_scope: "old_scope".to_string(),
        created_at: now,
        updated_at: now,
      })
      .await?;

    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/request-access", post(request_access_handler))
      .with_state(state);

    // Client sends version but DB has None - should be cache miss
    let resp = router
      .oneshot(Request::post("/auth/request-access").json(json! {{
        "app_client_id": app_client_id,
        "version": "v1.0.0"
      }})?)
      .await?;

    assert_eq!(StatusCode::OK, resp.status());
    let body: AppAccessResponse = resp.json().await?;
    assert_eq!(expected_scope, body.scope);
    assert_eq!(Some("v2.0.0".to_string()), body.app_client_config_version);

    token_mock.assert();
    access_mock.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_request_access_handler_auth_returns_none_version(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let app_client_id = "test_app_client_id";
    let expected_scope = "scope_resource_test-resource-server";

    // Mock auth server
    let mut server = Server::new_async().await;
    let url = server.url();

    // Mock token endpoint
    let token_mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_body(
        json!({
            "access_token": "test_access_token",
            "token_type": "Bearer",
            "expires_in": 300,
        })
        .to_string(),
      )
      .create();

    // Mock request-access endpoint without version field
    let access_mock = server
      .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
      .match_header("Authorization", "Bearer test_access_token")
      .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
      .with_status(200)
      .with_body(
        json!({
          "scope": expected_scope,
          "toolsets": [{
            "id": "builtin-exa-web-search",
            "scope": "scope_toolset-builtin-exa-web-search",
            "scope_id": "test-scope-uuid-123",
            "added_to_resource_client": true
          }]
        })
        .to_string(),
      )
      .create();

    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let auth_service = Arc::new(test_auth_service(&url));
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(auth_service)
      .with_db_service()
      .await
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/request-access", post(request_access_handler))
      .with_state(state);

    let resp = router
      .oneshot(Request::post("/auth/request-access").json(json! {{
        "app_client_id": app_client_id
      }})?)
      .await?;

    assert_eq!(StatusCode::OK, resp.status());
    let body: AppAccessResponse = resp.json().await?;
    assert_eq!(expected_scope, body.scope);
    assert_eq!(None, body.app_client_config_version);

    token_mock.assert();
    access_mock.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_request_access_handler_no_client_credentials(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new().with_app_status(&AppStatus::Ready); // No app_reg_info set
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_db_service()
      .await
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/request-access", post(request_access_handler))
      .with_state(state);

    let resp = router
      .oneshot(Request::post("/auth/request-access").json(json! {{
        "app_client_id": "test_app_client_id"
      }})?)
      .await?;

    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
    let error = resp.json::<Value>().await?;
    let expected_message = "Application is not registered. Please register the application first.";
    assert_eq!(
      json! {{
        "error": {
          "message": expected_message,
          "code": "login_error-app_reg_info_not_found",
          "type": "invalid_app_state"
        }
      }},
      error
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_request_access_handler_auth_service_error(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let app_client_id = "invalid_app_client_id";

    // Mock auth server with error response
    let mut server = Server::new_async().await;
    let url = server.url();

    // Mock token endpoint for client credentials
    let token_mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .with_status(200)
      .with_body(
        json!({
            "access_token": "test_access_token",
            "token_type": "Bearer",
            "expires_in": 300,
        })
        .to_string(),
      )
      .create();

    // Mock request-access endpoint with error
    let access_mock = server
      .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
      .with_status(400)
      .with_body(json!({"error": "app_client_not_found"}).to_string())
      .create();

    let dbfile = temp_bodhi_home.path().join("test.db");
    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
      })
      .with_app_status(&AppStatus::Ready);
    let auth_service = Arc::new(test_auth_service(&url));
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(auth_service)
      .with_db_service()
      .await
      .build_session_service(dbfile)
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service.clone(),
    ));

    let router = Router::new()
      .route("/auth/request-access", post(request_access_handler))
      .with_state(state);

    let resp = router
      .oneshot(Request::post("/auth/request-access").json(json! {{
        "app_client_id": app_client_id
      }})?)
      .await?;

    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
    let error = resp.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "Authentication service error: app_client_not_found.",
          "code": "auth_service_error-auth_service_api_error",
          "type": "internal_server_error",
          "param": {
            "var_0": "app_client_not_found"
          }
        }
      }},
      error
    );

    token_mock.assert();
    access_mock.assert();
    Ok(())
  }
}
