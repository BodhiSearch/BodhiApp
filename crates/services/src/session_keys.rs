/// Lock keys (not modeled as constants here) use `{client_id}:{session_id}:<lock_type>` format.

pub const SESSION_KEY_USER_ID: &str = "user_id";
pub const SESSION_KEY_ACTIVE_CLIENT_ID: &str = "active_client_id";
pub const DASHBOARD_ACCESS_TOKEN_KEY: &str = "dashboard:access_token";
pub const DASHBOARD_REFRESH_TOKEN_KEY: &str = "dashboard:refresh_token";

pub fn access_token_key(client_id: &str) -> String {
  format!("{client_id}:access_token")
}

pub fn refresh_token_key(client_id: &str) -> String {
  format!("{client_id}:refresh_token")
}

pub fn id_token_key(client_id: &str) -> String {
  format!("{client_id}:id_token")
}
