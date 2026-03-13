use crate::apps::{
  AccessRequestActionResponse, AccessRequestReviewResponse, AccessRequestStatusResponse,
  AppsRouteError, CreateAccessRequestResponse, ToolTypeReviewInfo,
};
use crate::{ApiError, AuthScope, OpenAIApiError, ValidatedJson, API_TAG_AUTH};
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  response::Json,
};
use serde::Deserialize;
use services::{
  AppAccessRequestStatus, ApprovalStatus, ApproveAccessRequest, CreateAccessRequest, FlowType,
  RequestedMcpServer, RequestedResources, ToolsetTypeRequest,
};
use services::{ResourceRole, UserScope};
use tracing::{debug, info};

pub const ENDPOINT_APPS_REQUEST_ACCESS: &str = "/bodhi/v1/apps/request-access";
pub const ENDPOINT_APPS_ACCESS_REQUESTS_ID: &str = "/bodhi/v1/apps/access-requests/{id}";
pub const ENDPOINT_ACCESS_REQUESTS_REVIEW: &str = "/bodhi/v1/access-requests/{id}/review";
pub const ENDPOINT_ACCESS_REQUESTS_APPROVE: &str = "/bodhi/v1/access-requests/{id}/approve";
pub const ENDPOINT_ACCESS_REQUESTS_DENY: &str = "/bodhi/v1/access-requests/{id}/deny";

// Query params for GET /apps/access-requests/:id (polling by apps)
#[derive(Debug, Deserialize)]
pub struct AccessRequestStatusQuery {
  pub app_client_id: String,
}

/// Create access request (POST /apps/request-access)
#[utoipa::path(
    post,
    path = ENDPOINT_APPS_REQUEST_ACCESS,
    tag = API_TAG_AUTH,
    operation_id = "createAccessRequest",
    summary = "Create Access Request",
    description = "Create an access request for an app to access user resources. Always creates a draft for user review. Unauthenticated endpoint.",
    request_body(
        content = CreateAccessRequest,
        description = "Access request details"
    ),
    responses(
        (status = 201, description = "Access request created", body = CreateAccessRequestResponse),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
        (status = 404, description = "App client not found", body = OpenAIApiError),
    ),
    security(())
)]
pub async fn apps_create_access_request(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<CreateAccessRequest>,
) -> Result<(StatusCode, Json<CreateAccessRequestResponse>), ApiError> {
  debug!(
    "Creating access request for app_client_id: {}",
    request.app_client_id
  );

  // Validate redirect_url for redirect flow
  if request.flow_type == FlowType::Redirect && request.redirect_url.is_none() {
    return Err(AppsRouteError::MissingRedirectUrl)?;
  }

  // Note: We skip fetching app client info here because:
  // 1. This endpoint is unauthenticated (no user token available)
  // 2. KC endpoint for app client info may not be implemented yet
  // 3. App info will be fetched during review (when user is authenticated)
  debug!(
    "Creating access request for app_client_id: {} (app info will be fetched during review)",
    request.app_client_id
  );

  let tool_types: Vec<ToolsetTypeRequest> = request
    .requested
    .as_ref()
    .map(|r| r.toolset_types.clone())
    .unwrap_or_default();

  let mcp_servers: Vec<RequestedMcpServer> = request
    .requested
    .as_ref()
    .map(|r| r.mcp_servers.clone())
    .unwrap_or_default();

  let tools = auth_scope.tools();
  for tool_type_req in &tool_types {
    tools.validate_type(&tool_type_req.toolset_type)?;
  }

  let access_request_service = auth_scope.access_request_service();
  let created = access_request_service
    .create_draft(
      request.app_client_id,
      request.flow_type,
      request.redirect_url,
      tool_types,
      mcp_servers,
      request.requested_role,
    )
    .await?;

  let review_url = access_request_service.build_review_url(&created.id);
  info!(
    "Access request {} created with review_url: {}",
    created.id, review_url
  );
  Ok((
    StatusCode::CREATED,
    Json(CreateAccessRequestResponse {
      id: created.id,
      status: AppAccessRequestStatus::Draft,
      review_url,
    }),
  ))
}

/// Get access request status (GET /apps/access-requests/:id)
#[utoipa::path(
    get,
    path = ENDPOINT_APPS_ACCESS_REQUESTS_ID,
    tag = API_TAG_AUTH,
    operation_id = "getAccessRequestStatus",
    summary = "Get Access Request Status",
    description = "Poll access request status. Requires app_client_id query parameter for security.",
    params(
        ("id" = String, Path, description = "Access request ID"),
        ("app_client_id" = String, Query, description = "App client ID for verification")
    ),
    responses(
        (status = 200, description = "Status retrieved", body = AccessRequestStatusResponse),
        (status = 404, description = "Not found or app_client_id mismatch", body = OpenAIApiError),
    ),
    security(())
)]
pub async fn apps_get_access_request_status(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  Query(query): Query<AccessRequestStatusQuery>,
) -> Result<Json<AccessRequestStatusResponse>, ApiError> {
  debug!("Getting access request status for id: {}", id);

  let access_request_service = auth_scope.access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppsRouteError::NotFound)?;

  // Verify app_client_id matches
  if request.app_client_id != query.app_client_id {
    return Err(AppsRouteError::NotFound)?;
  }

  let requested_role: UserScope = request.requested_role.parse()?;
  let approved_role: Option<UserScope> = request.approved_role.map(|r| r.parse()).transpose()?;

  Ok(Json(AccessRequestStatusResponse {
    id: request.id,
    status: request.status,
    requested_role,
    approved_role,
    access_request_scope: request.access_request_scope,
  }))
}

/// Get access request review data (GET /access-requests/:id/review)
#[utoipa::path(
    get,
    path = ENDPOINT_ACCESS_REQUESTS_REVIEW,
    tag = API_TAG_AUTH,
    operation_id = "getAccessRequestReview",
    summary = "Get Access Request Review",
    description = "Get full access request details for review page. Returns data regardless of status. Requires session auth.",
    params(
        ("id" = String, Path, description = "Access request ID")
    ),
    responses(
        (status = 200, description = "Review data retrieved", body = AccessRequestReviewResponse),
        (status = 404, description = "Not found", body = OpenAIApiError),
        (status = 410, description = "Request expired", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_get_access_request_review(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<AccessRequestReviewResponse>, ApiError> {
  let access_request_service = auth_scope.access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppsRouteError::NotFound)?;

  let requested: RequestedResources = serde_json::from_str(&request.requested).unwrap_or_default();

  let tools_svc = auth_scope.tools();
  let mcps_svc = auth_scope.mcps();
  let all_user_toolsets = tools_svc.list().await?;
  let all_user_mcp_entities = mcps_svc.list().await?;
  let all_user_mcps: Vec<services::Mcp> = all_user_mcp_entities
    .into_iter()
    .map(|e| e.into())
    .collect();

  let mut tools_info = Vec::new();

  for tool_type_req in &requested.toolset_types {
    let tool_def = tools_svc
      .get_type(&tool_type_req.toolset_type)
      .ok_or_else(|| AppsRouteError::InvalidToolType(tool_type_req.toolset_type.clone()))?;

    let instances: Vec<services::Toolset> = all_user_toolsets
      .iter()
      .filter(|t| t.toolset_type == tool_type_req.toolset_type)
      .cloned()
      .map(|e| e.into())
      .collect();

    tools_info.push(ToolTypeReviewInfo {
      toolset_type: tool_type_req.toolset_type.clone(),
      name: tool_def.name.clone(),
      description: tool_def.description.clone(),
      instances,
    });
  }

  let mut mcps_info = Vec::new();

  for mcp_server_req in &requested.mcp_servers {
    let instances = all_user_mcps
      .iter()
      .filter(|m| m.mcp_server.url == mcp_server_req.url)
      .cloned()
      .collect();

    mcps_info.push(crate::apps::McpServerReviewInfo {
      url: mcp_server_req.url.clone(),
      instances,
    });
  }

  Ok(Json(AccessRequestReviewResponse {
    id: request.id,
    app_client_id: request.app_client_id,
    app_name: request.app_name,
    app_description: request.app_description,
    flow_type: request.flow_type,
    status: request.status,
    requested_role: request.requested_role,
    requested,
    tools_info,
    mcps_info,
  }))
}

/// Approve access request (PUT /access-requests/:id/approve)
#[utoipa::path(
    put,
    path = ENDPOINT_ACCESS_REQUESTS_APPROVE,
    tag = API_TAG_AUTH,
    operation_id = "approveAppsAccessRequest",
    summary = "Approve Access Request",
    description = "Approve access request with tool instance selections. Requires session auth.",
    params(
        ("id" = String, Path, description = "Access request ID")
    ),
    request_body(
        content = ApproveAccessRequest,
        description = "Approval details with tool selections"
    ),
    responses(
        (status = 200, description = "Request approved", body = AccessRequestActionResponse),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
        (status = 404, description = "Not found", body = OpenAIApiError),
        (status = 409, description = "Already processed", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_approve_access_request(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(approval_input): ValidatedJson<ApproveAccessRequest>,
) -> Result<Json<AccessRequestActionResponse>, ApiError> {
  let user_id = auth_scope.require_user_id()?;
  let token = auth_scope
    .auth_context()
    .token()
    .ok_or(AppsRouteError::InsufficientPrivileges)?;
  let tenant_id = auth_scope.require_tenant_id()?;

  // Extract approver's role from session and compute max grantable scope
  let approver_role = match auth_scope.auth_context() {
    services::AuthContext::Session { role, .. } => {
      role.ok_or(AppsRouteError::InsufficientPrivileges)?
    }
    services::AuthContext::MultiTenantSession { role, .. } => {
      role.ok_or(AppsRouteError::InsufficientPrivileges)?
    }
    _ => return Err(AppsRouteError::InsufficientPrivileges)?,
  };
  let max_grantable = if approver_role >= ResourceRole::PowerUser {
    UserScope::PowerUser
  } else {
    UserScope::User
  };

  let approved_scope = approval_input.approved_role;

  // Fetch the access request to get requested_role for privilege escalation check
  let access_request_service = auth_scope.access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppsRouteError::NotFound)?;

  let requested_scope: UserScope = request.requested_role.parse()?;

  // Validate: approved can't exceed what was requested
  if approved_scope > requested_scope {
    return Err(AppsRouteError::PrivilegeEscalation {
      approved: approved_scope.to_string(),
      max_allowed: requested_scope.to_string(),
    })?;
  }
  // Validate: approved can't exceed what the approver is allowed to grant
  if approved_scope > max_grantable {
    return Err(AppsRouteError::PrivilegeEscalation {
      approved: approved_scope.to_string(),
      max_allowed: max_grantable.to_string(),
    })?;
  }

  // Validate tool instances using auth-scoped tool service (enforces ownership via user_id)
  for approval in &approval_input.approved.toolsets {
    if approval.status == ApprovalStatus::Approved {
      let instance = approval.instance.as_ref().ok_or_else(|| {
        AppsRouteError::ToolInstanceNotConfigured(format!(
          "instance required for approved toolset_type: {}",
          approval.toolset_type
        ))
      })?;

      let toolset_entity = auth_scope
        .tools()
        .get(&instance.id)
        .await?
        .ok_or_else(|| AppsRouteError::ToolInstanceNotOwned(instance.id.clone()))?;

      if toolset_entity.toolset_type != approval.toolset_type {
        return Err(AppsRouteError::InvalidToolType(format!(
          "Instance {} is not of type {}",
          instance.id, approval.toolset_type
        )))?;
      }

      if !toolset_entity.enabled {
        return Err(AppsRouteError::ToolInstanceNotConfigured(format!(
          "Instance {} is not enabled",
          instance.id
        )))?;
      }

      if toolset_entity.encrypted_api_key.is_none() {
        return Err(AppsRouteError::ToolInstanceNotConfigured(format!(
          "Instance {} does not have API key configured",
          instance.id
        )))?;
      }
    }
  }

  // Validate MCP instances using auth-scoped mcp service (enforces ownership via user_id)
  for approval in &approval_input.approved.mcps {
    if approval.status == ApprovalStatus::Approved {
      let instance = approval.instance.as_ref().ok_or_else(|| {
        AppsRouteError::ToolInstanceNotConfigured(format!(
          "instance required for approved MCP: {}",
          approval.url
        ))
      })?;

      let mcp_entity = auth_scope
        .mcps()
        .get(&instance.id)
        .await?
        .ok_or_else(|| AppsRouteError::ToolInstanceNotOwned(instance.id.clone()))?;

      if mcp_entity.server_url != approval.url {
        return Err(AppsRouteError::InvalidToolType(format!(
          "MCP instance {} is not connected to server {}",
          instance.id, approval.url
        )))?;
      }

      if !mcp_entity.enabled {
        return Err(AppsRouteError::ToolInstanceNotConfigured(format!(
          "MCP instance {} is not enabled",
          instance.id
        )))?;
      }
    }
  }

  let updated = access_request_service
    .approve_request(
      &id,
      user_id,
      tenant_id,
      token,
      approval_input.approved.toolsets,
      approval_input.approved.mcps,
      approved_scope,
    )
    .await?;

  Ok(Json(AccessRequestActionResponse {
    status: updated.status,
    flow_type: updated.flow_type,
    redirect_url: updated.redirect_uri,
  }))
}

/// Deny access request (POST /access-requests/:id/deny)
#[utoipa::path(
    post,
    path = ENDPOINT_ACCESS_REQUESTS_DENY,
    tag = API_TAG_AUTH,
    operation_id = "denyAccessRequest",
    summary = "Deny Access Request",
    description = "Deny access request. Requires session auth.",
    params(
        ("id" = String, Path, description = "Access request ID")
    ),
    responses(
        (status = 200, description = "Request denied", body = AccessRequestActionResponse),
        (status = 404, description = "Not found", body = OpenAIApiError),
        (status = 409, description = "Already processed", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_deny_access_request(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<AccessRequestActionResponse>, ApiError> {
  let user_id = auth_scope.require_user_id()?;

  let access_request_service = auth_scope.access_request_service();
  let updated = access_request_service.deny_request(&id, user_id).await?;

  Ok(Json(AccessRequestActionResponse {
    status: updated.status,
    flow_type: updated.flow_type,
    redirect_url: updated.redirect_uri,
  }))
}

#[cfg(test)]
#[path = "test_access_request.rs"]
mod test_access_request;
