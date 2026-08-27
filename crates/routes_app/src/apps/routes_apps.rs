use crate::apps::consent::{
  build_error_redirect, evaluate_consent_query, ConsentReady, OAUTH_ERROR_ACCESS_DENIED,
};
use crate::apps::{
  AppAccessSummary, AppsRouteError, ConsentAppInfo, ConsentContextResponse, ConsentDecision,
  ConsentPriorGrant, ConsentPriorGrantSource, ConsentScopeInfo, ListAppAccessResponse,
  SubmitConsentRequest, SubmitConsentResponse,
};
use crate::{AuthScope, BodhiErrorResponse, ValidatedJson, API_TAG_AUTH};
use axum::{
  extract::{Path, RawQuery},
  http::StatusCode,
  response::Json,
};
use services::{
  compose_keycloak_scope, validate_grant_against_scope, AppAccessRequest, AppAccessRequestStatus,
  ApprovalStatus, ApprovedResources, CreateApprovedAccessRequest, McpGrant, RequestedResources,
  RequestedResourcesV1, UserScope,
};
use services::{AuthContext, ResourceRole};

pub const ENDPOINT_APPS_ACCESS_REQUESTS: &str = "/bodhi/v1/apps/access-requests";
pub const ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT: &str = "/bodhi/v1/apps/access-requests/consent";
pub const ENDPOINT_ACCESS_REQUESTS_APPS: &str = "/bodhi/v1/access-requests/apps";
pub const ENDPOINT_ACCESS_REQUESTS_REVOKE: &str = "/bodhi/v1/access-requests/{id}/revoke";

/// Consent context (GET /apps/access-requests/consent)
#[utoipa::path(
    get,
    path = ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT,
    tag = API_TAG_AUTH,
    operation_id = "getConsentContext",
    summary = "Get Consent Context",
    description = "Validate an OAuth authorize request (passed as the raw query string, exactly as received by /ui/apps/auth/) and return everything the consent page needs: app identity, parsed scope sections, prior grant, and whether the caller may approve. Always 200 with an ok/error union; error outcomes carry a redirect_url only when the redirect target was validated. Requires session auth (any role — guests see a blocked state).",
    params(
        ("client_id" = Option<String>, Query, description = "OAuth client id of the requesting app"),
        ("redirect_uri" = Option<String>, Query, description = "App redirect target; exact-matched against the app's registered URIs"),
        ("response_type" = Option<String>, Query, description = "Forwarded to the auth server verbatim (typically 'code'); enforced there"),
        ("state" = Option<String>, Query, description = "Opaque app state, echoed back on redirects when present"),
        ("code_challenge" = Option<String>, Query, description = "PKCE challenge; forwarded verbatim, enforced by the auth server"),
        ("code_challenge_method" = Option<String>, Query, description = "PKCE method; forwarded verbatim, enforced by the auth server"),
        ("scope" = Option<String>, Query, description = "App-facing scope string (scope_user_*, scope_apps:*, passthrough tokens)"),
        ("source_access_request_id" = Option<String>, Query, description = "Prior grant id for reauthorization"),
    ),
    responses(
        (status = 200, description = "Consent context or a structured OAuth error", body = ConsentContextResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_get_consent_context(
  auth_scope: AuthScope,
  RawQuery(raw_query): RawQuery,
) -> Result<Json<ConsentContextResponse>, BodhiErrorResponse> {
  let token = auth_scope
    .auth_context()
    .token()
    .ok_or(AppsRouteError::InsufficientPrivileges)?;
  let auth_service = auth_scope.auth_service();

  let ready =
    match evaluate_consent_query(&*auth_service, token, raw_query.as_deref().unwrap_or("")).await {
      Ok(ready) => ready,
      Err(failure) => {
        return Ok(Json(ConsentContextResponse::Error {
          error: failure.error.to_string(),
          error_description: failure.error_description,
          redirect_url: failure.redirect_url,
        }))
      }
    };

  let can_approve = matches!(
    auth_scope.auth_context(),
    AuthContext::Session { role, .. } | AuthContext::MultiTenantSession { role, .. }
      if role.has_access_to(&ResourceRole::User)
  );
  let prior_grant = resolve_prior_grant(&auth_scope, &ready).await?;

  Ok(Json(ConsentContextResponse::Ok {
    app: ConsentAppInfo {
      client_id: ready.client_id,
      name: ready.app_info.name,
      description: ready.app_info.description,
      redirect_uri: ready.redirect_uri,
    },
    scope: ConsentScopeInfo {
      role: ready.scope.role,
      llms: ready.scope.llms,
      mcps: ready.scope.mcps,
      passthrough: ready.scope.passthrough,
    },
    prior_grant,
    can_approve,
  }))
}

/// Prior grant offered on the consent screen. An explicit `source_access_request_id` that
/// does not resolve to this app+user's approved grant is ignored (fresh authorization).
async fn resolve_prior_grant(
  auth_scope: &AuthScope,
  ready: &ConsentReady,
) -> Result<Option<ConsentPriorGrant>, BodhiErrorResponse> {
  let ctx = auth_scope.auth_context();
  let (Some(tenant_id), Some(user_id)) = (ctx.tenant_id(), ctx.user_id()) else {
    return Ok(None);
  };
  let access_request_service = auth_scope.access_request_service();

  if let Some(source_id) = &ready.source_access_request_id {
    let row = access_request_service
      .get_request(tenant_id, source_id)
      .await?;
    return Ok(row.and_then(|row| {
      to_prior_grant(
        row,
        &ready.client_id,
        user_id,
        ConsentPriorGrantSource::Explicit,
      )
    }));
  }

  let row = access_request_service
    .latest_approved_for_app_user(tenant_id, &ready.client_id, user_id)
    .await?;
  Ok(row.and_then(|row| {
    to_prior_grant(
      row,
      &ready.client_id,
      user_id,
      ConsentPriorGrantSource::Latest,
    )
  }))
}

fn to_prior_grant(
  row: AppAccessRequest,
  app_client_id: &str,
  user_id: &str,
  source: ConsentPriorGrantSource,
) -> Option<ConsentPriorGrant> {
  if row.status != AppAccessRequestStatus::Approved
    || row.app_client_id != app_client_id
    || row.user_id.as_deref() != Some(user_id)
  {
    return None;
  }
  let approved: ApprovedResources = serde_json::from_str(row.approved.as_deref()?).ok()?;
  let approved_role: UserScope = row.approved_role.as_deref()?.parse().ok()?;
  Some(ConsentPriorGrant {
    id: row.id,
    approved_role,
    approved,
    source,
  })
}

/// Submit consent decision (POST /apps/access-requests)
#[utoipa::path(
    post,
    path = ENDPOINT_APPS_ACCESS_REQUESTS,
    tag = API_TAG_AUTH,
    operation_id = "submitConsent",
    summary = "Submit Consent Decision",
    description = "The single mutating consent call. Re-validates the authorize request from the echoed query string; on approve creates an already-approved access request and returns the auth server's authorize URL with the composed scope; on deny returns the app's redirect_uri carrying error=access_denied. The page navigates to redirect_url unconditionally. Approve requires the User role; deny is allowed for any authenticated session.",
    request_body(
        content = SubmitConsentRequest,
        description = "Consent decision with the original query string"
    ),
    responses(
        (status = 201, description = "Grant created (approve)", body = SubmitConsentResponse),
        (status = 200, description = "Deny or redirectable request error — no grant created", body = SubmitConsentResponse),
        (status = 400, description = "Invalid request (rendered in-app)", body = BodhiErrorResponse),
        (status = 403, description = "Caller may not approve", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn apps_submit_consent(
  auth_scope: AuthScope,
  ValidatedJson(request): ValidatedJson<SubmitConsentRequest>,
) -> Result<(StatusCode, Json<SubmitConsentResponse>), BodhiErrorResponse> {
  let token = auth_scope
    .auth_context()
    .token()
    .ok_or(AppsRouteError::InsufficientPrivileges)?;
  let auth_service = auth_scope.auth_service();

  let ready = match evaluate_consent_query(&*auth_service, token, &request.query).await {
    Ok(ready) => ready,
    // A redirectable failure still completes the flow: the page navigates to the app.
    Err(failure) => match failure.redirect_url {
      Some(redirect_url) => {
        return Ok((
          StatusCode::OK,
          Json(SubmitConsentResponse {
            id: None,
            redirect_url,
          }),
        ))
      }
      None => {
        return Err(AppsRouteError::ConsentRejected {
          error: failure.error.to_string(),
          error_description: failure.error_description,
        })?
      }
    },
  };

  match request.decision {
    ConsentDecision::Deny => {
      // Evaluation already validated redirect_uri parses as a URL, so this cannot fail.
      let redirect_url = build_error_redirect(
        &ready.redirect_uri,
        OAUTH_ERROR_ACCESS_DENIED,
        "user denied the access request",
        ready.state.as_deref(),
      )
      .ok_or_else(|| AppsRouteError::ConsentRejected {
        error: "server_error".to_string(),
        error_description: "failed to compose the denial redirect".to_string(),
      })?;
      Ok((
        StatusCode::OK,
        Json(SubmitConsentResponse {
          id: None,
          redirect_url,
        }),
      ))
    }
    ConsentDecision::Approve => approve_consent(&auth_scope, request, ready).await,
  }
}

async fn approve_consent(
  auth_scope: &AuthScope,
  request: SubmitConsentRequest,
  ready: ConsentReady,
) -> Result<(StatusCode, Json<SubmitConsentResponse>), BodhiErrorResponse> {
  match auth_scope.auth_context() {
    AuthContext::Session { role, .. } | AuthContext::MultiTenantSession { role, .. }
      if role.has_access_to(&ResourceRole::User) => {}
    _ => return Err(AppsRouteError::InsufficientPrivileges)?,
  }
  let max_grantable =
    caller_max_user_scope(auth_scope).ok_or(AppsRouteError::InsufficientPrivileges)?;

  let approved_role = request
    .approved_role
    .ok_or_else(|| AppsRouteError::ConsentFieldMissing("approved_role".to_string()))?;
  let approved = request
    .approved
    .ok_or_else(|| AppsRouteError::ConsentFieldMissing("approved".to_string()))?;

  let requested_scope = ready.scope.role;
  if approved_role > requested_scope {
    return Err(AppsRouteError::PrivilegeEscalation {
      approved: approved_role.to_string(),
      max_allowed: requested_scope.to_string(),
    })?;
  }
  if approved_role > max_grantable {
    return Err(AppsRouteError::PrivilegeEscalation {
      approved: approved_role.to_string(),
      max_allowed: max_grantable.to_string(),
    })?;
  }

  // The grant may not exceed what the scope requested (tampered-POST guard).
  validate_grant_against_scope(&ready.scope, approved.v1()).map_err(AppsRouteError::from)?;

  // Validate MCP instances using auth-scoped services (enforces ownership via user_id)
  match &approved {
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

  let tenant_id = auth_scope.require_tenant_id()?;
  let user_id = auth_scope.require_user_id()?;
  let resource_client_id = auth_scope.require_client_id()?;
  let access_request_service = auth_scope.access_request_service();

  // Store the source id only when it resolves to this app+user's own approved grant.
  let source_access_request_id = match &ready.source_access_request_id {
    Some(source_id) => access_request_service
      .get_request(tenant_id, source_id)
      .await?
      .filter(|row| {
        row.status == AppAccessRequestStatus::Approved
          && row.app_client_id == ready.client_id
          && row.user_id.as_deref() == Some(user_id)
      })
      .map(|row| row.id),
    None => None,
  };

  // Audit record of what the scope requested, in the stored envelope shape.
  let requested = RequestedResources::V1(RequestedResourcesV1 {
    models_list: false,
    models_access: ready.scope.llms,
    mcps_list: false,
    mcps_access: ready.scope.mcps,
    mcp_servers: Vec::new(),
  });

  let row = access_request_service
    .create_approved(CreateApprovedAccessRequest {
      app_client_id: ready.client_id.clone(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      resource_client_id: resource_client_id.to_string(),
      requested,
      requested_role: ready.scope.role,
      approved,
      approved_role,
      source_access_request_id,
    })
    .await?;

  let access_request_scope = row
    .access_request_scope
    .as_deref()
    .ok_or(AppsRouteError::MissingAccessRequestScope)?;
  let kc_scope = compose_keycloak_scope(&ready.scope, access_request_scope);

  // Standard OAuth params are forwarded as received — Keycloak enforces them.
  let mut authorize_query = url::form_urlencoded::Serializer::new(String::new());
  if let Some(response_type) = &ready.response_type {
    authorize_query.append_pair("response_type", response_type);
  }
  authorize_query
    .append_pair("client_id", &ready.client_id)
    .append_pair("redirect_uri", &ready.redirect_uri);
  if let Some(state) = &ready.state {
    authorize_query.append_pair("state", state);
  }
  if let Some(code_challenge) = &ready.code_challenge {
    authorize_query.append_pair("code_challenge", code_challenge);
  }
  if let Some(code_challenge_method) = &ready.code_challenge_method {
    authorize_query.append_pair("code_challenge_method", code_challenge_method);
  }
  let authorize_query = authorize_query.append_pair("scope", &kc_scope).finish();
  let redirect_url = format!(
    "{}?{}",
    access_request_service.build_authorize_endpoint(),
    authorize_query
  );

  Ok((
    StatusCode::CREATED,
    Json(SubmitConsentResponse {
      id: Some(row.id),
      redirect_url,
    }),
  ))
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
    AuthContext::Session { role, .. } | AuthContext::MultiTenantSession { role, .. } => {
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
