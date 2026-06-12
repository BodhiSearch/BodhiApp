# services Crate Review

## Files Reviewed
- `crates/services/src/mcp_service/service.rs` (1545 lines) - McpService trait, DefaultMcpService implementation with OAuth config CRUD, token management, discovery, DCR, token refresh with concurrency control
- `crates/services/src/mcp_service/error.rs` (144 lines) - McpError and McpServerError domain error enums
- `crates/services/src/db/mcp_repository.rs` (112 lines) - McpRepository trait with auth header, OAuth config, and OAuth token CRUD methods
- `crates/services/src/db/service_mcp.rs` (941 lines) - SQLite implementation of McpRepository
- `crates/services/src/db/objs.rs` (364 lines) - Row types: McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow
- `crates/services/src/db/encryption.rs` (167 lines) - AES-256-GCM encryption with PBKDF2 key derivation
- `crates/services/migrations/0010_mcp_servers.up.sql` - MCP servers and instances tables
- `crates/services/migrations/0011_mcp_auth_headers.up.sql` - Auth headers table
- `crates/services/migrations/0012_mcp_oauth.up.sql` - OAuth configs and tokens tables

## Findings

### Finding 1: PBKDF2 uses only 1000 iterations
- **Priority**: Nice-to-have
- **File**: crates/services/src/db/encryption.rs
- **Location**: `PBKDF2_ITERATIONS` constant (line 13)
- **Issue**: PBKDF2 iteration count is set to 1000. OWASP recommends a minimum of 600,000 iterations for PBKDF2-HMAC-SHA256 as of 2023. While the master key is presumably a strong random key (not a human-chosen password), increasing iterations provides defense-in-depth against key compromise scenarios.
- **Recommendation**: Increase to at least 100,000 iterations. This is a "nice-to-have" per the Q&A constraints. The migration path would require re-encrypting all existing encrypted values, so this should be planned carefully.
- **Rationale**: Low iteration count weakens the key derivation step. Since the master key is likely machine-generated (not a human password), the practical risk is lower, but best practices recommend higher values.

### Finding 2: refresh_locks HashMap grows unboundedly
- **Priority**: Nice-to-have
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: `get_refresh_lock` method (lines 253-267), `refresh_locks` field (line 225)
- **Issue**: The `refresh_locks: RwLock<HashMap<String, Arc<Mutex<()>>>>` grows monotonically. Every unique `oauth_refresh:{config_id}` key creates a new entry that is never removed. Over the lifetime of a long-running server, this could accumulate thousands of entries (one per config_id ever refreshed), each holding an `Arc<Mutex<()>>` in memory.
- **Recommendation**: Add a cleanup mechanism, such as: (a) periodically removing locks for config IDs that have not been accessed recently, (b) using a bounded LRU cache instead of a plain HashMap, or (c) using a weak reference pattern so locks are dropped when not in active use.
- **Rationale**: Memory leak proportional to the number of unique OAuth config IDs over time. The per-config mutex pattern is correct for preventing concurrent refreshes, but the unbounded accumulation is a resource concern for long-running deployments.

### Finding 3: resolve_oauth_token uses get_mcp_oauth_token (user-scoped) but the auth_uuid refers to a config_id
- **Priority**: Critical
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: `resolve_oauth_token` method (lines 428-563), specifically line 440
- **Issue**: The method calls `self.db_service.get_mcp_oauth_token(user_id, auth_uuid)` where `auth_uuid` is the MCP instance's `auth_uuid` field. However, looking at the MCP instance creation flow, `auth_uuid` stores the **config ID** (the `mcp_oauth_configs.id`), not the **token ID** (`mcp_oauth_tokens.id`). The `get_mcp_oauth_token` method queries `WHERE id = ? AND created_by = ?`, meaning it looks for a token whose **primary key** matches `auth_uuid`. But the user's token ID is different from the config ID. This means the lookup would always return `None` unless the token ID happens to match the config ID, which it never will since both are independent UUIDs.

  Looking more carefully: The store_oauth_token method (line 1252) creates a token with a new UUID. The Mcp instance's `auth_uuid` stores the config ID. So when `resolve_oauth_token(user_id, auth_uuid)` is called with `auth_uuid = config_id`, the `get_mcp_oauth_token(user_id, config_id)` call tries to find a token with `id = config_id`, which will not exist. The correct method to use would be `get_latest_oauth_token_by_config(auth_uuid)` which queries by `mcp_oauth_config_id`.

  Wait -- let me re-examine. The `auth_uuid` on the MCP instance could store either a config ID or a token ID depending on how the frontend wires it. Let me check the token exchange flow. In `auth_configs.rs` (routes), `oauth_token_exchange_handler` calls `mcp_service.store_oauth_token(user_id, &config_id, ...)` which creates a token. But the token's ID is not stored back into any MCP instance's `auth_uuid`. The MCP instance's `auth_uuid` would be set by the user during MCP creation/update to the **config ID**.

  So indeed, `resolve_oauth_token` is called with a **config ID** as `auth_uuid`, but `get_mcp_oauth_token` queries by **token primary key** (`WHERE id = ?`). This would always fail to find the token. The method should be using `get_latest_oauth_token_by_config` instead.
- **Recommendation**: Replace `self.db_service.get_mcp_oauth_token(user_id, auth_uuid)` on line 440 with `self.db_service.get_latest_oauth_token_by_config(auth_uuid)`. Note that `get_latest_oauth_token_by_config` is not user-scoped -- consider adding a user-scoped variant or verifying user ownership of the config's token.
- **Rationale**: This is a logic bug that would cause all OAuth token resolution to fail at runtime. The MCP instance's `auth_uuid` stores a config ID, but the lookup treats it as a token ID.

### Finding 4: MCP delete deletes auth_header configs that are admin-managed shared resources
- **Priority**: Important
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: `delete` method (lines 923-950), specifically line 936
- **Issue**: When deleting an MCP instance, if it uses `McpAuthType::Header`, the code calls `self.db_service.delete_mcp_auth_header(auth_uuid)`. However, according to the design doc (03-services-mcp.md) and the update method comments (line 884-886), auth headers are "admin-managed resources that can be reused by other instances." The `update` method correctly does NOT delete auth headers when switching types. But the `delete` method inconsistently DOES delete them.
- **Recommendation**: Remove the `McpAuthType::Header` arm from the delete method's cleanup logic (line 935-937). Auth headers should only be deleted via the explicit `delete_auth_header` / `delete_auth_config` service methods.
- **Rationale**: Deleting an MCP instance should not destroy shared admin-managed resources that other MCP instances might reference. This is inconsistent with the design intent and with the `update` method's behavior.

### Finding 5: No unique constraint on mcp_oauth_tokens per (config_id, user_id)
- **Priority**: Important
- **File**: crates/services/migrations/0012_mcp_oauth.up.sql
- **Location**: `mcp_oauth_tokens` table (lines 24-38)
- **Issue**: The `mcp_oauth_tokens` table has no unique constraint on `(mcp_oauth_config_id, created_by)`. This means a single user can accumulate multiple token rows for the same OAuth config (e.g., by completing the authorization flow multiple times). The `get_latest_oauth_token_by_config` method mitigates this by selecting `ORDER BY created_at DESC LIMIT 1`, but older tokens will accumulate as orphaned rows.
- **Recommendation**: Either (a) add a `UNIQUE(mcp_oauth_config_id, created_by)` constraint using `INSERT OR REPLACE`, or (b) delete existing tokens for the same config+user before inserting a new one in `store_oauth_token`. Option (b) is likely simpler and aligns with the existing `delete_oauth_tokens_by_config` cascade delete pattern.
- **Rationale**: Without cleanup, the token table grows with orphaned rows over time. Each re-authorization creates a new token row without removing the old one.

### Finding 6: get_decrypted_client_secret returns None when there is no secret, not when config not found
- **Priority**: Nice-to-have
- **File**: crates/services/src/db/service_mcp.rs
- **Location**: `get_decrypted_client_secret` method (lines 717-738)
- **Issue**: The method returns `Ok(None)` for two distinct conditions: (1) the config does not exist, and (2) the config exists but has no client_secret (public client). The callers (token refresh in service.rs line 465-480, token exchange in auth_configs.rs line 258-287) handle `None` by falling back to sending only `client_id`. This is correct behavior for public clients (no secret), but it silently succeeds if the config was deleted between the start of the flow and the credential lookup.
- **Recommendation**: Consider splitting the return type to distinguish "config not found" from "no secret configured," or document that the caller must ensure the config exists before calling this method (which the callers currently do).
- **Rationale**: The ambiguity is unlikely to cause a bug in practice because callers fetch the config first, but it could be confusing for future maintainers.

### Finding 7: Token refresh does not log operations
- **Priority**: Nice-to-have
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: `resolve_oauth_token` method (lines 428-563)
- **Issue**: The token refresh flow (acquiring lock, detecting expiration, performing HTTP refresh, updating database) has no logging at any level. The Q&A specified that INFO logging should be present for OAuth operations. There are no `tracing::info!`, `tracing::debug!`, or `tracing::warn!` calls in the resolve/refresh path. A failed refresh only returns an error; a successful refresh leaves no trace in logs.
- **Recommendation**: Add `tracing::info!` for successful token refreshes (including config_id), `tracing::warn!` for expired tokens with no refresh token, and `tracing::debug!` for skipped refreshes (token still valid).
- **Rationale**: OAuth token refresh is a critical background operation that affects user experience. Without logging, diagnosing token refresh failures in production requires adding debug logging after the fact.

### Finding 8: Discovery and DCR also have no logging
- **Priority**: Nice-to-have
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: `discover_oauth_metadata` (lines 1308-1334), `discover_mcp_oauth_metadata` (lines 1336-1407), `dynamic_register_client` (lines 1409-1450)
- **Issue**: Same as Finding 7. These methods make external HTTP calls but have no logging for debugging.
- **Recommendation**: Add `tracing::info!` for the URLs being fetched and the success/failure of each step.
- **Rationale**: External HTTP calls are a common source of failures. Logging the request URL and response status aids debugging.

### Finding 9: Error enums follow domain convention correctly
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/services/src/mcp_service/error.rs
- **Location**: Entire file
- **Issue**: None. Both `McpServerError` and `McpError` follow the `errmeta_derive::ErrorMeta` pattern correctly with appropriate `ErrorType` mappings. The `From<EncryptionError>` and `From<McpClientError>` manual impls bridge external errors into domain-specific variants. Error messages follow the convention of sentence case ending with a period.
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation that the error handling architecture follows the crate conventions.

### Finding 10: TimeService is used correctly everywhere
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: All timestamp usage (lines 444, 630, 698, 805, 902, 977, 1086, 1140, 1195, 1271)
- **Issue**: None. All timestamp creation uses `self.time_service.utc_now().timestamp()`. No direct `Utc::now()` calls found.
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation of adherence to the TimeService pattern required by the crate's CLAUDE.md.

### Finding 11: Encryption implementation uses per-field salt and nonce correctly
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/services/src/db/encryption.rs
- **Location**: `encrypt_api_key` (lines 49-66), `generate_salt` (lines 31-35), `generate_nonce` (lines 37-41)
- **Issue**: None. Each encryption operation generates a unique random 32-byte salt and 12-byte nonce. The AES-256-GCM implementation correctly derives per-field keys from the master key using PBKDF2. The salt, nonce, and ciphertext are all Base64-encoded for database storage. Test coverage verifies roundtrip, different-salt uniqueness, and wrong-key rejection.
- **Recommendation**: No changes needed (except the iteration count noted in Finding 1).
- **Rationale**: The encryption design is sound. Per-field salt/nonce prevents nonce reuse attacks across fields.

### Finding 12: Migration 0012 missing IF NOT EXISTS on indexes
- **Priority**: Nice-to-have
- **File**: crates/services/migrations/0012_mcp_oauth.up.sql
- **Location**: Lines 40-41
- **Issue**: The `CREATE INDEX` statements on lines 40-41 do not use `IF NOT EXISTS`, unlike the other indexes in migration 0011 (line 19, 22) and migration 0010 (line 18). This means the migration will fail if re-run on a database where these indexes already exist. Migration 0011 inconsistently uses `IF NOT EXISTS` on some but not all index statements.
- **Recommendation**: Add `IF NOT EXISTS` to the two index creation statements on lines 40-41 for consistency and idempotency.
- **Rationale**: While migrations are typically run once, using `IF NOT EXISTS` provides a safety net and maintains consistency with other migrations in the project.

### Finding 13: delete_mcp_oauth_config does not cascade via FK -- relies on application-level cascade
- **Priority**: Nice-to-have
- **File**: crates/services/src/mcp_service/service.rs
- **Location**: `delete_oauth_config` method (lines 1244-1248)
- **Issue**: The service layer manually deletes tokens before deleting the config (`delete_oauth_tokens_by_config` then `delete_mcp_oauth_config`). SQLite FK constraints reference `mcp_oauth_configs(id)` from `mcp_oauth_tokens.mcp_oauth_config_id`, but SQLite does not enforce FK constraints by default unless `PRAGMA foreign_keys = ON` is explicitly set. The application-level cascade is the correct approach since FK enforcement cannot be relied upon. However, if `delete_oauth_tokens_by_config` fails, the config deletion proceeds, potentially orphaning the config if it has FK-dependent tokens.
- **Recommendation**: Wrap the two deletions in a single database transaction to ensure atomicity. If token deletion fails, the config should not be deleted either.
- **Rationale**: Without a transaction, a failure between the two operations could leave the database in an inconsistent state where tokens reference a non-existent config.
