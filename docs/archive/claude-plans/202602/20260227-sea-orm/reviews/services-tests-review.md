# Review 4: Test Infrastructure + Test Files

## Files Reviewed
- `crates/services/src/test_utils/sea.rs`, `db.rs`, `mod.rs`, `objs.rs`, `envs.rs`
- `crates/services/src/db/test_access_repository.rs`, `test_access_request_repository.rs`, `test_app_instance_repository.rs`, `test_mcp_repository.rs`, `test_model_repository.rs`, `test_settings_repository.rs`, `test_token_repository.rs`, `test_toolset_repository.rs`, `test_user_alias_repository.rs`

## Findings

### [Important] Missing encryption roundtrip tests for MCP entities
- **File**: `crates/services/src/db/test_mcp_repository.rs`
- **Issue**: While app_instance and toolset have encryption roundtrip tests, MCP entities (mcp_auth_header, mcp_oauth_config, mcp_oauth_token) are missing equivalent tests for:
  - (a) `get_decrypted_auth_header` returns correct plaintext after `create_mcp_auth_header`
  - (b) `get_decrypted_client_secret` returns correct plaintext after `create_mcp_oauth_config`
  - (c) `get_decrypted_oauth_bearer` returns correct Bearer token after `create_mcp_oauth_token`
- **Recommendation**: Add 3 tests in test_mcp_repository.rs, one for each decrypt method, verifying the full encrypt-store-retrieve-decrypt roundtrip produces the original plaintext.
- **Reference**: `test_app_instance_repository.rs` (upsert + get roundtrip), `test_toolset_repository.rs` (create + get_toolset_api_key roundtrip)

### [Nice-to-have] Missing tests for unique constraint violations
- **File**: All test_*_repository.rs files
- **Issue**: No tests verify behavior when unique constraints are violated (e.g., duplicate slug for MCP, duplicate prefix for api_model_alias, duplicate URL for mcp_server). These would verify the DB correctly rejects duplicates and the error is properly propagated as a DbError.
- **Recommendation**: Add tests for unique constraint violations on key columns: MCP slug uniqueness per user, api_model_alias prefix uniqueness, mcp_server URL uniqueness. Documents expected behavior and catches regressions if indexes change.
