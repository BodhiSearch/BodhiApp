use crate::tokens::error::TokenRouteError;
use crate::tokens::tokens_api_schemas::{
  CreateTokenRequest, PaginatedTokenResponse, TokenCreated, TokenDetail, UpdateTokenRequest,
};
use crate::{ApiError, BodhiApiError, ValidatedJson};
use crate::{AuthScope, PaginationSortParams, API_TAG_API_KEYS, ENDPOINT_TOKENS};
use axum::{
  extract::{Path, Query},
  http::{
    header::{CACHE_CONTROL, PRAGMA},
    StatusCode,
  },
  Json,
};
use services::{ResourceRole, TokenScope};

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
        content = CreateTokenRequest,
        description = "API token creation parameters",
        example = json!({
            "name": "My Integration Token",
            "scopes": ["scope_token_user"]
        })
    ),
    responses(
        (status = 201, description = "API token created successfully", body = TokenCreated,
         example = json!({
             "token": "bodhiapp_1234567890abcdef"
         })),
        (status = 403, description = "Forbidden - Session authentication required (API tokens cannot create tokens)", body = BodhiApiError),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn tokens_create(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<CreateTokenRequest>,
) -> Result<
  (
    StatusCode,
    [(axum::http::HeaderName, &'static str); 2],
    Json<TokenCreated>,
  ),
  ApiError,
> {
  let user_role = auth_scope
    .auth_context()
    .resource_role()
    .ok_or(TokenRouteError::AccessTokenMissing)?;

  // Guest/Anonymous cannot create tokens
  if !user_role.has_access_to(&ResourceRole::User) {
    return Err(TokenRouteError::AccessTokenMissing.into());
  }

  // Validate privilege escalation - users cannot create tokens with higher privileges than their role
  let token_scope = match (user_role, &request.scope) {
    // User can only create user-level tokens
    (ResourceRole::User, TokenScope::User) => Ok(request.scope),
    (ResourceRole::User, _) => Err(TokenRouteError::PrivilegeEscalation),

    // Other roles (PowerUser, Manager, Admin) can create user or power_user tokens
    (_, TokenScope::User | TokenScope::PowerUser) => Ok(request.scope),
  }?;

  let validated = CreateTokenRequest {
    name: request.name,
    scope: token_scope,
  };
  let token_created = auth_scope.tokens().create_token(validated).await?;

  Ok((
    StatusCode::CREATED,
    [
      (CACHE_CONTROL, "no-store, no-cache, must-revalidate"),
      (PRAGMA, "no-cache"),
    ],
    Json(token_created),
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
        content = UpdateTokenRequest,
        description = "Token update request",
        example = json!({
            "name": "Updated Token Name",
            "status": "inactive"
        })
    ),
    responses(
        (status = 200, description = "Token updated successfully", body = TokenDetail,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "user_id": "auth0|123456789",
             "name": "Updated Token Name",
             "token_id": "jwt-token-id-1",
             "status": "inactive",
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-11-10T04:52:06.786Z"
         })),
        (status = 404, description = "Token not found", body = BodhiApiError,
         example = json!({
             "error": {
                 "message": "Token not found",
                 "type": "not_found_error",
                 "code": "entity_error-not_found"
             }
         })),
        (status = 403, description = "Forbidden - Session authentication required (API tokens cannot manage tokens)", body = BodhiApiError),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn tokens_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(request): ValidatedJson<UpdateTokenRequest>,
) -> Result<Json<TokenDetail>, ApiError> {
  let token = auth_scope.tokens().update_token(&id, request).await?;
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
        (status = 200, description = "List of API tokens", body = PaginatedTokenResponse,
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
        (status = 403, description = "Forbidden - Session authentication required (API tokens cannot list tokens)", body = BodhiApiError),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn tokens_index(
  auth_scope: AuthScope,
  Query(query): Query<PaginationSortParams>,
) -> Result<Json<PaginatedTokenResponse>, ApiError> {
  let per_page = query.page_size.min(100);
  let paginated = auth_scope
    .tokens()
    .list_tokens(query.page, per_page)
    .await?;
  Ok(Json(paginated))
}
