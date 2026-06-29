use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Model inference grant. `All` is a wildcard that includes models added in the
/// future; `Specific` lists alias ids (empty ⇒ no model access).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelGrant {
  #[default]
  All,
  Specific {
    ids: Vec<String>,
  },
}

impl ModelGrant {
  /// Whether `id` is covered by this grant.
  pub fn allows(&self, id: &str) -> bool {
    match self {
      ModelGrant::All => true,
      ModelGrant::Specific { ids } => ids.iter().any(|m| m == id),
    }
  }
}

/// MCP connect grant. `All` is a wildcard (incl. future MCPs); `Specific` lists
/// the user's own instance ids (empty ⇒ no MCP access).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpGrant {
  #[default]
  All,
  Specific {
    ids: Vec<String>,
  },
}

impl McpGrant {
  /// Whether `id` is covered by this grant.
  pub fn allows(&self, id: &str) -> bool {
    match self {
      McpGrant::All => true,
      McpGrant::Specific { ids } => ids.iter().any(|m| m == id),
    }
  }
}

/// Per-resource access predicates implemented by every grant envelope — the
/// API-token `TokenGrantsV1` and the app-approval `ApprovedResourcesV1`. Lets a
/// single `AccessPolicy` ask both principals the same allow/deny questions.
///
/// Listing (`*_listable`) is separate from inference/connect: with listing off the
/// discovery endpoints return only the individually granted resources, but
/// inference/connect on a granted resource still succeeds.
pub trait ResourceGrants: Send + Sync {
  /// Whether inference on `model_id` is permitted.
  fn allows_model_inference(&self, model_id: &str) -> bool;
  /// Whether `model_id` is visible in listings.
  fn model_listable(&self, model_id: &str) -> bool;
  /// Whether connecting to / invoking MCP instance `mcp_id` is permitted.
  fn allows_mcp_connect(&self, mcp_id: &str) -> bool;
  /// Whether `mcp_id` is visible in listings.
  fn mcp_listable(&self, mcp_id: &str) -> bool;
}
