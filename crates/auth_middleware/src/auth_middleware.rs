use crate::app_status_or_default;
use axum::{
  extract::{Request, State},
  http::{header::AUTHORIZATION, HeaderMap, HeaderValue},
  middleware::Next,
  response::{IntoResponse, Redirect, Response},
};
use jsonwebtoken::{DecodingKey, Validation};
use oauth2::{ClientId, ClientSecret, RefreshToken};
use objs::{impl_error_from, ApiError, AppError, BadRequestError, ErrorType};
use server_core::RouterState;
use services::{
  decode_access_token, get_secret, AppRegInfo, AppService, AppStatus, AuthService,
  AuthServiceError, Claims, JsonWebTokenError, SecretService, SecretServiceError, APP_AUTHZ_FALSE,
  APP_AUTHZ_TRUE, KEY_APP_AUTHZ, KEY_APP_REG_INFO, KEY_RESOURCE_TOKEN,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthError {
  #[error("auth_header_not_found")]
  #[error_meta(error_type = ErrorType::Authentication, status = 401)]
  AuthHeaderNotFound,
  #[error(transparent)]
  JsonWebToken(#[from] JsonWebTokenError),
  #[error("kid_mismatch")]
  #[error_meta(error_type = ErrorType::Authentication, status = 401)]
  KidMismatch(String, String),
  #[error("alg_mismatch")]
  #[error_meta(error_type = ErrorType::Authentication, status = 401)]
  AlgMismatch(String, String),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, status = 401)]
  AuthService(#[from] AuthServiceError),
  #[error("app_reg_info_missing")]
  #[error_meta(error_type = ErrorType::Authentication, status = 500)]
  AppRegInfoMissing,
  #[error("refresh_token_not_found")]
  #[error_meta(error_type = ErrorType::Authentication, status = 500)]
  RefreshTokenNotFound,
  #[error(transparent)]
  BadRequest(#[from] BadRequestError),
  #[error(transparent)]
  SecretService(#[from] SecretServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, status = 401, code = "auth_error-tower_sessions", args_delegate = false)]
  TowerSession(#[from] tower_sessions::session::Error),
}

impl_error_from!(
  ::jsonwebtoken::errors::Error,
  AuthError::JsonWebToken,
  services::JsonWebTokenError
);

pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(_auth_middleware(session, State(state), headers, req, next).await?)
}

async fn _auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let app_service = state.app_service();
  let auth_service = app_service.auth_service();
  let secret_service = app_service.secret_service();

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
  if authz_status(&secret_service) == APP_AUTHZ_FALSE {
    return Ok(next.run(req).await);
  }

  // Check for token in session
  if let Some(access_token) = session.get::<String>("access_token").await? {
    let validated_token =
      validate_access_token(session, &auth_service, &secret_service, access_token).await?;
    req
      .headers_mut()
      .insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());
    return Ok(next.run(req).await);
  }

  // Check for token in header
  let token = headers
    .get(AUTHORIZATION)
    .ok_or(AuthError::AuthHeaderNotFound)?;
  let validated_token =
    validate_token_from_header(token, app_service, secret_service, state).await?;

  // Set header and continue
  req
    .headers_mut()
    .insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());

  Ok(next.run(req).await)
}

pub async fn optional_auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  Ok(_optional_auth_middleware(session, State(state), headers, req, next).await?)
}

async fn _optional_auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let app_service = state.app_service();
  let auth_service = app_service.auth_service();
  let secret_service = app_service.secret_service();

  // Check app status
  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Ok(next.run(req).await);
  }

  // Check if authorization is disabled
  if authz_status(&secret_service) == APP_AUTHZ_FALSE {
    return Ok(next.run(req).await);
  }

  // Try to get token from session
  if let Some(access_token) = session.get::<String>("access_token").await? {
    if let Ok(validated_token) = validate_access_token(
      session.clone(),
      &auth_service,
      &secret_service,
      access_token,
    )
    .await
    {
      req
        .headers_mut()
        .insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());
      return Ok(next.run(req).await);
    }
  }

  // Try to get token from header
  if let Some(token) = headers.get(AUTHORIZATION) {
    if let Ok(validated_token) =
      validate_token_from_header(token, app_service, secret_service, state).await
    {
      req
        .headers_mut()
        .insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());
    }
  }

  // Continue with the request, even if no valid token was found
  Ok(next.run(req).await)
}

async fn validate_token_from_header(
  header: &HeaderValue,
  app_service: Arc<dyn AppService>,
  secret_service: Arc<dyn SecretService>,
  state: Arc<dyn RouterState>,
) -> Result<String, AuthError> {
  let token = header
    .to_str()
    .map_err(|e| BadRequestError::new(format!("authorization header is not valid utf-8: {e}")))?
    .strip_prefix("Bearer ")
    .ok_or(BadRequestError::new(
      "authorization header is malformed".to_string(),
    ))?
    .to_string();
  if token.is_empty() {
    return Err(BadRequestError::new("token not found in authorization header".to_string()).into());
  }
  let token_data = decode_access_token(&token)?;
  let jti = &token_data.claims.jti;
  let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
  let cache_service = app_service.cache_service();
  let cache_key = format!("exchange-access-token-{}", jti);
  if let Some(cached_data) = cache_service.get(&cache_key) {
    if let Some((cached_token, cached_hash)) = cached_data.split_once(':') {
      if cached_hash == token_hash {
        return Ok(cached_token.to_string());
      }
    } else {
      tracing::warn!("malformed cached key: '{cache_key}', deleting from cache");
      cache_service.remove(&cache_key);
    }
  }
  let app_reg_info: AppRegInfo =
    get_secret(secret_service, KEY_APP_REG_INFO)?.ok_or_else(|| AuthError::AppRegInfoMissing)?;
  let header = jsonwebtoken::decode_header(&token)?;
  if header.kid != Some(app_reg_info.kid.clone()) {
    return Err(AuthError::KidMismatch(
      app_reg_info.kid.to_string(),
      header.kid.unwrap_or_default(),
    ));
  }
  if header.alg != app_reg_info.alg {
    return Err(AuthError::AlgMismatch(
      format!("{:?}", app_reg_info.alg),
      format!("{:?}", header.alg),
    ));
  }
  let key_pem = format!(
    "-----BEGIN RSA PUBLIC KEY-----\n{}\n-----END RSA PUBLIC KEY-----",
    app_reg_info.public_key
  );
  let key = DecodingKey::from_rsa_pem(key_pem.as_bytes())?;
  let mut validation = Validation::new(header.alg);
  validation.set_issuer(&[app_reg_info.issuer]);
  validation.validate_aud = false;
  let _ = jsonwebtoken::decode::<Claims>(&token, &key, &validation)?;
  let (access_token, refresh_token) = state
    .app_service()
    .auth_service()
    .exchange_for_resource_token(&token)
    .await?;
  let cache_value = format!("{}:{}", access_token.secret(), token_hash);
  cache_service.set(&cache_key, &cache_value);
  cache_service.set(
    &format!("exchange-refresh-token-{}", jti),
    refresh_token.secret(),
  );
  Ok(access_token.secret().to_string())
}

async fn validate_access_token(
  session: Session,
  auth_service: &Arc<dyn AuthService>,
  secret_service: &Arc<dyn SecretService>,
  access_token: String,
) -> Result<String, AuthError> {
  // Validate session token
  let token_data = decode_access_token(&access_token)?;
  // Check if token is expired
  let now = time::OffsetDateTime::now_utc();
  if now.unix_timestamp() < token_data.claims.exp as i64 {
    return Ok(access_token);
  }
  let Some(refresh_token) = session.get::<String>("refresh_token").await? else {
    return Err(AuthError::RefreshTokenNotFound);
  };
  // Token is expired, try to refresh
  let app_reg_info: AppRegInfo =
    get_secret(secret_service, KEY_APP_REG_INFO)?.ok_or(AuthError::AppRegInfoMissing)?;

  let (new_access_token, new_refresh_token) = auth_service
    .refresh_token(
      RefreshToken::new(refresh_token),
      ClientId::new(app_reg_info.client_id),
      ClientSecret::new(app_reg_info.client_secret),
    )
    .await?;

  // Store new tokens in session
  session
    .insert("access_token", new_access_token.secret())
    .await?;
  session
    .insert("refresh_token", new_refresh_token.secret())
    .await?;

  Ok(new_access_token.secret().to_string())
}

fn authz_status(secret_service: &Arc<dyn SecretService>) -> String {
  secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or_else(|_| Some(APP_AUTHZ_TRUE.to_string()))
    .unwrap_or_else(|| APP_AUTHZ_TRUE.to_string())
}

#[cfg(test)]
mod tests {
  use crate::{auth_middleware, optional_auth_middleware, test_utils::setup_l10n_middleware};
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
  };
  use jsonwebtoken::Algorithm;
  use mockall::predicate::{always, eq};
  use oauth2::{AccessToken, ClientId, RefreshToken};
  use objs::{test_utils::temp_bodhi_home, FluentLocalizationService, ReqwestError};
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use server_core::{
    test_utils::ResponseTestExt, DefaultRouterState, MockSharedContextRw, RouterState,
  };
  use services::{
    test_utils::{expired_token, token, AppServiceStubBuilder, SecretServiceStub},
    AppRegInfoBuilder, AuthServiceError, CacheService, MockAuthService, MokaCacheService,
    SqliteSessionService, APP_STATUS_READY, APP_STATUS_SETUP, KEY_RESOURCE_TOKEN,
  };
  use sha2::{Digest, Sha256};
  use std::{collections::HashMap, sync::Arc};
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
    let mut secret_service = SecretServiceStub::new();
    secret_service
      .with_app_authz_disabled()
      .with_app_status_ready();
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
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

  fn with_app_status_setup() -> HashMap<String, String> {
    maplit::hashmap! {
      APP_STATUS_SETUP.to_string() => APP_STATUS_READY.to_string()
    }
  }

  #[rstest]
  #[case(SecretServiceStub::with_map(with_app_status_setup()))]
  #[case(SecretServiceStub::new())]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_auth_middleware_redirects_to_setup(
    #[case] secret_service: SecretServiceStub,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .with_envs(maplit::hashmap! {"BODHI_FRONTEND_URL" => "https://bodhi.app"})
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
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
  #[case(None, StatusCode::UNAUTHORIZED, json! {{
    "error": {
      "message": "authorization header not found in header",
      "type": "authentication_error",
      "code": "auth_error-auth_header_not_found"
    }
  }})]
  #[case(
    Some("bearer foobar"),
    StatusCode::BAD_REQUEST,
    json!{{
      "error": {
        "message": "invalid request, reason: \u{2068}authorization header is malformed\u{2069}",
        "type": "invalid_request_error",
        "code": "bad_request_error"
      }
    }}
  )]
  #[case(
    Some("Bearer "),
    StatusCode::BAD_REQUEST,
    json!{{
      "error": {
        "message": "invalid request, reason: \u{2068}token not found in authorization header\u{2069}",
        "type": "invalid_request_error",
        "code": "bad_request_error"
      }
    }}
  )]
  #[tokio::test]
  async fn test_auth_middleware_auth_header_errors(
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
    #[case] auth_header: Option<&str>,
    #[case] expected_status: StatusCode,
    #[case] expected_error: Value,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));
    let mut req = Request::get("/with_auth");
    if let Some(header) = auth_header {
      req = req.header("Authorization", header);
    }
    let req = req.body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    assert_eq!(expected_status, response.status());
    let api_error = response.json_obj::<Value>().await?;
    assert_eq!(api_error, expected_error);

    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_algorithm_mismatch(
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .alg(Algorithm::HS256)
        .build()?,
    );
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(
      value,
      json! {{
        "error": {
          "message": "the algorithm in the token does not match the expected algorithm, expected: \u{2068}HS256\u{2069}, found: \u{2068}RS256\u{2069}",
          "type": "authentication_error",
          "code": "auth_error-alg_mismatch"
        }
      }}
    );

    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_kid_mismatch(
    token: (String, String, String),
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .kid("other-kid".to_string())
        .build()?,
    );
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(
      value,
      json! {{
        "error": {
          "message": "the secure key id in the token does not match the expected key id, expected: \u{2068}other-kid\u{2069}, found: \u{2068}test-kid\u{2069}",
          "type": "authentication_error",
          "code": "auth_error-kid_mismatch"
        }
      }}
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_public_key_missing(
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let actual: Value = response.json_obj().await?;
    assert_eq!(
      actual,
      json! {{
        "error": {
          "message": "app registration info is missing, not found in secure storage",
          "type": "authentication_error",
          "code": "auth_error-app_reg_info_missing"
        }
      }}
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_auth_middleware_token_issuer_error(
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
    token: (String, String, String),
  ) -> anyhow::Result<()> {
    let (_, token, public_key) = token;
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .public_key(public_key)
        .issuer("https://id.other-domain.com/realms/other-app".to_string())
        .build()?,
    );
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));

    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router(state);

    let response = router.clone().oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(
      value,
      json! {{
        "error": {
          "message": "authentication token issuer is invalid",
          "type": "authentication_error",
          "code": "json_web_token_error-InvalidIssuer"
        }
      }}
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_no_exchange_if_present_in_cache(
    token: (String, String, String),
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (jti, token, _) = token;
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_for_resource_token()
      .never();

    let cache_service = MokaCacheService::default();
    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    let cached_token = "token-from-cache";
    cache_service.set(
      &format!("exchange-access-token-{}", jti),
      &format!("{}:{}", cached_token, token_hash),
    );

    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(&AppRegInfoBuilder::test_default().build()?);
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .cache_service(Arc::new(cache_service))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));

    let req = Request::get(path)
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router(state);
    let response = router.oneshot(req).await?;
    let status = response.status();
    assert_eq!(StatusCode::IM_A_TEAPOT, status);

    let response_json = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        path: path.to_string(),
        authorization_header: Some(format!("Bearer {token}")),
        x_resource_token: Some(cached_token.to_string()),
      },
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_exchange_if_exchange_token_not_in_cache(
    token: (String, String, String),
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (_, token, public_key) = token;
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_for_resource_token()
      .with(eq(token.clone()))
      .return_once(|_| {
        Ok((
          AccessToken::new("token-from-exchange".to_string()),
          RefreshToken::new("refresh-token-from-exchange".to_string()),
        ))
      });
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .public_key(public_key)
        .build()?,
    );
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .cache_service(Arc::new(MokaCacheService::default()))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
      app_service.clone(),
    ));

    let req = Request::get(path)
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router(state);
    let response = router.oneshot(req).await?;
    let status = response.status();
    assert_eq!(StatusCode::IM_A_TEAPOT, status);
    let response_json = response.json::<TestResponse>().await?;
    assert_eq!(
      TestResponse {
        path: path.to_string(),
        authorization_header: Some(format!("Bearer {token}")),
        x_resource_token: Some("token-from-exchange".to_string()),
      },
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_with_valid_session_token(
    token: (String, String, String),
    temp_bodhi_home: TempDir,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
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
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
    expired_token: (String, String, String),
    temp_bodhi_home: TempDir,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (_, expired_token, _) = expired_token;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);

    // Create mock auth service
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_refresh_token()
      .with(
        always(),
        eq(ClientId::new("test_client_id".to_string())),
        always(),
      )
      .return_once(|_, _, _| {
        Ok((
          AccessToken::new("new_access_token".to_string()),
          RefreshToken::new("new_refresh_token".to_string()),
        ))
      });

    // Setup app service with mocks
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .client_id("test_client_id".to_string())
        .client_secret("test_client_secret".to_string())
        .build()?,
    );

    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(Arc::new(mock_auth_service))
      .session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
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
    #[from(setup_l10n_middleware)] _setup_l10n: Arc<FluentLocalizationService>,
    expired_token: (String, String, String),
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let (_, expired_token, _) = expired_token;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);

    // Create mock auth service that fails to refresh the token
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_refresh_token()
      .with(
        always(),
        eq(ClientId::new("test_client_id".to_string())),
        always(),
      )
      .return_once(|_, _, _| {
        Err(AuthServiceError::Reqwest(ReqwestError::new(
          "Failed to refresh token".to_string(),
        )))
      });

    // Setup app service with mocks
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(
      &AppRegInfoBuilder::test_default()
        .client_id("test_client_id".to_string())
        .client_secret("test_client_secret".to_string())
        .build()?,
    );

    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .auth_service(Arc::new(mock_auth_service))
      .session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContextRw::new()),
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
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let actual: Value = response.json().await?;
    assert_eq!(
      actual,
      json! {{
        "error": {
          "message": "error connecting to internal service: \u{2068}Failed to refresh token\u{2069}",
          "type": "authentication_error",
          "code": "reqwest_error"
        }
      }}
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
}
