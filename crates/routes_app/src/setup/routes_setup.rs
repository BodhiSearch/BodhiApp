use crate::middleware::standalone_app_status_or_default;
use crate::setup::error::SetupRouteError;
use crate::setup::setup_api_schemas::{AppInfo, SetupRequest, SetupResponse};
use crate::shared::{utils::extract_request_host, AuthScope};
use crate::{ApiError, JsonRejectionError};
use crate::{API_TAG_SETUP, API_TAG_SYSTEM, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP};
use axum::Json;
use axum_extra::extract::WithRejection;
use services::{AppStatus, AuthContext, DeploymentMode, LOGIN_CALLBACK_PATH};

pub const LOOPBACK_HOSTS: &[&str] = &["localhost", "127.0.0.1", "0.0.0.0"];

#[utoipa::path(
    get,
    path = ENDPOINT_APP_INFO,
    tag = API_TAG_SYSTEM,
    operation_id = "getAppInfo",
    summary = "Get Application Information",
    description = "Retrieves current application version and status information including setup state",
    responses(
        (status = 200, description = "Application information retrieved successfully", body = AppInfo,
         example = json!({
             "version": "0.1.0",
             "commit_sha": "abc1234",
             "status": "ready",
             "deployment": "standalone",
             "client_id": "my-client-id",
             "url": "https://example.com"
         })),
    )
)]
pub async fn setup_show(auth_scope: AuthScope) -> Result<Json<AppInfo>, ApiError> {
  let settings = auth_scope.settings();
  let deployment = settings.deployment_mode().await;

  let (status, client_id) = match auth_scope.auth_context() {
    // Standalone: authenticated session
    AuthContext::Session { client_id, .. } => (AppStatus::Ready, Some(client_id.clone())),
    // Multi-tenant: fully authenticated with a tenant
    AuthContext::MultiTenantSession {
      client_id: Some(cid),
      ..
    } => (AppStatus::Ready, Some(cid.clone())),
    // Multi-tenant: dashboard-only, check local memberships
    AuthContext::MultiTenantSession {
      client_id: None, ..
    } => {
      let has_memberships = auth_scope
        .tenants()
        .has_memberships()
        .await
        .unwrap_or(false);
      if has_memberships {
        (AppStatus::Ready, None)
      } else {
        (AppStatus::Setup, None)
      }
    }
    // Anonymous multi-tenant
    AuthContext::Anonymous {
      deployment: DeploymentMode::MultiTenant,
      ..
    } => (AppStatus::Ready, None),
    // Anonymous standalone
    AuthContext::Anonymous { .. } => {
      let tenant_svc = auth_scope.tenants();
      let standalone_app = tenant_svc.get_standalone_app().await?;
      let status = standalone_app
        .as_ref()
        .map(|t| t.status.clone())
        .unwrap_or_default();
      let cid = standalone_app.map(|t| t.client_id);
      (status, cid)
    }
    // API token / external app — always ready
    AuthContext::ApiToken { client_id, .. } | AuthContext::ExternalApp { client_id, .. } => {
      (AppStatus::Ready, Some(client_id.clone()))
    }
  };

  Ok(Json(AppInfo {
    version: settings.version().await,
    commit_sha: settings.commit_sha().await,
    status,
    deployment,
    client_id,
    url: settings.public_server_url().await,
  }))
}

#[utoipa::path(
    post,
    path = ENDPOINT_APP_SETUP,
    tag = API_TAG_SETUP,
    operation_id = "setupApp",
    summary = "Setup Application",
    description = "Initializes the application with authentication configuration and registers with the auth server",
    request_body(
        content = SetupRequest,
        description = "Application setup configuration",
        example = json!({
            "name": "My Bodhi Server",
            "description": "My personal AI server"
        })
    ),
    responses(
        (status = 200, description = "Application setup completed successfully", body = SetupResponse,
         example = json!({
             "status": "resource_admin"
         })),
    )
)]
pub async fn setup_create(
  auth_scope: AuthScope,
  headers: axum::http::HeaderMap,
  WithRejection(Json(request), _): WithRejection<Json<SetupRequest>, JsonRejectionError>,
) -> Result<SetupResponse, ApiError> {
  let settings = auth_scope.settings();
  if settings.is_multi_tenant().await {
    return Err(SetupRouteError::NotStandalone)?;
  }
  let tenant_svc = auth_scope.tenants();
  let auth_flow = auth_scope.auth_flow();
  let status = standalone_app_status_or_default(&tenant_svc).await?;
  if status != AppStatus::Setup {
    return Err(SetupRouteError::AlreadySetup)?;
  }

  // Validate server name (minimum 10 characters)
  if request.name.len() < 10 {
    return Err(SetupRouteError::ServerNameTooShort)?;
  }
  let settings = auth_scope.settings();
  let redirect_uris = if settings.get_public_host_explicit().await.is_some() {
    // Explicit configuration (including RunPod) - use only configured callback URL
    vec![settings.login_callback_url().await]
  } else {
    // Local/network installation mode - build comprehensive redirect URI list
    let scheme = settings.public_scheme().await;
    let port = settings.public_port().await;
    let mut redirect_uris = Vec::new();

    // Always add all loopback hosts for local development
    for host in LOOPBACK_HOSTS {
      redirect_uris.push(format!(
        "{}://{}:{}{}",
        scheme, host, port, LOGIN_CALLBACK_PATH
      ));
    }

    // Add request host if it's not a loopback host (for network access)
    if let Some(request_host) = extract_request_host(&headers) {
      if !LOOPBACK_HOSTS.contains(&request_host.as_str()) {
        redirect_uris.push(format!(
          "{}://{}:{}{}",
          scheme, request_host, port, LOGIN_CALLBACK_PATH
        ));
      }
    }

    // Add server IP for future-proofing (even if current request is from loopback)
    if let Some(server_ip) = auth_scope.network().get_server_ip() {
      let server_uri = format!("{}://{}:{}{}", scheme, server_ip, port, LOGIN_CALLBACK_PATH);
      // Only add if not already present
      if !redirect_uris.contains(&server_uri) {
        redirect_uris.push(server_uri);
      }
    }

    redirect_uris
  };
  let request_name = request.name.clone();
  let request_description = request.description.clone();
  let client_reg = auth_flow
    .register_client(
      request.name,
      request.description.unwrap_or_default(),
      redirect_uris,
    )
    .await?;
  tenant_svc
    .create_tenant(
      &client_reg.client_id,
      &client_reg.client_secret,
      &request_name,
      request_description,
      AppStatus::ResourceAdmin,
      None,
    )
    .await?;
  Ok(SetupResponse {
    status: AppStatus::ResourceAdmin,
  })
}
