use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_builder::Builder)]
pub struct AppRegInfo {
  pub client_id: String,
  pub client_secret: String,
  #[serde(default)]
  pub scope: String,
}

#[derive(
  Debug,
  Serialize,
  Deserialize,
  PartialEq,
  strum::Display,
  Clone,
  Default,
  strum::EnumString,
  ToSchema,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[schema(example = "ready")]
pub enum AppStatus {
  #[default]
  /// Initial setup required
  #[schema(rename = "setup")]
  Setup,
  /// Application is ready
  #[schema(rename = "ready")]
  Ready,
  /// Admin setup required
  #[schema(rename = "resource-admin")]
  ResourceAdmin,
}

// ============================================================================
// Access Request API Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppAccessRequest {
  pub app_client_id: String,
  pub flow_type: String, // "redirect" | "popup"
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redirect_uri: Option<String>, // Required if flow_type == "redirect"
  pub tools: Vec<objs::ToolsetTypeRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppAccessResponse {
  pub access_request_id: String,
  pub review_url: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub scopes: Vec<String>, // Empty on draft creation, populated after approval
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppAccessRequestDetail {
  pub id: String,
  pub app_client_id: String,
  pub flow_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redirect_uri: Option<String>,
  pub status: String,
  pub tools_requested: Vec<objs::ToolsetTypeRequest>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_approved: Option<Vec<objs::ToolsetApproval>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user_id: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub scopes: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_message: Option<String>,
  #[schema(value_type = String, format = "date-time")]
  pub expires_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}
