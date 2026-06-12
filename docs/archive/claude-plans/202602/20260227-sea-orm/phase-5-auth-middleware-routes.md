# Phase 5: Upstream Consumer Changes (`crates/auth_middleware/`, `crates/routes_app/`, `crates/services/src/`)

Update all upstream consumers of the repository layer to work with the new SeaORM domain types, DateTime fields, and ULID identifiers.

## auth_middleware

- `src/token_service/service.rs` (modified) -- TimeService injection for token validation
- `src/token_service/tests.rs` (modified) -- Updated for FrozenTimeService
- `src/access_request_auth_middleware/middleware.rs` (modified) -- Updated for new domain types
- `src/access_request_auth_middleware/tests.rs` (modified) -- Updated test assertions
- `src/auth_middleware/middleware.rs` (modified) -- Minor updates for new types
- `src/auth_middleware/tests.rs` (modified) -- Updated test setup
- `tests/test_live_auth_middleware.rs` (modified) -- Updated integration tests
- `CLAUDE.md` (modified) -- Updated docs for TimeService

## routes_app

- Route handler files (modified) -- Changes to route handlers for updated domain types

## services (Consumer-Facing Changes)

- `src/access_request_service/service.rs` (modified) -- Updated for SeaORM domain types
- `src/access_request_service/test_access_request_service.rs` (modified) -- Updated tests
- `src/app_instance_service.rs` (modified) -- Updated for AppInstanceRow changes
- `src/data_service.rs` (modified) -- Minor updates
- `src/mcp_service/service.rs` (modified) -- Updated for new McpRow tuple returns. Post-review: removed 3 conversion helpers (`auth_header_row_to_model`, `oauth_config_row_to_model`, `oauth_token_row_to_model`), simplified list/get consumers to pass-through (repo returns domain types), refactored `resolve_oauth_token` to use decrypt methods instead of raw encrypted fields, renamed `get_mcp_server_url_for_config` to `get_mcp_server_url`.
- `src/mcp_service/tests.rs` (modified) -- Updated test assertions
- `src/tool_service/service.rs` (modified) -- Updated for ToolsetRow changes
- `src/tool_service/tests.rs` (modified) -- Updated test assertions
- `src/queue_service.rs` (modified) -- Updated for domain type changes
- `src/progress_tracking.rs` (modified) -- Minor updates
- `src/objs.rs` (modified) -- Service-level domain object updates
- `src/setting_service/` (modified) -- Multiple files updated for SettingsRepository changes
