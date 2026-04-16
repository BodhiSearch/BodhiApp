use crate::mcps::{McpServerQuery, ENDPOINT_MCP_SERVERS};
use crate::{AuthScope, BodhiErrorResponse, ValidatedJson, API_TAG_MCPS};
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  Json,
};
use services::{McpServer, McpServerRequest};

// ============================================================================
// MCP Server Response types
// ============================================================================

/// MCP server response with computed mcp counts and optional auth config.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct McpServerResponse {
  #[serde(flatten)]
  pub server: McpServer,
  pub enabled_mcp_count: i64,
  pub disabled_mcp_count: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_config: Option<services::McpAuthConfigResponse>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ListMcpServersResponse {
  pub mcp_servers: Vec<McpServerResponse>,
}

// ============================================================================
// MCP Server Admin Handlers
// ============================================================================

/// Create a new MCP server entry (admin/manager only)
#[utoipa::path(
  post,
  path = ENDPOINT_MCP_SERVERS,
  tag = API_TAG_MCPS,
  operation_id = "createMcpServer",
  request_body = McpServerRequest,
  responses(
    (status = 201, description = "MCP server created", body = McpServerResponse),
    (status = 400, description = "Validation error"),
    (status = 409, description = "URL already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn mcp_servers_create(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<McpServerRequest>,
) -> Result<(StatusCode, Json<McpServerResponse>), BodhiErrorResponse> {
  let mcps = auth_scope.mcps();

  let auth_config_request = request.auth_config.clone();
  let entity = mcps.create_mcp_server(request).await?;
  let server_id = entity.id.clone();
  let server: McpServer = entity.into();

  let auth_config = if let Some(config_request) = auth_config_request {
    Some(mcps.create_auth_config(&server_id, config_request).await?)
  } else {
    None
  };

  let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server_id).await?;

  Ok((
    StatusCode::CREATED,
    Json(McpServerResponse {
      server,
      enabled_mcp_count: enabled_count,
      disabled_mcp_count: disabled_count,
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
  request_body = McpServerRequest,
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
  ValidatedJson(request): ValidatedJson<McpServerRequest>,
) -> Result<Json<McpServerResponse>, BodhiErrorResponse> {
  let mcps = auth_scope.mcps();

  let entity = mcps.update_mcp_server(&id, request).await?;
  let server_id = entity.id.clone();
  let server: McpServer = entity.into();

  let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server_id).await?;

  Ok(Json(McpServerResponse {
    server,
    enabled_mcp_count: enabled_count,
    disabled_mcp_count: disabled_count,
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
) -> Result<Json<McpServerResponse>, BodhiErrorResponse> {
  let mcps = auth_scope.mcps();

  let entity = mcps
    .get_mcp_server(&id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP server".to_string()))?;
  let server_id = entity.id.clone();
  let server: McpServer = entity.into();

  let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server_id).await?;

  Ok(Json(McpServerResponse {
    server,
    enabled_mcp_count: enabled_count,
    disabled_mcp_count: disabled_count,
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
) -> Result<Json<ListMcpServersResponse>, BodhiErrorResponse> {
  let mcps = auth_scope.mcps();

  let entities = mcps.list_mcp_servers(query.enabled).await?;

  let mut responses = Vec::with_capacity(entities.len());
  for entity in entities {
    let server_id = entity.id.clone();
    let server: McpServer = entity.into();
    let (enabled_count, disabled_count) = mcps.count_mcps_for_server(&server_id).await?;
    responses.push(McpServerResponse {
      server,
      enabled_mcp_count: enabled_count,
      disabled_mcp_count: disabled_count,
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
