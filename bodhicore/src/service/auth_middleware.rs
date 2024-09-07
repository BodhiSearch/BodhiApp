use super::{
  get_secret, AuthServiceError, HttpError, HttpErrorBuilder, SecretServiceError, APP_AUTHZ_FALSE,
  APP_AUTHZ_TRUE, APP_STATUS_SETUP, KEY_APP_AUTHZ, KEY_APP_REG_INFO, KEY_APP_STATUS,
  KEY_RESOURCE_TOKEN,
};
use crate::server::RouterStateFn;
use axum::{
  extract::{Request, State},
  http::{header::AUTHORIZATION, HeaderMap},
  middleware::Next,
  response::{IntoResponse, Redirect, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, Clone, strum::Display, thiserror::Error)]
pub enum AuthError {
  TokenNotFound,
  InvalidToken(String),
  InsufficientPermission,
  BadRequest(String),
  InternalServerError(String),
  #[error(transparent)]
  ExchangeError(#[from] AuthServiceError),
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
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
    }
  }
}

impl IntoResponse for AuthError {
  fn into_response(self) -> axum::response::Response {
    HttpError::from(self).into_response()
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  jti: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(derive_builder::Builder))]
pub struct AppRegInfo {
  pub public_key: String,
  pub alg: Algorithm,
  pub kid: String,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: String,
}

pub async fn auth_middleware(
  _session: Session,
  State(state): State<Arc<dyn RouterStateFn>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let app_service = state.app_service();
  let secret_service = &app_service.secret_service();

  // Check app status
  let app_status = secret_service
    .get_secret_string(KEY_APP_STATUS)
    .unwrap_or_else(|_| Some(APP_STATUS_SETUP.to_string()))
    .unwrap_or_else(|| APP_STATUS_SETUP.to_string());
  if app_status == APP_STATUS_SETUP {
    return Ok(
      Redirect::to(&format!(
        "{}/ui/setup",
        app_service.env_service().frontend_url()
      ))
      .into_response(),
    );
  }

  // Check if authorization is disabled
  let authz_status = &secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or_else(|_| Some(APP_AUTHZ_TRUE.to_string()))
    .unwrap_or_else(|| APP_AUTHZ_TRUE.to_string());

  if authz_status == APP_AUTHZ_FALSE {
    return Ok(next.run(req).await);
  }

  // Extract token from header
  let token = match headers.get(AUTHORIZATION) {
    None => return Err(AuthError::TokenNotFound),
    Some(header) => header
      .to_str()
      .map_err(|e| AuthError::BadRequest(format!("header is not valid utf-8: {e}")))?
      .strip_prefix("Bearer ")
      .ok_or(AuthError::BadRequest(
        "authorization header is malformed".to_string(),
      ))?
      .to_string(),
  };

  if token.is_empty() {
    return Err(AuthError::BadRequest(
      "authorization header is malformed".to_string(),
    ));
  }

  // Calculate token hash
  let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

  // Decode token without validation to get the JTI
  let mut validation = Validation::default();
  validation.insecure_disable_signature_validation();
  let token_data = jsonwebtoken::decode::<Claims>(
    &token,
    &DecodingKey::from_secret(&[]), // dummy key for parsing
    &validation,
  )
  .map_err(|e| AuthError::InvalidToken(format!("error decoding token: {e}")))?;

  let jti = &token_data.claims.jti;

  // Check cache for existing exchange token
  let cache_service = app_service.cache_service();
  let cache_key = format!("exchange-access-token-{}", jti);
  if let Some(cached_data) = cache_service.get(&cache_key) {
    let (cached_token, cached_hash) = cached_data
      .split_once(':')
      .ok_or_else(|| AuthError::InternalServerError("Invalid cache data format".to_string()))?;
    if cached_hash == token_hash {
      req
        .headers_mut()
        .insert(KEY_RESOURCE_TOKEN, cached_token.parse().unwrap());
      return Ok(next.run(req).await);
    }
  }

  // If not in cache or hash mismatch, proceed with full token validation and exchange
  let app_reg_info: AppRegInfo =
    get_secret(secret_service, KEY_APP_REG_INFO)?.ok_or_else(|| {
      AuthError::InternalServerError("app registration info is missing".to_string())
    })?;

  // Validate token
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

  // Exchange token
  let (access_token, refresh_token) = state
    .app_service()
    .auth_service()
    .exchange_for_resource_token(&token)
    .await
    .map_err(AuthError::ExchangeError)?;

  // Store in cache with hash
  let cache_value = format!("{}:{}", access_token.secret(), token_hash);
  cache_service.set(&cache_key, &cache_value);
  cache_service.set(
    &format!("exchange-refresh-token-{}", jti),
    refresh_token.secret(),
  );

  // Set header and continue
  req
    .headers_mut()
    .insert(KEY_RESOURCE_TOKEN, access_token.secret().parse().unwrap());

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use super::{auth_middleware, AuthError};
  use crate::{
    server::{RouterState, RouterStateFn},
    service::{
      auth_middleware::AppRegInfoBuilder, AppServiceFn, AuthServiceError, CacheService, HttpError,
      MockAuthService, MokaCacheService, SecretServiceError, APP_STATUS_READY, APP_STATUS_SETUP,
      KEY_RESOURCE_TOKEN,
    },
    test_utils::{
      token, AppServiceStubBuilder, MockSharedContext, ResponseTestExt, SecretServiceStub,
    },
  };
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
  };
  use jsonwebtoken::Algorithm;
  use mockall::predicate::eq;
  use oauth2::{AccessToken, RefreshToken};
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use sha2::{Digest, Sha256};
  use std::{collections::HashMap, sync::Arc};
  use tower::ServiceExt;

  #[derive(Debug, Serialize, Deserialize)]
  struct TestResponse {
    x_resource_token: String,
    authorization_header: String,
    path: String,
  }

  async fn test_handler(headers: HeaderMap, Path(path): Path<String>) -> Response {
    let empty = HeaderValue::from_str("").unwrap();
    let x_api_token = headers
      .get(KEY_RESOURCE_TOKEN)
      .unwrap_or(&empty)
      .to_str()
      .unwrap();
    let authorization_bearer_token = headers
      .get("Authorization")
      .unwrap_or(&empty)
      .to_str()
      .unwrap();
    (
      StatusCode::IM_A_TEAPOT,
      Json(TestResponse {
        x_resource_token: x_api_token.to_string(),
        authorization_header: authorization_bearer_token.to_string(),
        path,
      }),
    )
      .into_response()
  }

  fn test_router() -> Router<Arc<dyn RouterStateFn>> {
    Router::new().route("/*path", get(test_handler))
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_skips_if_app_status_ready_and_authz_false() -> anyhow::Result<()> {
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
    let req = Request::get("/v1/chat/completions").body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
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
    let req = Request::get("/v1/chat/completions").body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::SEE_OTHER, response.status());
    assert_eq!(
      "https://bodhi.app/ui/setup",
      response.headers().get("Location").unwrap()
    );
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
    let mut req = Request::get("/some-path");
    if let Some(header) = auth_header {
      req = req.header("Authorization", header);
    }
    let req = req.body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let response = router.oneshot(req).await?;
    assert_eq!(expected_status, response.status());
    let j: Value = response.json_obj().await?;
    assert_eq!(expected_message, j["message"].as_str().unwrap());
    assert_eq!("invalid_request_error", j["type"].as_str().unwrap());
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
    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let response = router.oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(
      "unknown algorithm: RS256, supported: HS256",
      value["message"].as_str().unwrap()
    );
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
    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let response = router.oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(
      "unknown key id: 'test-kid', supported: 'other-kid'",
      value["message"].as_str().unwrap()
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_middleware_public_key_missing(
    token: anyhow::Result<(String, String, String)>
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
    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);
    let response = router.oneshot(req).await?;
    // assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let j: Value = response.json_obj().await?;
    // assert_eq!("internal_server_error", j["type"].as_str().unwrap());
    assert_eq!(
      "app registration info is missing",
      j["message"].as_str().unwrap()
    );
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

    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let response = router.oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!("invalid_api_key", value["code"].as_str().unwrap());
    assert_eq!(
      "error decoding/validating token: InvalidIssuer",
      value["message"].as_str().unwrap()
    );
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_auth_middleware_no_exchange_if_present_in_cache(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (jti, token, _) = token?;
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_for_resource_token()
      .never();

    let cache_service = MokaCacheService::new(None, None);
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

    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let response = router.oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(value.get("message"), None);
    assert_eq!(StatusCode::IM_A_TEAPOT, status);
    assert_eq!(cached_token, value["x_resource_token"].as_str().unwrap());
    assert_eq!(
      format!("Bearer {token}"),
      value["authorization_header"].as_str().unwrap()
    );
    assert_eq!("some-path", value["path"].as_str().unwrap());
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_auth_middleware_exchange_if_exchange_token_not_in_cache(
    token: anyhow::Result<(String, String, String)>,
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
      .cache_service(Arc::new(MokaCacheService::new(None, None)))
      .with_session_service()
      .await
      .build()?;
    let app_service = Arc::new(app_service);
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      app_service.clone(),
    ));

    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .layer(app_service.session_service().session_layer())
      .with_state(state);

    let response = router.oneshot(req).await?;
    let status = response.status();
    let value: Value = response.json_obj().await?;
    assert_eq!(value.get("message"), None);
    assert_eq!(StatusCode::IM_A_TEAPOT, status);
    assert_eq!(
      "token-from-exchange",
      value["x_resource_token"].as_str().unwrap()
    );
    assert_eq!(
      format!("Bearer {token}"),
      value["authorization_header"].as_str().unwrap()
    );
    assert_eq!("some-path", value["path"].as_str().unwrap());
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
    use crate::service::ErrorBody;

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
