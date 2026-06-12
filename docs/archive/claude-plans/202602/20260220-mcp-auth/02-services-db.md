# MCP OAuth - Database Layer (`services` crate)

## Task

✅ **COMPLETED** - Add SQLite migrations, row types, and repository trait methods for MCP auth headers, OAuth configs, and OAuth tokens with AES-256-GCM encryption.

**Note**: This task was already completed in previous work. The OAuth refactor primarily impacted the `mcps` table auth_type enum values.

## Files

| File | Purpose |
|------|---------|
| `crates/services/migrations/0011_mcp_auth_headers.up.sql` | Auth headers table |
| `crates/services/migrations/0012_mcp_oauth.up.sql` | OAuth configs + tokens tables |
| `crates/services/src/db/objs.rs` | Row types: McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow |
| `crates/services/src/db/mcp_repository.rs` | McpRepository trait with all CRUD + decryption methods |
| `crates/services/src/db/service_mcp.rs` | SQLite implementation of McpRepository |
| `crates/services/src/db/encryption.rs` | AES-256-GCM encrypt/decrypt functions |
| `crates/services/src/db/service.rs` | Test cleanup: reset_all_tables() updated for new tables |
| `crates/services/src/db/test_mcp_repository.rs` | Repository tests |

## Migration 0010: mcp_servers (Updated)

✅ **UPDATED** - The `mcps` table's `auth_type` column comment updated from:
```sql
-- OLD: auth_type: 'public', 'header', 'oauth-pre-registered'
-- NEW: auth_type: 'public', 'header', 'oauth'
```

This aligns with the simplified 3-variant `McpAuthType` enum in the objs crate.

**Git diff from crates/services/migrations/0010_mcp_servers.up.sql:**
```sql
-    auth_type TEXT NOT NULL DEFAULT 'public', -- auth type: 'public', 'header', 'oauth-pre-registered'
+    auth_type TEXT NOT NULL DEFAULT 'public', -- auth type: 'public', 'header', 'oauth'
```

## Migration 0011: mcp_auth_headers

✅ **ALREADY COMPLETED** (pre-refactor) - Creates `mcp_auth_headers` table with columns: `id` (PK), `name`, `mcp_server_id` (FK to mcp_servers), `header_key`, `encrypted_header_value`, `header_value_salt`, `header_value_nonce`, `created_by`, `created_at`, `updated_at`.

Indexes: unique on `(mcp_server_id, name COLLATE NOCASE)`, non-unique on `mcp_server_id`.

## Migration 0012: mcp_oauth_configs + mcp_oauth_tokens

✅ **ALREADY COMPLETED** (pre-refactor) - **mcp_oauth_configs** (20 columns): `id` (PK), `name`, `mcp_server_id` (FK), `registration_type` (default `'pre-registered'`), `client_id`, `encrypted_client_secret`/`_salt`/`_nonce` (nullable 3-column pattern), `authorization_endpoint`, `token_endpoint`, `registration_endpoint` (nullable), `encrypted_registration_access_token`/`_salt`/`_nonce` (nullable), `client_id_issued_at` (nullable), `token_endpoint_auth_method` (nullable), `scopes` (nullable), `created_by`, `created_at`, `updated_at`.

**mcp_oauth_tokens** (13 columns): `id` (PK), `mcp_oauth_config_id` (FK to mcp_oauth_configs), `encrypted_access_token`/`_salt`/`_nonce`, `encrypted_refresh_token`/`_salt`/`_nonce` (nullable), `scopes_granted` (nullable), `expires_at` (nullable), `created_by`, `created_at`, `updated_at`.

Indexes: `idx_mcp_oauth_configs_server`, `idx_mcp_oauth_tokens_config`, unique on `(mcp_server_id, name COLLATE NOCASE)` for configs.

**Key insight**: The `registration_type` column already distinguished pre-registered from dynamic OAuth clients, making separate enum variants unnecessary.

## Row Types (objs.rs)

✅ **NO CHANGES REQUIRED** - Row types were already correct and didn't need updates for the enum simplification.

**McpAuthHeaderRow**: 10 fields mapping 1:1 to migration columns. Derives `Debug, Clone, PartialEq`.

**McpOAuthConfigRow**: 20 fields mapping 1:1 to migration columns. Encryption fields are `Option<String>` (nullable for public clients). The `registration_type` field carries `"pre-registered"` or `"dynamic"`.

**McpOAuthTokenRow**: 13 fields. Access token encryption fields are required (`String`); refresh token fields are `Option<String>`.

## McpRepository Trait (mcp_repository.rs)

✅ **NO CHANGES REQUIRED** - Repository methods already existed and work with simplified enum.

13 async methods organized by resource:

### Auth Headers (6 methods)
- `create_mcp_auth_header(&self, row) -> Result<McpAuthHeaderRow>`
- `get_mcp_auth_header(&self, id) -> Result<Option<McpAuthHeaderRow>>`
- `update_mcp_auth_header(&self, row) -> Result<McpAuthHeaderRow>`
- `delete_mcp_auth_header(&self, id) -> Result<()>`
- `list_mcp_auth_headers_by_server(&self, mcp_server_id) -> Result<Vec<McpAuthHeaderRow>>`
- `get_decrypted_auth_header(&self, id) -> Result<Option<(String, String)>>` - returns `(header_key, decrypted_value)`

### OAuth Configs (5 methods)
- `create_mcp_oauth_config(&self, row) -> Result<McpOAuthConfigRow>`
- `get_mcp_oauth_config(&self, id) -> Result<Option<McpOAuthConfigRow>>`
- `list_mcp_oauth_configs_by_server(&self, mcp_server_id) -> Result<Vec<McpOAuthConfigRow>>`
- `delete_mcp_oauth_config(&self, id) -> Result<()>`
- `get_decrypted_client_secret(&self, id) -> Result<Option<(String, String)>>` - returns `(client_id, decrypted_secret)`

### OAuth Tokens (7 methods)
- `create_mcp_oauth_token(&self, row) -> Result<McpOAuthTokenRow>`
- `get_mcp_oauth_token(&self, user_id, id) -> Result<Option<McpOAuthTokenRow>>` - user-scoped
- `get_latest_oauth_token_by_config(&self, config_id) -> Result<Option<McpOAuthTokenRow>>` - not user-scoped (for refresh)
- `update_mcp_oauth_token(&self, row) -> Result<McpOAuthTokenRow>`
- `delete_mcp_oauth_token(&self, user_id, id) -> Result<()>` - user-scoped
- `delete_oauth_tokens_by_config(&self, config_id) -> Result<()>` - cascade delete
- `get_decrypted_oauth_bearer(&self, id) -> Result<Option<(String, String)>>` - returns `("Authorization", "Bearer <token>")`, not user-scoped

## Encryption (encryption.rs)

Two functions using PBKDF2-HMAC-SHA256 key derivation + AES-256-GCM:

- `encrypt_api_key(master_key, plaintext) -> Result<(encrypted_b64, salt_b64, nonce_b64)>`
- `decrypt_api_key(master_key, encrypted, salt, nonce) -> Result<String>`

Parameters: 32-byte salt, 12-byte nonce, 1000 PBKDF2 iterations. All values Base64-encoded for DB storage.

## Test Cleanup (service.rs)

`reset_all_tables()` updated to delete in FK-safe order: `mcp_oauth_tokens` before `mcp_oauth_configs` before `mcp_auth_headers` before `mcp_servers`.

## Test Coverage (test_mcp_repository.rs)

Test fixture builders: `test_mcp_server_row()`, `test_oauth_config_row()`, `test_oauth_token_row()` with encryption.

Auth header tests (5): decrypt roundtrip, missing header returns None, MCP with auth_uuid, public MCP without auth_uuid, full CRUD.

OAuth config tests (4): create+read, list by server (ordered DESC), delete, decrypt client secret.

OAuth token tests (3): create+read, get latest by config (ordered DESC), cascade delete by config.

## Cross-References

- Domain types these rows map to: [01-objs.md](./01-objs.md)
- Service methods that use this repository: [03-services-mcp.md](./03-services-mcp.md)
