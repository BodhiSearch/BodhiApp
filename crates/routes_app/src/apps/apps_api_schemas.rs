use crate::ResourceAccess;
use serde::{Deserialize, Serialize};
use services::{
  AppAccessRequest, AppAccessRequestStatus, ApprovedResources, FlowType, RequestedResources,
  UserScope,
};
use utoipa::ToSchema;

// Response for POST /apps/request-access
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "draft",
    "review_url": "http://localhost:1135/ui/apps/access-requests/review?id=550e8400-e29b-41d4-a716-446655440000"
}))]
pub struct CreateAccessRequestResponse {
  pub id: String,
  /// Always "draft"
  pub status: AppAccessRequestStatus,
  pub review_url: String,
}

// Response for GET /apps/access-requests/:id (status polling by apps)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "approved",
    "requested_role": "scope_user_user",
    "approved_role": "scope_user_user",
    "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}))]
pub struct AccessRequestStatusResponse {
  pub id: String,
  /// One of: "draft", "approved", "denied", "failed"
  pub status: AppAccessRequestStatus,
  pub requested_role: UserScope,
  /// Present when approved
  pub approved_role: Option<UserScope>,
  /// Present when user-approved with tools
  pub access_request_scope: Option<String>,
}

// Response for GET /access-requests/:id/review (review page data)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessRequestReviewResponse {
  pub id: String,
  pub app_client_id: String,
  /// From KC, if available
  pub app_name: Option<String>,
  /// From KC, if available
  pub app_description: Option<String>,
  /// One of: "redirect", "popup"
  pub flow_type: FlowType,
  pub status: AppAccessRequestStatus,
  pub requested_role: String,
  pub requested: RequestedResources,
  #[serde(default)]
  pub mcps_info: Vec<McpServerReviewInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpServerReviewInfo {
  pub url: String,
  /// User's MCP instances connected to this server URL
  pub instances: Vec<services::Mcp>,
}

// Response for PUT /access-requests/:id/approve and POST /access-requests/:id/deny
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessRequestActionResponse {
  pub status: AppAccessRequestStatus,
  pub flow_type: FlowType,
  /// Present for redirect flow
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redirect_url: Option<String>,
}

/// One issued app token (approved access request) with its effective grant summary.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppAccessSummary {
  pub id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub status: AppAccessRequestStatus,
  pub approved_role: Option<UserScope>,
  /// Effective model access granted to this app.
  pub models: ResourceAccess,
  /// Effective MCP access granted to this app.
  pub mcps: ResourceAccess,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AppAccessSummary {
  /// Build a summary from an access-request row, reflecting its approved grants
  /// (defaults to no access when the approved JSON is missing/unparsable).
  pub fn from_row(row: AppAccessRequest) -> Self {
    let approved = row
      .approved
      .as_deref()
      .and_then(|json| serde_json::from_str::<ApprovedResources>(json).ok());
    let (models, mcps) = match approved.as_ref().map(|a| a.v1()) {
      Some(v1) => (ResourceAccess::app_models(v1), ResourceAccess::app_mcps(v1)),
      None => (
        ResourceAccess::Specific {
          list: false,
          ids: vec![],
        },
        ResourceAccess::Specific {
          list: false,
          ids: vec![],
        },
      ),
    };
    Self {
      id: row.id,
      app_client_id: row.app_client_id,
      app_name: row.app_name,
      app_description: row.app_description,
      status: row.status,
      approved_role: row.approved_role.and_then(|r| r.parse().ok()),
      models,
      mcps,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }
  }
}

/// Response for GET /access-requests/apps — the caller's issued app tokens.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListAppAccessResponse {
  pub data: Vec<AppAccessSummary>,
}
