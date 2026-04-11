use crate::shared::AuthScope;

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
