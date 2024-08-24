use crate::{
  oai::{ApiError, ApiErrorBuilder},
  server::RouterStateFn,
};
use axum::{
  extract::{Request, State},
  http::{header::AUTHORIZATION, HeaderMap, StatusCode},
  middleware::Next,
  response::{IntoResponse, Redirect, Response},
  Json,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{
  get_secret, AuthServiceError, SecretServiceError, APP_AUTHZ_FALSE, APP_AUTHZ_TRUE,
  APP_STATUS_SETUP, KEY_APP_AUTHZ, KEY_APP_REG_INFO, KEY_APP_STATUS, KEY_RESOURCE_TOKEN,
};

#[derive(Debug, strum::Display, thiserror::Error)]
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

impl From<&AuthError> for ApiError {
  fn from(value: &AuthError) -> Self {
    match value {
      AuthError::TokenNotFound => {
        ApiError::unauthorized("you didn't provide an API key".to_string(), None)
      }
      AuthError::InvalidToken(msg) => {
        ApiError::unauthorized(msg.clone(), Some("invalid_api_key".to_string()))
      }
      AuthError::InsufficientPermission => ApiError::unauthorized(
        "you have insufficient permissions for this operation".to_string(),
        None,
      ),
      AuthError::BadRequest(msg) => ApiErrorBuilder::default()
        .invalid_request_error(msg.to_owned())
        .build()
        .unwrap(),
      AuthError::InternalServerError(msg) => ApiErrorBuilder::default()
        .internal_server_error(msg.to_owned())
        .build()
        .unwrap(),
      AuthError::ExchangeError(err) => ApiErrorBuilder::default()
        .internal_server_error(err.to_string())
        .build()
        .unwrap(),
      AuthError::SecretServiceError(err) => ApiErrorBuilder::default()
        .internal_server_error(err.to_string())
        .build()
        .unwrap(),
    }
  }
}

impl From<&AuthError> for StatusCode {
  fn from(val: &AuthError) -> Self {
    match val {
      AuthError::TokenNotFound => StatusCode::UNAUTHORIZED,
      AuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
      AuthError::InsufficientPermission => StatusCode::FORBIDDEN,
      AuthError::BadRequest(_) => StatusCode::BAD_REQUEST,
      AuthError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AuthError::ExchangeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AuthError::SecretServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

impl IntoResponse for AuthError {
  fn into_response(self) -> Response {
    (StatusCode::from(&self), Json(ApiError::from(&self))).into_response()
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  jti: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(derive_builder::Builder))]
pub struct AppRegInfo {
  pub public_key: String,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: String,
}

pub async fn auth_middleware(
  State(state): State<Arc<dyn RouterStateFn>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let app_service = state.app_service();
  let secret_service = &app_service.secret_service();
  let app_status = secret_service
    .get_secret_string(KEY_APP_STATUS)
    .unwrap_or_else(|_| Some(APP_STATUS_SETUP.to_string()))
    .unwrap_or_else(|| APP_STATUS_SETUP.to_string());
  if app_status == APP_STATUS_SETUP {
    return Ok(Redirect::to("/ui/setup").into_response());
  }

  let authz_status = &secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or_else(|_| Some(APP_AUTHZ_TRUE.to_string()))
    .unwrap_or_else(|| APP_AUTHZ_TRUE.to_string());

  if authz_status == APP_AUTHZ_FALSE {
    return Ok(next.run(req).await);
  }
  let header = match headers.get(AUTHORIZATION) {
    None => return Err(AuthError::TokenNotFound),
    Some(header) => header
      .to_str()
      .map_err(|e| AuthError::BadRequest(format!("header is not valid utf-8: {e}")))?,
  };
  let token = header
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

  let app_reg_info: AppRegInfo =
    get_secret(secret_service, KEY_APP_REG_INFO)?.ok_or_else(|| {
      AuthError::InternalServerError("app registration info is missing".to_string())
    })?;

  let header = jsonwebtoken::decode_header(&token)
    .map_err(|e| AuthError::InvalidToken(format!("error decoding token header: {e}")))?;
  let decoded_pem = STANDARD
    .decode(&app_reg_info.public_key)
    .map_err(|e| AuthError::InternalServerError(format!("error decoding base64: {e}")))?;
  let pem_str = String::from_utf8(decoded_pem).map_err(|e| {
    AuthError::InternalServerError(format!(
      "error converting base64 public key to utf-8 string: {e}"
    ))
  })?;
  let key = DecodingKey::from_rsa_pem(pem_str.as_bytes())
    .map_err(|e| AuthError::InternalServerError(format!("error creating decoding key: {e}")))?;
  let mut validation = Validation::new(header.alg);
  validation.set_issuer(&[app_reg_info.issuer]);
  validation.validate_aud = false;
  let token_data = jsonwebtoken::decode::<Claims>(&token, &key, &validation)
    .map_err(|e| AuthError::InvalidToken(format!("error decoding/validating token: {e}")))?;

  let cache_service = app_service.cache_service();
  let exchange_token =
    match cache_service.get(&format!("exchange-access-token-{}", token_data.claims.jti)) {
      Some(token) => token,
      None => {
        let (access_token, refresh_token) = state
          .app_service()
          .auth_service()
          .exchange_for_resource_token(&token)
          .await
          .map_err(AuthError::ExchangeError)?;

        cache_service.set(
          &format!("exchange-access-token-{}", token_data.claims.jti),
          access_token.secret(),
        );
        cache_service.set(
          &format!("exchange-refresh-token-{}", token_data.claims.jti),
          refresh_token.secret(),
        );

        access_token.secret().to_string()
      }
    };
  req
    .headers_mut()
    .insert(KEY_RESOURCE_TOKEN, exchange_token.parse().unwrap());

  Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
  use super::auth_middleware;
  use crate::{
    server::{RouterState, RouterStateFn},
    service::{
      auth_middleware::AppRegInfoBuilder, CacheService, MockAuthService, MokaCacheService,
      APP_STATUS_READY, APP_STATUS_SETUP, KEY_RESOURCE_TOKEN,
    },
    test_utils::{AppServiceStubBuilder, MockSharedContext, ResponseTestExt, SecretServiceStub},
  };
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, HeaderValue, Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
  };
  use base64::{engine::general_purpose::STANDARD, Engine};
  use chrono::{Duration, Utc};
  use jsonwebtoken::{encode, EncodingKey, Header};
  use mockall::predicate::eq;
  use oauth2::{AccessToken, RefreshToken};
  use rsa::{
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    RsaPrivateKey,
  };
  use rstest::{fixture, rstest};
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use std::{collections::HashMap, sync::Arc};
  use tower::ServiceExt;
  use uuid::Uuid;

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
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let req = Request::get("/v1/chat/completions").body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
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
  async fn test_auth_middleware_redirects_to_setup(
    #[case] secret_service: SecretServiceStub,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let req = Request::get("/v1/chat/completions").body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .with_state(state);
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::SEE_OTHER, response.status());
    assert_eq!("/ui/setup", response.headers().get("Location").unwrap());
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
    let mut secret_service = SecretServiceStub::new();
    secret_service
      .with_app_status_ready()
      .with_app_authz_enabled();
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let mut req = Request::get("/some-path");
    if let Some(header) = auth_header {
      req = req.header("Authorization", header);
    }
    let req = req.body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
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
  async fn test_auth_middleware_public_key_missing() -> anyhow::Result<()> {
    let mut secret_service = SecretServiceStub::new();
    secret_service
      .with_app_status_ready()
      .with_app_authz_enabled();
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));
    let req = Request::get("/some-path")
      .header("Authorization", "Bearer foobar")
      .body(Body::empty())?;
    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .with_state(state);
    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let j: Value = response.json_obj().await?;
    assert_eq!("internal_server_error", j["type"].as_str().unwrap());
    assert_eq!(
      "app registration info is missing",
      j["message"].as_str().unwrap()
    );
    Ok(())
  }

  #[fixture]
  pub fn token() -> anyhow::Result<(String, String, String)> {
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = private_key.to_public_key();
    let jti = Uuid::new_v4().to_string();
    let claims = json!({
        "exp": (Utc::now() + Duration::hours(1)).timestamp(),
        "jti": jti,
        "iss": "https://id.mydomain.com/realms/myapp".to_string(),
        "sub": Uuid::new_v4().to_string(),
        "typ": "Bearer",
        "azp": "test-client",
        "session_state": Uuid::new_v4().to_string(),
        "scope": "openid scope_user profile email scope_power_user",
        "sid": Uuid::new_v4().to_string(),
        "email_verified": true,
        "name": "Test User",
        "preferred_username": "testuser@email.com",
        "given_name": "Test",
        "family_name": "User",
        "email": "testuser@email.com"
    });

    let header = Header {
      kid: Some(Uuid::new_v4().to_string()),
      alg: jsonwebtoken::Algorithm::RS256,
      ..Default::default()
    };

    let pem_file = private_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::CRLF)?;
    let token = encode(
      &header,
      &claims,
      &EncodingKey::from_rsa_pem(pem_file.as_bytes())?,
    )?;

    let public_key_base64 = public_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::CRLF)?;
    let output = STANDARD.encode(public_key_base64);
    Ok((jti, token, output))
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_auth_middleware_token_audience_error(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (_, token, public_key) = token?;
    let mut secret_service = SecretServiceStub::new();
    secret_service
      .with_app_status_ready()
      .with_app_authz_enabled()
      .with_app_reg_info(
        &AppRegInfoBuilder::test_default()
          .public_key(public_key)
          .issuer("https://id.other-domain.com/realms/other-app".to_string())
          .build()?,
      );
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .with_state(state);

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    let error: Value = response.json_obj().await?;
    assert_eq!("invalid_api_key", error["code"].as_str().unwrap());
    assert_eq!(
      "error decoding/validating token: InvalidIssuer",
      error["message"].as_str().unwrap()
    );
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_auth_middleware_no_exchange_if_present_in_cache(
    token: anyhow::Result<(String, String, String)>,
  ) -> anyhow::Result<()> {
    let (jti, token, public_key) = token?;
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_for_resource_token()
      .never();
    let cache_service = MokaCacheService::new(None, None);
    cache_service.set(
      &format!("exchange-access-token-{}", jti),
      "token-from-cache",
    );
    let mut secret_service = SecretServiceStub::new();
    secret_service
      .with_app_status_ready()
      .with_app_authz_enabled()
      .with_app_reg_info(
        &AppRegInfoBuilder::test_default()
          .public_key(public_key)
          .build()?,
      );
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .cache_service(Arc::new(cache_service))
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .with_state(state);

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let value: Value = response.json_obj().await?;
    assert_eq!(
      "token-from-cache",
      value["x_resource_token"].as_str().unwrap()
    );
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
    let mut secret_service = SecretServiceStub::new();
    secret_service
      .with_app_status_ready()
      .with_app_authz_enabled()
      .with_app_reg_info(
        &AppRegInfoBuilder::test_default()
          .public_key(public_key)
          .build()?,
      );
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .cache_service(Arc::new(MokaCacheService::new(None, None)))
      .build()?;
    let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let req = Request::get("/some-path")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let router = test_router()
      .layer(from_fn_with_state(state.clone(), auth_middleware))
      .with_state(state);

    let response = router.oneshot(req).await?;
    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
    let value: Value = response.json_obj().await?;
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
}
