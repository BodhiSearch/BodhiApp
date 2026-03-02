use crate::tokens::error::TokenRouteError;
use crate::tokens::tokens_api_schemas::{
  ApiTokenResponse, CreateApiTokenRequest, PaginatedApiTokenResponse, UpdateApiTokenRequest,
};
use crate::{AuthScope, PaginationSortParams, API_TAG_API_KEYS, ENDPOINT_TOKENS};
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  Json,
};
use axum_extra::extract::WithRejection;
use crate::{ApiError, JsonRejectionError, OpenAIApiError};
use services::{ApiToken, ResourceRole, TokenScope};

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
pub async fn tokens_create(
  auth_scope: AuthScope,
  WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, JsonRejectionError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError> {
  let AuthContext::Session {
    role: Some(user_role),
    ..
  } = auth_scope.auth_context()
  else {
    return Err(TokenRouteError::AccessTokenMissing.into());
  };

  // Validate privilege escalation - users cannot create tokens with higher privileges than their role
  let token_scope = match (user_role, &payload.scope) {
    // User can only create user-level tokens
    (ResourceRole::User, TokenScope::User) => Ok(payload.scope),
    (ResourceRole::User, _) => Err(TokenRouteError::PrivilegeEscalation),

    // Other roles (PowerUser, Manager, Admin) can create user or power_user tokens
    (_, TokenScope::User | TokenScope::PowerUser) => Ok(payload.scope),
  }?;

  let (token_str, _api_token) = auth_scope
    .tokens()
    .create_token(payload.name.unwrap_or_default(), token_scope)
    .await?;

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
pub async fn tokens_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateApiTokenRequest>, JsonRejectionError>,
) -> Result<Json<ApiToken>, ApiError> {
  let token = auth_scope
    .tokens()
    .update_token(&id, payload.name, payload.status)
    .await?;
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
pub async fn tokens_index(
  auth_scope: AuthScope,
  Query(query): Query<PaginationSortParams>,
) -> Result<Json<PaginatedApiTokenResponse>, ApiError> {
  let per_page = query.page_size.min(100);
  let (tokens, total) = auth_scope
    .tokens()
    .list_tokens(query.page, per_page)
    .await?;

  let paginated = PaginatedApiTokenResponse {
    data: tokens,
    total,
    page: query.page,
    page_size: per_page,
  };
  Ok(Json(paginated))
}
