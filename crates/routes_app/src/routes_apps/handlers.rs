use crate::routes_apps::{
  AccessRequestActionResponse, AccessRequestReviewResponse, AccessRequestStatusResponse,
  AppAccessRequestError, ApproveAccessRequestBody, CreateAccessRequestBody,
  CreateAccessRequestResponse, RequestedResources, ToolTypeReviewInfo,
};
use auth_middleware::AuthContext;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::Json,
  Extension,
};
use objs::{ApiError, OpenAIApiError, ToolsetTypeRequest, API_TAG_AUTH};
use serde::Deserialize;
use server_core::RouterState;
use std::sync::Arc;
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
    description = "Create an access request for an app to access user resources. If no tools requested, auto-approves. Unauthenticated endpoint.",
    request_body(
        content = CreateAccessRequestBody,
        description = "Access request details"
    ),
    responses(
        (status = 201, description = "Access request created", body = CreateAccessRequestResponse),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
        (status = 404, description = "App client not found", body = OpenAIApiError),
    ),
    security(())
)]
pub async fn create_access_request_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(body): Json<CreateAccessRequestBody>,
) -> Result<(StatusCode, Json<CreateAccessRequestResponse>), ApiError> {
  debug!(
    "Creating access request for app_client_id: {}",
    body.app_client_id
  );

  // Validate flow_type
  if body.flow_type != "redirect" && body.flow_type != "popup" {
    return Err(AppAccessRequestError::InvalidFlowType(body.flow_type))?;
  }

  // Validate redirect_url for redirect flow
  if body.flow_type == "redirect" && body.redirect_url.is_none() {
    return Err(AppAccessRequestError::MissingRedirectUrl)?;
  }

  // Note: We skip fetching app client info here because:
  // 1. This endpoint is unauthenticated (no user token available)
  // 2. KC endpoint for app client info may not be implemented yet
  // 3. App info will be fetched during review (when user is authenticated)
  debug!(
    "Creating access request for app_client_id: {} (app info will be fetched during review)",
    body.app_client_id
  );

  // Extract tool types from requested
  let tool_types: Vec<ToolsetTypeRequest> = body
    .requested
    .as_ref()
    .map(|r| r.toolset_types.clone())
    .unwrap_or_default();

  // Validate tool types exist
  let tool_service = state.app_service().tool_service();
  for tool_type_req in &tool_types {
    tool_service.validate_type(&tool_type_req.toolset_type)?;
  }

  // Create draft or auto-approve
  let access_request_service = state.app_service().access_request_service();
  let created = access_request_service
    .create_draft(
      body.app_client_id,
      body.flow_type,
      body.redirect_url,
      tool_types,
    )
    .await?;

  // Return response based on status
  if created.status == "approved" {
    info!("Access request {} auto-approved", created.id);
    Ok((
      StatusCode::CREATED,
      Json(CreateAccessRequestResponse::Approved {
        id: created.id,
        resource_scope: created
          .resource_scope
          .expect("resource_scope must be set for approved"),
      }),
    ))
  } else {
    // Draft status
    let review_url = access_request_service.build_review_url(&created.id);
    info!(
      "Access request {} created with review_url: {}",
      created.id, review_url
    );
    Ok((
      StatusCode::CREATED,
      Json(CreateAccessRequestResponse::Draft {
        id: created.id,
        review_url,
      }),
    ))
  }
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
pub async fn get_access_request_status_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  Query(query): Query<AccessRequestStatusQuery>,
) -> Result<Json<AccessRequestStatusResponse>, ApiError> {
  debug!("Getting access request status for id: {}", id);

  let access_request_service = state.app_service().access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppAccessRequestError::NotFound)?;

  // Verify app_client_id matches
  if request.app_client_id != query.app_client_id {
    return Err(AppAccessRequestError::NotFound)?;
  }

  Ok(Json(AccessRequestStatusResponse {
    id: request.id,
    status: request.status,
    resource_scope: request.resource_scope,
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
pub async fn get_access_request_review_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<AccessRequestReviewResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  debug!("Getting review data for access request: {}", id);

  let access_request_service = state.app_service().access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppAccessRequestError::NotFound)?;

  // Parse requested tools
  let requested: RequestedResources =
    serde_json::from_str(&request.requested).unwrap_or_else(|_| RequestedResources {
      toolset_types: vec![],
    });

  // Enrich with tool info
  let tool_service = state.app_service().tool_service();
  let mut tools_info = Vec::new();

  for tool_type_req in &requested.toolset_types {
    // Get tool type definition
    let tool_def = tool_service
      .get_type(&tool_type_req.toolset_type)
      .ok_or_else(|| AppAccessRequestError::InvalidToolType(tool_type_req.toolset_type.clone()))?;

    // Get user's instances of this tool type
    let user_toolsets = tool_service.list(user_id).await?;
    let instances = user_toolsets
      .into_iter()
      .filter(|t| t.toolset_type == tool_type_req.toolset_type)
      .collect();

    tools_info.push(ToolTypeReviewInfo {
      toolset_type: tool_type_req.toolset_type.clone(),
      name: tool_def.name.clone(),
      description: tool_def.description.clone(),
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
    requested,
    tools_info,
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
        content = ApproveAccessRequestBody,
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
pub async fn approve_access_request_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  Json(body): Json<ApproveAccessRequestBody>,
) -> Result<Json<AccessRequestActionResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let token = auth_context.token().expect("requires auth middleware");
  info!("User {} approving access request {}", user_id, id);

  // Validate tool instances
  let tool_service = state.app_service().tool_service();

  for approval in &body.approved.toolsets {
    if approval.status == "approved" {
      let instance = approval.instance.as_ref().ok_or_else(|| {
        AppAccessRequestError::ToolInstanceNotConfigured(format!(
          "instance required for approved toolset_type: {}",
          approval.toolset_type
        ))
      })?;

      let toolset = tool_service
        .get(user_id, &instance.id)
        .await?
        .ok_or_else(|| AppAccessRequestError::ToolInstanceNotOwned(instance.id.clone()))?;

      if toolset.toolset_type != approval.toolset_type {
        return Err(AppAccessRequestError::InvalidToolType(format!(
          "Instance {} is not of type {}",
          instance.id, approval.toolset_type
        )))?;
      }

      if !toolset.enabled {
        return Err(AppAccessRequestError::ToolInstanceNotConfigured(format!(
          "Instance {} is not enabled",
          instance.id
        )))?;
      }

      if !toolset.has_api_key {
        return Err(AppAccessRequestError::ToolInstanceNotConfigured(format!(
          "Instance {} does not have API key configured",
          instance.id
        )))?;
      }
    }
  }

  let access_request_service = state.app_service().access_request_service();
  let updated = access_request_service
    .approve_request(&id, user_id, token, body.approved.toolsets)
    .await?;

  info!("Access request {} approved by user {}", id, user_id);
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
pub async fn deny_access_request_handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<AccessRequestActionResponse>, ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
  info!("User {} denying access request {}", user_id, id);

  let access_request_service = state.app_service().access_request_service();
  let updated = access_request_service.deny_request(&id, user_id).await?;

  info!("Access request {} denied by user {}", id, user_id);
  Ok(Json(AccessRequestActionResponse {
    status: updated.status,
    flow_type: updated.flow_type,
    redirect_url: updated.redirect_uri,
  }))
}
