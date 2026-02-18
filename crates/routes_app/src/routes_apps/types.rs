use objs::{ApprovedResources, RequestedResources, Toolset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Request body for POST /apps/request-access
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "app_client_id": "my-app-client",
    "flow_type": "redirect",
    "redirect_url": "https://myapp.com/callback",
    "requested": {
        "toolset_types": [
            {"toolset_type": "builtin-exa-search"}
        ]
    }
}))]
pub struct CreateAccessRequestBody {
  /// App client ID from Keycloak
  pub app_client_id: String,
  /// Flow type: "redirect" or "popup"
  pub flow_type: String,
  /// Redirect URL for result notification (required for redirect flow)
  pub redirect_url: Option<String>,
  /// Resources requested (tools, etc.)
  pub requested: Option<RequestedResources>,
}

// Response for POST /apps/request-access
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status")]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "draft",
    "review_url": "http://localhost:1135/ui/apps/access-requests/review?id=550e8400-e29b-41d4-a716-446655440000"
}))]
pub enum CreateAccessRequestResponse {
  /// Draft status - requires user approval
  #[serde(rename = "draft")]
  Draft {
    /// Access request ID
    id: String,
    /// Review URL for user to approve/deny
    review_url: String,
  },
  /// Approved status - auto-approved when no tools requested
  #[serde(rename = "approved")]
  Approved {
    /// Access request ID
    id: String,
    /// Resource scope granted by KC
    resource_scope: String,
  },
}

// Response for GET /apps/access-requests/:id (status polling by apps)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "approved",
    "resource_scope": "scope_resource:550e8400-e29b-41d4-a716-446655440000",
    "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}))]
pub struct AccessRequestStatusResponse {
  /// Access request ID
  pub id: String,
  /// Current status: "draft", "approved", "denied", "failed"
  pub status: String,
  /// Resource scope (present when approved)
  pub resource_scope: Option<String>,
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
  pub flow_type: String,
  /// Current status
  pub status: String,
  /// Resources requested
  pub requested: RequestedResources,
  /// Tool type information with user instances
  pub tools_info: Vec<ToolTypeReviewInfo>,
  /// MCP server information with user instances
  #[serde(default)]
  pub mcps_info: Vec<McpServerReviewInfo>,
}

// Tool type review info with user instances
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolTypeReviewInfo {
  /// Tool type identifier
  pub toolset_type: String,
  /// Tool type display name
  pub name: String,
  /// Tool type description
  pub description: String,
  /// User's configured instances of this tool type
  pub instances: Vec<Toolset>,
}

// MCP server review info with user instances
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpServerReviewInfo {
  /// Requested MCP server URL
  pub url: String,
  /// User's MCP instances connected to this server URL
  pub instances: Vec<objs::Mcp>,
}

// Response for PUT /access-requests/:id/approve and POST /access-requests/:id/deny
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessRequestActionResponse {
  /// Updated status after action
  pub status: String,
  /// Flow type of the access request
  pub flow_type: String,
  /// Redirect URL (present for redirect flow)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redirect_url: Option<String>,
}

// Request body for PUT /access-requests/:id/approve
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "approved": {
        "toolsets": [
            {
                "toolset_type": "builtin-exa-search",
                "status": "approved",
                "instance": {"id": "instance-uuid"}
            }
        ],
        "mcps": [
            {
                "url": "https://mcp.deepwiki.com/mcp",
                "status": "approved",
                "instance": {"id": "instance-uuid"}
            }
        ]
    }
}))]
pub struct ApproveAccessRequestBody {
  /// Approved resources with selections
  pub approved: ApprovedResources,
}
