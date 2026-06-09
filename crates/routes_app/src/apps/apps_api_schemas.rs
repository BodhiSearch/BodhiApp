use serde::{Deserialize, Serialize};
use services::{AppAccessRequestStatus, FlowType, RequestedResources, UserScope};
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
