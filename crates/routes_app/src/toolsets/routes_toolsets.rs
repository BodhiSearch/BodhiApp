use crate::toolsets::error::ToolsetRouteError;
use crate::toolsets::toolsets_api_schemas::{
  ExecuteToolsetRequest, ListToolsetTypesResponse, ListToolsetsResponse, ToolsetResponse,
};
use crate::ApiError;
use crate::{
  AuthScope, ValidatedJson, API_TAG_APPS, API_TAG_TOOLSETS, ENDPOINT_APPS_TOOLSETS,
  ENDPOINT_TOOLSETS, ENDPOINT_TOOLSET_TYPES,
};
use axum::{
  extract::Path,
  http::StatusCode,
  routing::{delete, get, post, put},
  Json, Router,
};
use services::AppService;
use services::{
  AppToolsetConfig, ApprovalStatus, ApprovedResources, AuthContext, Toolset,
  ToolsetExecutionResponse, ToolsetRequest,
};
use std::sync::Arc;

// ============================================================================
// Router Configuration
// ============================================================================

pub fn routes_toolsets(state: Arc<dyn AppService>) -> Router {
  Router::new()
    // Toolset CRUD
    .route("/toolsets", get(toolsets_index))
    .route("/toolsets", post(toolsets_create))
    .route("/toolsets/{id}", get(toolsets_show))
    .route("/toolsets/{id}", put(toolsets_update))
    .route("/toolsets/{id}", delete(toolsets_destroy))
    // Execute (middleware at routes level)
    .route(
      "/toolsets/{id}/tools/{tool_name}/execute",
      post(toolsets_execute),
    )
    // Type listing (separate namespace avoids {id} collision)
    .route("/toolset_types", get(toolset_types_index))
    // Admin enable/disable
    .route(
      "/toolset_types/{toolset_type}/app-config",
      put(toolset_types_enable),
    )
    .route(
      "/toolset_types/{toolset_type}/app-config",
      delete(toolset_types_disable),
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
pub async fn toolsets_index(auth_scope: AuthScope) -> Result<Json<ListToolsetsResponse>, ApiError> {
  let entities = auth_scope.tools().list().await?;

  // Convert entities to domain Toolset type
  let toolsets: Vec<Toolset> = entities.into_iter().map(|e| e.into()).collect();

  // Filter toolset list for ExternalApp tokens: only return toolsets approved in the access request
  let toolsets = if let AuthContext::ExternalApp {
    access_request_id: Some(ar_id),
    ..
  } = auth_scope.auth_context()
  {
    let request = auth_scope
      .access_request_service()
      .get_request(ar_id)
      .await?;
    let approved_ids: std::collections::HashSet<String> = request
      .and_then(|r| r.approved)
      .map(|json| {
        serde_json::from_str::<ApprovedResources>(&json)
          .map_err(|_| ToolsetRouteError::InvalidApprovedJson)
      })
      .transpose()?
      .map(|res| match res {
        ApprovedResources::V1(v1) => v1
          .toolsets
          .into_iter()
          .filter(|a| a.status == ApprovalStatus::Approved)
          .filter_map(|a| a.instance.map(|i| i.id))
          .collect(),
      })
      .unwrap_or_default();
    toolsets
      .into_iter()
      .filter(|t| approved_ids.contains(&t.id))
      .collect()
  } else {
    toolsets
  };

  // Enrich each toolset with type information
  let tools = auth_scope.tools();
  let mut responses = Vec::new();
  for toolset in toolsets {
    responses.push(toolset_to_response(toolset, &tools).await?);
  }

  // Populate toolset_types from database
  let toolset_types = auth_scope.tools().list_app_toolset_configs().await?;

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
  request_body = ToolsetRequest,
  responses(
    (status = 201, description = "Toolset created", body = ToolsetResponse),
    (status = 400, description = "Validation error"),
    (status = 409, description = "Name already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn toolsets_create(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<ToolsetRequest>,
) -> Result<(StatusCode, Json<ToolsetResponse>), ApiError> {
  let tools = auth_scope.tools();
  let entity = tools.create(request).await?;
  let toolset: Toolset = entity.into();

  let response = toolset_to_response(toolset, &tools).await?;
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
pub async fn toolsets_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<ToolsetResponse>, ApiError> {
  let tools = auth_scope.tools();
  let entity = tools
    .get(&id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("Toolset".to_string()))?;
  let toolset: Toolset = entity.into();

  let response = toolset_to_response(toolset, &tools).await?;
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
  request_body = ToolsetRequest,
  responses(
    (status = 200, description = "Toolset updated", body = ToolsetResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "Toolset not found or not owned"),
    (status = 409, description = "Name already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn toolsets_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(request): ValidatedJson<ToolsetRequest>,
) -> Result<Json<ToolsetResponse>, ApiError> {
  let tools = auth_scope.tools();
  let entity = tools.update(&id, request).await?;
  let toolset: Toolset = entity.into();

  let response = toolset_to_response(toolset, &tools).await?;
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
pub async fn toolsets_destroy(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  auth_scope.tools().delete(&id).await?;

  Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Execute Handler
// ============================================================================

/// Execute a tool method on a toolset
#[utoipa::path(
  post,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}/tools/{tool_name}/execute",
  tag = API_TAG_TOOLSETS,
  operation_id = "executeToolsetTool",
  params(
    ("id" = String, Path, description = "Toolset instance UUID"),
    ("tool_name" = String, Path, description = "Tool name to execute")
  ),
  request_body = ExecuteToolsetRequest,
  responses(
    (status = 200, description = "Tool execution result", body = ToolsetExecutionResponse),
    (status = 400, description = "Validation error or toolset not configured"),
    (status = 404, description = "Toolset or method not found"),
  ),
  security(("bearer" = []))
)]
pub async fn toolsets_execute(
  auth_scope: AuthScope,
  Path((id, tool_name)): Path<(String, String)>,
  Json(request): Json<ExecuteToolsetRequest>,
) -> Result<Json<ToolsetExecutionResponse>, ApiError> {
  let response = auth_scope
    .tools()
    .execute(&id, &tool_name, request.into())
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
pub async fn toolset_types_index(
  auth_scope: AuthScope,
) -> Result<Json<ListToolsetTypesResponse>, ApiError> {
  let types = auth_scope.tools().list_types();
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
    (status = 200, description = "Toolset type enabled", body = AppToolsetConfig),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn toolset_types_enable(
  auth_scope: AuthScope,
  Path(toolset_type): Path<String>,
) -> Result<Json<AppToolsetConfig>, ApiError> {
  let config = auth_scope
    .tools()
    .set_app_toolset_enabled(&toolset_type, true)
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
    (status = 200, description = "Toolset type disabled", body = AppToolsetConfig),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn toolset_types_disable(
  auth_scope: AuthScope,
  Path(toolset_type): Path<String>,
) -> Result<Json<AppToolsetConfig>, ApiError> {
  let config = auth_scope
    .tools()
    .set_app_toolset_enabled(&toolset_type, false)
    .await?;

  Ok(Json(config))
}

// ============================================================================
// Conversion Helpers
// ============================================================================

async fn toolset_to_response(
  toolset: Toolset,
  tools: &services::AuthScopedToolService,
) -> Result<ToolsetResponse, ApiError> {
  // Get type information for enrichment
  let type_def = tools
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

// ============================================================================
// External App (/apps/) Wrappers
// ============================================================================

/// List toolsets accessible to the authenticated external app
#[utoipa::path(
  get,
  path = ENDPOINT_APPS_TOOLSETS,
  tag = API_TAG_APPS,
  operation_id = "appsListToolsets",
  responses(
    (status = 200, description = "List of toolsets accessible to the external app", body = ListToolsetsResponse),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn apps_toolsets_index(
  auth_scope: AuthScope,
) -> Result<Json<ListToolsetsResponse>, ApiError> {
  toolsets_index(auth_scope).await
}

/// Execute a tool on a toolset via external app
#[utoipa::path(
  post,
  path = ENDPOINT_APPS_TOOLSETS.to_owned() + "/{id}/tools/{tool_name}/execute",
  tag = API_TAG_APPS,
  operation_id = "appsExecuteToolsetTool",
  params(
    ("id" = String, Path, description = "Toolset instance UUID"),
    ("tool_name" = String, Path, description = "Tool name to execute")
  ),
  request_body = ExecuteToolsetRequest,
  responses(
    (status = 200, description = "Tool execution result", body = ToolsetExecutionResponse),
    (status = 400, description = "Validation error or toolset not configured"),
    (status = 404, description = "Toolset or method not found"),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn apps_toolsets_execute(
  auth_scope: AuthScope,
  Path((id, tool_name)): Path<(String, String)>,
  Json(request): Json<ExecuteToolsetRequest>,
) -> Result<Json<ToolsetExecutionResponse>, ApiError> {
  toolsets_execute(auth_scope, Path((id, tool_name)), Json(request)).await
}

#[cfg(test)]
#[path = "test_toolsets_crud.rs"]
mod test_toolsets_crud;

#[cfg(test)]
#[path = "test_toolsets_types.rs"]
mod test_toolsets_types;
