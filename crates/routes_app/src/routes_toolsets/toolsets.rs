use crate::{
  ApiKeyUpdateDto, CreateToolsetRequest, ExecuteToolsetRequest, ListToolsetTypesResponse,
  ListToolsetsResponse, ToolsetResponse, ToolsetValidationError, UpdateToolsetRequest,
  ENDPOINT_TOOLSETS, ENDPOINT_TOOLSET_TYPES,
};
use auth_middleware::ExtractUserId;
use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{delete, get, post, put},
  Json, Router,
};
use objs::API_TAG_TOOLSETS;
use objs::{ApiError, Toolset, ToolsetExecutionResponse};
use server_core::RouterState;
use services::db::ApiKeyUpdate;
use std::sync::Arc;
use validator::Validate;

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
    // Type listing (separate namespace avoids {id} collision)
    .route("/toolset_types", get(list_toolset_types_handler))
    // Admin enable/disable
    .route(
      "/toolset_types/{toolset_type}/app-config",
      put(enable_type_handler),
    )
    .route(
      "/toolset_types/{toolset_type}/app-config",
      delete(disable_type_handler),
    )
    .with_state(state)
}

// ============================================================================
// Toolset CRUD Handlers
// ============================================================================

/// List all toolsets for the authenticated user
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
) -> Result<Json<ListToolsetsResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let toolsets = tool_service.list(&user_id).await?;

  // Enrich each toolset with type information
  let mut responses = Vec::new();
  for toolset in toolsets {
    responses.push(toolset_to_response(toolset, &tool_service).await?);
  }

  // Populate toolset_types from database
  let toolset_types = tool_service.list_app_toolset_configs().await?;

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
      &request.toolset_type,
      &request.slug,
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
      &request.slug,
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
) -> Result<Json<ListToolsetTypesResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();
  let types = tool_service.list_types();
  Ok(Json(ListToolsetTypesResponse { types }))
}

/// Enable a toolset type at app level (Admin only)
#[utoipa::path(
  put,
  path = "/bodhi/v1/toolset_types/{toolset_type}/app-config",
  tag = API_TAG_TOOLSETS,
  operation_id = "enableToolsetType",
  params(
    ("toolset_type" = String, Path, description = "Toolset type identifier (e.g., 'builtin-exa-search')")
  ),
  responses(
    (status = 200, description = "Toolset type enabled", body = objs::AppToolsetConfig),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn enable_type_handler(
  ExtractUserId(updated_by): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(toolset_type): Path<String>,
) -> Result<Json<objs::AppToolsetConfig>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let config = tool_service
    .set_app_toolset_enabled(&toolset_type, true, &updated_by)
    .await?;

  Ok(Json(config))
}

/// Disable a toolset type at app level (Admin only)
#[utoipa::path(
  delete,
  path = "/bodhi/v1/toolset_types/{toolset_type}/app-config",
  tag = API_TAG_TOOLSETS,
  operation_id = "disableToolsetType",
  params(
    ("toolset_type" = String, Path, description = "Toolset type identifier (e.g., 'builtin-exa-search')")
  ),
  responses(
    (status = 200, description = "Toolset type disabled", body = objs::AppToolsetConfig),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn disable_type_handler(
  ExtractUserId(updated_by): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  Path(toolset_type): Path<String>,
) -> Result<Json<objs::AppToolsetConfig>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let config = tool_service
    .set_app_toolset_enabled(&toolset_type, false, &updated_by)
    .await?;

  Ok(Json(config))
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
    .get_type(&toolset.toolset_type)
    .ok_or_else(|| services::ToolsetError::InvalidToolsetType(toolset.toolset_type.clone()))?;

  Ok(ToolsetResponse {
    id: toolset.id,
    slug: toolset.slug,
    toolset_type: toolset.toolset_type,
    description: toolset.description,
    enabled: toolset.enabled,
    has_api_key: toolset.has_api_key,
    tools: type_def.tools,
    created_at: toolset.created_at,
    updated_at: toolset.updated_at,
  })
}
