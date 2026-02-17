use crate::{PaginatedApiTokenResponse, PaginationSortParams, ENDPOINT_TOKENS};
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Extension, Json,
};
use axum_extra::extract::WithRejection;
use base64::{engine::general_purpose, Engine};
use objs::{
  ApiError, AppError, EntityError, ErrorType, OpenAIApiError, ResourceRole, TokenScope,
  API_TAG_API_KEYS,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{
  db::{ApiToken, TokenStatus},
  extract_claims, AuthServiceError, IdClaims, TokenError,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
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
  pub(crate) token: String,
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
  #[error("Application is not registered. Cannot create API tokens.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AppRegMissing,
  #[error("Access token is missing.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AccessTokenMissing,
  #[error("Refresh token not received from authentication server.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  RefreshTokenMissing,
  #[error(transparent)]
  AuthService(#[from] AuthServiceError),
  #[error("Privilege escalation not allowed.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PrivilegeEscalation,
  #[error("Invalid token scope.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidScope,
  #[error("Invalid role: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRole(String),
}

/// Create a new API token
///
/// **Security Note:** This endpoint requires browser session authentication only.
/// API tokens cannot be used to create additional tokens, preventing token-based
/// privilege escalation. Login via `/bodhi/v1/auth/initiate` is required.
#[utoipa::path(
    post,
    path = ENDPOINT_TOKENS,
    tag = API_TAG_API_KEYS,
    operation_id = "createApiToken",
    summary = "Create API Token",
    description = "Creates a new API token for programmatic access to the API. The token can be used for bearer authentication in API requests. Tokens are scoped based on user role: User role receives scope_token_user (basic access), while Admin/Manager/PowerUser roles receive scope_token_power_user (administrative access).\n\n**Security Note:** Session-only authentication required to prevent token-based privilege escalation.",
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
        (status = 403, description = "Forbidden - Session authentication required (API tokens cannot create tokens)", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn create_token_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError> {
  let app_service = state.app_service();
  let db_service = app_service.db_service();

  let AuthContext::Session {
    ref token,
    role: Some(user_role),
    ..
  } = auth_context
  else {
    return Err(ApiTokenError::AccessTokenMissing.into());
  };
  let resource_token = token;
  let user_id = extract_claims::<IdClaims>(resource_token)?.sub;

  // Validate privilege escalation - users cannot create tokens with higher privileges than their role
  let token_scope = match (user_role, &payload.scope) {
    // User can only create user-level tokens
    (ResourceRole::User, TokenScope::User) => Ok(payload.scope),
    (ResourceRole::User, _) => Err(ApiTokenError::PrivilegeEscalation),

    // Other roles (PowerUser, Manager, Admin) can create user or power_user tokens only
    (_, TokenScope::User | TokenScope::PowerUser) => Ok(payload.scope),
    (_, _) => Err(ApiTokenError::InvalidScope),
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
///
/// **Security Note:** Session-only authentication required for token management.
/// API tokens cannot manage other tokens to prevent privilege escalation.
#[utoipa::path(
    put,
    path = ENDPOINT_TOKENS.to_owned() + "/{id}",
    tag = API_TAG_API_KEYS,
    operation_id = "updateApiToken",
    summary = "Update API Token",
    description = "Updates the name and status of an existing API token. Only the token owner can update their tokens. Token values cannot be changed after creation.\n\n**Security Note:** Session-only authentication required to prevent token-based privilege escalation.",
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
        (status = 403, description = "Forbidden - Session authentication required (API tokens cannot manage tokens)", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn update_token_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateApiTokenRequest>, ApiError>,
) -> Result<Json<ApiToken>, ApiError> {
  let app_service = state.app_service();
  let db_service = app_service.db_service();
  let resource_token = auth_context
    .token()
    .ok_or(ApiTokenError::AccessTokenMissing)?;

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
///
/// **Security Note:** Session-only authentication required for viewing tokens.
/// API tokens cannot list other tokens to prevent information disclosure.
#[utoipa::path(
    get,
    path = ENDPOINT_TOKENS,
    tag = API_TAG_API_KEYS,
    operation_id = "listApiTokens",
    summary = "List API Tokens",
    description = "Retrieves paginated list of API tokens owned by the current user. Includes token metadata but not the actual token values for security.\n\n**Security Note:** Session-only authentication required to prevent information disclosure via tokens.",
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
        (status = 403, description = "Forbidden - Session authentication required (API tokens cannot list tokens)", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn list_tokens_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<PaginationSortParams>,
) -> Result<Json<PaginatedApiTokenResponse>, ApiError> {
  let per_page = query.page_size.min(100);
  let resource_token = auth_context
    .token()
    .ok_or(ApiTokenError::AccessTokenMissing)?;
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
mod tests;
