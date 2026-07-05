use crate::apps::{
  AccessRequestActionResponse, AccessRequestReviewResponse, AccessRequestStatusResponse,
  AppAccessSummary, AppsRouteError, CreateAccessRequestResponse, ListAppAccessResponse,
};
use crate::{AuthScope, BodhiErrorResponse, ValidatedJson, API_TAG_AUTH};
use axum::{
  extract::{Path, Query},
  http::StatusCode,
  response::Json,
};
use serde::Deserialize;
use services::{
  AppAccessRequestStatus, ApprovalStatus, ApproveAccessRequest, ApprovedResources,
  CreateAccessRequest, McpGrant, RequestedResources,
};
use services::{ResourceRole, UserScope};
use tracing::{debug, info};

pub const ENDPOINT_APPS_REQUEST_ACCESS: &str = "/bodhi/v1/apps/request-access";
pub const ENDPOINT_APPS_ACCESS_REQUESTS_ID: &str = "/bodhi/v1/apps/access-requests/{id}";
pub const ENDPOINT_ACCESS_REQUESTS_REVIEW: &str = "/bodhi/v1/access-requests/{id}/review";
pub const ENDPOINT_ACCESS_REQUESTS_APPROVE: &str = "/bodhi/v1/access-requests/{id}/approve";
pub const ENDPOINT_ACCESS_REQUESTS_DENY: &str = "/bodhi/v1/access-requests/{id}/deny";
pub const ENDPOINT_ACCESS_REQUESTS_APPS: &str = "/bodhi/v1/access-requests/apps";
pub const ENDPOINT_ACCESS_REQUESTS_REVOKE: &str = "/bodhi/v1/access-requests/{id}/revoke";

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
    description = "Create an access request for an app to access user resources. Always creates a draft for user review. Anonymous, except `exchange: true` requires the app's current token in the Authorization header.",
    request_body(
        content = CreateAccessRequest,
        description = "Access request details"
    ),
    responses(
        (status = 201, description = "Access request created", body = CreateAccessRequestResponse),
        (status = 400, description = "Invalid request", body = BodhiErrorResponse),
        (status = 404, description = "App client not found", body = BodhiErrorResponse),
    ),
    security(())
)]
pub async fn apps_create_access_request(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<CreateAccessRequest>,
) -> Result<(StatusCode, Json<CreateAccessRequestResponse>), BodhiErrorResponse> {
  debug!(
    "Creating access request for app_client_id: {}",
    request.app_client_id
  );

  // Exchange: derive the prior request from the caller's own app token (never the body).
  let source_access_request_id = if request.exchange {
    match auth_scope.auth_context() {
      services::AuthContext::ExternalApp {
        app_client_id,
        access_request_id: Some(source_id),
        ..
      } if *app_client_id == request.app_client_id => Some(source_id.clone()),
      _ => return Err(AppsRouteError::ExchangeRequiresAuth)?,
    }
  } else {
    None
  };

  let access_request_service = auth_scope.access_request_service();
  let created = access_request_service
    .create_draft(
      request.app_client_id,
      request.requested,
      request.requested_role,
      source_access_request_id,
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
        (status = 404, description = "Not found or app_client_id mismatch", body = BodhiErrorResponse),
    ),
    security(())
)]
pub async fn apps_get_access_request_status(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  Query(query): Query<AccessRequestStatusQuery>,
) -> Result<Json<AccessRequestStatusResponse>, BodhiErrorResponse> {
  debug!("Getting access request status for id: {}", id);

  let access_request_service = auth_scope.access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppsRouteError::NotFound)?;

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
        (status = 404, description = "Not found", body = BodhiErrorResponse),
        (status = 410, description = "Request expired", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_get_access_request_review(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<AccessRequestReviewResponse>, BodhiErrorResponse> {
  let access_request_service = auth_scope.access_request_service();
  let request = access_request_service
    .get_request(&id)
    .await?
    .ok_or(AppsRouteError::NotFound)?;

  let requested: RequestedResources =
    serde_json::from_str(&request.requested).map_err(|_| AppsRouteError::InvalidRequestedJson)?;

  let mcps_svc = auth_scope.mcps();
  let all_user_mcp_entities = mcps_svc.list().await?;
  let all_user_mcps: Vec<services::Mcp> = all_user_mcp_entities
    .into_iter()
    .map(|e| e.into())
    .collect();

  let mut mcps_info = Vec::new();

  match &requested {
    RequestedResources::V1(v1) => {
      for mcp_server_req in &v1.mcp_servers {
        // Surface every configured instance so the user can connect the request to any of their
        // MCPs (e.g. the same tool reached via a gateway), with exact-URL matches sorted first.
        let (mut matches, others): (Vec<_>, Vec<_>) = all_user_mcps
          .iter()
          .cloned()
          .partition(|m| m.mcp_server.url == mcp_server_req.url);
        matches.extend(others);

        mcps_info.push(crate::apps::McpServerReviewInfo {
          url: mcp_server_req.url.clone(),
          instances: matches,
        });
      }
    }
  }

  let previous_grant = match &request.source_access_request_id {
    Some(source_id) => {
      resolve_previous_grant(&*access_request_service, source_id, &request.app_client_id).await
    }
    None => None,
  };

  Ok(Json(AccessRequestReviewResponse {
    id: request.id,
    app_client_id: request.app_client_id,
    app_name: request.app_name,
    app_description: request.app_description,
    status: request.status,
    requested_role: request.requested_role,
    requested,
    mcps_info,
    auth_endpoint: access_request_service.build_authorize_endpoint(),
    previous_grant,
  }))
}

/// Prior grant for an upgrade review; `None` (form uses defaults) when the source is
/// missing, not approved, from a different app, or unparsable.
async fn resolve_previous_grant(
  service: &dyn services::AccessRequestService,
  source_id: &str,
  app_client_id: &str,
) -> Option<crate::apps::PreviousGrantInfo> {
  let source = service.get_request(source_id).await.ok().flatten()?;
  if source.status != AppAccessRequestStatus::Approved || source.app_client_id != app_client_id {
    return None;
  }
  let approved: ApprovedResources = serde_json::from_str(source.approved.as_deref()?).ok()?;
  let approved_role: UserScope = source.approved_role.as_deref()?.parse().ok()?;
  Some(crate::apps::PreviousGrantInfo {
    approved_role,
    approved,
  })
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
        (status = 400, description = "Invalid request", body = BodhiErrorResponse),
        (status = 404, description = "Not found", body = BodhiErrorResponse),
        (status = 409, description = "Already processed", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_approve_access_request(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(approval_input): ValidatedJson<ApproveAccessRequest>,
) -> Result<Json<AccessRequestActionResponse>, BodhiErrorResponse> {
  let user_id = auth_scope.require_user_id()?;
  let token = auth_scope
    .auth_context()
    .token()
    .ok_or(AppsRouteError::InsufficientPrivileges)?;
  let tenant_id = auth_scope.require_tenant_id()?;

  let approver_role = match auth_scope.auth_context() {
    services::AuthContext::Session { role, .. }
    | services::AuthContext::MultiTenantSession { role, .. } => {
      if !role.has_access_to(&ResourceRole::User) {
        return Err(AppsRouteError::InsufficientPrivileges)?;
      }
      role
    }
    services::AuthContext::Anonymous { .. }
    | services::AuthContext::ApiToken { .. }
    | services::AuthContext::ExternalApp { .. } => {
      return Err(AppsRouteError::InsufficientPrivileges)?
    }
  };
  let max_grantable = if *approver_role >= ResourceRole::PowerUser {
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

  // Validate MCP instances using auth-scoped services (enforces ownership via user_id)
  match &approval_input.approved {
    ApprovedResources::V1(v1) => {
      for approval in &v1.mcps {
        if approval.status == ApprovalStatus::Approved {
          let instance = approval.instance.as_ref().ok_or_else(|| {
            AppsRouteError::McpInstanceNotConfigured(format!(
              "instance required for approved MCP: {}",
              approval.url
            ))
          })?;

          // Any owned + enabled instance may satisfy a requested URL — the user picks which of
          // their MCPs to connect, so we don't require the instance's server_url to match.
          let mcp_entity = auth_scope
            .mcps()
            .get(&instance.id)
            .await?
            .ok_or_else(|| AppsRouteError::McpInstanceNotOwned(instance.id.clone()))?;

          if !mcp_entity.enabled {
            return Err(AppsRouteError::McpInstanceNotConfigured(format!(
              "MCP instance {} is not enabled",
              instance.id
            )))?;
          }
        }
      }

      // Owner-extra MCP grants must reference the owner's own enabled instances too.
      if let McpGrant::Specific { ids } = &v1.mcps_access {
        for id in ids {
          let mcp_entity = auth_scope
            .mcps()
            .get(id)
            .await?
            .ok_or_else(|| AppsRouteError::McpInstanceNotOwned(id.clone()))?;
          if !mcp_entity.enabled {
            return Err(AppsRouteError::McpInstanceNotConfigured(format!(
              "MCP instance {} is not enabled",
              id
            )))?;
          }
        }
      }
    }
  }

  let updated = access_request_service
    .approve_request(
      &id,
      user_id,
      tenant_id,
      token,
      approval_input.approved,
      approved_scope,
    )
    .await?;

  Ok(Json(AccessRequestActionResponse {
    status: updated.status,
    access_request_scope: updated.access_request_scope,
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
        (status = 404, description = "Not found", body = BodhiErrorResponse),
        (status = 409, description = "Already processed", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_deny_access_request(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<AccessRequestActionResponse>, BodhiErrorResponse> {
  let user_id = auth_scope.require_user_id()?;

  let access_request_service = auth_scope.access_request_service();
  let updated = access_request_service.deny_request(&id, user_id).await?;

  Ok(Json(AccessRequestActionResponse {
    status: updated.status,
    access_request_scope: None,
  }))
}

/// List the caller's issued app tokens (GET /access-requests/apps)
#[utoipa::path(
    get,
    path = ENDPOINT_ACCESS_REQUESTS_APPS,
    tag = API_TAG_AUTH,
    operation_id = "listAppAccess",
    summary = "List Issued App Tokens",
    description = "List the caller's approved app access grants with their effective resource access. Requires session auth.",
    responses(
        (status = 200, description = "Issued app tokens", body = ListAppAccessResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_list_user_access(
  auth_scope: AuthScope,
) -> Result<Json<ListAppAccessResponse>, BodhiErrorResponse> {
  let user_id = auth_scope.require_user_id()?;
  let tenant_id = auth_scope.require_tenant_id()?;
  let ceiling = caller_max_user_scope(&auth_scope);
  let rows = auth_scope
    .access_request_service()
    .list_approved_for_user(tenant_id, user_id)
    .await?;
  let data = rows
    .into_iter()
    .map(|row| AppAccessSummary::from_row(row, ceiling))
    .collect();
  Ok(Json(ListAppAccessResponse { data }))
}

/// The maximum `UserScope` the session caller could have granted — used to clamp a
/// (possibly DB-tampered) stored `approved_role` for display, mirroring the
/// token-exchange privilege ceiling. Non-session principals ⇒ `None` (no clamp).
fn caller_max_user_scope(auth_scope: &AuthScope) -> Option<UserScope> {
  match auth_scope.auth_context() {
    services::AuthContext::Session { role, .. }
    | services::AuthContext::MultiTenantSession { role, .. } => {
      Some(if *role >= ResourceRole::PowerUser {
        UserScope::PowerUser
      } else {
        UserScope::User
      })
    }
    _ => None,
  }
}

/// Revoke an issued app token (POST /access-requests/:id/revoke)
#[utoipa::path(
    post,
    path = ENDPOINT_ACCESS_REQUESTS_REVOKE,
    tag = API_TAG_AUTH,
    operation_id = "revokeAppAccess",
    summary = "Revoke App Token",
    description = "Revoke a previously-approved app grant; the app token stops working. Requires session auth.",
    params(
        ("id" = String, Path, description = "Access request ID")
    ),
    responses(
        (status = 200, description = "Grant revoked", body = AppAccessSummary),
        (status = 404, description = "Not found", body = BodhiErrorResponse),
        (status = 409, description = "Not in a revocable state", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_revoke_access_request(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<AppAccessSummary>, BodhiErrorResponse> {
  let user_id = auth_scope.require_user_id()?;
  let tenant_id = auth_scope.require_tenant_id()?;
  let updated = auth_scope
    .access_request_service()
    .revoke_request(tenant_id, &id, user_id)
    .await?;

  // Evict any cached token-exchange results bound to this access request so the
  // revocation takes effect immediately on every path (not after the 5-min TTL).
  let needle = crate::middleware::token_service::access_request_cache_needle(&id);
  auth_scope
    .cache_service()
    .remove_entries_containing(&needle);

  let ceiling = caller_max_user_scope(&auth_scope);
  Ok(Json(AppAccessSummary::from_row(updated, ceiling)))
}

#[cfg(test)]
#[path = "test_access_request.rs"]
mod test_access_request;
