use crate::{PaginatedApiTokenResponse, PaginationSortParams, ENDPOINT_TOKENS};
use auth_middleware::{KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_TOKEN};
use axum::{
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
  Json,
};
use axum_extra::extract::WithRejection;
use base64::{engine::general_purpose, Engine};
use objs::{
  ApiError, AppError, BadRequestError, EntityError, ErrorType, OpenAIApiError, ResourceRole,
  TokenScope, API_TAG_API_KEYS,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  db::{ApiToken, TokenStatus},
  extract_claims, AuthServiceError, IdClaims, TokenError,
};
use sha2::{Digest, Sha256};
use std::{str::FromStr, sync::Arc};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to create a new API token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "My Integration Token",
    "scope": "scope_token_user"
}))]
pub struct CreateApiTokenRequest {
  /// Descriptive name for the API token (minimum 3 characters)
  #[serde(default)]
  #[schema(min_length = 3, max_length = 100, example = "My Integration Token")]
  pub name: Option<String>,
  /// Token scope defining access level
  #[schema(example = "scope_token_user")]
  pub scope: TokenScope,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "token": "bodhiapp_1234567890abcdef"
}))]
pub struct ApiTokenResponse {
  /// API token with bodhiapp_ prefix for programmatic access
  #[schema(example = "bodhiapp_1234567890abcdef")]
  token: String,
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
    description = "Creates a new API token for programmatic access to the API. The token can be used for bearer authentication in API requests. Tokens are scoped based on user role: User role receives scope_token_user (basic access), while Admin/Manager/PowerUser roles receive scope_token_power_user (administrative access).",
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
             "token": "bodhiapp_1234567890abcdef"
         })),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn create_token_handler(
  headers: HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError> {
  let app_service = state.app_service();
  let db_service = app_service.db_service();

  // Extract user info from headers
  let resource_token = headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .and_then(|token| token.to_str().ok())
    .ok_or(ApiTokenError::AccessTokenMissing)?;

  let user_id = extract_claims::<IdClaims>(resource_token)?.sub;
  let user_role_str = headers
    .get(KEY_HEADER_BODHIAPP_ROLE)
    .and_then(|role| role.to_str().ok())
    .ok_or(ApiTokenError::AccessTokenMissing)?;

  let user_role =
    ResourceRole::from_str(user_role_str).map_err(|e| BadRequestError::new(e.to_string()))?;

  // Validate privilege escalation - users cannot create tokens with higher privileges than their role
  let token_scope = match (user_role, &payload.scope) {
    // User can only create user-level tokens
    (ResourceRole::User, TokenScope::User) => Ok(payload.scope),
    (ResourceRole::User, _) => Err(BadRequestError::new(
      "privilege_escalation_not_allowed".to_string(),
    )),

    // Other roles (PowerUser, Manager, Admin) can create user or power_user tokens only
    (_, TokenScope::User | TokenScope::PowerUser) => Ok(payload.scope),
    (_, _) => Err(BadRequestError::new("invalid_token_scope".to_string())),
  }?;

  // Generate cryptographically secure random token
  let mut random_bytes = [0u8; 32];
  rand::rng().fill_bytes(&mut random_bytes);
  let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
  let token_str = format!("bodhiapp_{}", random_string);

  // Extract prefix (first 8 chars after "bodhiapp_")
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash token with SHA-256
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // CRITICAL: Use TimeService, NOT Utc::now()!
  let now = app_service.time_service().utc_now();

  // Create ApiToken
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id,
    name: payload.name.unwrap_or_default(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: token_scope.to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };

  // Store in database
  db_service.create_api_token(&mut api_token).await?;

  // Return token (shown only once!)
  Ok((
    StatusCode::CREATED,
    Json(ApiTokenResponse { token: token_str }),
  ))
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
        (status = 404, description = "Token not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Token not found",
                 "type": "not_found_error",
                 "code": "entity_error-not_found"
             }
         })),
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
    create_token_handler, ApiTokenError, ApiTokenResponse, CreateApiTokenRequest,
    PaginatedApiTokenResponse, UpdateApiTokenRequest,
  };
  use anyhow_trace::anyhow_trace;
  use auth_middleware::{KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_TOKEN};
  use axum::{
    body::Body,
    http::{Method, Request},
    routing::{get, post, put},
    Router,
  };
  use hyper::StatusCode;
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    AppError, FluentLocalizationService, TokenScope,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::{
    db::{ApiToken, DbService, TokenStatus},
    test_utils::{
      access_token_claims, build_token, test_db_service, AppServiceStub, AppServiceStubBuilder,
      TestDbService,
    },
    AppService,
  };
  use sha2::{Digest, Sha256};
  use std::sync::Arc;
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
        token_prefix: format!("bodhiapp_test{:02}", i),
        token_hash: "token_hash".to_string(),
        scopes: "scope_token_user".to_string(),
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

  #[anyhow_trace]
  #[rstest]
  #[case::user_role_user_scope("resource_user", TokenScope::User, "scope_token_user")]
  #[case::admin_role_user_scope("resource_admin", TokenScope::User, "scope_token_user")]
  #[case::admin_role_power_user_scope(
    "resource_admin",
    TokenScope::PowerUser,
    "scope_token_power_user"
  )]
  #[case::manager_role_user_scope("resource_manager", TokenScope::User, "scope_token_user")]
  #[case::manager_role_power_user_scope(
    "resource_manager",
    TokenScope::PowerUser,
    "scope_token_power_user"
  )]
  #[case::power_user_role_user_scope("resource_power_user", TokenScope::User, "scope_token_user")]
  #[case::power_user_role_power_user_scope(
    "resource_power_user",
    TokenScope::PowerUser,
    "scope_token_power_user"
  )]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_role_scope_mapping(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] role: &str,
    #[case] requested_scope: TokenScope,
    #[case] expected_scope: &str,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let user_id = claims["sub"].as_str().unwrap().to_string();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Make create request with specific role and scope
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          .header(KEY_HEADER_BODHIAPP_ROLE, role)
          .json(&CreateApiTokenRequest {
            name: Some(format!("Test Token for {}", role)),
            scope: requested_scope,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::CREATED, response.status());

    let token_response = response.json::<ApiTokenResponse>().await?;
    assert!(
      token_response.token.starts_with("bodhiapp_"),
      "Token should start with 'bodhiapp_' prefix"
    );

    // Verify token in database has correct scope
    let token_prefix = &token_response.token[.."bodhiapp_".len() + 8];
    let db_token = test_db_service
      .get_api_token_by_prefix(token_prefix)
      .await?
      .expect("Token should exist in database");

    assert_eq!(expected_scope, db_token.scopes);
    assert_eq!(user_id, db_token.user_id);
    assert_eq!(TokenStatus::Active, db_token.status);

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let user_id = claims["sub"].as_str().unwrap().to_string();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Make create request
    let token_name = "Test Integration Token";
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          .header(KEY_HEADER_BODHIAPP_ROLE, "resource_user")
          .json(&CreateApiTokenRequest {
            name: Some(token_name.to_string()),
            scope: TokenScope::User,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::CREATED, response.status());

    let token_response = response.json::<ApiTokenResponse>().await?;
    let token_str = &token_response.token;

    // Verify token format
    assert!(
      token_str.starts_with("bodhiapp_"),
      "Token should start with 'bodhiapp_' prefix"
    );
    assert!(
      token_str.len() > 50,
      "Token should be sufficiently long (prefix + base64)"
    );

    // Extract prefix for DB lookup
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Verify token exists in database
    let db_token = test_db_service
      .get_api_token_by_prefix(token_prefix)
      .await?
      .expect("Token should exist in database");

    assert_eq!(token_name, db_token.name);
    assert_eq!(user_id, db_token.user_id);
    assert_eq!(TokenStatus::Active, db_token.status);
    assert_eq!("scope_token_user", db_token.scopes);

    // Verify hash matches
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let calculated_hash = format!("{:x}", hasher.finalize());
    assert_eq!(calculated_hash, db_token.token_hash);

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_without_name(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Make create request without name
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          .header(KEY_HEADER_BODHIAPP_ROLE, "resource_user")
          .json(&CreateApiTokenRequest {
            name: None,
            scope: TokenScope::User,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::CREATED, response.status());

    let token_response = response.json::<ApiTokenResponse>().await?;
    let token_prefix = &token_response.token[.."bodhiapp_".len() + 8];

    // Verify token has empty name
    let db_token = test_db_service
      .get_api_token_by_prefix(token_prefix)
      .await?
      .expect("Token should exist in database");

    assert_eq!("", db_token.name);

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_missing_auth(
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Make create request WITHOUT authentication header
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .json(&CreateApiTokenRequest {
            name: Some("Test Token".to_string()),
            scope: TokenScope::User,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_invalid_role(
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Make create request with INVALID role header
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          .header(KEY_HEADER_BODHIAPP_ROLE, "invalid_role_value")
          .json(&CreateApiTokenRequest {
            name: Some("Test Token".to_string()),
            scope: TokenScope::User,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_token_handler_missing_role(
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Make create request WITHOUT role header (should fail)
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          // NO KEY_HEADER_BODHIAPP_ROLE header - should return error
          .json(&CreateApiTokenRequest {
            name: Some("Test Token".to_string()),
            scope: TokenScope::User,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

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

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;
    let now = app_service.time_service().utc_now();

    // Create initial token
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user_id.to_string(),
      name: "Initial Name".to_string(),
      token_prefix: "bodhiapp_test123".to_string(),
      token_hash: "token_hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    test_db_service.create_api_token(&mut token).await?;

    // Setup app with router
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

  #[anyhow_trace]
  #[rstest]
  #[case::user_to_power_user("resource_user", TokenScope::PowerUser)]
  #[case::user_to_manager("resource_user", TokenScope::Manager)]
  #[case::user_to_admin("resource_user", TokenScope::Admin)]
  #[awt]
  #[tokio::test]
  async fn test_create_token_privilege_escalation_user(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] role: &str,
    #[case] requested_scope: TokenScope,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // User attempting to create higher-privilege token
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          .header(KEY_HEADER_BODHIAPP_ROLE, role)
          .json(&CreateApiTokenRequest {
            name: Some("Escalation Attempt".to_string()),
            scope: requested_scope,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[case::power_user_to_manager("resource_power_user", TokenScope::Manager)]
  #[case::power_user_to_admin("resource_power_user", TokenScope::Admin)]
  #[case::manager_to_admin("resource_manager", TokenScope::Admin)]
  #[case::admin_to_manager("resource_admin", TokenScope::Manager)]
  #[awt]
  #[tokio::test]
  async fn test_create_token_privilege_escalation_invalid_scopes(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] role: &str,
    #[case] requested_scope: TokenScope,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let claims = access_token_claims();
    let (access_token, _) = build_token(claims)?;

    let test_db_service = Arc::new(test_db_service);
    let app_service = AppServiceStubBuilder::default()
      .db_service(test_db_service.clone())
      .build()?;

    let app = app(app_service).await;

    // Any role attempting to create Manager/Admin token
    let response = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/tokens")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &access_token)
          .header(KEY_HEADER_BODHIAPP_ROLE, role)
          .json(&CreateApiTokenRequest {
            name: Some("Invalid Scope Attempt".to_string()),
            scope: requested_scope,
          })?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    Ok(())
  }
}
