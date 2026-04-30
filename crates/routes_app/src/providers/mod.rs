use crate::shared::AuthScope;
use services::ai_apis::llm_liberty::{ensure_fresh_credentials, LlmLibertyRefreshError};
use services::models::llm_liberty_envelope::ResolvedLlmLibertyCredentials;

/// Resolve the stored API key for a given alias, returning None if no key is configured
/// or if the lookup fails. Used by oai and anthropic route handlers.
pub(crate) async fn resolve_api_key_for_alias(
  auth_scope: &AuthScope,
  api_alias_id: &str,
) -> Option<String> {
  let tenant_id = auth_scope.tenant_id().unwrap_or("").to_string();
  let user_id = auth_scope
    .auth_context()
    .user_id()
    .unwrap_or("")
    .to_string();
  auth_scope
    .db()
    .get_api_key_for_alias(&tenant_id, &user_id, api_alias_id)
    .await
    .unwrap_or_else(|e| {
      tracing::warn!("Failed to fetch API key for alias {}: {}", api_alias_id, e);
      None
    })
}

/// Resolve fresh LLM Liberty OAuth credentials for the given alias. Delegates to
/// `services::ai_apis::llm_liberty::ensure_fresh_credentials`, which serializes
/// concurrent refreshes for the same alias on a single node via a per-alias mutex.
///
/// Reuses the `AiApiClientFactory`'s shared `SafeReqwest` (Arc-shared internally) instead
/// of building a new one per call — the upstream OAuth-token endpoint is reached
/// from the same connection pool as model traffic.
pub(crate) async fn resolve_llm_liberty_credentials(
  auth_scope: &AuthScope,
  api_alias_id: &str,
) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError> {
  let tenant_id = auth_scope.tenant_id().unwrap_or("").to_string();
  let user_id = auth_scope
    .auth_context()
    .user_id()
    .unwrap_or("")
    .to_string();
  let http = auth_scope.ai_api().safe_http_client();
  let db = auth_scope.db();
  ensure_fresh_credentials(&*db, &http, &tenant_id, &user_id, api_alias_id).await
}
