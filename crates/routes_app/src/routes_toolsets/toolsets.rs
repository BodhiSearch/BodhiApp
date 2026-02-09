use crate::{
  ApiKeyUpdateDto, AppToolsetConfigResponse, CreateToolsetRequest, ExecuteToolsetRequest,
  ListToolsetTypesResponse, ListToolsetsResponse, ToolsetResponse, ToolsetTypeResponse,
  ToolsetValidationError, UpdateToolsetRequest, ENDPOINT_TOOLSETS, ENDPOINT_TOOLSET_TYPES,
};
use auth_middleware::{
  ExtractToken, ExtractUserId, KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE,
  KEY_HEADER_BODHIAPP_TOOL_SCOPES,
};
use axum::{
  extract::{Path, State},
  http::{HeaderMap, StatusCode},
  routing::{delete, get, post, put},
  Json, Router,
};
use objs::API_TAG_TOOLSETS;
use objs::{ApiError, Toolset, ToolsetExecutionResponse, ToolsetScope};
use server_core::RouterState;
use services::db::ApiKeyUpdate;
use services::ToolsetError;
use std::collections::HashSet;
use std::sync::Arc;
use validator::Validate;

// ============================================================================
// Helper Functions
// ============================================================================

fn is_oauth_auth(headers: &HeaderMap) -> bool {
  !headers.contains_key(KEY_HEADER_BODHIAPP_ROLE)
    && headers
      .get(KEY_HEADER_BODHIAPP_SCOPE)
      .and_then(|v| v.to_str().ok())
      .map(|s| s.starts_with("scope_user_"))
      .unwrap_or(false)
}

fn extract_allowed_toolset_scopes(headers: &HeaderMap) -> HashSet<String> {
  let toolset_scopes_header = headers
    .get(KEY_HEADER_BODHIAPP_TOOL_SCOPES)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

  let allowed_scopes = ToolsetScope::from_scope_string(toolset_scopes_header);
  allowed_scopes.iter().map(|s| s.to_string()).collect()
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn routes_toolsets(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    // Toolset CRUD
    .route("/toolsets", get(list_toolsets_handler))
    .route("/toolsets", post(create_toolset_handler))
    .route("/toolsets/{id}", get(get_toolset_handler))
    .route("/toolsets/{id}", put(update_toolset_handler))
    .route("/toolsets/{id}", delete(delete_toolset_handler))
    // Execute (middleware at routes level)
    .route(
      "/toolsets/{id}/execute/{method}",
      post(execute_toolset_handler),
    )
    // Type listing and admin (separate namespace avoids {id} collision)
    .route("/toolset_types", get(list_toolset_types_handler))
    .route(
      "/toolset_types/{scope}/app-config",
      put(enable_type_handler),
    )
    .route(
      "/toolset_types/{scope}/app-config",
      delete(disable_type_handler),
    )
    .with_state(state)
}

// ============================================================================
// Toolset CRUD Handlers
// ============================================================================

/// List all toolsets for the authenticated user
///
/// For OAuth tokens, filters toolsets by scope_toolset-* scopes in the token.
#[utoipa::path(
  get,
  path = ENDPOINT_TOOLSETS,
  tag = API_TAG_TOOLSETS,
  operation_id = "listToolsets",
  responses(
    (status = 200, description = "List of user's toolsets", body = ListToolsetsResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_toolsets_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsetsResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let toolsets = tool_service.list(&user_id).await?;

  // OAuth filtering: hide toolsets of types not in scopes
  let filtered_toolsets = if is_oauth_auth(&headers) {
    let allowed_scopes = extract_allowed_toolset_scopes(&headers);
    toolsets
      .into_iter()
      .filter(|toolset| allowed_scopes.contains(&toolset.scope))
      .collect()
  } else {
    toolsets
  };

  // Enrich each toolset with type information
  let mut responses = Vec::new();
  for toolset in filtered_toolsets {
    responses.push(toolset_to_response(toolset, &tool_service).await?);
  }

  // Fetch toolset_types based on auth type
  let toolset_types = if is_oauth_auth(&headers) {
    // For OAuth: fetch only configs for scopes in the token (efficient database query)
    let allowed_scopes = extract_allowed_toolset_scopes(&headers);
    let scopes_vec: Vec<String> = allowed_scopes.into_iter().collect();
    tool_service
      .list_app_toolset_configs_by_scopes(&scopes_vec)
      .await?
  } else {
    // For session: return all toolset configs (session users have access to all scopes)
    tool_service.list_app_toolset_configs().await?
  };

  Ok(Json(ListToolsetsResponse {
    toolsets: responses,
    toolset_types,
  }))
}

/// Create a new toolset
#[utoipa::path(
  post,
  path = ENDPOINT_TOOLSETS,
  tag = API_TAG_TOOLSETS,
  operation_id = "createToolset",
  request_body = CreateToolsetRequest,
  responses(
    (status = 201, description = "Toolset created", body = ToolsetResponse),
    (status = 400, description = "Validation error"),
    (status = 409, description = "Name already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn create_toolset_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<CreateToolsetRequest>,
) -> Result<(StatusCode, Json<ToolsetResponse>), ApiError> {
  request
    .validate()
    .map_err(|e| ToolsetValidationError::Validation(e.to_string()))?;
  let tool_service = state.app_service().tool_service();

  let toolset = tool_service
    .create(
      &user_id,
      &request.scope_uuid,
      &request.name,
      request.description,
      request.enabled,
      request.api_key,
    )
    .await?;

  let response = toolset_to_response(toolset, &tool_service).await?;
  Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific toolset by ID
#[utoipa::path(
  get,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}",
  tag = API_TAG_TOOLSETS,
  operation_id = "getToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID")
  ),
  responses(
    (status = 200, description = "Toolset", body = ToolsetResponse),
    (status = 404, description = "Toolset not found or not owned"),
  ),
  security(("bearer" = []))
)]
pub async fn get_toolset_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<ToolsetResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let toolset = tool_service
    .get(&user_id, &id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("Toolset".to_string()))?;

  let response = toolset_to_response(toolset, &tool_service).await?;
  Ok(Json(response))
}

/// Update a toolset (full PUT semantics)
#[utoipa::path(
  put,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}",
  tag = API_TAG_TOOLSETS,
  operation_id = "updateToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID")
  ),
  request_body = UpdateToolsetRequest,
  responses(
    (status = 200, description = "Toolset updated", body = ToolsetResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "Toolset not found or not owned"),
    (status = 409, description = "Name already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn update_toolset_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  Json(request): Json<UpdateToolsetRequest>,
) -> Result<Json<ToolsetResponse>, ApiError> {
  request
    .validate()
    .map_err(|e| ToolsetValidationError::Validation(e.to_string()))?;
  let tool_service = state.app_service().tool_service();

  let api_key_update = match request.api_key {
    ApiKeyUpdateDto::Keep => ApiKeyUpdate::Keep,
    ApiKeyUpdateDto::Set(value) => ApiKeyUpdate::Set(value),
  };

  let toolset = tool_service
    .update(
      &user_id,
      &id,
      &request.name,
      request.description,
      request.enabled,
      api_key_update,
    )
    .await?;

  let response = toolset_to_response(toolset, &tool_service).await?;
  Ok(Json(response))
}

/// Delete a toolset
#[utoipa::path(
  delete,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}",
  tag = API_TAG_TOOLSETS,
  operation_id = "deleteToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID")
  ),
  responses(
    (status = 204, description = "Toolset deleted"),
    (status = 404, description = "Toolset not found or not owned"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_toolset_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let tool_service = state.app_service().tool_service();

  tool_service.delete(&user_id, &id).await?;

  Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Execute Handler
// ============================================================================

/// Execute a tool method on a toolset
#[utoipa::path(
  post,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}/execute/{method}",
  tag = API_TAG_TOOLSETS,
  operation_id = "executeToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID"),
    ("method" = String, Path, description = "Tool method name")
  ),
  request_body = ExecuteToolsetRequest,
  responses(
    (status = 200, description = "Tool execution result", body = ToolsetExecutionResponse),
    (status = 400, description = "Validation error or toolset not configured"),
    (status = 404, description = "Toolset or method not found"),
  ),
  security(("bearer" = []))
)]
pub async fn execute_toolset_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path((id, method)): Path<(String, String)>,
  Json(request): Json<ExecuteToolsetRequest>,
) -> Result<Json<ToolsetExecutionResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let response = tool_service
    .execute(&user_id, &id, &method, request.into())
    .await?;

  Ok(Json(response))
}

// ============================================================================
// Toolset Type Handlers (Admin)
// ============================================================================

/// List all available toolset types with their tools
///
/// For OAuth tokens, filters types by scope_toolset-* scopes in the token.
#[utoipa::path(
  get,
  path = ENDPOINT_TOOLSET_TYPES,
  tag = API_TAG_TOOLSETS,
  operation_id = "listToolsetTypes",
  responses(
    (status = 200, description = "List of all toolset types", body = ListToolsetTypesResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_toolset_types_handler(
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsetTypesResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let toolsets = tool_service.list_all_toolsets().await?;

  // OAuth filtering: hide types not in scopes
  let filtered_toolsets = if is_oauth_auth(&headers) {
    let allowed_scopes = extract_allowed_toolset_scopes(&headers);
    toolsets
      .into_iter()
      .filter(|t| allowed_scopes.contains(&t.scope))
      .collect()
  } else {
    toolsets
  };

  let types: Vec<ToolsetTypeResponse> = filtered_toolsets
    .into_iter()
    .map(|t| ToolsetTypeResponse {
      scope_uuid: t.scope_uuid,
      scope: t.scope,
      name: t.name,
      description: t.description,
      app_enabled: t.app_enabled,
      tools: t.tools,
    })
    .collect();

  Ok(Json(ListToolsetTypesResponse { types }))
}

/// Enable a toolset type at app level (admin only - enforced by auth middleware)
#[utoipa::path(
  put,
  path = ENDPOINT_TOOLSET_TYPES.to_owned() + "/{type_id}/app-config",
  tag = API_TAG_TOOLSETS,
  operation_id = "enableToolsetType",
  params(
    ("type_id" = String, Path, description = "Toolset type identifier")
  ),
  responses(
    (status = 200, description = "Toolset type enabled", body = AppToolsetConfigResponse),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn enable_type_handler(
  ExtractToken(admin_token): ExtractToken,
  ExtractUserId(updated_by): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(scope): Path<String>,
) -> Result<Json<AppToolsetConfigResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  // Find the toolset definition to get scope_uuid
  let toolset_def = tool_service
    .list_types()
    .into_iter()
    .find(|def| def.scope == scope)
    .ok_or_else(|| services::ToolsetError::ToolsetNotFound(scope.clone()))?;

  let config = tool_service
    .set_app_toolset_enabled(
      &admin_token,
      &scope,
      &toolset_def.scope_uuid,
      true,
      &updated_by,
    )
    .await?;

  Ok(Json(AppToolsetConfigResponse { config }))
}

/// Disable a toolset type at app level (admin only - enforced by auth middleware)
#[utoipa::path(
  delete,
  path = ENDPOINT_TOOLSET_TYPES.to_owned() + "/{type_id}/app-config",
  tag = API_TAG_TOOLSETS,
  operation_id = "disableToolsetType",
  params(
    ("type_id" = String, Path, description = "Toolset type identifier")
  ),
  responses(
    (status = 200, description = "Toolset type disabled", body = AppToolsetConfigResponse),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn disable_type_handler(
  ExtractToken(admin_token): ExtractToken,
  ExtractUserId(updated_by): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(scope): Path<String>,
) -> Result<Json<AppToolsetConfigResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  // Find the toolset definition to get scope_uuid
  let toolset_def = tool_service
    .list_types()
    .into_iter()
    .find(|def| def.scope == scope)
    .ok_or_else(|| services::ToolsetError::ToolsetNotFound(scope.clone()))?;

  let config = tool_service
    .set_app_toolset_enabled(
      &admin_token,
      &scope,
      &toolset_def.scope_uuid,
      false,
      &updated_by,
    )
    .await?;

  Ok(Json(AppToolsetConfigResponse { config }))
}

// ============================================================================
// Conversion Helpers
// ============================================================================

async fn toolset_to_response(
  toolset: Toolset,
  tool_service: &Arc<dyn services::ToolService>,
) -> Result<ToolsetResponse, ApiError> {
  // Get type information for enrichment
  let type_def = tool_service
    .get_type(&toolset.scope_uuid)
    .ok_or_else(|| ToolsetError::InvalidToolsetType(toolset.scope_uuid.clone()))?;

  Ok(ToolsetResponse {
    id: toolset.id,
    name: toolset.name,
    scope_uuid: toolset.scope_uuid,
    scope: toolset.scope,
    description: toolset.description,
    enabled: toolset.enabled,
    has_api_key: toolset.has_api_key,
    tools: type_def.tools,
    created_at: toolset.created_at,
    updated_at: toolset.updated_at,
  })
}
