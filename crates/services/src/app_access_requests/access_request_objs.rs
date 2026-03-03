use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// AppAccessRequest - Database row for app access request consent tracking
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppAccessRequest {
  pub id: String,
  pub tenant_id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub flow_type: FlowType,
  pub redirect_uri: Option<String>,
  pub status: AppAccessRequestStatus,
  pub requested: String,
  pub approved: Option<String>,
  pub user_id: Option<String>,
  pub requested_role: String,
  pub approved_role: Option<String>,
  pub access_request_scope: Option<String>,
  pub error_message: Option<String>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ApprovalStatus {
  Approved,
  Denied,
}

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  strum::Display,
  strum::EnumIter,
  strum::EnumString,
  Serialize,
  Deserialize,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppAccessRequestStatus {
  Draft,
  Approved,
  Denied,
  Failed,
  Expired,
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  PartialEq,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FlowType {
  Redirect,
  Popup,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetTypeRequest {
  pub toolset_type: String, // e.g., "builtin-exa-search"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetApproval {
  pub toolset_type: String,
  pub status: ApprovalStatus,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instance: Option<ToolsetInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetInstance {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RequestedMcpServer {
  pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpApproval {
  pub url: String,
  pub status: ApprovalStatus,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instance: Option<McpInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpInstance {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Default)]
pub struct ApprovedResources {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub toolsets: Vec<ToolsetApproval>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcps: Vec<McpApproval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Default)]
pub struct RequestedResources {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub toolset_types: Vec<ToolsetTypeRequest>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcp_servers: Vec<RequestedMcpServer>,
}

// ============================================================================
// Request types for app access request domain
// ============================================================================

/// Request for creating an app access request
#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, ToSchema)]
#[schema(example = json!({
    "app_client_id": "my-app-client",
    "flow_type": "redirect",
    "redirect_url": "https://myapp.com/callback",
    "requested_role": "scope_user_user",
    "requested": {
        "toolset_types": [
            {"toolset_type": "builtin-exa-search"}
        ]
    }
}))]
pub struct CreateAccessRequest {
  /// App client ID from Keycloak
  pub app_client_id: String,
  /// Flow type: "redirect" or "popup"
  pub flow_type: FlowType,
  /// Redirect URL for result notification (required for redirect flow)
  pub redirect_url: Option<String>,
  /// Role requested for the external app (scope_user_user or scope_user_power_user)
  pub requested_role: crate::UserScope,
  /// Resources requested (tools, etc.)
  pub requested: Option<RequestedResources>,
}

/// Request for approving an app access request
#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, ToSchema)]
#[schema(example = json!({
    "approved_role": "scope_user_user",
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
pub struct ApproveAccessRequest {
  /// Role to grant for the approved request (scope_user_user or scope_user_power_user)
  pub approved_role: crate::UserScope,
  /// Approved resources with selections
  pub approved: ApprovedResources,
}
