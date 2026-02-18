use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppAccessRequestStatus {
  Draft,
  Approved,
  Denied,
  Failed,
}

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  strum::Display,
  strum::EnumString,
  Serialize,
  Deserialize,
  ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AccessRequestFlowType {
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
  pub status: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instance: Option<ToolsetInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetInstance {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpServerRequest {
  pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpApproval {
  pub url: String,
  pub status: String,
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
  pub mcp_servers: Vec<McpServerRequest>,
}
