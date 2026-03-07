use crate::middleware::{access_token_key, app_status_or_default, SESSION_KEY_ACTIVE_CLIENT_ID};
use crate::setup::error::SetupRouteError;
use crate::setup::setup_api_schemas::{AppInfo, SetupRequest, SetupResponse};
use crate::shared::{utils::extract_request_host, AuthScope};
use crate::tenants::{ensure_valid_dashboard_token, DASHBOARD_ACCESS_TOKEN_KEY};
use crate::{ApiError, JsonRejectionError};
use crate::{API_TAG_SETUP, API_TAG_SYSTEM, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP};
use axum::Json;
use axum_extra::extract::WithRejection;
use services::{AppStatus, LOGIN_CALLBACK_PATH};
use tower_sessions::Session;

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
             "client_id": "my-client-id"
         })),
    )
)]
pub async fn setup_show(
  auth_scope: AuthScope,
  session: Session,
) -> Result<Json<AppInfo>, ApiError> {
  let settings = auth_scope.settings();
  let deployment = settings.deployment_mode().await;

  let (status, client_id) = if settings.is_multi_tenant().await {
    resolve_multi_tenant_status(&auth_scope, &session).await?
  } else {
    let tenant_svc = auth_scope.tenant();
    let status = app_status_or_default(&tenant_svc).await;
    let client_id = auth_scope.auth_context().client_id().map(|s| s.to_string());
    (status, client_id)
  };

  Ok(Json(AppInfo {
    version: settings.version().await,
    commit_sha: settings.commit_sha().await,
    status,
    deployment,
    client_id,
  }))
}

async fn resolve_multi_tenant_status(
  auth_scope: &AuthScope,
  session: &Session,
) -> Result<(AppStatus, Option<String>), ApiError> {
  let settings = auth_scope.settings();

  // 1. Has active_client_id with valid resource token?
  if let Ok(Some(active_client_id)) = session.get::<String>(SESSION_KEY_ACTIVE_CLIENT_ID).await {
    if let Ok(Some(_token)) = session
      .get::<String>(&access_token_key(&active_client_id))
      .await
    {
      return Ok((AppStatus::Ready, Some(active_client_id)));
    }
  }

  // 2. Has dashboard token?
  if session
    .get::<String>(DASHBOARD_ACCESS_TOKEN_KEY)
    .await
    .unwrap_or(None)
    .is_some()
  {
    let auth_service = auth_scope.auth_service();
    match ensure_valid_dashboard_token(
      session,
      auth_service.as_ref(),
      settings.as_ref(),
      auth_scope.time().as_ref(),
    )
    .await
    {
      Ok(dashboard_token) => {
        match auth_service.list_tenants(&dashboard_token).await {
          Ok(spi_response) => {
            if spi_response.tenants.is_empty() {
              return Ok((AppStatus::Setup, None));
            } else {
              return Ok((AppStatus::TenantSelection, None));
            }
          }
          Err(_) => {
            // SPI call failed, treat as tenant_selection (user can retry)
            return Ok((AppStatus::TenantSelection, None));
          }
        }
      }
      Err(_) => {
        // Dashboard token invalid/expired and refresh failed
        return Ok((AppStatus::TenantSelection, None));
      }
    }
  }

  // 3. No dashboard token
  Ok((AppStatus::TenantSelection, None))
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
  let tenant_svc = auth_scope.tenant();
  let auth_flow = auth_scope.auth_flow();
  let status = app_status_or_default(&tenant_svc).await;
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
      AppStatus::ResourceAdmin,
      None,
    )
    .await?;
  Ok(SetupResponse {
    status: AppStatus::ResourceAdmin,
  })
}
