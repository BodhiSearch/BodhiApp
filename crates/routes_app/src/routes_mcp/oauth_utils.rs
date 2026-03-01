use crate::routes_mcp::{
  DynamicRegisterRequest, DynamicRegisterResponse, McpValidationError, OAuthDiscoverAsRequest,
  OAuthDiscoverAsResponse, OAuthDiscoverMcpRequest, OAuthDiscoverMcpResponse, OAuthTokenResponse,
  ENDPOINT_MCPS_OAUTH_DISCOVER_AS, ENDPOINT_MCPS_OAUTH_DISCOVER_MCP,
  ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE,
};
use crate::API_TAG_MCPS;
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, State},
  http::StatusCode,
  Extension, Json,
};
use server_core::RouterState;
use services::ApiError;
use std::sync::Arc;

// ============================================================================
// OAuth Discovery Handlers
// ============================================================================

/// Discover OAuth metadata from an authorization server URL (RFC 8414)
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_OAUTH_DISCOVER_AS,
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthDiscoverAs",
  request_body = OAuthDiscoverAsRequest,
  responses(
    (status = 200, description = "OAuth discovery metadata", body = OAuthDiscoverAsResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_discover_as_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<OAuthDiscoverAsRequest>,
) -> Result<Json<OAuthDiscoverAsResponse>, ApiError> {
  url::Url::parse(&request.url)
    .map_err(|e| McpValidationError::InvalidUrl(format!("url: {}", e)))?;

  let mcp_service = state.app_service().mcp_service();

  let metadata = mcp_service.discover_oauth_metadata(&request.url).await?;

  let authorization_endpoint = metadata["authorization_endpoint"]
    .as_str()
    .unwrap_or_default()
    .to_string();
  let token_endpoint = metadata["token_endpoint"]
    .as_str()
    .unwrap_or_default()
    .to_string();
  let scopes_supported = metadata["scopes_supported"].as_array().map(|arr| {
    arr
      .iter()
      .filter_map(|v| v.as_str().map(String::from))
      .collect()
  });

  Ok(Json(OAuthDiscoverAsResponse {
    authorization_endpoint,
    token_endpoint,
    scopes_supported,
  }))
}

/// Discover OAuth metadata from an MCP server URL (RFC 9728 + RFC 8414)
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_OAUTH_DISCOVER_MCP,
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthDiscoverMcp",
  request_body = OAuthDiscoverMcpRequest,
  responses(
    (status = 200, description = "MCP OAuth discovery metadata", body = OAuthDiscoverMcpResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_discover_mcp_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<OAuthDiscoverMcpRequest>,
) -> Result<Json<OAuthDiscoverMcpResponse>, ApiError> {
  url::Url::parse(&request.mcp_server_url)
    .map_err(|e| McpValidationError::InvalidUrl(format!("mcp_server_url: {}", e)))?;

  let mcp_service = state.app_service().mcp_service();
  let metadata = mcp_service
    .discover_mcp_oauth_metadata(&request.mcp_server_url)
    .await?;

  Ok(Json(OAuthDiscoverMcpResponse {
    authorization_endpoint: metadata["authorization_endpoint"]
      .as_str()
      .map(String::from),
    token_endpoint: metadata["token_endpoint"].as_str().map(String::from),
    registration_endpoint: metadata["registration_endpoint"].as_str().map(String::from),
    scopes_supported: metadata["scopes_supported"].as_array().map(|arr| {
      arr
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
    }),
    resource: metadata["resource"].as_str().map(String::from),
    authorization_server_url: metadata["authorization_server_url"]
      .as_str()
      .map(String::from),
  }))
}

// ============================================================================
// Dynamic Client Registration
// ============================================================================

/// Standalone dynamic client registration (no server_id required)
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE,
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthDynamicRegisterStandalone",
  request_body = DynamicRegisterRequest,
  responses(
    (status = 200, description = "Dynamic client registration result", body = DynamicRegisterResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn standalone_dynamic_register_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<DynamicRegisterRequest>,
) -> Result<Json<DynamicRegisterResponse>, ApiError> {
  url::Url::parse(&request.registration_endpoint)
    .map_err(|e| McpValidationError::InvalidUrl(format!("registration_endpoint: {}", e)))?;
  url::Url::parse(&request.redirect_uri)
    .map_err(|e| McpValidationError::InvalidRedirectUri(e.to_string()))?;

  let mcp_service = state.app_service().mcp_service();
  let metadata = mcp_service
    .dynamic_register_client(
      &request.registration_endpoint,
      &request.redirect_uri,
      request.scopes,
    )
    .await?;

  Ok(Json(DynamicRegisterResponse {
    client_id: metadata["client_id"]
      .as_str()
      .unwrap_or_default()
      .to_string(),
    client_secret: metadata["client_secret"].as_str().map(String::from),
    client_id_issued_at: metadata["client_id_issued_at"].as_i64(),
    token_endpoint_auth_method: metadata["token_endpoint_auth_method"]
      .as_str()
      .map(String::from),
    registration_access_token: metadata["registration_access_token"]
      .as_str()
      .map(String::from),
  }))
}

// ============================================================================
// OAuth Token Handlers
// ============================================================================

/// Get an OAuth token by ID
#[utoipa::path(
  get,
  path = "/bodhi/v1/mcps/oauth-tokens/{token_id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcpOAuthToken",
  params(
    ("token_id" = String, Path, description = "OAuth token UUID")
  ),
  responses(
    (status = 200, description = "OAuth token", body = OAuthTokenResponse),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_oauth_token_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(token_id): Path<String>,
) -> Result<Json<OAuthTokenResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();
  let token = mcp_service
    .get_oauth_token(user_id, &token_id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("OAuth token".to_string()))?;
  Ok(Json(OAuthTokenResponse::from(token)))
}

/// Delete an OAuth token by ID
#[utoipa::path(
  delete,
  path = "/bodhi/v1/mcps/oauth-tokens/{token_id}",
  tag = API_TAG_MCPS,
  operation_id = "deleteMcpOAuthToken",
  params(
    ("token_id" = String, Path, description = "OAuth token UUID")
  ),
  responses(
    (status = 204, description = "OAuth token deleted"),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_oauth_token_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(token_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let db_service = state.app_service().db_service();
  db_service
    .delete_mcp_oauth_token(user_id, &token_id)
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?;
  Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "test_oauth_utils.rs"]
mod test_oauth_utils;
