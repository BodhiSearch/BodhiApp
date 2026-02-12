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
            {"tool_type": "builtin-exa-search"}
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

// Wrapper for requested resources
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequestedResources {
  /// Toolset types being requested
  #[serde(default)]
  pub toolset_types: Vec<ToolTypeRequest>,
}

// Tool type request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolTypeRequest {
  /// Tool type identifier (e.g., "builtin-exa-search")
  pub tool_type: String,
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
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "app_client_id": "my-app-client",
    "app_name": "My Application",
    "app_description": "A sample application",
    "flow_type": "redirect",
    "status": "draft",
    "requested": {
        "toolset_types": [
            {"tool_type": "builtin-exa-search"}
        ]
    },
    "tools_info": [
        {
            "tool_type": "builtin-exa-search",
            "name": "Exa Search",
            "description": "Search the web using Exa AI",
            "instances": [
                {
                    "id": "instance-uuid",
                    "name": "My Exa Instance",
                    "enabled": true,
                    "has_api_key": true
                }
            ]
        }
    ]
}))]
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
}

// Tool type review info with user instances
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolTypeReviewInfo {
  /// Tool type identifier
  pub tool_type: String,
  /// Tool type display name
  pub name: String,
  /// Tool type description
  pub description: String,
  /// User's configured instances of this tool type
  pub instances: Vec<ToolInstanceInfo>,
}

// Tool instance info for review
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolInstanceInfo {
  /// Instance ID
  pub id: String,
  /// Instance name
  pub name: String,
  /// Whether instance is enabled
  pub enabled: bool,
  /// Whether instance has API key configured
  pub has_api_key: bool,
}

// Request body for PUT /access-requests/:id/approve
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "approved": {
        "toolset_types": [
            {
                "tool_type": "builtin-exa-search",
                "status": "approved",
                "instance_id": "instance-uuid"
            }
        ]
    }
}))]
pub struct ApproveAccessRequestBody {
  /// Approved resources with selections
  pub approved: ApprovedResources,
}

// Wrapper for approved resources
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApprovedResources {
  /// Toolset approvals with instance selections
  #[serde(default)]
  pub toolset_types: Vec<ToolApproval>,
}

// Tool approval with instance selection
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolApproval {
  /// Tool type identifier
  pub tool_type: String,
  /// Approval status: "approved" or "denied"
  pub status: String,
  /// Instance ID (required when status = "approved")
  pub instance_id: Option<String>,
}
