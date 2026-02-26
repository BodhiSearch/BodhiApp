use crate::shared::utils::extract_request_host;
use crate::{ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP};
use auth_middleware::app_status_or_default;
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, ErrorType, API_TAG_SETUP, API_TAG_SYSTEM};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{AppInstanceError, AppStatus, AuthServiceError, LOGIN_CALLBACK_PATH};
use std::sync::Arc;
use utoipa::ToSchema;

pub const LOOPBACK_HOSTS: &[&str] = &["localhost", "127.0.0.1", "0.0.0.0"];

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppServiceError {
  #[error("Application is already set up.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AlreadySetup,
  #[error("Server name must be at least 10 characters long.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ServerNameTooShort,
  #[error(transparent)]
  AppInstanceError(#[from] AppInstanceError),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
}

/// Application information and status
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[schema(example = json!({
    "version": "0.1.0",
    "commit_sha": "abc1234",
    "status": "ready"
}))]
pub struct AppInfo {
  /// Application version number (semantic versioning)
  #[schema(example = "0.1.0")]
  pub version: String,
  /// Git commit SHA of the build
  #[schema(example = "abc1234")]
  pub commit_sha: String,
  /// Current application setup and operational status
  #[schema(example = "ready")]
  pub status: AppStatus,
}

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
             "status": "ready"
         })),
    )
)]
pub async fn app_info_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<AppInfo>, ApiError> {
  let app_instance_service = state.app_service().app_instance_service();
  let status = app_status_or_default(&app_instance_service).await;
  let setting_service = &state.app_service().setting_service();
  Ok(Json(AppInfo {
    version: setting_service.version().await,
    commit_sha: setting_service.commit_sha().await,
    status,
  }))
}

/// Request to setup the application in authenticated mode
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
  "name": "My Bodhi Server",
  "description": "My personal AI server"
}))]
pub struct SetupRequest {
  /// Server name for identification (minimum 10 characters)
  #[schema(min_length = 10, max_length = 100, example = "My Bodhi Server")]
  pub name: String,
  /// Optional description of the server's purpose
  #[schema(max_length = 500, example = "My personal AI server")]
  pub description: Option<String>,
}

/// Response containing the updated application status after setup
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "resource-admin"
}))]
pub struct SetupResponse {
  /// New application status after successful setup
  #[schema(example = "resource-admin")]
  pub status: AppStatus,
}

impl IntoResponse for SetupResponse {
  fn into_response(self) -> Response {
    (StatusCode::OK, Json(self)).into_response()
  }
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
             "status": "resource-admin"
         })),
    )
)]
pub async fn setup_handler(
  headers: axum::http::HeaderMap,
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<SetupRequest>, ApiError>,
) -> Result<SetupResponse, ApiError> {
  let app_instance_service = state.app_service().app_instance_service();
  let auth_service = &state.app_service().auth_service();
  let status = app_status_or_default(&app_instance_service).await;
  if status != AppStatus::Setup {
    return Err(AppServiceError::AlreadySetup)?;
  }

  // Validate server name (minimum 10 characters)
  if request.name.len() < 10 {
    return Err(AppServiceError::ServerNameTooShort)?;
  }
  let setting_service = &state.app_service().setting_service();
  let redirect_uris = if setting_service.get_public_host_explicit().await.is_some() {
    // Explicit configuration (including RunPod) - use only configured callback URL
    vec![setting_service.login_callback_url().await]
  } else {
    // Local/network installation mode - build comprehensive redirect URI list
    let scheme = setting_service.public_scheme().await;
    let port = setting_service.public_port().await;
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
    if let Some(server_ip) = state.app_service().network_service().get_server_ip() {
      let server_uri = format!("{}://{}:{}{}", scheme, server_ip, port, LOGIN_CALLBACK_PATH);
      // Only add if not already present
      if !redirect_uris.contains(&server_uri) {
        redirect_uris.push(server_uri);
      }
    }

    redirect_uris
  };
  let client_reg = auth_service
    .register_client(
      request.name,
      request.description.unwrap_or_default(),
      redirect_uris,
    )
    .await?;
  app_instance_service
    .create_instance(
      &client_reg.client_id,
      &client_reg.client_secret,
      AppStatus::ResourceAdmin,
    )
    .await?;
  Ok(SetupResponse {
    status: AppStatus::ResourceAdmin,
  })
}

#[cfg(test)]
#[path = "test_setup.rs"]
mod test_setup;
