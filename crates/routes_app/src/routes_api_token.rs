use crate::{PaginatedApiTokenResponse, PaginationSortParams, ENDPOINT_TOKENS};
use auth_middleware::KEY_HEADER_BODHIAPP_TOKEN;
use axum::{
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{
  ApiError, AppError, EntityError, ErrorType, OpenAIApiError, ServiceUnavailableError,
  API_TAG_API_KEYS,
};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  db::{ApiToken, TokenStatus},
  extract_claims, AuthServiceError, IdClaims, TokenError,
};
use std::sync::Arc;
use utoipa::ToSchema;

/// Request to create a new API token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "My Integration Token"
}))]
pub struct CreateApiTokenRequest {
  /// Descriptive name for the API token (minimum 3 characters)
  #[serde(default)]
  #[schema(min_length = 3, max_length = 100, example = "My Integration Token")]
  pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "offline_token": "bapp_1234567890abcdef"
}))]
pub struct ApiTokenResponse {
  /// API token with bapp_ prefix for programmatic access
  #[schema(example = "bapp_1234567890abcdef")]
  offline_token: String,
}

/// Request to update an existing API token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Updated Token Name",
    "status": "inactive"
}))]
pub struct UpdateApiTokenRequest {
  /// New descriptive name for the token (minimum 3 characters)
  #[schema(min_length = 3, max_length = 100, example = "Updated Token Name")]
  pub name: String,
  /// New status for the token (active/inactive)
  #[schema(example = "inactive")]
  pub status: TokenStatus,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiTokenError {
  #[error(transparent)]
  Token(#[from] TokenError),
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

/// Create a new API token
#[utoipa::path(
    post,
    path = ENDPOINT_TOKENS,
    tag = API_TAG_API_KEYS,
    operation_id = "createApiToken",
    summary = "Create API Token",
    description = "Creates a new API token for programmatic access to the API. The token can be used for bearer authentication in API requests. This feature is currently not available.",
    request_body(
        content = CreateApiTokenRequest,
        description = "API token creation parameters",
        example = json!({
            "name": "My Integration Token",
            "scopes": ["scope_token_user"]
        })
    ),
    responses(
        (status = 201, description = "API token created successfully", body = ApiTokenResponse,
         example = json!({
             "offline_token": "bapp_1234567890abcdef"
         })),
        (status = 400, description = "Invalid request parameters or token name already exists", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Token name must be at least 3 characters long",
                 "type": "invalid_request_error",
                 "code": "validation_error"
             }
         })),
        (status = 500, description = "Internal server error during token creation", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Failed to generate secure token",
                 "type": "internal_server_error",
                 "code": "token_generation_error"
             }
         }))
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn create_token_handler(
  _headers: HeaderMap,
  State(_state): State<Arc<dyn RouterState>>,
  WithRejection(Json(_payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError> {
  Err(ServiceUnavailableError::new(
    "api token feature is not available".to_string(),
  ))?
}

/// Update an existing API token
#[utoipa::path(
    put,
    path = ENDPOINT_TOKENS.to_owned() + "/{id}",
    tag = API_TAG_API_KEYS,
    operation_id = "updateApiToken",
    summary = "Update API Token",
    description = "Updates the name and status of an existing API token. Only the token owner can update their tokens. Token values cannot be changed after creation.",
    params(
        ("id" = String, Path, 
         description = "Unique identifier of the API token to update",
         example = "550e8400-e29b-41d4-a716-446655440000")
    ),
    request_body(
        content = UpdateApiTokenRequest,
        description = "Token update request",
        example = json!({
            "name": "Updated Token Name",
            "status": "inactive"
        })
    ),
    responses(
        (status = 200, description = "Token updated successfully", body = ApiToken,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "user_id": "auth0|123456789",
             "name": "Updated Token Name",
             "token_id": "jwt-token-id-1",
             "status": "inactive",
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-11-10T04:52:06.786Z"
         })),
        (status = 401, description = "Unauthorized - Token missing or invalid", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "access token is not present in request",
                 "type": "invalid_request_error",
                 "code": "api_token_error-token_missing"
             }
         })),
        (status = 404, description = "Token not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Token not found",
                 "type": "not_found_error",
                 "code": "entity_error-not_found"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Internal server error occurred",
                 "type": "internal_server_error",
                 "code": "internal_error"
             }
         }))
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn update_token_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateApiTokenRequest>, ApiError>,
) -> Result<Json<ApiToken>, ApiError> {
  let app_service = state.app_service();
  let db_service = app_service.db_service();

  let resource_token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .map(|token| token.to_str());
  let Some(Ok(resource_token)) = resource_token else {
    return Err(ApiTokenError::AccessTokenMissing)?;
  };
  let user_id = extract_claims::<IdClaims>(resource_token)?.sub;
  let mut token = db_service
    .get_api_token_by_id(&user_id, &id)
    .await?
    .ok_or_else(|| EntityError::NotFound("Token".to_string()))?;

  // Update mutable fields
  token.name = payload.name;
  token.status = payload.status;

  db_service.update_api_token(&user_id, &mut token).await?;

  Ok(Json(token))
}

/// List all API tokens for the current user
#[utoipa::path(
    get,
    path = ENDPOINT_TOKENS,
    tag = API_TAG_API_KEYS,
    operation_id = "listApiTokens",
    summary = "List API Tokens",
    description = "Retrieves paginated list of API tokens owned by the current user. Includes token metadata but not the actual token values for security.",
    params(
        PaginationSortParams
    ),
    responses(
        (status = 200, description = "List of API tokens", body = PaginatedApiTokenResponse,
         example = json!({
             "data": [
                 {
                     "id": "550e8400-e29b-41d4-a716-446655440000",
                     "user_id": "user@email.com",
                     "name": "Development Token",
                     "token_id": "jwt-token-id-1",
                     "status": "active",
                     "created_at": "2024-11-10T04:52:06.786Z",
                     "updated_at": "2024-11-10T04:52:06.786Z"
                 },
                 {
                     "id": "660e8400-e29b-41d4-a716-446655440000",
                     "user_id": "user@email.com",
                     "name": "Test Token",
                     "token_id": "jwt-token-id-2",
                     "status": "inactive",
                     "created_at": "2024-11-10T04:52:06.786Z",
                     "updated_at": "2024-11-10T04:52:06.786Z"
                 }
             ],
             "total": 2,
             "page": 1,
             "page_size": 10
         })),
        (status = 401, description = "Unauthorized - Token missing or invalid", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "access token is not present in request",
                 "type": "invalid_request_error",
                 "code": "api_token_error-token_missing"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Internal server error occurred",
                 "type": "internal_server_error",
                 "code": "internal_error"
             }
         }))
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn list_tokens_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<PaginationSortParams>,
) -> Result<Json<PaginatedApiTokenResponse>, ApiError> {
  let per_page = query.page_size.min(100);
  let resource_token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .map(|token| token.to_str());
  let Some(Ok(resource_token)) = resource_token else {
    return Err(ApiTokenError::AccessTokenMissing)?;
  };
  let user_id = extract_claims::<IdClaims>(resource_token)?.sub;

  let (tokens, total) = state
    .app_service()
    .db_service()
    .list_api_tokens(&user_id, query.page, per_page)
    .await?;

  let paginated = PaginatedApiTokenResponse {
    data: tokens,
    total,
    page: query.page,
    page_size: per_page,
  };
  Ok(Json(paginated))
}

#[cfg(test)]
mod tests {
  use super::{list_tokens_handler, update_token_handler};
  use crate::{
    create_token_handler, wait_for_event, ApiTokenError, PaginatedApiTokenResponse,
    UpdateApiTokenRequest,
  };
  use anyhow_trace::anyhow_trace;
  use auth_middleware::KEY_HEADER_BODHIAPP_TOKEN;
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
      access_token_claims, build_token, test_db_service, AppServiceStub, AppServiceStubBuilder,
      FrozenTimeService, SecretServiceStub, TestDbService,
    },
    AppRegInfo, AppService, AppStatus, AuthServiceError, MockAuthService,
  };
  use std::{sync::Arc, time::Duration};
  use tower::ServiceExt;
  use uuid::Uuid;

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
      .route("/api/tokens/{token_id}", put(update_token_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[ignore = "enable when supporting creating api tokens"]
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
      .times(1)
      .return_once(|_, _, _, _, _| Ok(("new_access_token".to_string(), Some(offline_token_cl))));

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, "test_token")
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
    let (tokens, _) = test_db_service
      .list_api_tokens("test-user-id", 1, 10)
      .await?;
    assert_eq!(1, tokens.len());
    let created_token = &tokens[0];
    assert_eq!("test-jti", created_token.token_id);
    assert_eq!("test-user-id", created_token.user_id);
    assert_eq!("My API Token", created_token.name);
    assert_eq!(TokenStatus::Active, created_token.status);
    Ok(())
  }

  #[rstest]
  #[ignore = "enable when supporting creating api tokens"]
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
      .times(1)
      .return_once(|_, _, _, _, _| Ok(("some_access_token".to_string(), Some(offline_token))));

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, "test_token")
          .json(&payload)?,
      )
      .await?;
    assert_eq!(StatusCode::CREATED, response.status());
    Ok(())
  }

  #[rstest]
  #[ignore = "enable when supporting creating api tokens"]
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
      .times(1)
      .return_once(|_, _, _, _, _| {
        Err(AuthServiceError::TokenExchangeError(
          "test_error".to_string(),
        ))
      });

    let secret_service = SecretServiceStub::new()
      .with_app_reg_info(&AppRegInfo {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, "invalid_token")
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
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let user_id = claims["sub"].as_str().unwrap().to_string();
    let (token, _) = build_token(claims)?;
    let db_service = Arc::new(db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(db_service.clone())
      .build()?;

    // Create multiple tokens
    for i in 1..=15 {
      let mut token = ApiToken {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .uri("/api/tokens?page=1&page_size=10")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let list_response = response.json::<PaginatedApiTokenResponse>().await?;
    assert_eq!(list_response.data.len(), 10);
    assert_eq!(list_response.total, 15);
    assert_eq!(list_response.page, 1);
    assert_eq!(list_response.page_size, 10);

    // Test second page
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .uri("/api/tokens?page=2&page_size=10")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let list_response = response.json::<PaginatedApiTokenResponse>().await?;
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
    let (token, _) = build_token(access_token_claims())?;
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .uri("/api/tokens")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let list_response = response.json::<PaginatedApiTokenResponse>().await?;
    assert_eq!(list_response.data.len(), 0);
    assert_eq!(list_response.total, 0);
    assert_eq!(list_response.page, 1); // Default page
    assert_eq!(list_response.page_size, 30); // Default page size

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_token_handler_success(
    #[from(setup_l10n)] _l18n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let user_id = claims["sub"].as_str().unwrap().to_string();
    let (access_token, _) = build_token(claims)?;
    // Create initial token
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user_id.to_string(),
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
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
    let db_token = test_db_service
      .get_api_token_by_id(&user_id, &token.id)
      .await?
      .unwrap();
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
    let (token, _) = build_token(access_token_claims())?;
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
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
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
