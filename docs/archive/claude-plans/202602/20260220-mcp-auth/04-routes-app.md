# MCP OAuth - Routes (`routes_app` crate)

## Task

✅ **COMPLETED** - Implement HTTP handlers, DTOs, OpenAPI registration, and route wiring for MCP server admin, auth header CRUD, OAuth config CRUD, OAuth flow, discovery, dynamic registration, token management, and unified auth config endpoints.

**Major Refactor**: Unified `routes_mcp_servers/` and `routes_mcps/` into single `routes_mcp/` module with reorganized file structure.

## Files

✅ **RESTRUCTURED** - Old split structure consolidated:

| File | Purpose | Status |
|------|---------|--------|
| `crates/routes_app/src/routes_mcp/servers.rs` | MCP server CRUD handlers | ✅ Moved from routes_mcp_servers |
| `crates/routes_app/src/routes_mcp/mcps.rs` | MCP instance CRUD + tool handlers | ✅ Moved from routes_mcps |
| `crates/routes_app/src/routes_mcp/auth_configs.rs` | Unified auth config handlers (header + OAuth) | ✅ New file |
| `crates/routes_app/src/routes_mcp/oauth_utils.rs` | OAuth discovery + dynamic registration | ✅ New file |
| `crates/routes_app/src/routes_mcp/types.rs` | All request/response DTOs | ✅ Unified |
| `crates/routes_app/src/routes_mcp/error.rs` | McpValidationError enum | ✅ Unified |
| `crates/routes_app/src/routes_mcp/mod.rs` | Module exports + endpoint constants | ✅ |
| `crates/routes_app/src/routes.rs` | Route wiring | ✅ Updated |

## Route Wiring (routes.rs)

✅ **SIMPLIFIED** - Unified auth config endpoints under `/bodhi/v1/mcps/auth-configs`:

**Key route changes from git diff:**
```diff
-    .route(ENDPOINT_MCPS_AUTH_HEADERS, post(create_auth_header_handler))
-    .route("/bodhi/v1/mcp-servers/{server_id}/auth-headers", get(list_auth_headers_handler))
-    .route("/bodhi/v1/mcp-servers/{server_id}/oauth-configs", get(list_oauth_configs_handler))
-    .route("/bodhi/v1/mcp-servers/{server_id}/oauth-configs", post(create_oauth_config_handler))
+    .route(ENDPOINT_MCPS_AUTH_CONFIGS, post(create_auth_config_handler))
+    .route(ENDPOINT_MCPS_AUTH_CONFIGS, get(list_auth_configs_handler))
+    .route(&format!("{ENDPOINT_MCPS_AUTH_CONFIGS}/{{id}}"), get(get_auth_config_handler))
+    .route(&format!("{ENDPOINT_MCPS_AUTH_CONFIGS}/{{id}}"), delete(delete_auth_config_handler))
```

OAuth login/token endpoints simplified:
```diff
-    "/bodhi/v1/mcp-servers/{server_id}/oauth-configs/{config_id}/login"
+    "/bodhi/v1/mcps/auth-configs/{id}/login"
```

**Removed endpoints**:
- Removed server_id nesting - auth configs now globally scoped with `?mcp_server_id` query param
- Removed separate header-specific and OAuth-specific routes
- Removed server-scoped dynamic registration handler (only standalone version remains)

## Handler Functions

### MCP Server Admin Handlers (servers.rs)

✅ **NO CHANGES** - Server CRUD unchanged:
- `create_mcp_server_handler` (POST) - Creates server with optional `auth_config` field (discriminated union)
- `update_mcp_server_handler` (PUT) - Updates url, name, description, enabled
- `get_mcp_server_handler` (GET) - Returns server with `enabled_mcp_count` and `disabled_mcp_count`
- `list_mcp_servers_handler` (GET) - Optional `?enabled=true/false` filter

### Unified Auth Config Handlers (auth_configs.rs)

✅ **NEW FILE** - Consolidated header + OAuth handlers:
- `create_auth_config_handler` (POST `/bodhi/v1/mcps/auth-configs`) - Accepts `CreateMcpAuthConfigRequest` discriminated union
- `list_auth_configs_handler` (GET `/bodhi/v1/mcps/auth-configs?mcp_server_id={id}`) - Returns mixed array of header + OAuth configs
- `get_auth_config_handler` (GET `/bodhi/v1/mcps/auth-configs/{id}`) - Looks up in both tables
- `delete_auth_config_handler` (DELETE `/bodhi/v1/mcps/auth-configs/{id}`) - Cascades to tokens if OAuth

### OAuth Flow Handlers (auth_configs.rs)

✅ **UPDATED** - Path params simplified:
- `oauth_login_handler` (POST `/bodhi/v1/mcps/auth-configs/{id}/login`) - PKCE flow with session state
- `oauth_token_exchange_handler` (POST `/bodhi/v1/mcps/auth-configs/{id}/token`) - Code exchange with CSRF validation

### OAuth Utility Handlers (oauth_utils.rs)

✅ **NEW FILE** - Discovery + dynamic registration:
- `oauth_discover_as_handler` (POST `/bodhi/v1/mcps/oauth/discover-as`) - RFC 8414 discovery
- `oauth_discover_mcp_handler` (POST `/bodhi/v1/mcps/oauth/discover-mcp`) - RFC 9728 + 8414 discovery
- `standalone_dynamic_register_handler` (POST `/bodhi/v1/mcps/oauth/dynamic-register`) - RFC 7591 registration

**Removed**: Server-scoped dynamic registration handler (duplicate functionality).

### OAuth Token Handlers

- `get_oauth_token_handler` (GET) - User-scoped. Returns metadata only (boolean flags, not actual tokens).
- `delete_oauth_token_handler` (DELETE) - User-scoped. Returns 204.

### Unified Auth Config Handlers

- `create_auth_config_handler` (POST) - Accepts `CreateMcpAuthConfigRequest` discriminated union (from objs). Dispatches to header or OAuth creation based on variant type.
- `list_auth_configs_handler` (GET) - Fetches both headers and OAuth configs, converts to `McpAuthConfigResponse` union, returns mixed array.
- `get_auth_config_handler` (GET) - Looks up by config_id in both tables, returns appropriate union variant.
- `delete_auth_config_handler` (DELETE) - Deletes from appropriate table. If OAuth config, cascades to delete associated tokens. Returns 204.

## DTOs (types.rs)

### MCP Server DTOs
- `CreateMcpServerRequest` - `url`, `name`, `description?`, `enabled`, `auth_config?: CreateMcpAuthConfigRequest`
- `UpdateMcpServerRequest` - `url`, `name`, `description?`, `enabled`
- `McpServerResponse` - includes `enabled_mcp_count`, `disabled_mcp_count`, `auth_config?: McpAuthConfigResponse`
- `McpServerQuery` - `enabled?: bool`
- `ListMcpServersResponse` - `mcp_servers: Vec<McpServerResponse>`

### Auth Header DTOs
- `CreateAuthHeaderRequest` - `name`, `mcp_server_id`, `header_key`, `header_value`
- `UpdateAuthHeaderRequest` - `name`, `header_key`, `header_value`
- `AuthHeaderResponse` - secrets masked via `has_header_value: bool`
- `AuthHeadersListResponse` - `auth_headers: Vec<AuthHeaderResponse>`

### OAuth Config DTOs
- `CreateOAuthConfigRequest` - `name?`, `client_id`, `client_secret?`, `authorization_endpoint`, `token_endpoint`, `scopes?`, `registration_type?`, `registration_endpoint?`, `token_endpoint_auth_method?`, `client_id_issued_at?`, `registration_access_token?`
- `OAuthConfigResponse` - secrets masked via `has_client_secret`, `has_registration_access_token`
- `OAuthConfigsListResponse` - `oauth_configs: Vec<OAuthConfigResponse>`
- `OAuthTokenResponse` - secrets masked via `has_access_token`, `has_refresh_token`

### OAuth Flow DTOs
- `OAuthLoginRequest` - `redirect_uri`
- `OAuthLoginResponse` - `authorization_url`
- `OAuthTokenExchangeRequest` - `code`, `redirect_uri`, `state`

### Discovery DTOs
- `OAuthDiscoverAsRequest` - `url`
- `OAuthDiscoverAsResponse` - `authorization_endpoint`, `token_endpoint`, `scopes_supported?`
- `OAuthDiscoverMcpRequest` - `mcp_server_url`
- `OAuthDiscoverMcpResponse` - `authorization_endpoint?`, `token_endpoint?`, `registration_endpoint?`, `scopes_supported?`, `resource?`, `authorization_server_url?`

### Dynamic Registration DTOs
- `DynamicRegisterRequest` - `registration_endpoint`, `redirect_uri`, `scopes?`
- `DynamicRegisterResponse` - `client_id`, `client_secret?`, `client_id_issued_at?`, `token_endpoint_auth_method?`, `registration_access_token?`

## OpenAPI Registration (openapi.rs)

All DTOs registered in `components(schemas(...))`. All handler functions registered in `paths(...)`. See openapi.rs lines 400-440 for schema list and lines 536-573 for path list.

## Test Files

| File | Coverage |
|------|----------|
| `crates/routes_app/src/routes_mcp/test_servers.rs` | Server CRUD, atomic auth config creation |
| `crates/routes_app/src/routes_mcp/test_auth_configs.rs` | Unified auth config CRUD (header + OAuth); uses `build_test_router()` with real services |
| `crates/routes_app/src/routes_mcp/test_oauth_flow.rs` | PKCE flow, CSRF validation; uses custom router with real `SqliteSessionService` |
| `crates/routes_app/src/routes_mcp/test_oauth_utils.rs` | RFC 8414/9728 discovery, standalone DCR |
| `crates/routes_app/src/routes_mcp/test_mcps.rs` | MCP instance CRUD |

**Test infrastructure**: `crates/routes_app/src/test_utils/mcp.rs` (exposed via `routes_app::test_utils`) provides shared helpers — `build_mcp_test_state()`, `build_mcp_test_state_with_app_service()` (for mock-service tests), and `#[cfg(test)]`-only API-driving helpers `setup_mcp_server_in_db()`, `create_header_auth_config_in_db()`, `create_oauth_auth_config_in_db()` (for real-service tests). Re-exports `objs::test_utils::fixed_dt` under `#[cfg(test)]`.

## Cross-References

- Domain types used in DTOs and handlers: [01-objs.md](./01-objs.md)
- Service methods called by handlers: [03-services-mcp.md](./03-services-mcp.md)
- Frontend hooks consuming these endpoints: [05-frontend.md](./05-frontend.md)
