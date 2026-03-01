use crate::routes_mcp::{
  CreateMcpServerRequest, ListMcpServersResponse, McpServerQuery, McpServerResponse,
  UpdateMcpServerRequest,
};
use crate::API_TAG_MCPS;
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Extension, Json,
};
use server_core::RouterState;
use services::ApiError;
use std::sync::Arc;

use super::ENDPOINT_MCP_SERVERS;

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

  let auth_config = if let Some(config_request) = request.auth_config {
    Some(
      mcp_service
        .create_auth_config(user_id, &server.id, config_request)
        .await?,
    )
  } else {
    None
  };

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
pub async fn get_mcp_server_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<McpServerResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();

  let server = mcp_service
    .get_mcp_server(&id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP server".to_string()))?;

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
