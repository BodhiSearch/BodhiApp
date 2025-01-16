use auth_middleware::KEY_RESOURCE_TOKEN;
use axum::{
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, EntityError, ErrorType};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  db::{ApiToken, TokenStatus},
  AuthServiceError, SecretServiceExt,
};
use std::{cmp::min, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiTokenRequest {
  name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTokenResponse {
  offline_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateApiTokenRequest {
  name: String,
  status: TokenStatus,
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
  let db_service = app_service.db_service();

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
  let _ = db_service
    .create_api_token_from(payload.name.unwrap_or_default().as_str(), &offline_token)
    .await?;
  let response = ApiTokenResponse { offline_token };
  Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_token_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(token_id): Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateApiTokenRequest>, ApiError>,
) -> Result<Json<ApiToken>, ApiError> {
  let app_service = state.app_service();
  let db_service = app_service.db_service();

  let mut token = db_service
    .get_api_token(&token_id)
    .await?
    .ok_or_else(|| EntityError::NotFound("Token".to_string()))?;

  // Update mutable fields
  token.name = payload.name;
  token.status = payload.status;

  db_service.update_api_token(&mut token).await?;

  Ok(Json(token))
}

#[derive(Debug, Deserialize)]
pub struct ListApiTokensQuery {
  #[serde(default = "default_page")]
  pub page: u32,
  #[serde(default = "default_page_size")]
  pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListApiTokensResponse {
  pub data: Vec<ApiToken>,
  pub total: u32,
  pub page: u32,
  pub page_size: u32,
}

fn default_page() -> u32 {
  1
}

fn default_page_size() -> u32 {
  10
}

pub async fn list_tokens_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<ListApiTokensQuery>,
) -> Result<Json<ListApiTokensResponse>, ApiError> {
  let per_page = min(query.page_size, 100);
  let (tokens, total) = state
    .app_service()
    .db_service()
    .list_api_tokens(query.page, per_page)
    .await?;

  Ok(Json(ListApiTokensResponse {
    data: tokens,
    total,
    page: query.page,
    page_size: per_page,
  }))
}

#[cfg(test)]
mod tests {
  use crate::{
    create_token_handler, wait_for_event, ApiTokenError, ListApiTokensResponse,
    UpdateApiTokenRequest,
  };
  use anyhow_trace::anyhow_trace;
  use auth_middleware::KEY_RESOURCE_TOKEN;
  use axum::{
    body::Body,
    http::{Method, Request},
    routing::{get, post, put},
    Router,
  };
  use chrono::Utc;
  use hyper::StatusCode;
  use mockall::predicate::eq;
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    AppError, FluentLocalizationService,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::{
    db::{ApiToken, DbService, TokenStatus},
    test_utils::{
      build_token, test_db_service, AppServiceStub, AppServiceStubBuilder, FrozenTimeService,
      SecretServiceStub, TestDbService,
    },
    AppRegInfo, AppService, AppStatus, AuthServiceError, MockAuthService,
  };
  use std::{sync::Arc, time::Duration};
  use tower::ServiceExt;
  use uuid::Uuid;

  use super::{list_tokens_handler, update_token_handler};

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
      .route("/api/tokens", post(create_token_handler))
      .route("/api/tokens", get(list_tokens_handler))
      .route("/api/tokens/:token_id", put(update_token_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_success(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let mut rx = test_db_service.subscribe();
    let (offline_token, _) = build_token(json! {
      {
        "iat": Utc::now(),
        "jti": "test-jti",
        "iss": "https://test-id.app/realms/test",
        "aud": "https://test-id.app/realms/test",
        "sub": "test-user-id",
        "typ": "Offline",
        "azp": "test-resource",
        "session_state": "test-session-id",
        "scope": "openid offline_access scope_token_user",
        "sid": "test-session-id"
      }
    })?;
    let offline_token_cl = offline_token.clone();
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
      .return_once(|_, _, _, _, _| Ok(("new_access_token".to_string(), Some(offline_token_cl))));

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
    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .auth_service(Arc::new(mock_auth_service))
      .secret_service(Arc::new(secret_service))
      .time_service(Arc::new(time_service))
      .db_service(test_db_service.clone())
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
          .uri("/api/tokens")
          .header(KEY_RESOURCE_TOKEN, "test_token")
          .json(&payload)?,
      )
      .await?;

    assert_eq!(StatusCode::CREATED, response.status());
    let response = response.json::<Value>().await?;
    let response_obj = response.as_object().unwrap();
    assert_eq!(
      offline_token,
      response_obj.get("offline_token").unwrap().as_str().unwrap()
    );
    let event_received = wait_for_event!(rx, "create_api_token_from", Duration::from_millis(500));
    assert!(
      event_received,
      "Timed out waiting for create_api_token_from event"
    );

    // List tokens to verify creation
    let (tokens, _) = test_db_service.list_api_tokens(1, 10).await?;
    assert_eq!(1, tokens.len());
    let created_token = &tokens[0];
    assert_eq!("test-jti", created_token.token_id);
    assert_eq!("test-user-id", created_token.user_id);
    assert_eq!("My API Token", created_token.name);
    assert_eq!(TokenStatus::Active, created_token.status);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_no_name(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let (offline_token, _) = build_token(json! {{"jti": "test-jti", "sub": "test-user"}})?;
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
      .return_once(|_, _, _, _, _| Ok(("some_access_token".to_string(), Some(offline_token))));

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
      .db_service(Arc::new(test_db_service))
      .build()
      .unwrap();
    let app = app(app_service).await;
    let payload = json!({});

    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_RESOURCE_TOKEN, "test_token")
          .json(&payload)?,
      )
      .await?;
    assert_eq!(StatusCode::CREATED, response.status());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_exchange_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
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
      .db_service(Arc::new(test_db_service))
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
          .uri("/api/tokens")
          .header(KEY_RESOURCE_TOKEN, "invalid_token")
          .json(&payload)?,
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

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_tokens_pagination(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(db_service.clone())
      .build()?;

    // Create multiple tokens
    for i in 1..=15 {
      let mut token = ApiToken {
        id: Uuid::new_v4().to_string(),
        user_id: "test-user".to_string(),
        name: format!("Test Token {}", i),
        token_id: Uuid::new_v4().to_string(),
        token_hash: "token_hash".to_string(),
        status: TokenStatus::Active,
        created_at: app_service.time_service().utc_now(),
        updated_at: app_service.time_service().utc_now(),
      };
      db_service.create_api_token(&mut token).await?;
    }

    let router = app(app_service).await;

    // Test first page
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/api/tokens?page=1&page_size=10")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let list_response = response.json::<ListApiTokensResponse>().await?;
    assert_eq!(list_response.data.len(), 10);
    assert_eq!(list_response.total, 15);
    assert_eq!(list_response.page, 1);
    assert_eq!(list_response.page_size, 10);

    // Test second page
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/api/tokens?page=2&page_size=10")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let list_response = response.json::<ListApiTokensResponse>().await?;
    assert_eq!(list_response.data.len(), 5);
    assert_eq!(list_response.total, 15);
    assert_eq!(list_response.page, 2);
    assert_eq!(list_response.page_size, 10);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_tokens_empty(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(db_service.clone())
      .build()?;

    let router = app(app_service).await;

    // Test empty results
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/api/tokens")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let list_response = response.json::<ListApiTokensResponse>().await?;
    assert_eq!(list_response.data.len(), 0);
    assert_eq!(list_response.total, 0);
    assert_eq!(list_response.page, 1); // Default page
    assert_eq!(list_response.page_size, 10); // Default page size

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_token_handler_success(
    #[from(setup_l10n)] _l18n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create initial token
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test_user".to_string(),
      name: "Initial Name".to_string(),
      token_id: "token123".to_string(),
      token_hash: "token_hash".to_string(),
      status: TokenStatus::Active,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    test_db_service.create_api_token(&mut token).await?;
    let test_db_service = Arc::new(test_db_service);

    // Setup app with router
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;
    let app = app(app_service).await;

    // Make update request
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri(format!("/api/tokens/{}", token.id))
          .json(&UpdateApiTokenRequest {
            name: "Updated Name".to_string(),
            status: TokenStatus::Inactive,
          })?,
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let updated_token = response.json::<ApiToken>().await?;
    assert_eq!(updated_token.name, "Updated Name");
    assert_eq!(updated_token.status, TokenStatus::Inactive);
    assert_eq!(updated_token.id, token.id);

    // Verify DB was updated
    let db_token = test_db_service.get_api_token(&token.id).await?.unwrap();
    assert_eq!(db_token.name, "Updated Name");
    assert_eq!(db_token.status, TokenStatus::Inactive);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_token_handler_not_found(
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Setup app with router
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(test_db_service))
      .build()?;
    let app = app(app_service).await;

    // Make update request with non-existent ID
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/tokens/non-existent-id")
          .json(&UpdateApiTokenRequest {
            name: "Updated Name".to_string(),
            status: TokenStatus::Inactive,
          })?,
      )
      .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
  }
}
