/// Session key constants and namespaced key format functions.
///
/// Global session keys (not namespaced by tenant):
/// - `user_id` — the authenticated user's ID
/// - `active_client_id` — currently selected tenant's client_id
///
/// Dashboard session keys (multi-tenant dashboard):
/// - `dashboard:access_token`
/// - `dashboard:refresh_token`
///
/// Tenant-namespaced keys use `{client_id}:<key_type>` format:
/// - `{client_id}:access_token`
/// - `{client_id}:refresh_token`
///
/// Lock keys use `{client_id}:{session_id}:<lock_type>` format.

pub const SESSION_KEY_USER_ID: &str = "user_id";
pub const SESSION_KEY_ACTIVE_CLIENT_ID: &str = "active_client_id";
pub const DASHBOARD_ACCESS_TOKEN_KEY: &str = "dashboard:access_token";
pub const DASHBOARD_REFRESH_TOKEN_KEY: &str = "dashboard:refresh_token";

/// Returns the namespaced session key for an access token: `{client_id}:access_token`
pub fn access_token_key(client_id: &str) -> String {
  format!("{client_id}:access_token")
}

/// Returns the namespaced session key for a refresh token: `{client_id}:refresh_token`
pub fn refresh_token_key(client_id: &str) -> String {
  format!("{client_id}:refresh_token")
}
