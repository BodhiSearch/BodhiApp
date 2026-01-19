use crate::{app_status_or_default, DefaultTokenService};
use axum::{
  extract::{Request, State},
  http::{header::HOST, HeaderMap},
  middleware::Next,
  response::Response,
};

use objs::{
  ApiError, AppError, AppRegInfoMissingError, ErrorType, RoleError, TokenScopeError, UserScopeError,
};
use server_core::RouterState;
use services::{
  db::DbError, extract_claims, AppStatus, AuthServiceError, ScopeClaims, SecretServiceError,
  TokenError, UserIdClaims,
};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::debug;

pub const SESSION_KEY_ACCESS_TOKEN: &str = "access_token";
pub const SESSION_KEY_REFRESH_TOKEN: &str = "refresh_token";
pub const SESSION_KEY_USER_ID: &str = "user_id";

macro_rules! bodhi_header {
  ($name:literal) => {
    concat!("X-BodhiApp-", $name)
  };
}

pub const KEY_PREFIX_HEADER_BODHIAPP: &str = "X-BodhiApp-";
pub const KEY_HEADER_BODHIAPP_TOKEN: &str = bodhi_header!("Token");
pub const KEY_HEADER_BODHIAPP_USERNAME: &str = bodhi_header!("Username");
pub const KEY_HEADER_BODHIAPP_ROLE: &str = bodhi_header!("Role");
pub const KEY_HEADER_BODHIAPP_SCOPE: &str = bodhi_header!("Scope");
pub const KEY_HEADER_BODHIAPP_USER_ID: &str = bodhi_header!("User-Id");
// Phase 7.6: External app tool authorization headers
pub const KEY_HEADER_BODHIAPP_TOOL_SCOPES: &str = bodhi_header!("Tool-Scopes");
pub const KEY_HEADER_BODHIAPP_AZP: &str = bodhi_header!("Azp");

const SEC_FETCH_SITE_HEADER: &str = "sec-fetch-site";

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
  #[error("missing_roles")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingRoles,
  #[error("invalid_access")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidAccess,
  #[error("token_inactive")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenInactive,
  #[error("token_not_found")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenNotFound,
  #[error(transparent)]
  AuthService(#[from] AuthServiceError),
  #[error(transparent)]
  AppRegInfoMissing(#[from] AppRegInfoMissingError),
  #[error(transparent)]
  DbError(#[from] DbError),
  #[error("refresh_token_not_found")]
  #[error_meta(error_type = ErrorType::Authentication)]
  RefreshTokenNotFound,
  #[error(transparent)]
  SecretService(#[from] SecretServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "auth_error-tower_sessions", args_delegate = false)]
  TowerSession(#[from] tower_sessions::session::Error),
  #[error("signature_key")]
  #[error_meta(error_type = ErrorType::Authentication)]
  SignatureKey(String),
  #[error("invalid_token")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidToken(String),
  #[error("signature_mismatch")]
  #[error_meta(error_type = ErrorType::Authentication)]
  SignatureMismatch(String),
  #[error("app_status_invalid")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppStatusInvalid(AppStatus),
}

pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);

  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
  );

  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Err(AuthError::AppStatusInvalid(AppStatus::Setup).into());
  }

  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    // Check for bearer token in request header
    let header = header
      .to_str()
      .map_err(|err| AuthError::InvalidToken(err.to_string()))?;
    let (access_token, resource_scope) = token_service.validate_bearer_token(header).await?;
    tracing::debug!(resource_scope = %resource_scope, "auth_middleware: validated bearer token");
    req
      .headers_mut()
      .insert(KEY_HEADER_BODHIAPP_TOKEN, access_token.parse().unwrap());

    req.headers_mut().insert(
      KEY_HEADER_BODHIAPP_SCOPE,
      resource_scope.to_string().parse().unwrap(),
    );

    // Extract and set toolset scopes, user_id, and azp from the exchanged token
    // These are used by toolset_auth_middleware for external app toolset authorization
    if let Ok(scope_claims) = extract_claims::<ScopeClaims>(&access_token) {
      // Extract toolset scopes (space-separated, matches JWT scope format)
      let toolset_scopes: Vec<&str> = scope_claims
        .scope
        .split_whitespace()
        .filter(|s| s.starts_with("scope_toolset-"))
        .collect();
      if !toolset_scopes.is_empty() {
        req.headers_mut().insert(
          KEY_HEADER_BODHIAPP_TOOL_SCOPES,
          toolset_scopes.join(" ").parse().unwrap(),
        );
      }
      // Set user_id from sub claim (required for toolset execution)
      req.headers_mut().insert(
        KEY_HEADER_BODHIAPP_USER_ID,
        scope_claims.sub.parse().unwrap(),
      );
      // Set azp (authorized party / app-client ID)
      req
        .headers_mut()
        .insert(KEY_HEADER_BODHIAPP_AZP, scope_claims.azp.parse().unwrap());
    }

    Ok(next.run(req).await)
  } else if is_same_origin(&_headers) {
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
      req
        .headers_mut()
        .insert(KEY_HEADER_BODHIAPP_TOKEN, access_token.parse().unwrap());
      if let Some(role) = role {
        req
          .headers_mut()
          .insert(KEY_HEADER_BODHIAPP_ROLE, role.to_string().parse().unwrap());
      }
      // username only for session tokens for now, will implement for bearer once we implement access token story
      let claims = extract_claims::<UserIdClaims>(&access_token)?;
      let user_id = claims.sub.clone();
      req.headers_mut().insert(
        KEY_HEADER_BODHIAPP_USERNAME,
        claims.preferred_username.parse().unwrap(),
      );
      req
        .headers_mut()
        .insert(KEY_HEADER_BODHIAPP_USER_ID, user_id.parse().unwrap());
      debug!(
        "auth_middleware: session token validated successfully for user: {}",
        user_id
      );
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

pub async fn inject_optional_auth_info(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
    app_service.concurrency_service(),
  );

  // Check app status
  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Ok(next.run(req).await);
  }
  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    // Bearer token
    if let Ok(header) = header.to_str() {
      if let Ok((access_token, resource_scope)) = token_service.validate_bearer_token(header).await
      {
        req
          .headers_mut()
          .insert(KEY_HEADER_BODHIAPP_TOKEN, access_token.parse().unwrap());
        req.headers_mut().insert(
          KEY_HEADER_BODHIAPP_SCOPE,
          resource_scope.to_string().parse().unwrap(),
        );
        // Extract and set toolset scopes and azp from the exchanged token
        if let Ok(scope_claims) = extract_claims::<ScopeClaims>(&access_token) {
          let toolset_scopes: Vec<&str> = scope_claims
            .scope
            .split_whitespace()
            .filter(|s| s.starts_with("scope_toolset-"))
            .collect();
          if !toolset_scopes.is_empty() {
            req.headers_mut().insert(
              KEY_HEADER_BODHIAPP_TOOL_SCOPES,
              toolset_scopes.join(" ").parse().unwrap(),
            );
          }
          req
            .headers_mut()
            .insert(KEY_HEADER_BODHIAPP_AZP, scope_claims.azp.parse().unwrap());
        }
      }
    }
  } else if is_same_origin(req.headers()) {
    // session token
    if let Ok(Some(access_token)) = session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await {
      match token_service
        .get_valid_session_token(session.clone(), access_token.clone())
        .await
      {
        Ok((validated_token, role)) => {
          debug!("inject_session_auth_info: session token injected successfully");
          req
            .headers_mut()
            .insert(KEY_HEADER_BODHIAPP_TOKEN, validated_token.parse().unwrap());
          if let Some(role) = role {
            req
              .headers_mut()
              .insert(KEY_HEADER_BODHIAPP_ROLE, role.to_string().parse().unwrap());
          }
          let claims = extract_claims::<UserIdClaims>(&validated_token)?;
          req.headers_mut().insert(
            KEY_HEADER_BODHIAPP_USERNAME,
            claims.preferred_username.parse().unwrap(),
          );
          req
            .headers_mut()
            .insert(KEY_HEADER_BODHIAPP_USER_ID, claims.sub.parse().unwrap());
        }
        Err(AuthError::RefreshTokenNotFound) => {
          // Log this specific case - user needs to re-login
          // Clear the invalid session data to prevent repeated failed refresh attempts
          debug!("inject_session_auth_info: session has no refresh token - clearing session data");
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
        Err(err) => {
          // Log other errors and clear session on unrecoverable errors
          debug!(
            ?err,
            "inject_session_auth_info: token validation/refresh failed"
          );

          // Check if this is an auth error that indicates session should be cleared
          if matches!(
            err,
            AuthError::Token(_) | AuthError::AuthService(_) | AuthError::InvalidToken(_)
          ) {
            debug!("inject_session_auth_info: clearing invalid session due to auth error");
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
        }
      }
    } else {
      debug!("inject_session_auth_info: no access token in session");
    }
  } else {
    debug!("inject_session_auth_info: is_same_origin is false");
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
mod tests {
  use super::{
    KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
    KEY_HEADER_BODHIAPP_USERNAME, KEY_HEADER_BODHIAPP_USER_ID,
  };
  use crate::{auth_middleware, inject_optional_auth_info};
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
  };
  use base64::{engine::general_purpose, Engine};
  use mockall::predicate::eq;
  use objs::{
    test_utils::{setup_l10n, temp_bodhi_home},
    FluentLocalizationService, ReqwestError,
  };
  use pretty_assertions::assert_eq;
  use rand::RngCore;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::{
    db::{ApiToken, TokenStatus},
    test_utils::{
      access_token_claims, build_token, expired_token, AppServiceStubBuilder, SecretServiceStub,
      TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, AppService, AuthServiceError, MockAuthService, SqliteSessionService,
    BODHI_HOST, BODHI_PORT, BODHI_SCHEME,
  };
  use sha2::{Digest, Sha256};
  use std::sync::Arc;
  use tempfile::TempDir;
  use time::{Duration, OffsetDateTime};
  use tower::ServiceExt;
  use tower_sessions::{
    session::{Id, Record},
    SessionStore,
  };
  use uuid::Uuid;

  #[derive(Debug, Serialize, Deserialize, PartialEq)]
  struct TestResponse {
    x_resource_token: Option<String>,
    x_resource_role: Option<String>,
    x_resource_scope: Option<String>,
    authorization_header: Option<String>,
    path: String,
  }

  async fn test_handler_teapot(headers: HeaderMap, path: &str) -> Response {
    let authorization_header = headers
      .get("Authorization")
      .map(|v| v.to_str().unwrap().to_string());
    let x_resource_token = headers
      .get(KEY_HEADER_BODHIAPP_TOKEN)
      .map(|v| v.to_str().unwrap().to_string());
    let x_resource_role = headers
      .get(KEY_HEADER_BODHIAPP_ROLE)
      .map(|v| v.to_str().unwrap().to_string());
    let x_resource_scope = headers
      .get(KEY_HEADER_BODHIAPP_SCOPE)
      .map(|v| v.to_str().unwrap().to_string());
    (
      StatusCode::IM_A_TEAPOT,
      Json(TestResponse {
        x_resource_token,
        x_resource_role,
        x_resource_scope,
        authorization_header,
        path: path.to_string(),
      }),
    )
      .into_response()
  }

  fn router_with_auth() -> Router<Arc<dyn RouterState>> {
    Router::new().route(
      "/with_auth",
      get(|headers: HeaderMap| async move { test_handler_teapot(headers, "/with_auth").await }),
    )
  }

  fn router_with_optional_auth() -> Router<Arc<dyn RouterState>> {
    Router::new().route(
      "/with_optional_auth",
      get(|headers: HeaderMap| async move { test_handler_teapot(headers, "/with_optional_auth").await }),
    )
  }

  fn test_router(state: Arc<dyn RouterState>) -> Router {
    Router::new()
      .merge(router_with_auth().route_layer(from_fn_with_state(state.clone(), auth_middleware)))
      .merge(
        router_with_optional_auth()
          .route_layer(from_fn_with_state(state.clone(), inject_optional_auth_info)),
      )
      .layer(state.app_service().session_service().session_layer())
      .with_state(state)
  }

  async fn assert_optional_auth_passthrough(router: &Router) -> anyhow::Result<()> {
    assert_optional_auth(router, None, None).await?;
    Ok(())
  }

  async fn assert_optional_auth(
    router: &Router,
    authorization_header: Option<String>,
    x_resource_token: Option<String>,
  ) -> anyhow::Result<()> {
    let response = router
      .clone()
      .oneshot(Request::get("/with_optional_auth").body(Body::empty())?)
      .await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let response_json = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        x_resource_token,
        x_resource_role: None,
        x_resource_scope: None,
        authorization_header,
        path: "/with_optional_auth".to_string(),
      },
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case::authz_setup(SecretServiceStub::new().with_app_status_setup())]
  #[case::authz_missing(SecretServiceStub::new())]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_auth_middleware_returns_app_status_invalid_for_app_status_setup_or_missing(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] secret_service: SecretServiceStub,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .with_db_service()
      .await
      .with_settings(maplit::hashmap! {
        BODHI_SCHEME => "https",
        BODHI_HOST => "bodhi.app",
        BODHI_PORT => "443",
      })
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));
    let req = Request::get("/with_auth").body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let body: Value = response.json().await?;
    assert_eq!(
      json! {{
        "error": {
          "code": "auth_error-app_status_invalid",
          "type": "invalid_app_state",
          "message": "app status is invalid for this operation: \u{2068}setup\u{2069}"
        }
      }},
      body
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[case::with_auth_role_admin("/with_auth", &["resource_admin"], "resource_admin")]
  #[case::with_auth_role_user("/with_auth", &["resource_user"], "resource_user")]
  #[case::with_auth_role_manager("/with_auth", &["resource_manager"], "resource_manager")]
  #[case::with_auth_role_power_user("/with_auth", &["resource_power_user"], "resource_power_user")]
  #[case::with_auth_role_user_admin("/with_auth", &["resource_user", "resource_admin"], "resource_admin")]
  #[case::with_auth_role_user_manager("/with_auth", &["resource_user", "resource_manager"], "resource_manager")]
  #[case::with_optional_auth_role_admin("/with_optional_auth", &["resource_admin"], "resource_admin")]
  #[case::with_optional_auth_role_user("/with_optional_auth", &["resource_user"], "resource_user")]
  #[case::with_optional_auth_role_manager("/with_optional_auth", &["resource_manager"], "resource_manager")]
  #[case::with_optional_auth_role_power_user("/with_optional_auth", &["resource_power_user"], "resource_power_user")]
  #[case::with_optional_auth_role_user_admin("/with_optional_auth", &["resource_user", "resource_admin"], "resource_admin")]
  #[case::with_optional_auth_role_user_manager("/with_optional_auth", &["resource_user", "resource_manager"], "resource_manager")]
  #[tokio::test]
  async fn test_auth_middleware_with_valid_session_token(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    temp_bodhi_home: TempDir,
    #[case] path: &str,
    #[case] roles: &[&str],
    #[case] expected_role: &str,
  ) -> anyhow::Result<()> {
    let mut claims = access_token_claims();
    claims["resource_access"][TEST_CLIENT_ID]["roles"] =
      Value::Array(roles.iter().map(|r| Value::String(r.to_string())).collect());
    let (token, _) = build_token(claims)?;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let id = Id::default();
    let mut record = Record {
      id,
      data: maplit::hashmap! {
        "access_token".to_string() => Value::String(token.clone()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));

    let req = Request::get(path)
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "same-origin")
      .body(Body::empty())?;
    let router = test_router(state);

    let response = router.oneshot(req).await?;
    // assert_eq!("", response.text().await?);
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let body = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        path: path.to_string(),
        authorization_header: None,
        x_resource_token: Some(token),
        x_resource_role: Some(expected_role.to_string()),
        x_resource_scope: None,
      },
      body
    );
    Ok(())
  }

  #[rstest]
  #[case::with_auth_role_user("/with_auth", &["resource_user"], "resource_user")]
  #[case::with_auth_role_manager("/with_auth", &["resource_manager"], "resource_manager")]
  #[case::with_auth_role_power_user("/with_auth", &["resource_power_user"], "resource_power_user")]
  #[case::with_auth_role_admin("/with_auth", &["resource_admin"], "resource_admin")]
  #[case::with_auth_role_user_admin("/with_auth", &["resource_user", "resource_admin"], "resource_admin")]
  #[case::with_auth_role_user_manager("/with_auth", &["resource_user", "resource_manager"], "resource_manager")]
  #[case::with_optional_auth_role_user("/with_optional_auth", &["resource_user"], "resource_user")]
  #[case::with_optional_auth_role_manager("/with_optional_auth", &["resource_manager"], "resource_manager")]
  #[case::with_optional_auth_role_power_user("/with_optional_auth", &["resource_power_user"], "resource_power_user")]
  #[case::with_optional_auth_role_admin("/with_optional_auth", &["resource_admin"], "resource_admin")]
  #[case::with_optional_auth_role_user_admin("/with_optional_auth", &["resource_user", "resource_admin"], "resource_admin")]
  #[case::with_optional_auth_role_user_manager("/with_optional_auth", &["resource_user", "resource_manager"], "resource_manager")]
  #[tokio::test]
  async fn test_auth_middleware_with_expired_session_token(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    expired_token: (String, String),
    temp_bodhi_home: TempDir,
    #[case] path: &str,
    #[case] roles: &[&str],
    #[case] expected_role: &str,
  ) -> anyhow::Result<()> {
    let (expired_token, _) = expired_token;
    let mut access_token_claims = access_token_claims();
    access_token_claims["resource_access"][TEST_CLIENT_ID]["roles"] =
      Value::Array(roles.iter().map(|r| Value::String(r.to_string())).collect());
    let (exchanged_token, _) = build_token(access_token_claims)?;
    let exchanged_token_cl = exchanged_token.clone();
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let id = Id::default();
    let mut record = Record {
      id,
      data: maplit::hashmap! {
          "access_token".to_string() => Value::String(expired_token.clone()),
          "refresh_token".to_string() => Value::String("valid_refresh_token".to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;
    // Create mock auth service
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq("valid_refresh_token".to_string()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((exchanged_token_cl, Some("new_refresh_token".to_string()))));

    // Setup app service with mocks
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .auth_service(Arc::new(mock_auth_service))
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));

    let req = Request::get(path)
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "same-origin")
      .body(Body::empty())?;
    let router = test_router(state);

    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let body = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        path: path.to_string(),
        authorization_header: None,
        x_resource_token: Some(exchanged_token.clone()),
        x_resource_role: Some(expected_role.to_string()),
        x_resource_scope: None,
      },
      body
    );

    // Verify that the session was updated with the new tokens
    let updated_record = session_service.session_store.load(&id).await?.unwrap();
    assert_eq!(
      exchanged_token,
      updated_record
        .data
        .get("access_token")
        .unwrap()
        .as_str()
        .unwrap()
    );
    assert_eq!(
      "new_refresh_token",
      updated_record
        .data
        .get("refresh_token")
        .unwrap()
        .as_str()
        .unwrap()
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_token_refresh_persists_to_session(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    expired_token: (String, String),
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let (expired_token, _) = expired_token;
    let mut access_token_claims = access_token_claims();
    access_token_claims["resource_access"][TEST_CLIENT_ID]["roles"] =
      Value::Array(vec![Value::String("resource_admin".to_string())]);
    let (new_access_token, _) = build_token(access_token_claims)?;
    let new_access_token_cl = new_access_token.clone();

    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let id = Id::default();
    let mut record = Record {
      id,
      data: maplit::hashmap! {
          "access_token".to_string() => Value::String(expired_token.clone()),
          "refresh_token".to_string() => Value::String("valid_refresh_token".to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;

    // Create mock auth service that succeeds refresh
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq("valid_refresh_token".to_string()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((new_access_token_cl, Some("new_refresh_token".to_string()))));

    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .auth_service(Arc::new(mock_auth_service))
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));

    // First request should trigger refresh
    let req1 = Request::get("/with_auth")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "same-origin")
      .body(Body::empty())?;
    let router = test_router(state.clone());

    let response1 = router.clone().oneshot(req1).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response1.status());

    // Verify session was updated with new tokens
    let updated_record = session_service.session_store.load(&id).await?.unwrap();
    assert_eq!(
      new_access_token,
      updated_record
        .data
        .get("access_token")
        .unwrap()
        .as_str()
        .unwrap()
    );
    assert_eq!(
      "new_refresh_token",
      updated_record
        .data
        .get("refresh_token")
        .unwrap()
        .as_str()
        .unwrap()
    );

    // Second request should use the refreshed token without another refresh
    let req2 = Request::get("/with_auth")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "same-origin")
      .body(Body::empty())?;

    let response2 = router.oneshot(req2).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response2.status());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_with_expired_session_token_and_failed_refresh(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    expired_token: (String, String),
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let (expired_token, _) = expired_token;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);

    // Create mock auth service that fails to refresh the token
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_refresh_token()
      .with(
        eq("test_client_id"),
        eq("test_client_secret"),
        eq("refresh_token"),
      )
      .times(1)
      .return_once(|_, _, _| {
        Err(AuthServiceError::Reqwest(ReqwestError::new(
          "Failed to refresh token".to_string(),
        )))
      });

    // Setup app service with mocks
    let secret_service = SecretServiceStub::default().with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .client_id("test_client_id".to_string())
        .client_secret("test_client_secret".to_string())
        .build()?,
    );

    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(Arc::new(mock_auth_service))
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));

    let id = Id::default();
    let mut record = Record {
      id,
      data: maplit::hashmap! {
          "access_token".to_string() => Value::String(expired_token.clone()),
          "refresh_token".to_string() => Value::String("refresh_token".to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;

    let req = Request::get("/with_auth")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "same-origin")
      .body(Body::empty())?;
    let router = test_router(state);

    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let actual: Value = response.json().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "error connecting to internal service: \u{2068}Failed to refresh token\u{2069}",
          "type": "internal_server_error",
          "code": "reqwest_error"
        }
      }},
      actual
    );
    // Verify that the session was not updated
    let updated_record = session_service.session_store.load(&id).await?.unwrap();
    assert_eq!(
      expired_token,
      updated_record
        .data
        .get("access_token")
        .unwrap()
        .as_str()
        .unwrap()
    );
    assert_eq!(
      "refresh_token",
      updated_record
        .data
        .get("refresh_token")
        .unwrap()
        .as_str()
        .unwrap()
    );

    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_returns_invalid_access_when_no_token_in_session(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));

    let req = Request::get("/with_auth").body(Body::empty())?;
    let router = test_router(state);
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let body: Value = response.json().await?;
    assert_eq!(
      json! {{
        "error": {
          "code": "auth_error-invalid_access",
          "type": "authentication_error",
          "message": "access denied"
        }
      }},
      body
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_removes_internal_token_headers(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);
    let req = Request::get("/with_optional_auth")
      .header(KEY_HEADER_BODHIAPP_TOKEN, "user-sent-token")
      .header(KEY_HEADER_BODHIAPP_ROLE, "user-sent-role")
      .header(KEY_HEADER_BODHIAPP_SCOPE, "user-sent-scope")
      .header(KEY_HEADER_BODHIAPP_USERNAME, "user-sent-username")
      .header(KEY_HEADER_BODHIAPP_USER_ID, "user-sent-userid")
      .json(json! {{}})?;
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let actual: TestResponse = response.json().await?;
    assert_eq!(
      TestResponse {
        path: "/with_optional_auth".to_string(),
        x_resource_token: None,
        authorization_header: None,
        x_resource_role: None,
        x_resource_scope: None,
      },
      actual
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_session_ignored_when_cross_site(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    // create dummy session record
    let id = tower_sessions::session::Id::default();
    let mut record = tower_sessions::session::Record {
      id,
      data: maplit::hashmap! {
        "access_token".to_string() => serde_json::Value::String("dummy".to_string()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;

    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);
    // cross-site header
    let req = Request::get("/with_auth")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "cross-site")
      .body(Body::empty())?;
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[case::user("scope_token_user")]
  #[case::power_user("scope_token_power_user")]
  #[case::manager("scope_token_manager")]
  #[case::admin("scope_token_admin")]
  #[tokio::test]
  async fn test_auth_middleware_bodhiapp_token_scope_variations(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] scope_str: &str,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let db_service = app_service.db_service.as_ref().unwrap();

    // Generate test token
    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let token_str = format!("bodhiapp_{}", random_string);
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash the token
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database with specified scope
    let now = app_service.time_service().utc_now();
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user-id".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: scope_str.to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    db_service.create_api_token(&mut api_token).await?;

    // Create test router
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);

    // Make request with bearer token
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token_str))
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;

    // Assert response is successful
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());

    // Assert headers are set correctly
    let actual: TestResponse = response.json().await?;
    assert_eq!(
      TestResponse {
        path: "/with_auth".to_string(),
        x_resource_token: Some(token_str.clone()),
        x_resource_role: None,
        x_resource_scope: Some(scope_str.to_string()),
        authorization_header: Some(format!("Bearer {}", token_str)),
      },
      actual
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_bodhiapp_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let db_service = app_service.db_service.as_ref().unwrap();

    // Generate test token
    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let token_str = format!("bodhiapp_{}", random_string);
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash the token
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database
    let now = app_service.time_service().utc_now();
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user-id".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    db_service.create_api_token(&mut api_token).await?;

    // Create test router
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);

    // Make request with bearer token
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token_str))
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;

    // Assert response is successful
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());

    // Assert headers are set correctly
    let actual: TestResponse = response.json().await?;
    assert_eq!(
      TestResponse {
        path: "/with_auth".to_string(),
        x_resource_token: Some(token_str),
        x_resource_role: None,
        x_resource_scope: Some("scope_token_user".to_string()),
        authorization_header: Some(format!("Bearer bodhiapp_{}", random_string)),
      },
      actual
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_bodhiapp_token_inactive(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let db_service = app_service.db_service.as_ref().unwrap();

    // Generate test token
    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let token_str = format!("bodhiapp_{}", random_string);
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash the token
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database with Inactive status
    let now = app_service.time_service().utc_now();
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user-id".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Inactive,
      created_at: now,
      updated_at: now,
    };
    db_service.create_api_token(&mut api_token).await?;

    // Create test router
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);

    // Make request with bearer token
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token_str))
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;

    // Assert request returns 401 Unauthorized
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let body: Value = response.json().await?;
    assert_eq!(
      json! {{
        "error": {
          "code": "auth_error-token_inactive",
          "type": "authentication_error",
          "message": "API token is inactive"
        }
      }},
      body
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_bodhiapp_token_invalid_hash(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let db_service = app_service.db_service.as_ref().unwrap();

    // Generate test token
    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let token_str = format!("bodhiapp_{}", random_string);
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash a DIFFERENT token for storage
    let different_token = format!(
      "bodhiapp_{}",
      general_purpose::URL_SAFE_NO_PAD.encode([1u8; 32])
    );
    let mut hasher = Sha256::new();
    hasher.update(different_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database with different hash
    let now = app_service.time_service().utc_now();
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user-id".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    db_service.create_api_token(&mut api_token).await?;

    // Create test router
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);

    // Make request with bearer token (different from stored hash)
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token_str))
      .body(Body::empty())?;

    let response = router.oneshot(req).await?;

    // Assert request returns 401 Unauthorized
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let body: Value = response.json().await?;
    assert_eq!(
      json! {{
        "error": {
          "code": "token_error-invalid_token",
          "type": "authentication_error",
          "message": "token is invalid: \u{2068}Invalid token\u{2069}"
        }
      }},
      body
    );
    Ok(())
  }
}
