use crate::{
  CreateMcpRequest, EnableMcpServerRequest, ListMcpServersResponse, ListMcpsResponse,
  McpExecuteRequest, McpExecuteResponse, McpResponse, McpServerUrlQuery, McpToolsResponse,
  McpValidationError, UpdateMcpRequest, ENDPOINT_MCPS, ENDPOINT_MCP_SERVERS,
};
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Extension, Json,
};
use objs::{ApiError, API_TAG_MCPS};
use server_core::RouterState;
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
  let responses: Vec<McpResponse> = mcps.into_iter().map(McpResponse::from).collect();

  Ok(Json(ListMcpsResponse { mcps: responses }))
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
  if request.url.is_empty() {
    return Err(McpValidationError::Validation("url is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let mcp = mcp_service
    .create(
      user_id,
      &request.name,
      &request.slug,
      &request.url,
      request.description,
      request.enabled,
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
    .ok_or_else(|| objs::EntityError::NotFound("MCP".to_string()))?;

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
    .ok_or_else(|| objs::EntityError::NotFound("MCP".to_string()))?;

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

  let exec_request = objs::McpExecutionRequest {
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

// ============================================================================
// MCP Server Admin Handlers
// ============================================================================

/// List MCP server allowlist entries, optionally filtered by URL
#[utoipa::path(
  get,
  path = ENDPOINT_MCP_SERVERS,
  tag = API_TAG_MCPS,
  operation_id = "listMcpServers",
  params(
    ("url" = Option<String>, Query, description = "Filter by exact URL match")
  ),
  responses(
    (status = 200, description = "List of MCP servers", body = ListMcpServersResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_mcp_servers_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<McpServerUrlQuery>,
) -> Result<Json<ListMcpServersResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();

  let mcp_servers = if let Some(url) = query.url {
    match mcp_service.get_mcp_server_by_url(&url).await? {
      Some(server) => vec![server],
      None => vec![],
    }
  } else {
    mcp_service.list_mcp_servers().await?
  };

  Ok(Json(ListMcpServersResponse { mcp_servers }))
}

/// Enable an MCP server URL in the allowlist
#[utoipa::path(
  put,
  path = ENDPOINT_MCP_SERVERS,
  tag = API_TAG_MCPS,
  operation_id = "enableMcpServer",
  request_body = EnableMcpServerRequest,
  responses(
    (status = 200, description = "MCP server enabled", body = objs::McpServer),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn enable_mcp_server_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<EnableMcpServerRequest>,
) -> Result<Json<objs::McpServer>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.url.is_empty() {
    return Err(McpValidationError::Validation("url is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let server = mcp_service
    .set_mcp_server_enabled(&request.url, request.enabled, user_id)
    .await?;

  Ok(Json(server))
}

/// Disable an MCP server URL in the allowlist
#[utoipa::path(
  delete,
  path = ENDPOINT_MCP_SERVERS,
  tag = API_TAG_MCPS,
  operation_id = "disableMcpServer",
  request_body = EnableMcpServerRequest,
  responses(
    (status = 200, description = "MCP server disabled", body = objs::McpServer),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn disable_mcp_server_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<EnableMcpServerRequest>,
) -> Result<Json<objs::McpServer>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.url.is_empty() {
    return Err(McpValidationError::Validation("url is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let server = mcp_service
    .set_mcp_server_enabled(&request.url, false, user_id)
    .await?;

  Ok(Json(server))
}
