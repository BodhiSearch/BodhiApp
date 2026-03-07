use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use services::AppStatus;
use utoipa::ToSchema;

/// Application information and status
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[schema(example = json!({
    "version": "0.1.0",
    "commit_sha": "abc1234",
    "status": "ready",
    "deployment": "standalone",
    "client_id": "my-client-id"
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
  /// Deployment mode: "standalone" or "multi_tenant"
  #[schema(example = "standalone")]
  pub deployment: String,
  /// Active tenant's OAuth client_id (present when authenticated with an active tenant)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[schema(example = "my-client-id", nullable)]
  pub client_id: Option<String>,
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
    "status": "resource_admin"
}))]
pub struct SetupResponse {
  /// New application status after successful setup
  #[schema(example = "resource_admin")]
  pub status: AppStatus,
}

impl IntoResponse for SetupResponse {
  fn into_response(self) -> Response {
    (StatusCode::OK, Json(self)).into_response()
  }
}
