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
  pub status: String, // "approved" | "denied"
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instance_id: Option<String>, // Present if status == "approved"
}
