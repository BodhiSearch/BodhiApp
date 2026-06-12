# Phase 4: Test Infrastructure (`crates/services/src/test_utils/` and test files)

Rebuild the test infrastructure around SeaORM with dual SQLite/PostgreSQL parameterized testing via SeaTestContext, and rewrite all repository tests.

## Test Infrastructure

- `test_utils/sea.rs` (new) -- SeaTestContext fixture (dual SQLite/PostgreSQL), sea_context() function
- `test_utils/db.rs` (modified) -- TestDbService now wraps DefaultDbService (was SqliteDbService). Post-review: updated 6 MCP delegation methods for domain type returns, added `get_decrypted_refresh_token` delegation. MockDbService updated with new return types.
- `test_utils/mod.rs` (modified) -- Added sea module, updated exports
- `test_utils/objs.rs` (modified) -- Updated test builders for ULID and DateTime types
- `test_utils/envs.rs` (modified) -- Added PostgreSQL test env vars

## Repository Test Files

- `test_access_repository.rs` (modified) -- Rewritten for SeaORM, dual-DB parameterized
- `test_access_request_repository.rs` (modified) -- Rewritten, dual-DB parameterized
- `test_app_instance_repository.rs` (new) -- AppInstance CRUD with encryption tests
- `test_mcp_repository.rs` (modified) -- Expanded for 35 MCP methods, dual-DB parameterized. Post-review: assertions updated from encrypted field checks to `has_*` flag checks and decrypt method calls (e.g., `get_decrypted_oauth_bearer`, `get_decrypted_refresh_token`).
- `test_model_repository.rs` (modified) -- Rewritten for SeaORM entities
- `test_settings_repository.rs` (new) -- Settings upsert/get tests
- `test_token_repository.rs` (modified) -- Rewritten, dual-DB parameterized
- `test_toolset_repository.rs` (new) -- Toolset CRUD with encryption tests
- `test_user_alias_repository.rs` (new) -- UserAlias CRUD tests

## Other Test Changes

- `.env.test.example` (new) -- PostgreSQL test database URL template
- `test_app_instance_service.rs` (modified) -- Updated for DefaultDbService
- `test_session_service.rs` (modified) -- Updated for DefaultDbService
