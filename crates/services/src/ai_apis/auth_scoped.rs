use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::ai_api_client_factory::{AiApiClientFactory, LibertySource};
use crate::ai_apis::error::Result;
use crate::auth::AuthContext;
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::{Alias, ApiAlias};
use crate::AppService;
use crate::SafeReqwest;
use std::sync::Arc;

/// Auto-injects tenant_id/user_id from AuthContext when building Liberty clients.
/// Aliases must come from `auth_scope.data().find_alias(...)` (already tenant-scoped).
pub struct AuthScopedAiApiClientFactory {
  inner: Arc<dyn AiApiClientFactory>,
  auth_context: AuthContext,
}

impl AuthScopedAiApiClientFactory {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      inner: app_service.ai_api_client_factory(),
      auth_context,
    }
  }

  pub fn for_alias(&self, alias: &Alias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>> {
    self.inner.for_alias(alias, api_key)
  }

  /// Validation-only Liberty client from a freshly-pasted envelope.
  pub fn for_envelope(&self, envelope: &LlmLibertyEnvelope) -> Result<Box<dyn AiApiClient>> {
    self.inner.for_liberty(LibertySource::Envelope(envelope))
  }

  /// Request-time Liberty client from resolved (decrypted) credentials.
  pub fn for_resolved(
    &self,
    creds: &ResolvedLlmLibertyCredentials,
    alias: &ApiAlias,
  ) -> Result<Box<dyn AiApiClient>> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self.inner.for_liberty(LibertySource::Resolved {
      creds,
      alias_id: &alias.id,
      prefix: alias.prefix.clone(),
      tenant_id,
      user_id,
    })
  }

  pub fn safe_http_client(&self) -> SafeReqwest {
    self.inner.safe_http_client()
  }
}

impl std::fmt::Debug for AuthScopedAiApiClientFactory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AuthScopedAiApiClientFactory")
      .field("auth_context", &self.auth_context)
      .finish_non_exhaustive()
  }
}

#[cfg(test)]
#[path = "test_auth_scoped.rs"]
mod test_auth_scoped;
