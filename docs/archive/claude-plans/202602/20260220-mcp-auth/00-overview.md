# MCP OAuth 2.1 Authentication - Overview

## Implementation Status

✅ **FULLY COMPLETED** - OAuth 2.1 authentication for MCP (Model Context Protocol) servers implemented and refactored.

**Major Refactor (2026-02-22)**: Simplified authentication architecture by consolidating OAuth types and unifying API endpoints.

## Feature Summary

OAuth 2.1 authentication for MCP (Model Context Protocol) servers, enabling users to connect to OAuth-protected MCP tool servers. Supports two registration modes:

- **Pre-registered clients**: Admin provides client_id/client_secret from a pre-existing OAuth application
- **Dynamic Client Registration (DCR)**: Auto-registers a new OAuth client via RFC 7591

Both modes now use a unified `oauth` authentication type with differentiation via the `registration_type` field.

## Architecture

Auth configs are **per-MCP-server admin resources**; tokens are **per-user**. The data flow:

1. Admin creates an MCP server entry (URL, name)
2. Admin creates auth config(s) on that server (header, OAuth pre-registered, or OAuth dynamic)
3. User creates an MCP instance, selects an auth config from the dropdown
4. For OAuth: user initiates authorization flow (PKCE), completes consent, token stored
5. At tool execution time, the stored token is resolved and sent as Bearer header
6. Proactive token refresh occurs 60 seconds before expiry

## Security Design

- **PKCE S256**: All OAuth flows use Proof Key for Code Exchange with SHA-256
- **CSRF state validation**: Random state parameter validated between login and token exchange
- **Encrypted secrets**: Client secrets, header values, access/refresh tokens all AES-256-GCM encrypted at rest with per-field salt/nonce
- **Per-config refresh locks**: Mutex-based concurrency control prevents duplicate concurrent token refreshes
- **Secrets never exposed in API**: Responses use boolean flags (`has_client_secret`, `has_access_token`) instead of actual values

## Dependency Changes

- `reqwest 0.13.2` - HTTP client for OAuth discovery, token exchange, dynamic registration
- `rmcp 0.16.0` - MCP SDK with Streamable HTTP transport support

## Endpoint Inventory (Refactored)

✅ **SIMPLIFIED** - Consolidated from dual route structure to unified `/mcps/auth-configs` endpoints.

### MCP Server Admin (4 routes) - No changes
| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| POST | `/bodhi/v1/mcps/servers` | `create_mcp_server_handler` | Create server with optional atomic auth_config |
| PUT | `/bodhi/v1/mcps/servers/{id}` | `update_mcp_server_handler` | Update server |
| GET | `/bodhi/v1/mcps/servers/{id}` | `get_mcp_server_handler` | Get server with MCP counts |
| GET | `/bodhi/v1/mcps/servers` | `list_mcp_servers_handler` | List servers (optional enabled filter) |

### Unified Auth Configs (4 routes) ✅ **REFACTORED**
| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| POST | `/bodhi/v1/mcps/auth-configs` | `create_auth_config_handler` | Create any auth config type (discriminated union) |
| GET | `/bodhi/v1/mcps/auth-configs?mcp_server_id={id}` | `list_auth_configs_handler` | List all auth configs (mixed types) |
| GET | `/bodhi/v1/mcps/auth-configs/{id}` | `get_auth_config_handler` | Get any auth config type |
| DELETE | `/bodhi/v1/mcps/auth-configs/{id}` | `delete_auth_config_handler` | Delete any auth config (cascades tokens) |

**Key changes**:
- Removed server_id path nesting - now uses query parameter `?mcp_server_id=`
- Single endpoint prefix `/mcps/auth-configs` for all CRUD operations
- ~~Removed~~ type-specific routes (`/mcps/auth-headers`, `/mcp-servers/{id}/oauth-configs`)

### OAuth Flow (2 routes) ✅ **UPDATED**
| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| POST | `/bodhi/v1/mcps/auth-configs/{id}/login` | `oauth_login_handler` | Generate PKCE + authorization URL |
| POST | `/bodhi/v1/mcps/auth-configs/{id}/token` | `oauth_token_exchange_handler` | Exchange code for tokens (CSRF validated) |

**Changed**: Path params simplified from `/mcp-servers/{server_id}/oauth-configs/{config_id}/` to `/mcps/auth-configs/{id}/`

### OAuth Discovery (2 routes) - No changes
| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| POST | `/bodhi/v1/mcps/oauth/discover-as` | `oauth_discover_as_handler` | RFC 8414 AS metadata discovery |
| POST | `/bodhi/v1/mcps/oauth/discover-mcp` | `oauth_discover_mcp_handler` | RFC 9728 protected resource + AS discovery |

### Dynamic Client Registration (1 route) ✅ **SIMPLIFIED**
| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| POST | `/bodhi/v1/mcps/oauth/dynamic-register` | `standalone_dynamic_register_handler` | Standalone DCR (no server_id) |

**Removed**: ~~Server-scoped DCR endpoint~~ (duplicate functionality)

### OAuth Tokens (2 routes) - No changes
| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| GET | `/bodhi/v1/mcps/oauth-tokens/{token_id}` | `get_oauth_token_handler` | Get token metadata (user-scoped) |
| DELETE | `/bodhi/v1/mcps/oauth-tokens/{token_id}` | `delete_oauth_token_handler` | Delete token (user-scoped) |

## Refactoring Summary

**Before**: Dual route structure with type-specific endpoints
- `/mcps/auth-headers/` for header configs
- `/mcp-servers/{server_id}/oauth-configs/` for OAuth configs
- `/mcp-servers/{server_id}/auth-configs/` for unified access

**After**: Single unified structure
- `/mcps/auth-configs/` for ALL auth config types (Header + OAuth)
- Discriminated union (`type: "header"` or `type: "oauth"`) determines handling
- Query parameter `?mcp_server_id=` replaces path nesting
- Simpler client code with consistent endpoint patterns

## Cross-References

- Domain types: [01-objs.md](./01-objs.md)
- Database schema and repository: [02-services-db.md](./02-services-db.md)
- Service layer business logic: [03-services-mcp.md](./03-services-mcp.md)
- Route handlers and DTOs: [04-routes-app.md](./04-routes-app.md)
- Frontend implementation: [05-frontend.md](./05-frontend.md)
- E2E test infrastructure: [06-e2e.md](./06-e2e.md)
