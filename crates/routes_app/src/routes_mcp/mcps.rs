use crate::routes_mcp::{
  CreateMcpRequest, FetchMcpToolsRequest, ListMcpsResponse, McpAuth, McpExecuteRequest,
  McpExecuteResponse, McpResponse, McpToolsResponse, McpValidationError, UpdateMcpRequest,
  ENDPOINT_MCPS, ENDPOINT_MCPS_FETCH_TOOLS,
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
use services::{ApprovalStatus, ApprovedResources};
use std::sync::Arc;

// ============================================================================
// MCP Instance CRUD Handlers
// ============================================================================

/// List all MCP instances for the authenticated user
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS,
  tag = API_TAG_MCPS,
  operation_id = "listMcps",
  responses(
    (status = 200, description = "List of user's MCP instances", body = ListMcpsResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_mcps_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<ListMcpsResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let mcps = mcp_service.list(user_id).await?;

  let responses: Vec<McpResponse> = match &auth_context {
    AuthContext::ExternalApp {
      access_request_id: Some(ar_id),
      ..
    } => {
      let db_service = state.app_service().db_service();
      let approved_ids = extract_approved_mcp_ids(&db_service, ar_id).await;
      mcps
        .into_iter()
        .filter(|m| approved_ids.contains(&m.id))
        .map(McpResponse::from)
        .collect()
    }
    AuthContext::ExternalApp {
      access_request_id: None,
      ..
    } => vec![],
    _ => mcps.into_iter().map(McpResponse::from).collect(),
  };

  Ok(Json(ListMcpsResponse { mcps: responses }))
}

async fn extract_approved_mcp_ids(
  db_service: &std::sync::Arc<dyn services::db::DbService>,
  access_request_id: &str,
) -> Vec<String> {
  let Some(ar) = db_service.get(access_request_id).await.ok().flatten() else {
    return vec![];
  };
  let Some(approved_json) = &ar.approved else {
    return vec![];
  };
  let Ok(approvals) = serde_json::from_str::<ApprovedResources>(approved_json) else {
    return vec![];
  };
  approvals
    .mcps
    .iter()
    .filter(|a| a.status == ApprovalStatus::Approved)
    .filter_map(|a| a.instance.as_ref().map(|i| i.id.clone()))
    .collect()
}

/// Create a new MCP instance
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS,
  tag = API_TAG_MCPS,
  operation_id = "createMcp",
  request_body = CreateMcpRequest,
  responses(
    (status = 201, description = "MCP created", body = McpResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn create_mcp_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<CreateMcpRequest>,
) -> Result<(StatusCode, Json<McpResponse>), ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.name.is_empty() {
    return Err(McpValidationError::Validation("name is required".to_string()).into());
  }
  if request.slug.is_empty() {
    return Err(McpValidationError::Validation("slug is required".to_string()).into());
  }
  if request.mcp_server_id.is_empty() {
    return Err(McpValidationError::Validation("mcp_server_id is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let mcp = mcp_service
    .create(
      user_id,
      &request.name,
      &request.slug,
      &request.mcp_server_id,
      request.description,
      request.enabled,
      request.tools_cache,
      request.tools_filter,
      request.auth_type.clone(),
      request.auth_uuid,
    )
    .await?;

  Ok((StatusCode::CREATED, Json(McpResponse::from(mcp))))
}

/// Get a specific MCP instance by ID
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcp",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 200, description = "MCP instance", body = McpResponse),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_mcp_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<McpResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let mcp = mcp_service
    .get(user_id, &id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP".to_string()))?;

  Ok(Json(McpResponse::from(mcp)))
}

/// Update an MCP instance
#[utoipa::path(
  put,
  path = ENDPOINT_MCPS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "updateMcp",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  request_body = UpdateMcpRequest,
  responses(
    (status = 200, description = "MCP updated", body = McpResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn update_mcp_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  Json(request): Json<UpdateMcpRequest>,
) -> Result<Json<McpResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.name.is_empty() {
    return Err(McpValidationError::Validation("name is required".to_string()).into());
  }
  if request.slug.is_empty() {
    return Err(McpValidationError::Validation("slug is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let mcp = mcp_service
    .update(
      user_id,
      &id,
      &request.name,
      &request.slug,
      request.description,
      request.enabled,
      request.tools_filter,
      request.tools_cache,
      request.auth_type,
      request.auth_uuid,
    )
    .await?;

  Ok(Json(McpResponse::from(mcp)))
}

/// Delete an MCP instance
#[utoipa::path(
  delete,
  path = ENDPOINT_MCPS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "deleteMcp",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 204, description = "MCP deleted"),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_mcp_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  mcp_service.delete(user_id, &id).await?;

  Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// MCP Tool Discovery Handlers
// ============================================================================

/// Fetch tools from an MCP server without creating an MCP instance
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_FETCH_TOOLS,
  tag = API_TAG_MCPS,
  operation_id = "fetchMcpTools",
  request_body = FetchMcpToolsRequest,
  responses(
    (status = 200, description = "List of tools from MCP server", body = McpToolsResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "MCP server not found"),
  ),
  security(("bearer" = []))
)]
pub async fn fetch_mcp_tools_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<FetchMcpToolsRequest>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  if request.mcp_server_id.is_empty() {
    return Err(McpValidationError::Validation("mcp_server_id is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let (auth_header_key, auth_header_value) = match request.auth {
    Some(McpAuth::Header {
      header_key,
      header_value,
    }) => (Some(header_key), Some(header_value)),
    _ => (None, None),
  };

  let tools = mcp_service
    .fetch_tools_for_server(
      &request.mcp_server_id,
      auth_header_key,
      auth_header_value,
      request.auth_uuid,
    )
    .await?;

  Ok(Json(McpToolsResponse { tools }))
}

// ============================================================================
// MCP Tool Execution Handlers
// ============================================================================

/// List cached tools for an MCP instance
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS.to_owned() + "/{id}/tools",
  tag = API_TAG_MCPS,
  operation_id = "listMcpTools",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 200, description = "List of cached tools", body = McpToolsResponse),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn list_mcp_tools_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let mcp = mcp_service
    .get(user_id, &id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP".to_string()))?;

  let tools = mcp.tools_cache.unwrap_or_default();
  Ok(Json(McpToolsResponse { tools }))
}

/// Refresh tools by connecting to the MCP server
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS.to_owned() + "/{id}/tools/refresh",
  tag = API_TAG_MCPS,
  operation_id = "refreshMcpTools",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 200, description = "Refreshed list of tools", body = McpToolsResponse),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn refresh_mcp_tools_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let tools = mcp_service.fetch_tools(user_id, &id).await?;

  Ok(Json(McpToolsResponse { tools }))
}

/// Execute a tool on an MCP server
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS.to_owned() + "/{id}/tools/{tool_name}/execute",
  tag = API_TAG_MCPS,
  operation_id = "executeMcpTool",
  params(
    ("id" = String, Path, description = "MCP instance UUID"),
    ("tool_name" = String, Path, description = "Tool name to execute")
  ),
  request_body = McpExecuteRequest,
  responses(
    (status = 200, description = "Tool execution result", body = McpExecuteResponse),
    (status = 404, description = "MCP or tool not found"),
  ),
  security(("bearer" = []))
)]
pub async fn execute_mcp_tool_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path((id, tool_name)): Path<(String, String)>,
  Json(request): Json<McpExecuteRequest>,
) -> Result<Json<McpExecuteResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let exec_request = services::McpExecutionRequest {
    params: request.params,
  };

  let exec_response = mcp_service
    .execute(user_id, &id, &tool_name, exec_request)
    .await?;

  Ok(Json(McpExecuteResponse {
    result: exec_response.result,
    error: exec_response.error,
  }))
}

#[cfg(test)]
#[path = "test_mcps.rs"]
mod test_mcps;
