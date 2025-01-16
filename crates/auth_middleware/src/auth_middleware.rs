use crate::{app_status_or_default, DefaultTokenService};
use axum::{
  extract::{Request, State},
  http::HeaderMap,
  middleware::Next,
  response::{IntoResponse, Redirect, Response},
};
use objs::{ApiError, AppError, ErrorType};
use server_core::RouterState;
use services::{
  AppStatus, AuthServiceError, SecretService, SecretServiceError, SecretServiceExt, TokenError,
};
use std::sync::Arc;
use tower_sessions::Session;

pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";
pub const KEY_USER_ROLES: &str = "X-User-Roles";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthError {
  #[error(transparent)]
  Token(#[from] TokenError),
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
  #[error("app_reg_info_missing")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AppRegInfoMissing,
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
}

pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  req.headers_mut().remove(KEY_RESOURCE_TOKEN);
  req.headers_mut().remove(KEY_USER_ROLES);

  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
  );
  // Check app status
  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Ok(
      Redirect::to(&format!(
        "{}/ui/setup",
        app_service.env_service().frontend_url()
      ))
      .into_response(),
    );
  }

  // Check if authorization is disabled
  if !authz_status(&secret_service) {
    return Ok(next.run(req).await);
  }

  // Check for token in session
  if let Some(access_token) = session
    .get::<String>("access_token")
    .await
    .map_err(AuthError::from)?
  {
    let access_token = token_service
      .get_valid_session_token(session, access_token)
      .await?;
    req
      .headers_mut()
      .insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
    Ok(next.run(req).await)
  } else if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    let header = header
      .to_str()
      .map_err(|err| AuthError::InvalidToken(err.to_string()))?;
    let access_token = token_service.validate_bearer_token(header).await?;
    req
      .headers_mut()
      .insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
    Ok(next.run(req).await)
  } else {
    Err(AuthError::InvalidAccess)?
  }
}

pub async fn optional_auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  req.headers_mut().remove(KEY_RESOURCE_TOKEN);
  req.headers_mut().remove(KEY_USER_ROLES);
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
  );

  // Check app status
  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Ok(next.run(req).await);
  }

  // Check if authorization is disabled
  if !authz_status(&secret_service) {
    return Ok(next.run(req).await);
  }

  // Try to get token from session
  if let Some(access_token) = session
    .get::<String>("access_token")
    .await
    .map_err(AuthError::from)?
  {
    if let Ok(validated_token) = token_service
      .get_valid_session_token(session, access_token)
      .await
    {
      req
        .headers_mut()
        .insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());
      return Ok(next.run(req).await);
    }
  }
  // Continue with the request, even if no valid token was found
  Ok(next.run(req).await)
}

fn authz_status(secret_service: &Arc<dyn SecretService>) -> bool {
  secret_service.authz().unwrap_or(true)
}

#[cfg(test)]
mod tests {
  use super::KEY_RESOURCE_TOKEN;
  use crate::{auth_middleware, optional_auth_middleware};
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
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext, RouterState,
  };
  use services::{
    test_utils::{
      build_token, expired_token, offline_access_token_claims, offline_token_claims, sign_token,
      token, AppServiceStubBuilder, SecretServiceStub, OTHER_KEY, OTHER_PRIVATE_KEY,
      TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, AuthServiceError, MockAuthService, SqliteSessionService,
    GRANT_REFRESH_TOKEN,
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
    authorization_header: Option<String>,
    path: String,
  }

  async fn test_handler_teapot(headers: HeaderMap, path: &str) -> Response {
    let x_resource_token = headers
      .get(KEY_RESOURCE_TOKEN)
      .map(|v| v.to_str().unwrap().to_string());
    let authorization_bearer_token = headers
      .get("Authorization")
      .map(|v| v.to_str().unwrap().to_string());
    (
      StatusCode::IM_A_TEAPOT,
      Json(TestResponse {
        x_resource_token,
        authorization_header: authorization_bearer_token,
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
          .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware)),
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
        authorization_header,
        path: "/with_optional_auth".to_string(),
      },
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_skips_if_app_status_ready_and_authz_false(
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::new()
      .with_authz_disabled()
      .with_app_status_ready();
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));
    let req = Request::get(path).body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let response_json = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        x_resource_token: None,
        authorization_header: None,
        path: path.to_string(),
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
  async fn test_auth_middleware_redirects_to_setup_for_app_status_setup_or_missing(
    #[case] secret_service: SecretServiceStub,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .with_db_service()
      .await
      .with_envs(maplit::hashmap! {"BODHI_FRONTEND_URL" => "https://bodhi.app"})
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));
    let req = Request::get("/with_auth").body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::SEE_OTHER, response.status());
    assert_eq!(
      "https://bodhi.app/ui/setup",
      response.headers().get("Location").unwrap()
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_with_valid_session_token(
    token: (String, String),
    temp_bodhi_home: TempDir,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (token, _) = token;
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
    let id = Id::default();
    let mut record = Record {
      id,
      data: maplit::hashmap! {
        "access_token".to_string() => Value::String(token.clone()),
      },
      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    session_service.session_store.create(&mut record).await?;

    let req = Request::get(path)
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .body(Body::empty())?;
    let router = test_router(state);

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let body = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        path: path.to_string(),
        authorization_header: None,
        x_resource_token: Some(token),
      },
      body
    );
    Ok(())
  }

  #[rstest]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_with_expired_session_token(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    expired_token: (String, String),
    temp_bodhi_home: TempDir,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (expired_token, _) = expired_token;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);

    // Create mock auth service
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_refresh_token()
      .with(
        eq("test_client_id"),
        eq("test_client_secret"),
        eq("refresh_token"),
      )
      .return_once(|_, _, _| {
        Ok((
          "new_access_token".to_string(),
          Some("new_refresh_token".to_string()),
        ))
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

    let req = Request::get(path)
      .header("Cookie", format!("bodhiapp_session_id={}", id))
      .body(Body::empty())?;
    let router = test_router(state);

    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let body = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        path: path.to_string(),
        authorization_header: None,
        x_resource_token: Some("new_access_token".to_string()),
      },
      body
    );

    // Verify that the session was updated with the new tokens
    let updated_record = session_service.session_store.load(&id).await?.unwrap();
    assert_eq!(
      "new_access_token",
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
      .json(json! {{}})?;
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let actual: Value = response.json().await?;
    assert_eq!(
      json! {{
        "x_resource_token": Option::<String>::None,
        "authorization_header": Option::<String>::None,
        "path": "/with_optional_auth",
      }},
      actual
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_bearer_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let (bearer_token, _) = build_token(offline_token_claims())?;
    let (access_token, _) = build_token(offline_access_token_claims())?;
    let access_token_cl = access_token.clone();
    let mut auth_service = MockAuthService::default();
    auth_service
      .expect_exchange_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(bearer_token.clone()),
        eq(GRANT_REFRESH_TOKEN),
        eq(vec![]),
      )
      .return_once(|_, _, _, _, _| Ok((access_token_cl, Some("refresh_token".to_string()))));
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
    let actual: Value = response.json().await?;
    assert_eq!(
      json! {{
        "x_resource_token": access_token,
        "authorization_header": Some(format!("Bearer {}", bearer_token)),
        "path": "/with_auth",
      }},
      actual
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_with_other_key_bearer_token(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let token = offline_token_claims();
    let (token, _) = sign_token(&OTHER_PRIVATE_KEY, &OTHER_KEY, token)?;
    let app_service = AppServiceStubBuilder::default()
      .with_secret_service()
      .with_session_service()
      .await
      .with_db_service()
      .await
      .build()?;
    let db = app_service.db_service.as_ref().unwrap();
    let _ = db.create_api_token_from("test-token-id", &token).await?;
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let router = test_router(state);
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token))
      .json(json! {{}})?;
    let response = router.clone().oneshot(req).await?;
    // assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let actual: Value = response.json().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "signature mismatch, error: \u{2068}InvalidSignature\u{2069}",
          "type": "authentication_error",
          "code": "auth_error-signature_mismatch"
        }
      }},
      actual
    );
    Ok(())
  }
}
