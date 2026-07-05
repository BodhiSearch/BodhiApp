use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{deserialize_versioned, mcp_proxy_path, McpGrant, ModelGrant, ResourceGrants};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppAccessRequest {
  pub id: String,
  pub tenant_id: Option<String>,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub status: AppAccessRequestStatus,
  pub requested: String,
  pub approved: Option<String>,
  pub user_id: Option<String>,
  pub requested_role: String,
  pub approved_role: Option<String>,
  pub access_request_scope: Option<String>,
  /// Prior approved request this upgrade elevates; `None` for a fresh request.
  pub source_access_request_id: Option<String>,
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
  /// Owner revoked a previously-approved grant; the app token stops working.
  Revoked,
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

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
pub struct McpInstance {
  pub id: String,
  /// MCP proxy path for this instance (e.g. `/bodhi/v1/apps/mcps/{id}/mcp`)
  pub path: String,
}

impl<'de> Deserialize<'de> for McpInstance {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct Helper {
      id: String,
    }
    let helper = Helper::deserialize(deserializer)?;
    let path = mcp_proxy_path(&helper.id);
    Ok(McpInstance {
      id: helper.id,
      path,
    })
  }
}

/// The `version` tag is mandatory — clients must specify which version they are using.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
#[serde(tag = "version")]
pub enum RequestedResources {
  #[serde(rename = "1")]
  V1(RequestedResourcesV1),
}

impl<'de> Deserialize<'de> for RequestedResources {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self::V1(deserialize_versioned(deserializer, "resources")?))
  }
}

impl RequestedResources {
  pub fn version(&self) -> &str {
    match self {
      Self::V1(_) => "1",
    }
  }
}

impl Default for RequestedResources {
  fn default() -> Self {
    Self::V1(RequestedResourcesV1::default())
  }
}

/// The `version` tag is mandatory and must match the corresponding `RequestedResources` version.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
#[serde(tag = "version")]
pub enum ApprovedResources {
  #[serde(rename = "1")]
  V1(ApprovedResourcesV1),
}

impl<'de> Deserialize<'de> for ApprovedResources {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self::V1(deserialize_versioned(deserializer, "resources")?))
  }
}

impl ApprovedResources {
  pub fn version(&self) -> &str {
    match self {
      Self::V1(_) => "1",
    }
  }

  pub fn v1(&self) -> &ApprovedResourcesV1 {
    match self {
      Self::V1(v1) => v1,
    }
  }
}

impl Default for ApprovedResources {
  fn default() -> Self {
    Self::V1(ApprovedResourcesV1::default())
  }
}

/// What the external app asks for. The four booleans are **UI drivers**: they tell
/// the consent screen which controls to render (the owner decides the actual grant).
/// Fields are domain-first (`models_*` / `mcps_*`), matching `ApprovedResourcesV1`.
/// `mcp_servers` is the existing by-url MCP request and is unchanged.
///
/// `models_access` defaults to **true**: unless the app explicitly opts out
/// (`models_access: false`), the consent screen shows the model-access selector so
/// the owner can always scope models. (The other UI-driver flags default to false.)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RequestedResourcesV1 {
  /// Render the "list all models" toggle.
  #[serde(default)]
  pub models_list: bool,
  /// Render the model All/Specific access selector. Defaults to `true` (shown).
  #[serde(default = "default_true")]
  pub models_access: bool,
  /// Render the "list all MCPs" toggle.
  #[serde(default)]
  pub mcps_list: bool,
  /// Render the owner-extra MCP All/Specific access selector.
  #[serde(default)]
  pub mcps_access: bool,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcp_servers: Vec<RequestedMcpServer>,
}

/// serde/Default helper: `models_access` is shown by default (see `RequestedResourcesV1`).
fn default_true() -> bool {
  true
}

impl Default for RequestedResourcesV1 {
  fn default() -> Self {
    Self {
      models_list: false,
      models_access: default_true(),
      mcps_list: false,
      mcps_access: false,
      mcp_servers: Vec::new(),
    }
  }
}

/// What the owner granted at consent. Field names mirror `RequestedResourcesV1`
/// (`models_list` / `models_access` / `mcps_list` / `mcps_access`) — there the
/// values are UI-driver booleans, here they are the actual grants. `mcps` holds
/// the by-url instance approvals; `mcps_access` is the owner-granted set beyond them.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ApprovedResourcesV1 {
  #[serde(default)]
  pub models_list: bool,
  #[serde(default)]
  pub models_access: ModelGrant,
  #[serde(default)]
  pub mcps_list: bool,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcps: Vec<McpApproval>,
  /// Owner-granted MCP instances beyond the by-url requests. Defaults to none
  /// (empty `Specific`) — unlike a token's all-access default.
  #[serde(default = "no_extra_mcps")]
  pub mcps_access: McpGrant,
}

/// Default for `mcps_access`: no extra MCPs (empty `Specific`), not the
/// all-access `McpGrant::default()`.
fn no_extra_mcps() -> McpGrant {
  McpGrant::Specific { ids: Vec::new() }
}

impl Default for ApprovedResourcesV1 {
  fn default() -> Self {
    Self {
      models_list: false,
      models_access: ModelGrant::default(),
      mcps_list: false,
      mcps: Vec::new(),
      mcps_access: no_extra_mcps(),
    }
  }
}

impl ResourceGrants for ApprovedResourcesV1 {
  fn allows_model_inference(&self, model_id: &str) -> bool {
    self.models_access.allows(model_id)
  }

  fn allows_mcp_connect(&self, mcp_id: &str) -> bool {
    self.mcps.iter().any(|a| {
      a.status == ApprovalStatus::Approved && a.instance.as_ref().is_some_and(|i| i.id == mcp_id)
    }) || self.mcps_access.allows(mcp_id)
  }

  fn models_list(&self) -> bool {
    self.models_list
  }

  fn mcps_list(&self) -> bool {
    self.mcps_list
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, ToSchema)]
#[schema(example = json!({
    "app_client_id": "my-app-client",
    "requested_role": "scope_user_user",
    "requested": {
        "version": "1",
        "mcp_servers": [
            {"url": "https://mcp.example.com/mcp"}
        ]
    }
}))]
pub struct CreateAccessRequest {
  /// App client ID from Keycloak
  pub app_client_id: String,
  /// Role requested for the external app (scope_user_user or scope_user_power_user)
  pub requested_role: crate::UserScope,
  /// Resources requested (tools, etc.)
  pub requested: RequestedResources,
  /// Upgrade the app's current token: the caller must present it in the `Authorization`
  /// header; the server derives the prior request from the token, never the body.
  #[serde(default)]
  pub exchange: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, ToSchema)]
#[schema(example = json!({
    "approved_role": "scope_user_user",
    "approved": {
        "version": "1",
        "mcps": [
            {
                "url": "https://mcp.deepwiki.com/mcp",
                "status": "approved",
                "instance": {"id": "instance-uuid", "path": "/bodhi/v1/apps/mcps/instance-uuid/mcp"}
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
