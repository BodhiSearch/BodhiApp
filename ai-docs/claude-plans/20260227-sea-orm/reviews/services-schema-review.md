# Review 2: Entities + Migrations

## Files Reviewed

**Entity files**: access_request.rs, api_model_alias.rs, api_token.rs, app_access_request.rs, app_instance.rs, app_toolset_config.rs, download_request.rs, mcp.rs, mcp_auth_header.rs, mcp_oauth_config.rs, mcp_oauth_token.rs, mcp_server.rs, model_metadata.rs, setting.rs, toolset.rs, user_alias.rs, mod.rs

**Migration files**: m20250101_000000_extensions.rs through m20250101_000013_apps.rs, mod.rs

## Findings

### [Important] MCP encrypted entities don't use Pattern B views for read operations
- **File**: `crates/services/src/db/entities/mcp_auth_header.rs`, `crates/services/src/db/entities/mcp_oauth_config.rs`, `crates/services/src/db/entities/mcp_oauth_token.rs`
- **Location**: `From<Model>` impls in each file
- **Issue**: The implementation-conventions.md (line 366-368) planned for these entities to use Pattern B to "hide encrypted columns from domain struct." However, the current implementation uses `From<Model>` that copies ALL fields including encrypted ones (`encrypted_header_value`, `header_value_salt`, `header_value_nonce`, etc.) directly to the Row types. By contrast, `api_model_alias.rs` properly implements Pattern B using `ApiAliasView` (a `DerivePartialModel` struct) that excludes encrypted fields from read operations.
- **Recommendation**: Create `McpAuthHeaderView`, `McpOAuthConfigView`, `McpOAuthTokenView` structs (using `DerivePartialModel`) that exclude encrypted columns. Use these views in list/get repository methods. Keep full Model access only for create/update and decrypt operations.
- **Cross-layer dependency**: Requires changes in both `entities/` (new View structs) and `service_mcp.rs` (use Views in list/get methods).
- **Reference**: `entities/api_model_alias.rs:34-48` (ApiAliasView) and `service_model.rs:145-153` (usage of `into_partial_model`)

### [Important] if_not_exists() on all tables and indexes should be removed
- **File**: All migration files (m000001 through m000013)
- **Location**: Every `Table::create().if_not_exists()` and `Index::create().if_not_exists()` call
- **Issue**: The `if_not_exists()` was needed during the transition period when both sqlx and SeaORM migrations co-existed. Now that sqlx migrations are deleted, `if_not_exists()` masks migration ordering bugs and should be removed.
- **Recommendation**: Remove `if_not_exists()` from all `Table::create()` and `Index::create()` calls across all 14 migration files.
- **Locations**: m000001 (line 28, line 46), m000002 (line 31, lines 91/101/111), m000003, m000004, m000005, m000006 (lines 40/94, lines 57-88), m000007, m000008, m000009 (lines 45/63, lines 90-139), m000010 (line 34, line 60), m000011 (lines 64/105, lines 132-171), m000012, m000013.
