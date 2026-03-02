use crate::mcps::{
  CreateMcpServerRequest, ListMcpServersResponse, McpServerQuery, McpServerResponse,
  UpdateMcpServerRequest, ENDPOINT_MCP_SERVERS,
};
use crate::{ApiError, AuthScope, API_TAG_MCPS};
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  Json,
};

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
pub async fn mcp_servers_create(
  auth_scope: AuthScope,
  Json(request): Json<CreateMcpServerRequest>,
) -> Result<(StatusCode, Json<McpServerResponse>), ApiError> {
  let mcps = auth_scope.mcps();

  let server = mcps
    .create_mcp_server(
      &request.name,
      &request.url,
      request.description,
      request.enabled,
    )
    .await?;

  let auth_config = if let Some(config_request) = request.auth_config {
    Some(
      mcps
        .create_auth_config(&server.id, config_request)
        .await?,
    )
  } else {
    None
  };

  let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server.id).await?;

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
      auth_config,
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
pub async fn mcp_servers_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  Json(request): Json<UpdateMcpServerRequest>,
) -> Result<Json<McpServerResponse>, ApiError> {
  let mcps = auth_scope.mcps();

  let server = mcps
    .update_mcp_server(
      &id,
      &request.name,
      &request.url,
      request.description,
      request.enabled,
    )
    .await?;

  let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server.id).await?;

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
    auth_config: None,
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
pub async fn mcp_servers_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<McpServerResponse>, ApiError> {
  let mcps = auth_scope.mcps();

  let server = mcps
    .get_mcp_server(&id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP server".to_string()))?;

  let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server.id).await?;

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
    auth_config: None,
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
pub async fn mcp_servers_index(
  auth_scope: AuthScope,
  Query(query): Query<McpServerQuery>,
) -> Result<Json<ListMcpServersResponse>, ApiError> {
  let mcps = auth_scope.mcps();

  let servers = mcps.list_mcp_servers(query.enabled).await?;

  let mut responses = Vec::with_capacity(servers.len());
  for server in servers {
    let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server.id).await?;
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
      auth_config: None,
    });
  }

  Ok(Json(ListMcpServersResponse {
    mcp_servers: responses,
  }))
}

#[cfg(test)]
#[path = "test_servers.rs"]
mod test_servers;
