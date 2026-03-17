use crate::mcps::{
  FetchMcpToolsRequest, McpAuth, McpExecuteRequest, McpExecuteResponse, McpRouteError,
  McpToolsResponse, ENDPOINT_MCPS, ENDPOINT_MCPS_FETCH_TOOLS,
};
use crate::{ApiError, AuthScope, ValidatedJson, API_TAG_APPS, API_TAG_MCPS, ENDPOINT_APPS_MCPS};
use axum::{extract::Path, http::StatusCode, Json};
use services::{
  ApprovalStatus, ApprovedResources, AuthContext, Mcp, McpAuthParamInput, McpAuthParamType,
  McpRequest, McpWithServerEntity,
};

// ============================================================================
// MCP Instance Response types
// ============================================================================

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ListMcpsResponse {
  pub mcps: Vec<Mcp>,
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
pub async fn mcps_index(auth_scope: AuthScope) -> Result<Json<ListMcpsResponse>, ApiError> {
  let entities = auth_scope.mcps().list().await?;

  // Filter MCP list for ExternalApp tokens: only return MCPs approved in the access request
  let entities: Vec<McpWithServerEntity> = if let AuthContext::ExternalApp {
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
      .and_then(|json| serde_json::from_str::<ApprovedResources>(&json).ok())
      .map(|res| {
        res
          .mcps
          .into_iter()
          .filter(|a| a.status == ApprovalStatus::Approved)
          .filter_map(|a| a.instance.map(|i| i.id))
          .collect()
      })
      .unwrap_or_default();
    entities
      .into_iter()
      .filter(|m| approved_ids.contains(&m.id))
      .collect()
  } else {
    entities
  };

  let mcps: Vec<Mcp> = entities.into_iter().map(|e| e.into()).collect();
  Ok(Json(ListMcpsResponse { mcps }))
}

/// Create a new MCP instance
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS,
  tag = API_TAG_MCPS,
  operation_id = "createMcp",
  request_body = McpRequest,
  responses(
    (status = 201, description = "MCP created", body = Mcp),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn mcps_create(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<McpRequest>,
) -> Result<(StatusCode, Json<Mcp>), ApiError> {
  if request.mcp_server_id.is_none() || request.mcp_server_id.as_deref() == Some("") {
    return Err(McpRouteError::Validation("mcp_server_id is required".to_string()).into());
  }

  let entity = auth_scope.mcps().create(request).await?;
  let mcp: Mcp = entity.into();

  Ok((StatusCode::CREATED, Json(mcp)))
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
    (status = 200, description = "MCP instance", body = Mcp),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn mcps_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<Mcp>, ApiError> {
  let entity = auth_scope
    .mcps()
    .get(&id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP".to_string()))?;

  Ok(Json(entity.into()))
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
  request_body = McpRequest,
  responses(
    (status = 200, description = "MCP updated", body = Mcp),
    (status = 400, description = "Validation error"),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer" = []))
)]
pub async fn mcps_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(request): ValidatedJson<McpRequest>,
) -> Result<Json<Mcp>, ApiError> {
  let entity = auth_scope.mcps().update(&id, request).await?;

  Ok(Json(entity.into()))
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
pub async fn mcps_destroy(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  auth_scope.mcps().delete(&id).await?;

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
pub async fn mcps_fetch_tools(
  auth_scope: AuthScope,
  Json(request): Json<FetchMcpToolsRequest>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  if request.mcp_server_id.is_empty() {
    return Err(McpRouteError::Validation("mcp_server_id is required".to_string()).into());
  }

  // Prefer new `credentials` field; fall back to legacy `McpAuth::Header` for backward compat
  let credentials = if request.credentials.is_some() {
    request.credentials
  } else {
    match request.auth {
      Some(McpAuth::Header {
        header_key,
        header_value,
      }) => Some(vec![McpAuthParamInput {
        param_type: McpAuthParamType::Header,
        param_key: header_key,
        value: header_value,
      }]),
      _ => None,
    }
  };

  let tools = auth_scope
    .mcps()
    .fetch_tools_for_server(
      &request.mcp_server_id,
      credentials,
      request.auth_config_id,
      request.oauth_token_id,
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
pub async fn mcps_list_tools(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  let entity = auth_scope
    .mcps()
    .get(&id)
    .await?
    .ok_or_else(|| services::EntityError::NotFound("MCP".to_string()))?;

  let mcp: Mcp = entity.into();
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
pub async fn mcps_refresh_tools(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  let tools = auth_scope.mcps().fetch_tools(&id).await?;

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
pub async fn mcps_execute_tool(
  auth_scope: AuthScope,
  Path((id, tool_name)): Path<(String, String)>,
  Json(request): Json<McpExecuteRequest>,
) -> Result<Json<McpExecuteResponse>, ApiError> {
  let exec_request = services::McpExecutionRequest {
    params: request.params,
  };

  let exec_response = auth_scope
    .mcps()
    .execute(&id, &tool_name, exec_request)
    .await?;

  Ok(Json(McpExecuteResponse {
    result: exec_response.result,
    error: exec_response.error,
  }))
}

// ============================================================================
// External App (/apps/) Wrappers
// ============================================================================

/// List MCP instances accessible to the authenticated external app
#[utoipa::path(
  get,
  path = ENDPOINT_APPS_MCPS,
  tag = API_TAG_APPS,
  operation_id = "appsListMcps",
  responses(
    (status = 200, description = "List of MCP instances accessible to the external app", body = ListMcpsResponse),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn apps_mcps_index(auth_scope: AuthScope) -> Result<Json<ListMcpsResponse>, ApiError> {
  mcps_index(auth_scope).await
}

/// Get a specific MCP instance by ID via external app
#[utoipa::path(
  get,
  path = ENDPOINT_APPS_MCPS.to_owned() + "/{id}",
  tag = API_TAG_APPS,
  operation_id = "appsGetMcp",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 200, description = "MCP instance", body = Mcp),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn apps_mcps_show(
  auth_scope: AuthScope,
  path: Path<String>,
) -> Result<Json<Mcp>, ApiError> {
  mcps_show(auth_scope, path).await
}

/// Refresh tools for an MCP instance via external app
#[utoipa::path(
  post,
  path = ENDPOINT_APPS_MCPS.to_owned() + "/{id}/tools/refresh",
  tag = API_TAG_APPS,
  operation_id = "appsRefreshMcpTools",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 200, description = "Refreshed list of tools", body = McpToolsResponse),
    (status = 404, description = "MCP not found"),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn apps_mcps_refresh_tools(
  auth_scope: AuthScope,
  path: Path<String>,
) -> Result<Json<McpToolsResponse>, ApiError> {
  mcps_refresh_tools(auth_scope, path).await
}

/// Execute a tool on an MCP server via external app
#[utoipa::path(
  post,
  path = ENDPOINT_APPS_MCPS.to_owned() + "/{id}/tools/{tool_name}/execute",
  tag = API_TAG_APPS,
  operation_id = "appsExecuteMcpTool",
  params(
    ("id" = String, Path, description = "MCP instance UUID"),
    ("tool_name" = String, Path, description = "Tool name to execute")
  ),
  request_body = McpExecuteRequest,
  responses(
    (status = 200, description = "Tool execution result", body = McpExecuteResponse),
    (status = 404, description = "MCP or tool not found"),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn apps_mcps_execute_tool(
  auth_scope: AuthScope,
  path: Path<(String, String)>,
  json: Json<McpExecuteRequest>,
) -> Result<Json<McpExecuteResponse>, ApiError> {
  mcps_execute_tool(auth_scope, path, json).await
}

#[cfg(test)]
#[path = "test_mcps.rs"]
mod test_mcps;
