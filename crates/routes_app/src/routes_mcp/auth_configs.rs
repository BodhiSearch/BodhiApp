use crate::routes_mcp::{
  AuthConfigsQuery, CreateAuthConfigBody, McpValidationError, OAuthLoginRequest,
  OAuthLoginResponse, OAuthTokenExchangeRequest, OAuthTokenResponse, ENDPOINT_MCPS_AUTH_CONFIGS,
};
use auth_middleware::{generate_random_string, AuthContext};
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Extension, Json,
};
use base64::{engine::general_purpose, Engine};
use objs::{ApiError, McpAuthConfigResponse, McpAuthConfigsListResponse, API_TAG_MCPS};
use server_core::RouterState;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

// ============================================================================
// Unified Auth Config Handlers
// ============================================================================

/// Create a new auth config (header, OAuth pre-registered, or OAuth dynamic)
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_AUTH_CONFIGS,
  tag = API_TAG_MCPS,
  operation_id = "createMcpAuthConfig",
  request_body = CreateAuthConfigBody,
  responses(
    (status = 201, description = "Auth config created", body = McpAuthConfigResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn create_auth_config_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Json(body): Json<CreateAuthConfigBody>,
) -> Result<(StatusCode, Json<McpAuthConfigResponse>), ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();
  let config = mcp_service
    .create_auth_config(user_id, &body.mcp_server_id, body.config)
    .await?;
  Ok((StatusCode::CREATED, Json(config)))
}

/// List all auth configs for an MCP server (headers + OAuth configs)
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS_AUTH_CONFIGS,
  tag = API_TAG_MCPS,
  operation_id = "listMcpAuthConfigs",
  params(AuthConfigsQuery),
  responses(
    (status = 200, description = "List of auth configs", body = McpAuthConfigsListResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_auth_configs_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<AuthConfigsQuery>,
) -> Result<Json<McpAuthConfigsListResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();
  let auth_configs = mcp_service.list_auth_configs(&query.mcp_server_id).await?;
  Ok(Json(McpAuthConfigsListResponse { auth_configs }))
}

/// Get an auth config by ID
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS_AUTH_CONFIGS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcpAuthConfig",
  params(
    ("id" = String, Path, description = "Auth config UUID")
  ),
  responses(
    (status = 200, description = "Auth config", body = McpAuthConfigResponse),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_auth_config_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(config_id): Path<String>,
) -> Result<Json<McpAuthConfigResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();
  let config = mcp_service
    .get_auth_config(&config_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("Auth config".to_string()))?;
  Ok(Json(config))
}

/// Delete an auth config by ID (cascades to tokens if OAuth)
#[utoipa::path(
  delete,
  path = ENDPOINT_MCPS_AUTH_CONFIGS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "deleteMcpAuthConfig",
  params(
    ("id" = String, Path, description = "Auth config UUID")
  ),
  responses(
    (status = 204, description = "Auth config deleted"),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_auth_config_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(config_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  // Check ownership: caller must be the config creator or have Admin/Manager role
  let config = mcp_service
    .get_auth_config(&config_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("Auth config".to_string()))?;

  let is_owner = config.created_by() == user_id;
  let is_privileged = matches!(
    auth_context,
    AuthContext::Session {
      role: Some(objs::ResourceRole::Admin | objs::ResourceRole::Manager),
      ..
    }
  );

  if !is_owner && !is_privileged {
    return Err(
      McpValidationError::Validation(
        "insufficient privileges to delete this auth config".to_string(),
      )
      .into(),
    );
  }

  mcp_service.delete_auth_config(&config_id).await?;
  Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// OAuth Flow Handlers
// ============================================================================

/// Initiate OAuth login for a config
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_AUTH_CONFIGS.to_owned() + "/{id}/login",
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthLogin",
  params(
    ("id" = String, Path, description = "Auth config UUID")
  ),
  request_body = OAuthLoginRequest,
  responses(
    (status = 200, description = "Authorization URL", body = OAuthLoginResponse),
    (status = 404, description = "Auth config not found"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_login_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Path(config_id): Path<String>,
  Json(request): Json<OAuthLoginRequest>,
) -> Result<Json<OAuthLoginResponse>, ApiError> {
  url::Url::parse(&request.redirect_uri)
    .map_err(|e| McpValidationError::InvalidRedirectUri(e.to_string()))?;

  let mcp_service = state.app_service().mcp_service();

  let config = mcp_service
    .get_oauth_config(&config_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("OAuth config".to_string()))?;

  let code_verifier = generate_random_string(43);
  let code_challenge =
    general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
  let oauth_state = uuid::Uuid::new_v4().to_string();

  let created_at = state.app_service().time_service().utc_now().timestamp();
  let session_key = format!("mcp_oauth_{}", config_id);
  session
    .insert(
      &session_key,
      serde_json::json!({
        "code_verifier": code_verifier,
        "state": oauth_state,
        "created_at": created_at,
      }),
    )
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?;

  let mut auth_url = url::Url::parse(&config.authorization_endpoint).map_err(|e| {
    McpValidationError::Validation(format!("invalid authorization endpoint: {}", e))
  })?;
  auth_url
    .query_pairs_mut()
    .append_pair("response_type", "code")
    .append_pair("client_id", &config.client_id)
    .append_pair("redirect_uri", &request.redirect_uri)
    .append_pair("code_challenge", &code_challenge)
    .append_pair("code_challenge_method", "S256")
    .append_pair("state", &oauth_state);
  if let Some(scopes) = &config.scopes {
    auth_url.query_pairs_mut().append_pair("scope", scopes);
  }

  let mcp_server = mcp_service
    .get_mcp_server(&config.mcp_server_id)
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?
    .ok_or_else(|| objs::EntityError::NotFound("MCP server".to_string()))?;
  auth_url
    .query_pairs_mut()
    .append_pair("resource", &mcp_server.url);

  Ok(Json(OAuthLoginResponse {
    authorization_url: auth_url.to_string(),
  }))
}

/// Exchange authorization code for tokens
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_AUTH_CONFIGS.to_owned() + "/{id}/token",
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthTokenExchange",
  params(
    ("id" = String, Path, description = "Auth config UUID")
  ),
  request_body = OAuthTokenExchangeRequest,
  responses(
    (status = 200, description = "Token stored", body = OAuthTokenResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "Auth config not found"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_token_exchange_handler(
  Extension(auth_context): Extension<AuthContext>,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Path(config_id): Path<String>,
  Json(request): Json<OAuthTokenExchangeRequest>,
) -> Result<Json<OAuthTokenResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  url::Url::parse(&request.redirect_uri)
    .map_err(|e| McpValidationError::InvalidRedirectUri(e.to_string()))?;

  let mcp_service = state.app_service().mcp_service();

  let session_key = format!("mcp_oauth_{}", config_id);
  let session_data: serde_json::Value = session
    .get(&session_key)
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?
    .ok_or(McpValidationError::SessionDataMissing)?;

  let code_verifier = session_data["code_verifier"]
    .as_str()
    .ok_or(McpValidationError::SessionDataMissing)?
    .to_string();

  let expected_state = session_data["state"]
    .as_str()
    .ok_or(McpValidationError::SessionDataMissing)?
    .to_string();

  // Validate CSRF state TTL (10 minutes)
  const CSRF_STATE_TTL_SECS: i64 = 600;
  if let Some(created_at) = session_data["created_at"].as_i64() {
    let now = state.app_service().time_service().utc_now().timestamp();
    if now - created_at > CSRF_STATE_TTL_SECS {
      let _ = session.remove::<serde_json::Value>(&session_key).await;
      return Err(McpValidationError::CsrfStateExpired.into());
    }
  }

  if request.state != expected_state {
    return Err(McpValidationError::CsrfStateMismatch.into());
  }

  let _ = session.remove::<serde_json::Value>(&session_key).await;

  let token = mcp_service
    .exchange_oauth_token(
      user_id,
      &config_id,
      &request.code,
      &request.redirect_uri,
      &code_verifier,
    )
    .await?;

  Ok(Json(OAuthTokenResponse::from(token)))
}

#[cfg(test)]
#[path = "test_auth_configs.rs"]
mod test_auth_configs;

#[cfg(test)]
#[path = "test_oauth_flow.rs"]
mod test_oauth_flow;
