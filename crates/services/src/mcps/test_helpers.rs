//! Shared test helper factories for MCP repository tests.
use crate::db::encryption::encrypt_api_key;
use crate::db::EncryptionKeys;
use crate::mcps::{
  McpAuthConfigEntity, McpAuthConfigParamEntity, McpAuthParamEntity, McpAuthType, McpEntity,
  McpServerEntity,
};
use crate::test_utils::{test_encryption_keys, TEST_TENANT_ID};
use chrono::DateTime;
use chrono::Utc;

pub(crate) fn encryption_keys() -> EncryptionKeys {
  test_encryption_keys()
}

pub(crate) fn make_server(id: &str, url: &str, now: DateTime<Utc>) -> McpServerEntity {
  McpServerEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
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
) -> McpEntity {
  McpEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: user_id.to_string(),
    mcp_server_id: server_id.to_string(),
    name: format!("MCP {}", id),
    slug: slug.to_string(),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    created_at: now,
    updated_at: now,
  }
}

pub(crate) fn make_auth_config_row(
  id: &str,
  server_id: &str,
  config_type: &str,
  now: DateTime<Utc>,
) -> McpAuthConfigEntity {
  McpAuthConfigEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    mcp_server_id: server_id.to_string(),
    config_type: config_type.to_string(),
    name: format!("Config {}", id),
    created_by: "admin".to_string(),
    created_at: now,
    updated_at: now,
  }
}

pub(crate) fn make_auth_config_param_row(
  id: &str,
  auth_config_id: &str,
  param_type: &str,
  param_key: &str,
  now: DateTime<Utc>,
) -> McpAuthConfigParamEntity {
  McpAuthConfigParamEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    auth_config_id: auth_config_id.to_string(),
    param_type: param_type.to_string(),
    param_key: param_key.to_string(),
    created_at: now,
    updated_at: now,
  }
}

pub(crate) fn make_auth_param_row(
  id: &str,
  mcp_id: &str,
  param_type: &str,
  param_key: &str,
  value: &str,
  now: DateTime<Utc>,
) -> McpAuthParamEntity {
  let (encrypted, salt, nonce) =
    encrypt_api_key(&encryption_keys(), value).expect("encryption failed");
  McpAuthParamEntity {
    id: id.to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    mcp_id: mcp_id.to_string(),
    param_type: param_type.to_string(),
    param_key: param_key.to_string(),
    encrypted_value: encrypted,
    value_salt: salt,
    value_nonce: nonce,
    created_at: now,
    updated_at: now,
  }
}
