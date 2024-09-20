use crate::server::{HttpError, HttpErrorBuilder};
use crate::{
  server::RouterStateFn,
  utils::{decode_access_token, Claims},
};
use axum::{
  extract::{Request, State},
  http::{header::AUTHORIZATION, HeaderMap, HeaderValue},
  middleware::Next,
  response::{IntoResponse, Redirect, Response},
};
use jsonwebtoken::{DecodingKey, Validation};
use oauth2::{ClientId, ClientSecret, RefreshToken};
use objs::AppRegInfo;
use services::{
  get_secret, AppServiceFn, AuthService, AuthServiceError, ISecretService, SecretServiceError,
  APP_AUTHZ_FALSE, APP_AUTHZ_TRUE, APP_STATUS_SETUP, KEY_APP_AUTHZ, KEY_APP_REG_INFO,
  KEY_APP_STATUS, KEY_RESOURCE_TOKEN,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
  #[error("TokenNotFound")]
  TokenNotFound,
  #[error("{0}")]
  InvalidToken(String),
  #[error("InsufficientPermission")]
  InsufficientPermission,
  #[error("{0}")]
  BadRequest(String),
  #[error("{0}")]
  InternalServerError(String),
  #[error(transparent)]
  ExchangeError(#[from] AuthServiceError),
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error("{0}")]
  SessionError(String),
}

impl From<AuthError> for HttpError {
  fn from(error: AuthError) -> Self {
    match error {
      AuthError::TokenNotFound => HttpErrorBuilder::default()
        .unauthorized("you didn't provide an API key", None)
        .build()
        .unwrap(),
      AuthError::InvalidToken(msg) => HttpErrorBuilder::default()
        .unauthorized(&msg, Some("invalid_api_key"))
        .build()
        .unwrap(),
      AuthError::InsufficientPermission => HttpErrorBuilder::default()
        .forbidden("you have insufficient permissions for this operation")
        .build()
        .unwrap(),
      AuthError::BadRequest(msg) => HttpErrorBuilder::default()
        .bad_request(&msg)
        .build()
        .unwrap(),
      AuthError::InternalServerError(msg) => HttpErrorBuilder::default()
        .internal_server(Some(&msg))
        .build()
        .unwrap(),
      AuthError::ExchangeError(err) => HttpErrorBuilder::default()
        .internal_server(Some(&err.to_string()))
        .build()
        .unwrap(),
      AuthError::SecretServiceError(err) => err.into(),
      AuthError::SessionError(err) => HttpErrorBuilder::default()
        .internal_server(Some(&err))
        .build()
        .unwrap(),
    }
  }
}

impl From<tower_sessions::session::Error> for AuthError {
  fn from(error: tower_sessions::session::Error) -> Self {
    AuthError::SessionError(error.to_string())
  }
}

impl IntoResponse for AuthError {
  fn into_response(self) -> axum::response::Response {
    HttpError::from(self).into_response()
  }
}

pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterStateFn>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let app_service = state.app_service();
  let auth_service = app_service.auth_service();
  let secret_service = app_service.secret_service();

  // Check app status
  if app_status(&secret_service) == APP_STATUS_SETUP {
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
  let token = headers.get(AUTHORIZATION).ok_or(AuthError::TokenNotFound)?;
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
  State(state): State<Arc<dyn RouterStateFn>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let app_service = state.app_service();
  let auth_service = app_service.auth_service();
  let secret_service = app_service.secret_service();

  // Check app status
  if app_status(&secret_service) == APP_STATUS_SETUP {
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
  app_service: Arc<dyn AppServiceFn>,
  secret_service: Arc<dyn ISecretService>,
  state: Arc<dyn RouterStateFn>,
) -> Result<String, AuthError> {
  let token = header
    .to_str()
    .map_err(|e| AuthError::BadRequest(format!("header is not valid utf-8: {e}")))?
    .strip_prefix("Bearer ")
    .ok_or(AuthError::BadRequest(
      "authorization header is malformed".to_string(),
    ))?
    .to_string();
  if token.is_empty() {
    return Err(AuthError::BadRequest(
      "authorization header is malformed".to_string(),
    ));
  }
  let token_data =
    decode_access_token(&token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;
  let jti = &token_data.claims.jti;
  let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
  let cache_service = app_service.cache_service();
  let cache_key = format!("exchange-access-token-{}", jti);
  if let Some(cached_data) = cache_service.get(&cache_key) {
    let (cached_token, cached_hash) = cached_data
      .split_once(':')
      .ok_or_else(|| AuthError::InternalServerError("Invalid cache data format".to_string()))?;
    if cached_hash == token_hash {
      return Ok(cached_token.to_string());
    }
  }
  let app_reg_info: AppRegInfo =
    get_secret(secret_service, KEY_APP_REG_INFO)?.ok_or_else(|| {
      AuthError::InternalServerError("app registration info is missing".to_string())
    })?;
  let header = jsonwebtoken::decode_header(&token)
    .map_err(|e| AuthError::InvalidToken(format!("error decoding token header: {e}")))?;
  if header.kid != Some(app_reg_info.kid.clone()) {
    return Err(AuthError::InvalidToken(format!(
      "unknown key id: '{}', supported: '{}'",
      header.kid.unwrap_or_default(),
      app_reg_info.kid
    )));
  }
  if header.alg != app_reg_info.alg {
    return Err(AuthError::InvalidToken(format!(
      "unknown algorithm: {:?}, supported: {:?}",
      header.alg, app_reg_info.alg
    )));
  }
  let key_pem = format!(
    "-----BEGIN RSA PUBLIC KEY-----\n{}\n-----END RSA PUBLIC KEY-----",
    app_reg_info.public_key
  );
  let key = DecodingKey::from_rsa_pem(key_pem.as_bytes())
    .map_err(|e| AuthError::InternalServerError(format!("error parsing public key: {e}")))?;
  let mut validation = Validation::new(header.alg);
  validation.set_issuer(&[app_reg_info.issuer]);
  validation.validate_aud = false;
  jsonwebtoken::decode::<Claims>(&token, &key, &validation)
    .map_err(|e| AuthError::InvalidToken(format!("error decoding/validating token: {e}")))?;
  let (access_token, refresh_token) = state
    .app_service()
    .auth_service()
    .exchange_for_resource_token(&token)
    .await
    .map_err(AuthError::ExchangeError)?;
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
  secret_service: &Arc<dyn ISecretService>,
  access_token: String,
) -> Result<String, AuthError> {
  // Validate session token
  let token_data =
    decode_access_token(&access_token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;

  // Check if token is expired
  let now = time::OffsetDateTime::now_utc();
  if now.unix_timestamp() >= token_data.claims.exp as i64 {
    // Token is expired, try to refresh
    if let Some(refresh_token) = session.get::<String>("refresh_token").await? {
      let app_reg_info: AppRegInfo =
        get_secret(secret_service, KEY_APP_REG_INFO)?.ok_or_else(|| {
          AuthError::InternalServerError("app registration info is missing".to_string())
        })?;

      let result = auth_service
        .refresh_token(
          RefreshToken::new(refresh_token),
          ClientId::new(app_reg_info.client_id),
          ClientSecret::new(app_reg_info.client_secret),
        )
        .await;

      if let Ok((new_access_token, new_refresh_token)) = result {
        // Store new tokens in session
        session
          .insert("access_token", new_access_token.secret())
          .await?;
        session
          .insert("refresh_token", new_refresh_token.secret())
          .await?;

        return Ok(new_access_token.secret().to_string());
      } else {
        return Err(AuthError::InvalidToken(
          "Cannot refresh access token, please logout and login again.".to_string(),
        ));
      }
    } else {
      return Err(AuthError::InvalidToken(
        "Session expired. Please log in again.".to_string(),
      ));
    }
  }
  Ok(access_token)
}

fn authz_status(secret_service: &Arc<dyn ISecretService>) -> String {
  secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or_else(|_| Some(APP_AUTHZ_TRUE.to_string()))
    .unwrap_or_else(|| APP_AUTHZ_TRUE.to_string())
}

fn app_status(secret_service: &Arc<dyn ISecretService>) -> String {
  secret_service
    .get_secret_string(KEY_APP_STATUS)
    .unwrap_or_else(|_| Some(APP_STATUS_SETUP.to_string()))
    .unwrap_or_else(|| APP_STATUS_SETUP.to_string())
}

#[cfg(test)]
mod tests {
  use super::{auth_middleware, AuthError};
  use crate::{
    server::{optional_auth_middleware, HttpError, RouterState, RouterStateFn},
    test_utils::{MockSharedContext, ResponseTestExt},
  };
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
  };
  use jsonwebtoken::Algorithm;
  use mockall::predicate::{always, eq};
  use oauth2::{AccessToken, ClientId, RefreshToken};
  use objs::test_utils::temp_bodhi_home;
  use objs::AppRegInfoBuilder;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use services::{
    test_utils::{expired_token, token, AppServiceStubBuilder, SecretServiceStub},
    AuthServiceError, CacheService, MockAuthService, MokaCacheService, SecretServiceError,
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

  fn router_with_auth() -> Router<Arc<dyn RouterStateFn>> {
    Router::new().route(
      "/with_auth",
      get(|headers: HeaderMap| async move { test_handler_teapot(headers, "/with_auth").await }),
    )
  }

  fn router_with_optional_auth() -> Router<Arc<dyn RouterStateFn>> {
    Router::new().route(
      "/with_optional_auth",
      get(|headers: HeaderMap| async move { test_handler_teapot(headers, "/with_optional_auth").await }),
    )
  }

  fn test_router(state: Arc<dyn RouterStateFn>) -> Router {
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
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
  #[case(None, StatusCode::UNAUTHORIZED, "you didn't provide an API key")]
  #[case(
    Some("bearer foobar"),
    StatusCode::BAD_REQUEST,
    "authorization header is malformed"
  )]
  #[case(
    Some("Bearer "),
    StatusCode::BAD_REQUEST,
    "authorization header is malformed"
  )]
  #[tokio::test]
  async fn test_auth_middleware_auth_header_errors(
    #[case] auth_header: Option<&str>,
    #[case] expected_status: StatusCode,
    #[case] expected_message: &str,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
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
    let response_json = response.json_obj::<Value>().await?;
    assert_eq!(expected_message, response_json["message"].as_str().unwrap());
    assert_eq!(
      "invalid_request_error",
      response_json["type"].as_str().unwrap()
    );

    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_algorithm_mismatch(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token?;
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
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
      "unknown algorithm: RS256, supported: HS256",
      value["message"].as_str().unwrap()
    );

    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_kid_mismatch(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token?;
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
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
      "unknown key id: 'test-kid', supported: 'other-kid'",
      value["message"].as_str().unwrap()
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_public_key_missing(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token?;
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));
    let req = Request::get("/with_auth")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router(state);
    let response = router.clone().oneshot(req).await?;
    // assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let j: Value = response.json_obj().await?;
    // assert_eq!("internal_server_error", j["type"].as_str().unwrap());
    assert_eq!(
      "app registration info is missing",
      j["message"].as_str().unwrap()
    );
    assert_optional_auth_passthrough(&router).await?;
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_auth_middleware_token_issuer_error(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (_, token, public_key) = token?;
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
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
    assert_eq!("invalid_api_key", value["code"].as_str().unwrap());
    assert_eq!(
      "error decoding/validating token: InvalidIssuer",
      value["message"].as_str().unwrap()
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
    token: anyhow::Result<(String, String, String)>,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (jti, token, _) = token?;
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
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
    token: anyhow::Result<(String, String, String)>,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (_, token, public_key) = token?;
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
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

  async fn error_handler(State(error): State<AuthError>) -> Result<(), HttpError> {
    Err(error)?
  }

  fn test_error_router(error: AuthError) -> Router {
    Router::new()
      .route("/test-error", get(error_handler))
      .with_state(error)
  }

  #[rstest]
  #[case("/with_auth")]
  #[case("/with_optional_auth")]
  #[tokio::test]
  async fn test_auth_middleware_with_valid_session_token(
    token: anyhow::Result<(String, String, String)>,
    temp_bodhi_home: TempDir,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (_, token, _) = token?;
    let dbfile = temp_bodhi_home.path().join("test.db");
    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(SecretServiceStub::default()))
      .session_service(session_service.clone())
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
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
    expired_token: anyhow::Result<(String, String, String)>,
    temp_bodhi_home: TempDir,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    let (_, expired_token, _) = expired_token?;
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
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
    expired_token: anyhow::Result<(String, String, String)>,
    temp_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let (_, expired_token, _) = expired_token?;
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
        Err(AuthServiceError::Reqwest(
          "Failed to refresh token".to_string(),
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
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
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
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let body: Value = response.json().await?;

    // Check that the error message is correct
    assert_eq!(
      "Cannot refresh access token, please logout and login again.",
      body["message"].as_str().unwrap()
    );
    assert_eq!("invalid_request_error", body["type"].as_str().unwrap());
    assert_eq!("invalid_api_key", body["code"].as_str().unwrap());

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
  #[case::token_not_found(
    AuthError::TokenNotFound,
    StatusCode::UNAUTHORIZED,
    json!({
      "message": "you didn't provide an API key",
      "type": "invalid_request_error",
      "param": null,
      "code": null
    })
  )]
  #[case::invalid_token(
    AuthError::InvalidToken("Invalid token".to_string()),
    StatusCode::UNAUTHORIZED,
    json!({
      "message": "Invalid token",
      "type": "invalid_request_error",
      "param": null,
      "code": "invalid_api_key"
    })
  )]
  #[case::insufficient_permission(
    AuthError::InsufficientPermission,
    StatusCode::FORBIDDEN,
    json!({
      "message": "you have insufficient permissions for this operation",
      "type": "invalid_request_error",
      "param": null,
      "code": null
    })
  )]
  #[case::bad_request(
    AuthError::BadRequest("Bad request".to_string()),
    StatusCode::BAD_REQUEST,
    json!({
      "message": "Bad request",
      "type": "invalid_request_error",
      "param": null,
      "code": "invalid_value"
    })
  )]
  #[case::internal_server_error(
    AuthError::InternalServerError("Internal error".to_string()),
    StatusCode::INTERNAL_SERVER_ERROR,
    json!({
      "message": "Internal error",
      "type": "internal_server_error",
      "param": null,
      "code": "internal_server_error",
    })
  )]
  #[tokio::test]
  async fn test_auth_error_conversion_and_response(
    #[case] error: AuthError,
    #[case] expected_status: StatusCode,
    #[case] expected_body: serde_json::Value,
  ) -> anyhow::Result<()> {
    let app = test_error_router(error);

    let request = Request::builder()
      .uri("/test-error")
      .method("GET")
      .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), expected_status);
    let body: Value = response.json().await?;
    assert_eq!(body, expected_body);
    Ok(())
  }

  #[rstest]
  #[case::exchange_error(
    AuthError::ExchangeError(AuthServiceError::Reqwest("failed to register as resource server".to_string())),
    StatusCode::INTERNAL_SERVER_ERROR,
    "reqwest: failed to register as resource server"
  )]
  #[case::secret_service_error(
    AuthError::SecretServiceError(SecretServiceError::SecretNotFound),
    StatusCode::INTERNAL_SERVER_ERROR,
    "Secret not found"
  )]
  #[tokio::test]
  async fn test_auth_error_conversion_for_wrapped_errors(
    #[case] error: AuthError,
    #[case] expected_status: StatusCode,
    #[case] expected_message: &str,
  ) -> anyhow::Result<()> {
    use crate::server::ErrorBody;

    let app = test_error_router(error);

    let request = Request::builder()
      .uri("/test-error")
      .method("GET")
      .body(Body::empty())?;

    let response = app.oneshot(request).await?;

    assert_eq!(response.status(), expected_status);

    let body: ErrorBody = response.json().await?;
    assert_eq!(
      ErrorBody {
        message: expected_message.to_string(),
        r#type: "internal_server_error".to_string(),
        param: None,
        code: Some("internal_server_error".to_string())
      },
      body
    );
    Ok(())
  }
}
