# MCP Servers Feature - Planning Q&A Context

## Session Summary

Planning session to add MCP (Model Context Protocol) server support to BodhiApp. Analyzed the reference implementation at `rmcp-app` and the existing toolsets feature to design a parallel but distinct MCP integration. Conducted 8 rounds of detailed Q&A covering transport, configuration, architecture, API design, connection management, OAuth, and testing.

---

## 1. Starting State

BodhiApp has an existing **toolsets** feature providing LLM function calling via external APIs (currently Exa Web Search). The toolsets pattern includes:
- Per-user CRUD with slug-based identification
- AES-GCM encrypted API key storage
- Tool execution via backend API (`POST /bodhi/v1/toolsets/{id}/execute/{method}`)
- Frontend-driven agentic loop (detect tool_calls, execute, feed results back)
- Tool name convention: `toolset__{slug}__{method}`

The reference `rmcp-app` app demonstrates MCP client connectivity using the `rmcp` crate with:
- Streamable HTTP and SSE transports (no stdio)
- OAuth 2.1 PKCE, custom headers, and no-auth support
- In-memory connection pool with `Arc<RwLock<HashMap>>`
- REST endpoints for connect/disconnect/list_tools/execute_tool
- Frontend handles MCP-to-OpenAI tool format conversion and LLM integration

Key difference: rmcp-app is a standalone testing tool; BodhiApp needs MCP as an integrated feature within its existing multi-user, multi-crate architecture.

---

## 2. Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Transport support | HTTP only (Streamable HTTP) | Modern MCP standard; SSE is legacy, stdio not needed since BodhiApp is primarily a web/Docker server (Tauri is only for tray icon) |
| Configuration method | UI + Database only | Consistent with toolsets pattern; no config file support (unlike Claude Desktop) |
| User scope | Per-user only | Consistent with toolsets; each user manages their own MCP servers |
| Connection lifecycle | On-demand with 15min idle timeout | Connect when created (fetch tools, seed whitelist), lazy reconnect on execute, disconnect after idle |
| MCP library | `rmcp` crate with features: `client`, `transport-streamable-http-client`, `transport-streamable-http-client-reqwest`, `auth`, `reqwest` | De-facto Rust MCP SDK, proven in rmcp-app reference |
| New crate | `crates/mcp_client` | Focused rmcp wrapper + connection pool management; services crate handles CRUD only |
| Service architecture | McpService in services crate + McpClient trait in mcp_client crate | McpClient: rmcp wrapper + pool. McpService: CRUD + delegates to McpClient for connections |
| API prefix | `/bodhi/v1/mcps/` | Plural form, consistent with REST conventions |
| Tool naming | `mcp__{server-slug}__{tool-name}` | Parallel to toolsets (`toolset__{slug}__{method}`), prevents collisions across servers |
| Tools filter | Whitelist (empty = block all) | Seeded with all tools on first connection; user removes tools they don't want |
| Tool cache storage | JSON column (`tools_cache`) on `mcp_servers` table | Persists across restarts, re-fetched on explicit refresh |
| Auth support | No-auth + custom headers + OAuth 2.1 PKCE | Full support from day one, matching rmcp-app capability |
| OAuth redirect | Frontend callback page at `/ui/mcp/oauth/callback` (popup window) | Avoids losing page state; standard web OAuth pattern |
| OAuth token refresh | Prefer rmcp AuthClient; otherwise proactive (near-expiry) + reactive (on 401) | rmcp handles natively; fallback covers edge cases |
| Connection failure | Auto-retry once (refresh OAuth token if applicable), then return error | Balances resilience with responsiveness |
| Credential storage | Separate tables; encrypted same pattern as toolset API keys | `mcp_servers` for config + `mcp_oauth_tokens` for OAuth tokens, both AES-GCM encrypted |
| Chat UI integration | Out of scope | Deferred to future frontend phase |
| Auto-detect/probe | Deferred to later phase | Start with manual config; user specifies URL and auth type |
| Endpoint naming | `/mcps/{id}/tools` and `/mcps/{id}/tools/{tool_name}/execute` | Aligns with MCP protocol terminology ("tools"), not toolsets ("methods") |
| Nav placement (future UI) | Under Settings, alongside Toolsets | Consistent grouping for integrations |
| Agentic loop | Frontend-driven (consistent with toolsets) | Deferred to UI phase |
| Phase count | 5 phases | Domain types + DB, MCP client crate, Service CRUD, Routes + tool execution, OAuth flow |

---

## 3. Architecture: BodhiApp is NOT Desktop-First

Critical clarification from user: BodhiApp is **not** primarily a Tauri desktop app. It runs as:
- **Docker HTTP server** (primary deployment)
- **Standalone HTTP server** (non-native variant)
- **Tauri desktop app** (only for system tray icon)

This means:
- No stdio transport needed (no local child process spawning)
- HTTP-only transport is the correct choice
- Config file support (like Claude Desktop's `claude_desktop_config.json`) not applicable
- All MCP connections are remote over HTTP

---

## 4. Connection Lifecycle Details

### On Create
When a user creates an MCP server via `POST /bodhi/v1/mcps`:
1. Validate config (slug, URL, auth_type)
2. Store server record in DB
3. Attempt connection to MCP server
4. Fetch tools list via MCP protocol (`tools/list`)
5. Seed `tools_filter` with ALL discovered tool names (whitelist)
6. Cache tool schemas in `tools_cache` JSON column
7. Connection stays in pool, will disconnect after 15min idle

### On Execute
When `POST /mcps/{id}/tools/{tool_name}/execute` is called:
1. Check if connection exists in pool
2. If not connected, lazy reconnect (use cached auth credentials)
3. Validate tool_name against `tools_filter` whitelist
4. Call tool via MCP protocol (`tools/call`)
5. If connection fails, auto-retry once (refresh OAuth if applicable)
6. Return tool result or error

### On Refresh
When `POST /mcps/{id}/tools/refresh` is called:
1. Connect if not connected
2. Fetch fresh tools list from MCP server
3. Update `tools_cache` in DB
4. `tools_filter` is NOT reset (user's whitelist preserved)
5. New tools from server that aren't in filter are NOT auto-added

### Idle Timeout
- Background task checks connection pool periodically
- Connections idle > 15 minutes are disconnected
- Connection state is in-memory only; DB records persist

---

## 5. Database Schema Design

### `mcp_servers` table
Follows `toolsets` table pattern with additions for URL, auth, and tool caching.

Key differences from toolsets:
- `url` field (toolsets don't have URLs, they use known APIs)
- `auth_type` enum-like field (`none`, `headers`, `oauth`)
- `encrypted_auth_data` replaces `encrypted_api_key` (stores JSON blob: headers map, bearer token, etc.)
- `tools_cache` JSON column (toolsets have static tool definitions in code)
- `tools_filter` JSON column (toolsets don't filter tools)
- No `toolset_type` equivalent (MCP servers are all the same "type")

### `mcp_oauth_tokens` table
Separate table for OAuth token lifecycle management:
- Tied to `mcp_server_id` (1:1 relationship)
- Stores encrypted access token + refresh token
- Tracks expiry for proactive refresh
- Separate from server config to allow independent token rotation

### Encryption
Same AES-GCM pattern as toolsets: `encrypted_*`, `salt`, `nonce` columns. Uses existing `encrypt_api_key`/`decrypt_api_key` from `crates/services/src/db/encryption.rs`.

---

## 6. API Endpoint Details

### CRUD Endpoints
All require session auth (same as toolsets: session tokens only, API tokens rejected).

- `POST /bodhi/v1/mcps` - Create: validates slug, connects, fetches tools, seeds whitelist
- `GET /bodhi/v1/mcps` - List: returns all user's MCP servers with connection status
- `GET /bodhi/v1/mcps/{id}` - Get: includes tools_cache and tools_filter
- `PUT /bodhi/v1/mcps/{id}` - Update: slug, description, enabled, URL, auth config
- `DELETE /bodhi/v1/mcps/{id}` - Delete: disconnects from pool, removes DB records + OAuth tokens

### Tool Endpoints
- `GET /bodhi/v1/mcps/{id}/tools` - Returns cached tools, auto-connects if stale
- `POST /bodhi/v1/mcps/{id}/tools/{tool_name}/execute` - Execute tool, validates against whitelist
- `POST /bodhi/v1/mcps/{id}/tools/refresh` - Force re-fetch tools from server

### Status Endpoint
- `GET /bodhi/v1/mcps/{id}/status` - Returns connection state (connected/disconnected/error)

### OAuth Endpoints
- `POST /bodhi/v1/mcps/{id}/oauth/initiate` - Generates PKCE challenge, returns authorization URL
- `POST /bodhi/v1/mcps/{id}/oauth/complete` - Receives auth code, exchanges for tokens, stores encrypted

---

## 7. Testing Strategy

### Unit Tests (routes_app)
- mockall for McpService trait
- Test each endpoint handler: request validation, auth checks, error responses
- Follow existing `routes_toolsets/tests/` patterns

### Unit Tests (services)
- mockall for McpClient trait (from mcp_client crate)
- Test McpService CRUD logic: validation, encryption, DB operations
- Follow existing `tool_service/tests.rs` patterns

### Unit Tests (mcp_client)
- mockall for rmcp client interactions
- Test connection pool: add/remove/idle timeout
- Test retry logic

### Integration Tests (server_app)
- No mock MCP server needed for basic CRUD
- For tool execution: explore rmcp's server capabilities to create in-process mock MCP server
- Simulate OAuth flow: create MCP server, initiate OAuth, simulate callback with code, complete exchange, verify connection
- Multi-step session tests: create server → list tools → execute tool → verify result

---

## 8. Reference Implementation Analysis (rmcp-app)

### What to Reuse
- rmcp crate and feature set for MCP protocol handling
- `StreamableHttpClientTransport` for HTTP connections
- Connection pool pattern (`HashMap<server_id, McpConnection>` behind `RwLock`)
- OAuth 2.1 PKCE flow via rmcp's `AuthClient`
- Tool listing via `client.list_tools()` and execution via `client.call_tool()`

### What NOT to Reuse
- SSE transport (legacy, not needed for HTTP-only)
- Custom `SseClientTransport` implementation
- JWT auth middleware (BodhiApp has its own)
- SQLite via rusqlite (BodhiApp uses sqlx)
- Frontend tool conversion (deferred to UI phase)
- Auto-detect/probe (deferred)

### Key rmcp APIs Used
```rust
// Connect
let transport = StreamableHttpClientTransport::with_client(
  http_client,
  StreamableHttpClientTransportConfig::with_uri(url)
);
let client = create_client_info().serve(transport).await?;

// List tools
let tools_response = client.list_tools(Default::default()).await?;

// Call tool
let result = client.call_tool(CallToolRequestParams {
  name: Cow::Owned(tool_name.to_string()),
  arguments: args.as_object().cloned(),
  ..Default::default()
}).await?;
```

---

## 9. Crate Dependency Graph

```
mcp_client (new)
  └── rmcp (external: client, transport, auth)
  └── reqwest (HTTP client)
  └── tokio (async runtime, background tasks for idle timeout)

services
  └── mcp_client (for McpClient trait)
  └── objs (for domain types)
  └── sqlx (for DB operations)

routes_app
  └── services (for McpService trait)
  └── objs (for API types)

objs (no new deps, just new types in mcp.rs)
```

---

## 10. Scope Boundaries

### In Scope (This Plan)
- Backend CRUD for MCP servers
- MCP client connectivity over Streamable HTTP
- Tool discovery and caching
- Tool execution via REST API
- Tools whitelist filter
- Full OAuth 2.1 PKCE support
- No-auth and custom headers auth
- In-memory connection pool with idle timeout
- Auto-retry on connection failure
- Encrypted credential storage
- Unit and integration tests

### Out of Scope (Future Work)
- Frontend UI (CRUD pages, settings page, navigation item)
- Chat UI integration (MCP tools in chat popover, agentic loop)
- SSE transport
- Stdio transport
- Auto-detect/probe server capabilities
- MCP resources support
- MCP prompts support
- MCP sampling support (LLM-in-the-loop)
- App-level MCP server type configs (admin enable/disable, like `app_toolset_configs`)
