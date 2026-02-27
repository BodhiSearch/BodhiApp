# Review 3: Repository Implementations

## Files Reviewed
- `crates/services/src/db/service_mcp.rs` (590 lines)
- `crates/services/src/db/service_model.rs` (438 lines)
- `crates/services/src/db/service_access.rs`, `service_access_request.rs`, `service_app_instance.rs`, `service_token.rs`, `service_toolset.rs`, `service_user_alias.rs`, `service_settings.rs`
- `crates/services/src/db/default_service.rs`, `error.rs`, `objs.rs`, `service.rs`

## Findings

### [Deferred] Missing transaction wrapping for multi-step MCP operations at service layer
- **File**: `crates/services/src/db/service_mcp.rs`
- **Location**: Individual create methods (create_mcp_server line 15, create_mcp line 122, create_mcp_auth_header line 270, create_mcp_oauth_config line 358)
- **Issue**: Each repository method operates independently without transaction coordination. If multi-step creation fails midway, the database is left inconsistent. The only method using transactions is `delete_oauth_config_cascade` (lines 417-433).
- **Status**: Deferred — not in current scope.

### [Important] MCP auth header/OAuth encrypted fields leak to callers via list operations
- **File**: `crates/services/src/db/service_mcp.rs`
- **Location**: `list_mcp_auth_headers_by_server` (lines 324-335), `list_mcp_oauth_configs_by_server` (lines 396-407)
- **Issue**: List operations return full Row types including encrypted ciphertext fields (`encrypted_header_value`, `encrypted_client_secret`, etc.) to all callers. Contrasts with `api_model_alias` which uses `ApiAliasView` via `into_partial_model::<ApiAliasView>()` to exclude encrypted fields (see `service_model.rs` lines 145-153, 233-241).
- **Recommendation**: Create View structs (DerivePartialModel) for MCP auth entities excluding encrypted columns. Use `into_partial_model()` in list/get methods.
- **Dependency**: Requires I-1 (MCP View structs in entities/) to be done first.

### [Nice-to-have] create_toolset returns input clone instead of inserted model
- **File**: `crates/services/src/db/service_toolset.rs`
- **Location**: Line 59
- **Issue**: `create_toolset()` returns `Ok(row.clone())` after insert instead of converting the inserted model via `From<Model>`. Since all fields are explicitly set (no DB-generated defaults), the returned value is identical. However, other create methods (like `create_mcp_server`) return `From<Model>` on the inserted result, creating an inconsistency.
- **Recommendation**: Return `ToolsetRow::from(model)` for consistency, where `model` is the insert result. This would also catch any future DB-side transformations.

### [Nice-to-have] SqlxError wrapper still present in error.rs
- **File**: `crates/services/src/db/error.rs`
- **Location**: Lines 3-18
- **Issue**: `SqlxError` struct wraps `sqlx::Error` — a remnant from the pre-SeaORM era. It's still used by the session service which uses sqlx directly, so it's not dead code. However, it's a SeaORM migration leftover that ties the DB error module to two ORMs.
- **Recommendation**: When session service eventually migrates to SeaORM, remove `SqlxError` and the `sqlx` dependency from this module.
