//! Per-resource access enforcement for the current request principal.
//!
//! `AccessPolicy` is the single place that maps an `AuthContext` to allow/deny
//! decisions. Handlers obtain it via [`crate::AuthScope::access_policy`] and ask
//! uniform questions — they never match on `AuthContext` or grant internals.
//!
//! Only `AuthContext::ApiToken` is grant-restricted. Sessions are `Unrestricted`,
//! and `ExternalApp` keeps its own access-request enforcement (`mcps_index`) — so
//! it is `Unrestricted` here too. The actual grant logic lives on
//! `TokenGrantsV1` (the domain owns it); this type only resolves the principal
//! and turns a denial into the right HTTP error.

use services::{AppError, AuthContext, ErrorType, ResourceGrants, TokenGrantsV1};

/// Effective resource-access policy for the current principal.
pub enum AccessPolicy<'a> {
  /// No grant restrictions (session / external-app).
  Unrestricted,
  /// API token restricted to its grants.
  Token(&'a TokenGrantsV1),
}

impl<'a> AccessPolicy<'a> {
  /// Resolve the policy from the request's auth context.
  pub fn of(ctx: &'a AuthContext) -> Self {
    match ctx {
      AuthContext::ApiToken { grants, .. } => AccessPolicy::Token(grants.v1()),
      _ => AccessPolicy::Unrestricted,
    }
  }

  /// Whether `model_id` should appear in model listings.
  pub fn model_listable(&self, model_id: &str) -> bool {
    match self {
      AccessPolicy::Unrestricted => true,
      AccessPolicy::Token(grants) => grants.model_listable(model_id),
    }
  }

  /// Whether `mcp_id` should appear in MCP listings.
  pub fn mcp_listable(&self, mcp_id: &str) -> bool {
    match self {
      AccessPolicy::Unrestricted => true,
      AccessPolicy::Token(grants) => grants.mcp_listable(mcp_id),
    }
  }

  /// Guard inference on `model_id`; `Err(ModelForbidden)` (403) when not granted.
  pub fn ensure_model_inference(&self, model_id: &str) -> Result<(), TokenGrantError> {
    match self {
      AccessPolicy::Unrestricted => Ok(()),
      AccessPolicy::Token(grants) if grants.allows_model_inference(model_id) => Ok(()),
      AccessPolicy::Token(_) => Err(TokenGrantError::ModelForbidden(model_id.to_string())),
    }
  }

  /// Guard connecting to / invoking MCP `mcp_id`; `Err(McpForbidden)` (403) when not granted.
  pub fn ensure_mcp_connect(&self, mcp_id: &str) -> Result<(), TokenGrantError> {
    match self {
      AccessPolicy::Unrestricted => Ok(()),
      AccessPolicy::Token(grants) if grants.allows_mcp_connect(mcp_id) => Ok(()),
      AccessPolicy::Token(_) => Err(TokenGrantError::McpForbidden(mcp_id.to_string())),
    }
  }
}

/// Forbidden errors raised when a token addresses a resource outside its grant.
/// Converts to a 403 in every wire envelope via the blanket `From<AppError>`.
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum TokenGrantError {
  #[error("API token does not have access to model '{0}'.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  ModelForbidden(String),
  #[error("API token does not have access to MCP '{0}'.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  McpForbidden(String),
}

#[cfg(test)]
#[path = "test_token_grants.rs"]
mod test_token_grants;
