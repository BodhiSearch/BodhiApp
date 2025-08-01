use crate::{app_status_or_default, DefaultTokenService};
use axum::{
  extract::{Request, State},
  http::HeaderMap,
  middleware::Next,
  response::Response,
};

use objs::{
  ApiError, AppError, AppRegInfoMissingError, ErrorType, RoleError, TokenScopeError, UserScopeError,
};
use server_core::RouterState;
use services::{AppStatus, AuthServiceError, SecretServiceError, TokenError};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::instrument;

pub const SESSION_KEY_ACCESS_TOKEN: &str = "access_token";
pub const SESSION_KEY_REFRESH_TOKEN: &str = "refresh_token";

pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";
pub const KEY_RESOURCE_ROLE: &str = "X-Resource-Access";
pub const KEY_RESOURCE_SCOPE: &str = "X-Resource-Scope";

const SEC_FETCH_SITE_HEADER: &str = "sec-fetch-site";

/// Returns true if the request originates from the same site ("same-origin").
fn is_same_origin(headers: &HeaderMap) -> bool {
  matches!(
    headers
      .get(SEC_FETCH_SITE_HEADER)
      .and_then(|v| v.to_str().ok()),
    Some("same-origin")
  )
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

#[instrument(skip_all, level = "debug")]
pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  req.headers_mut().remove(KEY_RESOURCE_TOKEN);
  req.headers_mut().remove(KEY_RESOURCE_ROLE);
  req.headers_mut().remove(KEY_RESOURCE_SCOPE);

  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
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
    req
      .headers_mut()
      .insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());

    req.headers_mut().insert(
      KEY_RESOURCE_SCOPE,
      resource_scope.to_string().parse().unwrap(),
    );
    Ok(next.run(req).await)
  } else if is_same_origin(&_headers) {
    // Check for token in session
    if let Some(access_token) = session
      .get::<String>(SESSION_KEY_ACCESS_TOKEN)
      .await
      .map_err(AuthError::from)?
    {
      let (access_token, role) = token_service
        .get_valid_session_token(session, access_token)
        .await?;
      req
        .headers_mut()
        .insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
      req
        .headers_mut()
        .insert(KEY_RESOURCE_ROLE, role.to_string().parse().unwrap());
      Ok(next.run(req).await)
    } else {
      Err(AuthError::InvalidAccess)?
    }
  } else {
    Err(AuthError::InvalidAccess)?
  }
}

#[instrument(skip_all, level = "debug")]
pub async fn inject_session_auth_info(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  req.headers_mut().remove(KEY_RESOURCE_TOKEN);
  req.headers_mut().remove(KEY_RESOURCE_ROLE);
  req.headers_mut().remove(KEY_RESOURCE_SCOPE);
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
  );

  // Check app status
  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Ok(next.run(req).await);
  }
  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    if let Ok(header) = header.to_str() {
      if let Ok((access_token, resource_scope)) = token_service.validate_bearer_token(header).await
      {
        req
          .headers_mut()
          .insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
        req.headers_mut().insert(
          KEY_RESOURCE_SCOPE,
          resource_scope.to_string().parse().unwrap(),
        );
      }
    }
  } else if is_same_origin(req.headers()) {
    // Check for token in session
    if let Ok(Some(access_token)) = session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await {
      if let Ok((validated_token, role)) = token_service
        .get_valid_session_token(session, access_token)
        .await
      {
        req
          .headers_mut()
          .insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());
        req
          .headers_mut()
          .insert(KEY_RESOURCE_ROLE, role.to_string().parse().unwrap());
      }
    }
  }
  // Continue with the request
  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use super::{KEY_RESOURCE_ROLE, KEY_RESOURCE_TOKEN};
  use crate::{auth_middleware, inject_session_auth_info, KEY_RESOURCE_SCOPE};
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
  };
  use mockall::predicate::eq;
  use objs::{
    test_utils::{setup_l10n, temp_bodhi_home},
    FluentLocalizationService, ReqwestError,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::{
    test_utils::{
      access_token_claims, build_token, expired_token, offline_access_token_claims,
      offline_token_claims, token, AppServiceStubBuilder, SecretServiceStub, TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, AuthServiceError, MockAuthService, SqliteSessionService, BODHI_HOST,
    BODHI_PORT, BODHI_SCHEME,
  };
  use std::sync::Arc;
  use tempfile::TempDir;
  use time::{Duration, OffsetDateTime};
  use tower::ServiceExt;
  use tower_sessions::{
    session::{Id, Record},
    SessionStore,
  };

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
      .get(KEY_RESOURCE_TOKEN)
      .map(|v| v.to_str().unwrap().to_string());
    let x_resource_role = headers
      .get(KEY_RESOURCE_ROLE)
      .map(|v| v.to_str().unwrap().to_string());
    let x_resource_scope = headers
      .get(KEY_RESOURCE_SCOPE)
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
          .route_layer(from_fn_with_state(state.clone(), inject_session_auth_info)),
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
      .header(KEY_RESOURCE_TOKEN, "user-sent-token")
      .header(KEY_RESOURCE_ROLE, "user-sent-role")
      .header(KEY_RESOURCE_SCOPE, "user-sent-scope")
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
  #[case::scope_token_user("offline_access scope_token_user", "scope_token_user")]
  #[case::scope_token_user("offline_access scope_token_power_user", "scope_token_power_user")]
  #[case::scope_token_user("offline_access scope_token_manager", "scope_token_manager")]
  #[case::scope_token_user("offline_access scope_token_admin", "scope_token_admin")]
  #[case::scope_token_user(
    "offline_access scope_token_user scope_token_manager",
    "scope_token_manager"
  )]
  #[case::scope_token_user(
    "offline_access scope_token_user scope_token_power_user",
    "scope_token_power_user"
  )]
  #[tokio::test]
  async fn test_auth_middleware_bearer_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] scope: &str,
    #[case] expected_header: &str,
  ) -> anyhow::Result<()> {
    let (bearer_token, _) = build_token(offline_token_claims())?;
    let mut access_token_claims = offline_access_token_claims();
    access_token_claims["scope"] = Value::String(scope.to_string());
    let (access_token, _) = build_token(access_token_claims)?;
    let access_token_cl = access_token.clone();
    let mut auth_service = MockAuthService::default();
    auth_service
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(bearer_token.clone()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((access_token_cl, Some("refresh_token".to_string()))));
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .auth_service(Arc::new(auth_service))
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let db = app_service.db_service.as_ref().unwrap();
    let _ = db
      .create_api_token_from("test-token-id", &bearer_token)
      .await?;
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", bearer_token))
      .json(json! {{}})?;
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let actual: TestResponse = response.json().await?;
    assert_eq!(
      TestResponse {
        path: "/with_auth".to_string(),
        x_resource_token: Some(access_token),
        x_resource_role: None,
        x_resource_scope: Some(expected_header.to_string()),
        authorization_header: Some(format!("Bearer {}", bearer_token)),
      },
      actual
    );
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_auth_middleware_gives_precedence_to_token_over_session(
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let (token, _) = token();
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
    let offline_token_claims = offline_token_claims();
    let (offline_token, _) = build_token(offline_token_claims)?;
    let (offline_access_token, _) = build_token(offline_access_token_claims())?;
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .session_service(session_service.clone())
      .with_db_service()
      .await
      .build()?;
    let api_token = app_service
      .db_service
      .as_ref()
      .unwrap()
      .create_api_token_from("test-token", &offline_token)
      .await?;
    app_service.cache_service.as_ref().unwrap().set(
      &format!("token:{}", api_token.token_id),
      &offline_access_token,
    );
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);
    let req = Request::get("/with_auth")
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .header("Sec-Fetch-Site", "same-origin")
      .header("Authorization", format!("Bearer {}", offline_token))
      .body(Body::empty())?;
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let actual: TestResponse = response.json().await?;
    assert_eq!(
      TestResponse {
        path: "/with_auth".to_string(),
        x_resource_token: Some(offline_access_token),
        x_resource_role: None,
        x_resource_scope: Some("scope_token_user".to_string()),
        authorization_header: Some(format!("Bearer {}", offline_token)),
      },
      actual
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_session_ignored_when_cross_site(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
    use axum::http::StatusCode;
    use pretty_assertions::assert_eq;

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
}
