use crate::{
  CreateMcpRequest, CreateMcpServerRequest, ListMcpServersResponse, ListMcpsResponse,
  McpExecuteRequest, McpExecuteResponse, McpResponse, McpServerQuery, McpServerResponse,
  McpToolsResponse, McpValidationError, UpdateMcpRequest, UpdateMcpServerRequest, ENDPOINT_MCPS,
  ENDPOINT_MCP_SERVERS,
};
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Extension, Json,
};
use objs::{ApiError, ApprovedResources, API_TAG_MCPS};
use server_core::RouterState;
use std::sync::Arc;

// ============================================================================
// MCP Server Admin Handlers
// ============================================================================

/// Create a new MCP server entry (admin/manager only)
#[utoipa::path(
  post,
  path = ENDPOINT_MCP_SERVERS,
  tag = API_TAG_MCPS,
  operation_id = "createMcpServer",
  request_body = CreateMcpServerRequest,
  responses(
    (status = 201, description = "MCP server created", body = McpServerResponse),
    (status = 400, description = "Validation error"),
    (status = 409, description = "URL already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn create_mcp_server_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<CreateMcpServerRequest>,
) -> Result<(StatusCode, Json<McpServerResponse>), ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let server = mcp_service
    .create_mcp_server(
      &request.name,
      &request.url,
      request.description,
      request.enabled,
      user_id,
    )
    .await?;

  let (enabled_count, disabled_count) = mcp_service.count_mcps_for_server(&server.id).await?;

  Ok((
    StatusCode::CREATED,
    Json(McpServerResponse {
      id: server.id,
      url: server.url,
      name: server.name,
      description: server.description,
      enabled: server.enabled,
      created_by: server.created_by,
      updated_by: server.updated_by,
      enabled_mcp_count: enabled_count,
      disabled_mcp_count: disabled_count,
      created_at: server.created_at.to_rfc3339(),
      updated_at: server.updated_at.to_rfc3339(),
    }),
  ))
}

/// Update an existing MCP server entry (admin/manager only)
#[utoipa::path(
  put,
  path = ENDPOINT_MCP_SERVERS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "updateMcpServer",
  params(
    ("id" = String, Path, description = "MCP server UUID")
  ),
  request_body = UpdateMcpServerRequest,
  responses(
    (status = 200, description = "MCP server updated", body = McpServerResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "Not found"),
    (status = 409, description = "URL already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn update_mcp_server_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  Json(request): Json<UpdateMcpServerRequest>,
) -> Result<Json<McpServerResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let server = mcp_service
    .update_mcp_server(
      &id,
      &request.name,
      &request.url,
      request.description,
      request.enabled,
      user_id,
    )
    .await?;

  let (enabled_count, disabled_count) = mcp_service.count_mcps_for_server(&server.id).await?;

  Ok(Json(McpServerResponse {
    id: server.id,
    url: server.url,
    name: server.name,
    description: server.description,
    enabled: server.enabled,
    created_by: server.created_by,
    updated_by: server.updated_by,
    enabled_mcp_count: enabled_count,
    disabled_mcp_count: disabled_count,
    created_at: server.created_at.to_rfc3339(),
    updated_at: server.updated_at.to_rfc3339(),
  }))
}

/// Get a specific MCP server by ID
#[utoipa::path(
  get,
  path = ENDPOINT_MCP_SERVERS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcpServer",
  params(
    ("id" = String, Path, description = "MCP server UUID")
  ),
  responses(
    (status = 200, description = "MCP server", body = McpServerResponse),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_mcp_server_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<McpServerResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();

  let server = mcp_service
    .get_mcp_server(&id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("MCP server".to_string()))?;

  let (enabled_count, disabled_count) = mcp_service.count_mcps_for_server(&server.id).await?;

  Ok(Json(McpServerResponse {
    id: server.id,
    url: server.url,
    name: server.name,
    description: server.description,
    enabled: server.enabled,
    created_by: server.created_by,
    updated_by: server.updated_by,
    enabled_mcp_count: enabled_count,
    disabled_mcp_count: disabled_count,
    created_at: server.created_at.to_rfc3339(),
    updated_at: server.updated_at.to_rfc3339(),
  }))
}

/// List MCP servers, optionally filtered by enabled status
#[utoipa::path(
  get,
  path = ENDPOINT_MCP_SERVERS,
  tag = API_TAG_MCPS,
  operation_id = "listMcpServers",
  params(
    ("enabled" = Option<bool>, Query, description = "Filter by enabled status")
  ),
  responses(
    (status = 200, description = "List of MCP servers", body = ListMcpServersResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_mcp_servers_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<McpServerQuery>,
) -> Result<Json<ListMcpServersResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();

  let servers = mcp_service.list_mcp_servers(query.enabled).await?;

  let mut responses = Vec::with_capacity(servers.len());
  for server in servers {
    let (enabled_count, disabled_count) = mcp_service.count_mcps_for_server(&server.id).await?;
    responses.push(McpServerResponse {
      id: server.id,
      url: server.url,
      name: server.name,
      description: server.description,
      enabled: server.enabled,
      created_by: server.created_by,
      updated_by: server.updated_by,
      enabled_mcp_count: enabled_count,
      disabled_mcp_count: disabled_count,
      created_at: server.created_at.to_rfc3339(),
      updated_at: server.updated_at.to_rfc3339(),
    });
  }

  Ok(Json(ListMcpServersResponse {
    mcp_servers: responses,
  }))
}

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
    .filter(|a| a.status == "approved")
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
      request.tools_filter,
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
