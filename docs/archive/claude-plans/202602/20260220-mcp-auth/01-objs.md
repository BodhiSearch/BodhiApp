# MCP OAuth - Domain Objects (`objs` crate)

## Task

✅ **COMPLETED** - Define domain types for MCP authentication: auth type enum, OAuth config/token models, discriminated union request/response types, and validation.

## File

All types in `crates/objs/src/mcp.rs`, exported via `pub use mcp::*` in `lib.rs`.

## Types

### McpAuthType (enum)

✅ **IMPLEMENTED** - 3-variant enum (simplified from planned 4 variants) with `#[serde(rename_all = "kebab-case")]` and `#[derive(Default)]` (defaults to `Public`).

| Variant | JSON | Purpose | Status |
|---------|------|---------|--------|
| `Public` (default) | `"public"` | No authentication | ✅ |
| `Header` | `"header"` | Static header key/value | ✅ |
| `Oauth` | `"oauth"` | OAuth authentication (pre-registered or dynamic) | ✅ |

**Key Change**: Collapsed `OauthPreRegistered` and `OauthDynamic` into single `Oauth` variant. The `registration_type` field in `McpOAuthConfig` distinguishes pre-registered vs dynamic registration.

Implements `Display`, `FromStr`, `as_str() -> &'static str`. Used as `ToSchema` for OpenAPI.

```rust
// Actual implementation from crates/objs/src/mcp.rs
impl McpAuthType {
  pub fn as_str(&self) -> &'static str {
    match self {
      McpAuthType::Public => "public",
      McpAuthType::Header => "header",
      McpAuthType::Oauth => "oauth",
    }
  }
}
```

### Mcp (struct) - Modified

✅ **IMPLEMENTED** - Two new fields added to the existing `Mcp` struct:

- `auth_type: McpAuthType` - authentication mechanism for this MCP instance (Public/Header/Oauth)
- `auth_uuid: Option<String>` - reference to auth config ID (from `mcp_auth_headers` or `mcp_oauth_configs` table), skip_serializing_if None

### McpAuthHeader (struct)

✅ **IMPLEMENTED** - Public API model for header-based auth config. Secrets masked: actual header value replaced by `has_header_value: bool`.

Fields: `id`, `name`, `mcp_server_id`, `header_key`, `has_header_value`, `created_by`, `created_at`, `updated_at`

### McpOAuthConfig (struct)

✅ **IMPLEMENTED** - Public API model for OAuth config. Secrets masked via boolean flags.

Required fields: `id`, `name`, `mcp_server_id`, `registration_type` (String: `"pre-registered"` or `"dynamic"`), `client_id`, `authorization_endpoint`, `token_endpoint`, `has_client_secret`, `has_registration_access_token`, `created_by`, `created_at`, `updated_at`

Optional fields (skip_serializing_if None): `registration_endpoint`, `client_id_issued_at`, `token_endpoint_auth_method`, `scopes`

### McpOAuthToken (struct)

✅ **IMPLEMENTED** - Public API model for OAuth token. Secrets masked via boolean flags.

Required fields: `id`, `mcp_oauth_config_id`, `has_access_token`, `has_refresh_token`, `created_by`, `created_at`, `updated_at`

Optional fields (skip_serializing_if None): `scopes_granted`, `expires_at`

### CreateMcpAuthConfigRequest (discriminated union enum)

✅ **IMPLEMENTED** - Tagged with `#[serde(tag = "type", rename_all = "kebab-case")]`. Two variants (simplified from planned three):

**Header** (`"type": "header"`): `name`, `header_key`, `header_value` (plaintext in request)

**Oauth** (`"type": "oauth"`): Unified OAuth variant with `registration_type` field:
- Required: `name`, `client_id`, `authorization_endpoint`, `token_endpoint`
- Optional (with `#[serde(default)]`): `client_secret`, `scopes`, `registration_access_token`, `registration_endpoint`, `token_endpoint_auth_method`, `client_id_issued_at`
- `registration_type`: `"pre-registered"` (default) or `"dynamic-registration"`

```rust
// Actual implementation from crates/objs/src/mcp.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CreateMcpAuthConfigRequest {
  Header {
    name: String,
    header_key: String,
    header_value: String,
  },
  Oauth {
    name: String,
    client_id: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    scopes: Option<String>,
    #[serde(default = "default_registration_type")]
    registration_type: String,
    #[serde(default)]
    registration_access_token: Option<String>,
    #[serde(default)]
    registration_endpoint: Option<String>,
    #[serde(default)]
    token_endpoint_auth_method: Option<String>,
    #[serde(default)]
    client_id_issued_at: Option<i64>,
  },
}

fn default_registration_type() -> String {
  "pre-registered".to_string()
}
```

### McpAuthConfigResponse (discriminated union enum)

✅ **IMPLEMENTED** - Tagged with `#[serde(tag = "type", rename_all = "kebab-case")]`. Two variants mirroring `CreateMcpAuthConfigRequest`:
- IDs and timestamps added
- Secrets replaced with boolean flags
- `mcp_server_id` and `created_by` included

**Header variant**: `id`, `name`, `mcp_server_id`, `header_key`, `has_header_value`, `created_by`, `created_at`, `updated_at`

**Oauth variant**: `id`, `name`, `mcp_server_id`, `registration_type`, `client_id`, `authorization_endpoint`, `token_endpoint`, optional fields (`registration_endpoint`, `scopes`, `client_id_issued_at`, `token_endpoint_auth_method`), `has_client_secret`, `has_registration_access_token`, `created_by`, `created_at`, `updated_at`

Accessor methods: `id() -> &str`, `mcp_server_id() -> &str`

`From<McpAuthHeader>` converts to `Header` variant. `From<McpOAuthConfig>` converts to `Oauth` variant (simplified from conditional logic).

```rust
// Simplified conversion from crates/objs/src/mcp.rs
impl From<McpOAuthConfig> for McpAuthConfigResponse {
  fn from(o: McpOAuthConfig) -> Self {
    McpAuthConfigResponse::Oauth {
      id: o.id,
      name: o.name,
      mcp_server_id: o.mcp_server_id,
      registration_type: o.registration_type, // preserves "pre-registered" or "dynamic"
      client_id: o.client_id,
      // ... all fields from McpOAuthConfig
    }
  }
}
```

### McpAuthConfigsListResponse (struct)

✅ **IMPLEMENTED** - Wrapper: `auth_configs: Vec<McpAuthConfigResponse>`. Non-paginated, uses resource-plural field name per project convention.

## Validation

✅ **IMPLEMENTED** - **Constant**: `MAX_MCP_AUTH_CONFIG_NAME_LEN = 100`

**Function**: `validate_mcp_auth_config_name(name: &str) -> Result<(), String>` - rejects empty and names exceeding 100 chars.

Tests at bottom of `mcp.rs`: `test_validate_mcp_auth_config_name_accepts_valid`, `_rejects_empty`, `_rejects_too_long`.

## Implementation Summary

**Key Architectural Decision**: Simplified `McpAuthType` from 4 variants to 3 by merging OAuth types:
- **Original plan**: `Public`, `Header`, `OauthPreRegistered`, `OauthDynamic`
- **Implemented**: `Public`, `Header`, `Oauth`

The `registration_type` field in `CreateMcpAuthConfigRequest::Oauth` and `McpAuthConfigResponse::Oauth` distinguishes between pre-registered (`"pre-registered"`) and dynamic registration (`"dynamic-registration"`) OAuth clients.

**Benefits**:
- Simpler enum reduces match statement complexity across codebase
- Database stores single `"oauth"` auth_type value instead of two separate types
- Frontend logic simplified with unified OAuth handling
- registration_type field provides same information at data model level

## Test Utilities

`objs::test_utils::fixed_dt() -> DateTime<Utc>` — returns the fixed deterministic timestamp `2025-01-01T00:00:00Z` matching `FrozenTimeService`'s default. Used by downstream test helpers (e.g., `routes_app::test_utils::fixed_dt`) to construct expected values without duplicating the constant.

## Cross-References

- Row types mapping these to DB columns: [02-services-db.md](./02-services-db.md)
- Service methods consuming these types: [03-services-mcp.md](./03-services-mcp.md)
- Route DTOs that wrap/convert these: [04-routes-app.md](./04-routes-app.md)
