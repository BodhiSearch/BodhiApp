use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{mcp_proxy_path, McpGrant, ModelGrant, ResourceGrants};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppAccessRequest {
  pub id: String,
  pub tenant_id: Option<String>,
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
  /// Owner revoked a previously-approved grant; the app token stops working.
  Revoked,
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
    let value = serde_json::Value::deserialize(deserializer)?;
    let version = value
      .get("version")
      .and_then(|v| v.as_str())
      .ok_or_else(|| serde::de::Error::missing_field("version"))?;
    match version {
      "1" => {
        let v1: RequestedResourcesV1 =
          serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self::V1(v1))
      }
      unknown => Err(serde::de::Error::custom(format!(
        "Unsupported resources version '{}'. Supported versions: [1]",
        unknown
      ))),
    }
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
    let value = serde_json::Value::deserialize(deserializer)?;
    let version = value
      .get("version")
      .and_then(|v| v.as_str())
      .ok_or_else(|| serde::de::Error::missing_field("version"))?;
    match version {
      "1" => {
        let v1: ApprovedResourcesV1 =
          serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self::V1(v1))
      }
      unknown => Err(serde::de::Error::custom(format!(
        "Unsupported resources version '{}'. Supported versions: [1]",
        unknown
      ))),
    }
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
/// `mcp_servers` is the existing by-url MCP request and is unchanged.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Default)]
pub struct RequestedResourcesV1 {
  /// Render the "list all models" toggle.
  #[serde(default)]
  pub models_list: bool,
  /// Render the model All/Specific access selector.
  #[serde(default)]
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

/// What the owner granted at consent. Model grants mirror API tokens
/// (`list_models` + `models`). MCP grants combine the existing by-url instance
/// approvals (`mcps`) with an owner-granted-beyond-requested set (`mcps_extra`).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ApprovedResourcesV1 {
  #[serde(default)]
  pub list_models: bool,
  #[serde(default)]
  pub models: ModelGrant,
  #[serde(default)]
  pub list_mcps: bool,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcps: Vec<McpApproval>,
  /// Owner-granted MCP instances beyond the by-url requests. Defaults to none
  /// (empty `Specific`) — unlike a token's all-access default.
  #[serde(default = "no_extra_mcps")]
  pub mcps_extra: McpGrant,
}

/// Default for `mcps_extra`: no extra MCPs (empty `Specific`), not the
/// all-access `McpGrant::default()`.
fn no_extra_mcps() -> McpGrant {
  McpGrant::Specific { ids: Vec::new() }
}

impl Default for ApprovedResourcesV1 {
  fn default() -> Self {
    Self {
      list_models: false,
      models: ModelGrant::default(),
      list_mcps: false,
      mcps: Vec::new(),
      mcps_extra: no_extra_mcps(),
    }
  }
}

impl ResourceGrants for ApprovedResourcesV1 {
  fn allows_model_inference(&self, model_id: &str) -> bool {
    self.models.allows(model_id)
  }

  fn model_listable(&self, model_id: &str) -> bool {
    self.list_models || self.allows_model_inference(model_id)
  }

  fn allows_mcp_connect(&self, mcp_id: &str) -> bool {
    self.mcps.iter().any(|a| {
      a.status == ApprovalStatus::Approved && a.instance.as_ref().is_some_and(|i| i.id == mcp_id)
    }) || self.mcps_extra.allows(mcp_id)
  }

  fn mcp_listable(&self, mcp_id: &str) -> bool {
    self.list_mcps || self.allows_mcp_connect(mcp_id)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, ToSchema)]
#[schema(example = json!({
    "app_client_id": "my-app-client",
    "flow_type": "redirect",
    "redirect_url": "https://myapp.com/callback",
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
  /// Flow type: "redirect" or "popup"
  pub flow_type: FlowType,
  /// Redirect URL for result notification (required for redirect flow)
  #[validate(custom(function = "validate_redirect_url_scheme"))]
  pub redirect_url: Option<String>,
  /// Role requested for the external app (scope_user_user or scope_user_power_user)
  pub requested_role: crate::UserScope,
  /// Resources requested (tools, etc.)
  pub requested: RequestedResources,
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

fn validate_redirect_url_scheme(url: &str) -> Result<(), validator::ValidationError> {
  match url::Url::parse(url) {
    Ok(parsed) if parsed.scheme() == "http" || parsed.scheme() == "https" => Ok(()),
    _ => Err(validator::ValidationError::new(
      "invalid_redirect_url_scheme",
    )),
  }
}
