//! Shared test helper factories for MCP repository tests.
use crate::db::encryption::encrypt_api_key;
use crate::mcps::{McpAuthHeaderRow, McpAuthType, McpRow, McpServerRow};
use chrono::DateTime;
use chrono::Utc;

pub(crate) const ENCRYPTION_KEY: &[u8] = b"01234567890123456789012345678901";

pub(crate) fn make_server(id: &str, url: &str, now: DateTime<Utc>) -> McpServerRow {
  McpServerRow {
    id: id.to_string(),
    url: url.to_string(),
    name: format!("Server {}", id),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: now,
    updated_at: now,
  }
}

pub(crate) fn make_mcp(
  id: &str,
  server_id: &str,
  slug: &str,
  user_id: &str,
  now: DateTime<Utc>,
) -> McpRow {
  McpRow {
    id: id.to_string(),
    created_by: user_id.to_string(),
    mcp_server_id: server_id.to_string(),
    name: format!("MCP {}", id),
    slug: slug.to_string(),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: McpAuthType::Public,
    auth_uuid: None,
    created_at: now,
    updated_at: now,
  }
}

pub(crate) fn make_auth_header_row(
  id: &str,
  server_id: &str,
  now: DateTime<Utc>,
) -> McpAuthHeaderRow {
  let (encrypted, salt, nonce) =
    encrypt_api_key(ENCRYPTION_KEY, "Bearer sk-secret-token-123").expect("encryption failed");
  McpAuthHeaderRow {
    id: id.to_string(),
    name: "Header".to_string(),
    mcp_server_id: server_id.to_string(),
    header_key: "Authorization".to_string(),
    encrypted_header_value: encrypted,
    header_value_salt: salt,
    header_value_nonce: nonce,
    created_by: "user-1".to_string(),
    created_at: now,
    updated_at: now,
  }
}
