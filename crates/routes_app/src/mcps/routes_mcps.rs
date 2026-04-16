use crate::mcps::{McpRouteError, ENDPOINT_MCPS};
use crate::{
  AuthScope, BodhiErrorResponse, ValidatedJson, API_TAG_APPS, API_TAG_MCPS, ENDPOINT_APPS_MCPS,
};
use axum::{extract::Path, http::StatusCode, Json};
use services::{
  ApprovalStatus, ApprovedResources, AuthContext, Mcp, McpRequest, McpWithServerEntity,
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
pub async fn mcps_index(
  auth_scope: AuthScope,
) -> Result<Json<ListMcpsResponse>, BodhiErrorResponse> {
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
      .map(|json| {
        serde_json::from_str::<ApprovedResources>(&json)
          .map_err(|_| McpRouteError::InvalidApprovedJson)
      })
      .transpose()?
      .map(|res| match res {
        ApprovedResources::V1(v1) => v1
          .mcps
          .into_iter()
          .filter(|a| a.status == ApprovalStatus::Approved)
          .filter_map(|a| a.instance.map(|i| i.id))
          .collect(),
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
) -> Result<(StatusCode, Json<Mcp>), BodhiErrorResponse> {
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
) -> Result<Json<Mcp>, BodhiErrorResponse> {
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
) -> Result<Json<Mcp>, BodhiErrorResponse> {
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
) -> Result<StatusCode, BodhiErrorResponse> {
  auth_scope.mcps().delete(&id).await?;

  Ok(StatusCode::NO_CONTENT)
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
pub async fn apps_mcps_index(
  auth_scope: AuthScope,
) -> Result<Json<ListMcpsResponse>, BodhiErrorResponse> {
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
) -> Result<Json<Mcp>, BodhiErrorResponse> {
  mcps_show(auth_scope, path).await
}

#[cfg(test)]
#[path = "test_mcps.rs"]
mod test_mcps;
