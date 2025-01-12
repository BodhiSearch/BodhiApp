use auth_middleware::KEY_RESOURCE_TOKEN;
use axum::{
  extract::State,
  http::{HeaderMap, StatusCode},
  routing::post,
  Json, Router,
};
use axum_extra::extract::WithRejection;
use chrono::{DateTime, Utc};
use objs::{ApiError, AppError, ErrorType};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{AuthServiceError, SecretServiceExt};
use std::sync::Arc;

pub fn create_api_tokens_router() -> Router<Arc<dyn RouterState>> {
  Router::new().route("/api/tokens/create", post(create_token_handler))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiTokenRequest {
  name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTokenResponse {
  name: Option<String>,
  status: String,
  offline_token: String,
  last_used: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiTokenError {
  #[error("app_reg_info_missing")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AppRegMissing,
  #[error("token_missing")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AccessTokenMissing,
  #[error("refresh_token_missing")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  RefreshTokenMissing,
  #[error(transparent)]
  AuthService(#[from] AuthServiceError),
}

pub async fn create_token_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError> {
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();

  // Get client credentials
  let app_reg_info = secret_service.app_reg_info()?;
  let app_reg_info = app_reg_info.ok_or(ApiTokenError::AppRegMissing)?;

  let token = headers.get(KEY_RESOURCE_TOKEN).map(|token| token.to_str());
  let Some(Ok(token)) = token else {
    return Err(ApiTokenError::AccessTokenMissing)?;
  };
  // Exchange token
  let (_, offline_token) = auth_service
    .exchange_token(
      &app_reg_info.client_id,
      &app_reg_info.client_secret,
      token,
      "urn:ietf:params:oauth:token-type:refresh_token",
      vec![
        "openid".to_string(),
        "offline_access".to_string(),
        "scope_token_user".to_string(),
      ],
    )
    .await?;

  let offline_token = offline_token.ok_or(ApiTokenError::RefreshTokenMissing)?;
  let now = state.app_service().time_service().utc_now();
  let response = ApiTokenResponse {
    name: payload.name,
    status: "active".to_string(),
    offline_token,
    last_used: None,
    created_at: now,
    updated_at: now,
  };

  Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
  use crate::{create_token_handler, ApiTokenError, ApiTokenResponse};
  use auth_middleware::KEY_RESOURCE_TOKEN;
  use axum::{
    body::Body,
    http::{Method, Request},
    routing::post,
    Router,
  };
  use hyper::StatusCode;
  use mockall::predicate::eq;
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    AppError, FluentLocalizationService,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::{
    db::TimeService,
    test_utils::{AppServiceStub, AppServiceStubBuilder, FrozenTimeService, SecretServiceStub},
    AppRegInfo, AppStatus, AuthServiceError, MockAuthService,
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  #[rstest]
  #[case::app_reg_missing(
        &ApiTokenError::AppRegMissing,
        "app is not registered, cannot create api tokens"
    )]
  #[case::token_missing(
        &ApiTokenError::AccessTokenMissing,
        "access token is not present in request"
    )]
  #[case::refresh_token_missing(
        &ApiTokenError::RefreshTokenMissing,
        "refresh token not received from auth server"
    )]
  fn test_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }

  async fn app(app_service_stub: AppServiceStub) -> Router {
    let router_state = DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service_stub),
    );
    Router::new()
      .route("/api/tokens/create", post(create_token_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_success() -> anyhow::Result<()> {
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_token()
      .with(
        eq("test_client_id"),
        eq("test_client_secret"),
        eq("test_token"),
        eq("urn:ietf:params:oauth:token-type:refresh_token"),
        eq(vec![
          "openid".to_string(),
          "offline_access".to_string(),
          "scope_token_user".to_string(),
        ]),
      )
      .returning(|_, _, _, _, _| {
        Ok((
          "new_access_token".to_string(),
          Some("new_offline_token".to_string()),
        ))
      });

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      })
      .with_app_status(&AppStatus::Ready);

    let time_service = FrozenTimeService::default();
    let now = time_service.utc_now();
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .time_service(Arc::new(time_service))
      .build()
      .unwrap();
    let app = app(app_service).await;
    let payload = json!({
        "name": "My API Token"
    });
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens/create")
          .header("Content-Type", "application/json")
          .header(KEY_RESOURCE_TOKEN, "test_token")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(StatusCode::CREATED, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
          "name": "My API Token",
          "status": "active",
          "offline_token": "new_offline_token",
          "last_used": null,
          "created_at": now,
          "updated_at": now
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_no_name() -> anyhow::Result<()> {
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_token()
      .with(
        eq("test_client_id"),
        eq("test_client_secret"),
        eq("test_token"),
        eq("urn:ietf:params:oauth:token-type:refresh_token"),
        eq(vec![
          "openid".to_string(),
          "offline_access".to_string(),
          "scope_token_user".to_string(),
        ]),
      )
      .returning(|_, _, _, _, _| {
        Ok((
          "some_access_token".to_string(),
          Some("new_offline_token".to_string()),
        ))
      });

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      })
      .with_app_status(&AppStatus::Ready);

    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .build()
      .unwrap();
    let app = app(app_service).await;
    let payload = json!({});

    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens/create")
          .header("Content-Type", "application/json")
          .header(KEY_RESOURCE_TOKEN, "test_token")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(StatusCode::CREATED, response.status());
    let response = response.json::<ApiTokenResponse>().await?;
    assert!(response.name.is_none());
    assert_eq!("active", response.status);
    assert_eq!("new_offline_token", response.offline_token);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_exchange_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let mut mock_auth_service = MockAuthService::default();
    mock_auth_service
      .expect_exchange_token()
      .with(
        eq("test_client_id"),
        eq("test_client_secret"),
        eq("invalid_token"),
        eq("urn:ietf:params:oauth:token-type:refresh_token"),
        eq(vec![
          "openid".to_string(),
          "offline_access".to_string(),
          "scope_token_user".to_string(),
        ]),
      )
      .returning(|_, _, _, _, _| {
        Err(AuthServiceError::TokenExchangeError(
          "test_error".to_string(),
        ))
      });

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        public_key: "test_public_key".to_string(),
        alg: jsonwebtoken::Algorithm::RS256,
        kid: "test_kid".to_string(),
        issuer: "test_issuer".to_string(),
      })
      .with_app_status(&AppStatus::Ready);

    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .build()
      .unwrap();
    let app = app(app_service).await;
    let payload = json!({
        "name": "My API Token"
    });

    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens/create")
          .header("Content-Type", "application/json")
          .header(KEY_RESOURCE_TOKEN, "invalid_token")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json!({
          "error": {
              "type": "invalid_request_error",
              "code": "auth_service_error-token_exchange_error",
              "message": "token exchange failed: \u{2068}test_error\u{2069}"
          }
      }),
      response
    );

    Ok(())
  }
}
