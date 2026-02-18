---
name: MCP Servers Feature Plan
overview: "Add MCP (Model Context Protocol) server support to BodhiApp as a backend-only feature across 5 phases: domain types + DB schema, mcp_client crate, service CRUD, API routes with tool execution, and OAuth 2.1 PKCE flow. No frontend UI in this scope."
todos:
  - id: plan-readme
    content: Create README.md index for 20260217-mcps plan folder
    status: pending
  - id: plan-decisions
    content: Create decisions-ctx.md with all Q&A decisions from interview
    status: pending
  - id: plan-phase1
    content: Create phase-1-domain-types-db.md plan
    status: pending
  - id: plan-phase2
    content: Create phase-2-mcp-client-crate.md plan
    status: pending
  - id: plan-phase3
    content: Create phase-3-service-crud.md plan
    status: pending
  - id: plan-phase4
    content: Create phase-4-routes-tool-execution.md plan
    status: pending
  - id: plan-phase5
    content: Create phase-5-oauth-flow.md plan
    status: pending
isProject: false
---

# MCP Servers Feature - Backend Implementation Plan

## Summary

Add MCP server support to BodhiApp, enabling users to connect to remote MCP servers over Streamable HTTP, discover tools, and execute them via REST API. This is a backend-only implementation (no frontend UI) following the existing toolsets pattern. The feature will be implemented across 5 phases, each producing testable increments.

## Architecture Decisions (from Q&A)

- **Transport**: HTTP only (Streamable HTTP, modern MCP standard)
- **Configuration**: UI + Database (consistent with toolsets), no config file support
- **User scope**: Per-user only (consistent with toolsets)
- **Connection lifecycle**: On-demand with 15min idle timeout
- **MCP library**: `rmcp` crate v0.14+ with `client`, `transport-streamable-http-client`, `auth` features
- **Tool naming**: `mcp__{server-slug}__{tool-name}` pattern
- **Tools filter**: Whitelist (empty = block all), seeded with all tools on first connection
- **Auth support**: No-auth + custom headers + OAuth 2.1 PKCE (full from day one)
- **Agentic loop**: Frontend-driven (consistent with toolsets, deferred to UI phase)
- **Auto-detect/probe**: Deferred to later phase
- **Chat UI integration**: Out of scope (deferred)
- **Navigation**: Under Settings when UI is eventually added

## New Crate: `crates/mcp_client`

Focused MCP client wrapper around `rmcp` + connection pool management.

**Responsibilities**:

- Wrap rmcp client initialization and transport setup
- Manage in-memory connection pool (`HashMap<server_id, McpConnection>`)
- Handle idle timeout (15 min default) with background cleanup
- Provide trait-based interface for connect/disconnect/list_tools/call_tool
- Handle auto-retry on connection failure (refresh OAuth token, then retry once)

## Database Schema: Migration `0010_mcp_servers`

`**mcp_servers` table**:

- `id` TEXT PK (UUID)
- `user_id` TEXT NOT NULL
- `name` TEXT NOT NULL
- `slug` TEXT NOT NULL
- `description` TEXT
- `url` TEXT NOT NULL
- `auth_type` TEXT NOT NULL (none, headers, oauth)
- `encrypted_auth_data` TEXT (AES-GCM encrypted JSON blob for headers/API keys)
- `salt` TEXT, `nonce` TEXT
- `enabled` INTEGER NOT NULL DEFAULT 1
- `tools_cache` TEXT (JSON array of tool schemas from last fetch)
- `tools_filter` TEXT (JSON array of whitelisted tool names)
- `created_at` INTEGER, `updated_at` INTEGER
- UNIQUE(`user_id`, `slug` COLLATE NOCASE)

`**mcp_oauth_tokens` table**:

- `id` INTEGER PK AUTOINCREMENT
- `mcp_server_id` TEXT NOT NULL (FK to mcp_servers.id)
- `encrypted_access_token` TEXT, `salt_at` TEXT, `nonce_at` TEXT
- `encrypted_refresh_token` TEXT, `salt_rt` TEXT, `nonce_rt` TEXT
- `token_type` TEXT
- `expires_at` INTEGER
- `scope` TEXT
- `created_at` INTEGER, `updated_at` INTEGER
- UNIQUE(`mcp_server_id`)

## API Endpoints: `/bodhi/v1/mcps/`


| Method | Path                                            | Description                                                  |
| ------ | ----------------------------------------------- | ------------------------------------------------------------ |
| POST   | `/bodhi/v1/mcps`                                | Create MCP server (connects, fetches tools, seeds whitelist) |
| GET    | `/bodhi/v1/mcps`                                | List user's MCP servers                                      |
| GET    | `/bodhi/v1/mcps/{id}`                           | Get MCP server details                                       |
| PUT    | `/bodhi/v1/mcps/{id}`                           | Update MCP server config                                     |
| DELETE | `/bodhi/v1/mcps/{id}`                           | Delete MCP server                                            |
| GET    | `/bodhi/v1/mcps/{id}/tools`                     | List tools (returns cached, auto-connects if stale)          |
| POST   | `/bodhi/v1/mcps/{id}/tools/{tool_name}/execute` | Execute a tool                                               |
| POST   | `/bodhi/v1/mcps/{id}/tools/refresh`             | Force refresh tool list from server                          |
| GET    | `/bodhi/v1/mcps/{id}/status`                    | Get connection status                                        |
| POST   | `/bodhi/v1/mcps/{id}/oauth/initiate`            | Start OAuth 2.1 PKCE flow                                    |
| POST   | `/bodhi/v1/mcps/{id}/oauth/complete`            | Exchange auth code for tokens                                |


## Testing Strategy

- **Unit tests (routes_app)**: mockall for McpService trait, test each endpoint handler
- **Unit tests (services)**: mockall for McpClient trait, test McpService CRUD logic
- **Unit tests (mcp_client)**: mockall for rmcp client interactions
- **Integration tests (server_app)**: Create MCP server records, simulate OAuth flows, test multi-step execute flows with session tokens; explore rmcp mock server for real MCP protocol testing

## Phase Breakdown

### Phase 1: Domain Types + DB Schema

- Domain types in `crates/objs/src/mcp.rs`
- DB migration `0010_mcp_servers.up.sql` and `.down.sql`
- DB row types in `crates/services/src/db/objs.rs`
- Repository layer in `crates/services/src/db/mcp_repository.rs`

### Phase 2: MCP Client Crate

- New crate `crates/mcp_client` with rmcp dependency
- McpClient trait + DefaultMcpClient impl
- ConnectionPool with idle timeout management
- Connect/disconnect/list_tools/call_tool operations

### Phase 3: Service CRUD

- McpService trait in `crates/services/src/mcp_service/`
- CRUD operations: list, get, create, update, delete
- Slug/description validation (reuse toolset validators)
- Credential encryption (reuse AES-GCM pattern from toolsets)
- Tools cache and filter management

### Phase 4: API Routes + Tool Execution

- Route handlers in `crates/routes_app/src/routes_mcps/`
- CRUD endpoints with session auth
- Tools listing and execution endpoints
- Auto-connect on tools/execute if not connected
- Status endpoint
- Unit tests with mockall

### Phase 5: OAuth 2.1 PKCE Flow

- OAuth initiate endpoint (generate PKCE, return auth URL)
- OAuth complete endpoint (exchange code for tokens)
- Token storage in mcp_oauth_tokens table
- Token refresh via rmcp AuthClient
- Auto-retry with token refresh on 401

## Key File References

- Toolsets pattern to follow: [crates/objs/src/toolsets.rs](crates/objs/src/toolsets.rs), [crates/services/src/tool_service/](crates/services/src/tool_service/), [crates/routes_app/src/routes_toolsets/](crates/routes_app/src/routes_toolsets/)
- DB migration pattern: [crates/services/migrations/0007_toolsets_config.up.sql](crates/services/migrations/0007_toolsets_config.up.sql)
- Encryption pattern: [crates/services/src/db/encryption.rs](crates/services/src/db/encryption.rs) (AES-GCM)
- rmcp reference: [rmcp-app/src/mcp/pool.rs](../rmcp-app/src/mcp/pool.rs) and [rmcp-app/src/mcp/oauth.rs](../rmcp-app/src/mcp/oauth.rs)
- rmcp features needed: `client`, `transport-streamable-http-client`, `transport-streamable-http-client-reqwest`, `auth`, `reqwest`

