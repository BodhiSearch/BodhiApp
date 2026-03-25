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
  /// Access request ID
  pub id: String,
  /// Status (always "draft")
  pub status: AppAccessRequestStatus,
  /// Review URL for user to approve/deny
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
  /// Access request ID
  pub id: String,
  /// Current status: "draft", "approved", "denied", "failed"
  pub status: AppAccessRequestStatus,
  /// Role requested by the app
  pub requested_role: UserScope,
  /// Role approved (present when approved)
  pub approved_role: Option<UserScope>,
  /// Access request scope (present when user-approved with tools)
  pub access_request_scope: Option<String>,
}

// Response for GET /access-requests/:id/review (review page data)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessRequestReviewResponse {
  /// Access request ID
  pub id: String,
  /// App client ID
  pub app_client_id: String,
  /// App name from KC (if available)
  pub app_name: Option<String>,
  /// App description from KC (if available)
  pub app_description: Option<String>,
  /// Flow type: "redirect" or "popup"
  pub flow_type: FlowType,
  /// Current status
  pub status: AppAccessRequestStatus,
  /// Role requested by the app
  pub requested_role: String,
  /// Resources requested
  pub requested: RequestedResources,
  /// MCP server information with user instances
  #[serde(default)]
  pub mcps_info: Vec<McpServerReviewInfo>,
}

// MCP server review info with user instances
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpServerReviewInfo {
  /// Requested MCP server URL
  pub url: String,
  /// User's MCP instances connected to this server URL
  pub instances: Vec<services::Mcp>,
}

// Response for PUT /access-requests/:id/approve and POST /access-requests/:id/deny
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessRequestActionResponse {
  /// Updated status after action
  pub status: AppAccessRequestStatus,
  /// Flow type of the access request
  pub flow_type: FlowType,
  /// Redirect URL (present for redirect flow)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redirect_url: Option<String>,
}
