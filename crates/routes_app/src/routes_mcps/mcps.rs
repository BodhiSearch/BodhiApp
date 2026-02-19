use crate::{
  AuthHeaderResponse, CreateAuthHeaderRequest, CreateMcpRequest, CreateMcpServerRequest,
  CreateOAuthConfigRequest, FetchMcpToolsRequest, ListMcpServersResponse, ListMcpsResponse,
  McpAuth, McpExecuteRequest, McpExecuteResponse, McpResponse, McpServerQuery, McpServerResponse,
  McpToolsResponse, McpValidationError, OAuthConfigResponse, OAuthConfigsListResponse,
  OAuthDiscoverRequest, OAuthDiscoverResponse, OAuthLoginRequest, OAuthLoginResponse,
  OAuthTokenExchangeRequest, OAuthTokenResponse, UpdateAuthHeaderRequest, UpdateMcpRequest,
  UpdateMcpServerRequest, ENDPOINT_MCPS, ENDPOINT_MCPS_AUTH_HEADERS, ENDPOINT_MCPS_FETCH_TOOLS,
  ENDPOINT_MCPS_OAUTH_DISCOVER, ENDPOINT_MCP_SERVERS,
};
use auth_middleware::{generate_random_string, AuthContext};
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Extension, Json,
};
use base64::{engine::general_purpose, Engine};
use objs::{ApiError, ApprovedResources, API_TAG_MCPS};
use server_core::RouterState;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

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
// MCP Auth Header CRUD Handlers
// ============================================================================

/// Create a new auth header config
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_AUTH_HEADERS,
  tag = API_TAG_MCPS,
  operation_id = "createMcpAuthHeader",
  request_body = CreateAuthHeaderRequest,
  responses(
    (status = 201, description = "Auth header config created", body = AuthHeaderResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn create_auth_header_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<CreateAuthHeaderRequest>,
) -> Result<(StatusCode, Json<AuthHeaderResponse>), ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.header_key.is_empty() {
    return Err(McpValidationError::Validation("header_key is required".to_string()).into());
  }
  if request.header_value.is_empty() {
    return Err(McpValidationError::Validation("header_value is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let auth_header = mcp_service
    .create_auth_header(user_id, &request.header_key, &request.header_value)
    .await?;

  Ok((
    StatusCode::CREATED,
    Json(AuthHeaderResponse::from(auth_header)),
  ))
}

/// Get an auth header config by ID
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS_AUTH_HEADERS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcpAuthHeader",
  params(
    ("id" = String, Path, description = "Auth header config UUID")
  ),
  responses(
    (status = 200, description = "Auth header config", body = AuthHeaderResponse),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_auth_header_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<AuthHeaderResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let auth_header = mcp_service
    .get_auth_header(user_id, &id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("Auth header config".to_string()))?;

  Ok(Json(AuthHeaderResponse::from(auth_header)))
}

/// Update an auth header config
#[utoipa::path(
  put,
  path = ENDPOINT_MCPS_AUTH_HEADERS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "updateMcpAuthHeader",
  params(
    ("id" = String, Path, description = "Auth header config UUID")
  ),
  request_body = UpdateAuthHeaderRequest,
  responses(
    (status = 200, description = "Auth header config updated", body = AuthHeaderResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn update_auth_header_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  Json(request): Json<UpdateAuthHeaderRequest>,
) -> Result<Json<AuthHeaderResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.header_key.is_empty() {
    return Err(McpValidationError::Validation("header_key is required".to_string()).into());
  }
  if request.header_value.is_empty() {
    return Err(McpValidationError::Validation("header_value is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let auth_header = mcp_service
    .update_auth_header(user_id, &id, &request.header_key, &request.header_value)
    .await?;

  Ok(Json(AuthHeaderResponse::from(auth_header)))
}

/// Delete an auth header config
#[utoipa::path(
  delete,
  path = ENDPOINT_MCPS_AUTH_HEADERS.to_owned() + "/{id}",
  tag = API_TAG_MCPS,
  operation_id = "deleteMcpAuthHeader",
  params(
    ("id" = String, Path, description = "Auth header config UUID")
  ),
  responses(
    (status = 204, description = "Auth header config deleted"),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_auth_header_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  mcp_service.delete_auth_header(user_id, &id).await?;

  Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// MCP OAuth Config CRUD Handlers
// ============================================================================

/// Create a new OAuth config for an MCP server
#[utoipa::path(
  post,
  path = "/bodhi/v1/mcp-servers/{server_id}/oauth-configs",
  tag = API_TAG_MCPS,
  operation_id = "createMcpOAuthConfig",
  params(
    ("server_id" = String, Path, description = "MCP server UUID")
  ),
  request_body = CreateOAuthConfigRequest,
  responses(
    (status = 201, description = "OAuth config created", body = OAuthConfigResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn create_oauth_config_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(server_id): Path<String>,
  Json(request): Json<CreateOAuthConfigRequest>,
) -> Result<(StatusCode, Json<OAuthConfigResponse>), ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");

  if request.client_id.is_empty() {
    return Err(McpValidationError::Validation("client_id is required".to_string()).into());
  }
  if request.client_secret.is_empty() {
    return Err(McpValidationError::Validation("client_secret is required".to_string()).into());
  }
  if request.authorization_endpoint.is_empty() {
    return Err(
      McpValidationError::Validation("authorization_endpoint is required".to_string()).into(),
    );
  }
  if request.token_endpoint.is_empty() {
    return Err(McpValidationError::Validation("token_endpoint is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let config = mcp_service
    .create_oauth_config(
      user_id,
      &server_id,
      &request.client_id,
      &request.client_secret,
      &request.authorization_endpoint,
      &request.token_endpoint,
      request.scopes,
    )
    .await?;

  Ok((StatusCode::CREATED, Json(OAuthConfigResponse::from(config))))
}

/// Get an OAuth config by ID
#[utoipa::path(
  get,
  path = "/bodhi/v1/mcp-servers/{server_id}/oauth-configs/{config_id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcpOAuthConfig",
  params(
    ("server_id" = String, Path, description = "MCP server UUID"),
    ("config_id" = String, Path, description = "OAuth config UUID")
  ),
  responses(
    (status = 200, description = "OAuth config", body = OAuthConfigResponse),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_oauth_config_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path((_server_id, config_id)): Path<(String, String)>,
) -> Result<Json<OAuthConfigResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();

  let config = mcp_service
    .get_oauth_config(&config_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("OAuth config".to_string()))?;

  Ok(Json(OAuthConfigResponse::from(config)))
}

// ============================================================================
// OAuth Flow Handlers
// ============================================================================

/// Initiate OAuth login for a config
#[utoipa::path(
  post,
  path = "/bodhi/v1/mcp-servers/{server_id}/oauth-configs/{config_id}/login",
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthLogin",
  params(
    ("server_id" = String, Path, description = "MCP server UUID"),
    ("config_id" = String, Path, description = "OAuth config UUID")
  ),
  request_body = OAuthLoginRequest,
  responses(
    (status = 200, description = "Authorization URL", body = OAuthLoginResponse),
    (status = 404, description = "OAuth config not found"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_login_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Path((_server_id, config_id)): Path<(String, String)>,
  Json(request): Json<OAuthLoginRequest>,
) -> Result<Json<OAuthLoginResponse>, ApiError> {
  let mcp_service = state.app_service().mcp_service();

  let config = mcp_service
    .get_oauth_config(&config_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("OAuth config".to_string()))?;

  let code_verifier = generate_random_string(43);
  let code_challenge =
    general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
  let oauth_state = uuid::Uuid::new_v4().to_string();

  let session_key = format!("mcp_oauth_{}", config_id);
  session
    .insert(
      &session_key,
      serde_json::json!({
        "code_verifier": code_verifier,
        "state": oauth_state,
      }),
    )
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?;

  let mut auth_url = url::Url::parse(&config.authorization_endpoint).map_err(|e| {
    McpValidationError::Validation(format!("invalid authorization endpoint: {}", e))
  })?;
  auth_url
    .query_pairs_mut()
    .append_pair("response_type", "code")
    .append_pair("client_id", &config.client_id)
    .append_pair("redirect_uri", &request.redirect_uri)
    .append_pair("code_challenge", &code_challenge)
    .append_pair("code_challenge_method", "S256")
    .append_pair("state", &oauth_state);
  if let Some(scopes) = &config.scopes {
    auth_url.query_pairs_mut().append_pair("scope", scopes);
  }

  Ok(Json(OAuthLoginResponse {
    authorization_url: auth_url.to_string(),
  }))
}

/// Exchange authorization code for tokens
#[utoipa::path(
  post,
  path = "/bodhi/v1/mcp-servers/{server_id}/oauth-configs/{config_id}/token",
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthTokenExchange",
  params(
    ("server_id" = String, Path, description = "MCP server UUID"),
    ("config_id" = String, Path, description = "OAuth config UUID")
  ),
  request_body = OAuthTokenExchangeRequest,
  responses(
    (status = 200, description = "Token stored", body = OAuthTokenResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "OAuth config not found"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_token_exchange_handler(
  Extension(auth_context): Extension<AuthContext>,
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  Path((_server_id, config_id)): Path<(String, String)>,
  Json(request): Json<OAuthTokenExchangeRequest>,
) -> Result<Json<OAuthTokenResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();

  let session_key = format!("mcp_oauth_{}", config_id);
  let session_data: serde_json::Value = session
    .get(&session_key)
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?
    .ok_or_else(|| {
      McpValidationError::Validation(
        "OAuth session data not found. Initiate login first.".to_string(),
      )
    })?;

  let code_verifier = session_data["code_verifier"]
    .as_str()
    .ok_or_else(|| {
      McpValidationError::Validation("code_verifier not found in session".to_string())
    })?
    .to_string();

  let expected_state = session_data["state"]
    .as_str()
    .ok_or_else(|| McpValidationError::Validation("state not found in session".to_string()))?
    .to_string();

  if request.state != expected_state {
    return Err(
      McpValidationError::Validation("OAuth state mismatch (CSRF protection)".to_string()).into(),
    );
  }

  let _ = session.remove::<serde_json::Value>(&session_key).await;

  let db_service = state.app_service().db_service();
  let (client_id, client_secret) = db_service
    .get_decrypted_client_secret(&config_id)
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?
    .ok_or_else(|| objs::EntityError::NotFound("OAuth config".to_string()))?;

  let config = mcp_service
    .get_oauth_config(&config_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("OAuth config".to_string()))?;

  let http_client = reqwest::Client::new();
  let params = &[
    ("grant_type", "authorization_code"),
    ("code", &request.code),
    ("redirect_uri", &request.redirect_uri),
    ("client_id", &client_id),
    ("client_secret", &client_secret),
    ("code_verifier", &code_verifier),
  ];
  let resp = http_client
    .post(&config.token_endpoint)
    .header("Accept", "application/json")
    .form(params)
    .send()
    .await
    .map_err(|e| McpValidationError::Validation(format!("Token exchange failed: {}", e)))?;

  let status = resp.status();
  if !status.is_success() {
    let body = resp.text().await.unwrap_or_default();
    return Err(
      McpValidationError::Validation(format!("Token exchange failed (HTTP {}): {}", status, body))
        .into(),
    );
  }
  let body = resp.text().await.unwrap_or_default();
  let token_resp: serde_json::Value = serde_json::from_str(&body)
    .map_err(|e| McpValidationError::Validation(format!("Invalid token response: {}", e)))?;

  let access_token = token_resp["access_token"]
    .as_str()
    .ok_or_else(|| {
      McpValidationError::Validation("missing access_token in token response".to_string())
    })?
    .to_string();

  let refresh_token = token_resp["refresh_token"].as_str().map(|s| s.to_string());
  let expires_in = token_resp["expires_in"].as_i64();
  let scopes_granted = token_resp["scope"].as_str().map(|s| s.to_string());

  let token = mcp_service
    .store_oauth_token(
      user_id,
      &config_id,
      &access_token,
      refresh_token,
      scopes_granted,
      expires_in,
    )
    .await?;

  Ok(Json(OAuthTokenResponse::from(token)))
}

// ============================================================================
// OAuth Discovery Handler
// ============================================================================

/// Discover OAuth metadata from a server URL
#[utoipa::path(
  post,
  path = ENDPOINT_MCPS_OAUTH_DISCOVER,
  tag = API_TAG_MCPS,
  operation_id = "mcpOAuthDiscover",
  request_body = OAuthDiscoverRequest,
  responses(
    (status = 200, description = "OAuth discovery metadata", body = OAuthDiscoverResponse),
    (status = 400, description = "Validation error"),
  ),
  security(("bearer" = []))
)]
pub async fn oauth_discover_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<OAuthDiscoverRequest>,
) -> Result<Json<OAuthDiscoverResponse>, ApiError> {
  if request.url.is_empty() {
    return Err(McpValidationError::Validation("url is required".to_string()).into());
  }

  let mcp_service = state.app_service().mcp_service();

  let metadata = mcp_service.discover_oauth_metadata(&request.url).await?;

  let authorization_endpoint = metadata["authorization_endpoint"]
    .as_str()
    .unwrap_or_default()
    .to_string();
  let token_endpoint = metadata["token_endpoint"]
    .as_str()
    .unwrap_or_default()
    .to_string();
  let scopes_supported = metadata["scopes_supported"].as_array().map(|arr| {
    arr
      .iter()
      .filter_map(|v| v.as_str().map(String::from))
      .collect()
  });

  Ok(Json(OAuthDiscoverResponse {
    authorization_endpoint,
    token_endpoint,
    scopes_supported,
  }))
}

// ============================================================================
// OAuth Config List Handler
// ============================================================================

/// List OAuth configs for an MCP server
#[utoipa::path(
  get,
  path = "/bodhi/v1/mcp-servers/{server_id}/oauth-configs",
  tag = API_TAG_MCPS,
  operation_id = "listMcpOAuthConfigs",
  params(
    ("server_id" = String, Path, description = "MCP server UUID")
  ),
  responses(
    (status = 200, description = "List of OAuth configs", body = OAuthConfigsListResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_oauth_configs_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(server_id): Path<String>,
) -> Result<Json<OAuthConfigsListResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();
  let configs = mcp_service
    .list_oauth_configs_by_server(&server_id, user_id)
    .await?;
  let oauth_configs: Vec<OAuthConfigResponse> =
    configs.into_iter().map(OAuthConfigResponse::from).collect();
  Ok(Json(OAuthConfigsListResponse { oauth_configs }))
}

// ============================================================================
// OAuth Token Handlers
// ============================================================================

/// Get an OAuth token by ID
#[utoipa::path(
  get,
  path = "/bodhi/v1/mcps/oauth-tokens/{token_id}",
  tag = API_TAG_MCPS,
  operation_id = "getMcpOAuthToken",
  params(
    ("token_id" = String, Path, description = "OAuth token UUID")
  ),
  responses(
    (status = 200, description = "OAuth token", body = OAuthTokenResponse),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn get_oauth_token_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(token_id): Path<String>,
) -> Result<Json<OAuthTokenResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let mcp_service = state.app_service().mcp_service();
  let token = mcp_service
    .get_oauth_token(user_id, &token_id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("OAuth token".to_string()))?;
  Ok(Json(OAuthTokenResponse::from(token)))
}

/// Delete an OAuth token by ID
#[utoipa::path(
  delete,
  path = "/bodhi/v1/mcps/oauth-tokens/{token_id}",
  tag = API_TAG_MCPS,
  operation_id = "deleteMcpOAuthToken",
  params(
    ("token_id" = String, Path, description = "OAuth token UUID")
  ),
  responses(
    (status = 204, description = "OAuth token deleted"),
    (status = 404, description = "Not found"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_oauth_token_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(token_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let db_service = state.app_service().db_service();
  db_service
    .delete_mcp_oauth_token(user_id, &token_id)
    .await
    .map_err(|e| McpValidationError::Validation(e.to_string()))?;
  Ok(StatusCode::NO_CONTENT)
}
