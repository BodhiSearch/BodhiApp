# MCP Integration Plan for BodhiApp

## Summary

Integrate MCP (Model Context Protocol) servers into BodhiApp following the existing toolset pattern. Two-table architecture: admin registry + user instances. Support OAuth 2.1, Bearer, Custom Header, and Public auth types.

## Reference Documents

- `ai-docs/claude-plans/mcp-decisions.md` - All Q&A decisions
- `ai-docs/claude-plans/mcp-oauth-questions.md` - OAuth client registration analysis

## Key Decisions

| Area | Decision |
|------|----------|
| OAuth Registration | Dynamic (primary) + Pre-registered (fallback) |
| OAuth Discovery | MCP spec discovery only (.well-known) |
| Transport | streamable_http only |
| MCP Features | Tools only (no resources/prompts) |
| Sessions | Fresh per chat |
| Capabilities | No caching - fetch fresh |
| Public MCP | Instance required |
| URL Paths | /ui/mcp (user), /ui/admin/mcp (admin) |
| OAuth Redirect | /ui/auth/mcp/callback (same-window) |
| PKCE Storage | Database with state key |
| Tool Naming | mcp_{slug}__{tool_name} |
| Error Display | Full error always |
| Multi-instance | Yes, per server |

---

## Phase mcp-schema: Database Migration

**File:** `crates/services/migrations/0009_mcp_servers.up.sql`

```sql
CREATE TABLE IF NOT EXISTS mcp_server_registry (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    server_url TEXT NOT NULL UNIQUE,
    auth_type TEXT NOT NULL,  -- 'public' | 'oauth' | 'bearer' | 'custom_header'
    oauth_registration_mode TEXT,  -- 'dynamic' | 'pre_registered'
    oauth_client_id_encrypted TEXT,
    oauth_client_id_salt TEXT,
    oauth_client_id_nonce TEXT,
    oauth_client_secret_encrypted TEXT,
    oauth_client_secret_salt TEXT,
    oauth_client_secret_nonce TEXT,
    custom_header_name TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mcp_instances (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    server_id TEXT NOT NULL,
    slug TEXT NOT NULL,
    name TEXT,
    status TEXT NOT NULL DEFAULT 'not_ready',  -- 'not_ready' | 'ready' | 'needs_reauth'
    enabled INTEGER NOT NULL DEFAULT 1,
    bearer_token_encrypted TEXT,
    bearer_token_salt TEXT,
    bearer_token_nonce TEXT,
    custom_header_value_encrypted TEXT,
    custom_header_value_salt TEXT,
    custom_header_value_nonce TEXT,
    oauth_access_token_encrypted TEXT,
    oauth_access_token_salt TEXT,
    oauth_access_token_nonce TEXT,
    oauth_refresh_token_encrypted TEXT,
    oauth_refresh_token_salt TEXT,
    oauth_refresh_token_nonce TEXT,
    oauth_token_expires_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (server_id) REFERENCES mcp_server_registry(id) ON DELETE CASCADE,
    UNIQUE(user_id, slug COLLATE NOCASE)
);

CREATE TABLE IF NOT EXISTS mcp_oauth_pending (
    state TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    code_verifier TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (instance_id) REFERENCES mcp_instances(id) ON DELETE CASCADE
);
```

---

## Phase mcp-objs: Domain Types

**File:** `crates/objs/src/mcp.rs`

- `McpAuthType`: Public, OAuth, Bearer, CustomHeader
- `OAuthRegistrationMode`: Dynamic, PreRegistered
- `McpInstanceStatus`: NotReady, Ready, NeedsReauth
- `McpServerRegistry`: admin registry entry
- `McpInstance`: user instance
- `McpToolDefinition`: tool from MCP server
- `McpError`: thiserror error enum
- Validation: `validate_mcp_slug()`, `extract_slug_from_url()`

---

## Phase mcp-client: MCP Protocol Client

**New crate:** `crates/mcp_client/`

### McpClient (Streamable HTTP)
- `initialize()` - MCP session handshake
- `list_tools()` - JSON-RPC tools/list
- `call_tool()` - JSON-RPC tools/call
- `close()` - session termination

### McpOAuthClient (OAuth 2.1)
- `discover_oauth_metadata()` - .well-known discovery
- `register_client_dynamic()` - RFC 7591
- `generate_auth_url()` - with PKCE S256
- `exchange_code()` - authorization code exchange
- `refresh_token()` - token refresh

---

## Phase mcp-service: Service Layer

**File:** `crates/services/src/mcp_service.rs`

### Admin Operations
- `list_servers()`, `get_server()`, `create_server()`, `update_server()`, `delete_server()`
- `preview_server()` - fetch metadata on add/view

### User Operations
- `list_instances()`, `get_instance()`, `create_instance()`, `update_instance()`, `delete_instance()`
- `set_bearer_token()`, `set_custom_header_value()`
- `initiate_oauth()` - generate auth URL, store PKCE
- `complete_oauth()` - exchange code, store tokens
- `list_tools()`, `execute_tool()`
- `get_tools_for_user()` - aggregated for chat

### DbService additions
- CRUD for `mcp_server_registry`, `mcp_instances`, `mcp_oauth_pending`

---

## Phase mcp-routes: API Endpoints

**File:** `crates/routes_app/src/routes_mcp.rs`

| Method | Path | Purpose |
|--------|------|---------|
| GET | /mcp/servers | List registry (admin) |
| POST | /mcp/servers | Create server (admin) |
| GET | /mcp/servers/{id} | Get server (admin) |
| PUT | /mcp/servers/{id} | Update server (admin) |
| DELETE | /mcp/servers/{id} | Delete server (admin) |
| POST | /mcp/servers/preview | Preview capabilities |
| GET | /mcp/instances | List user instances |
| POST | /mcp/instances | Create instance |
| GET | /mcp/instances/{id} | Get instance |
| PUT | /mcp/instances/{id} | Update instance |
| DELETE | /mcp/instances/{id} | Delete instance |
| PUT | /mcp/instances/{id}/bearer-token | Set bearer token |
| PUT | /mcp/instances/{id}/custom-header | Set header value |
| POST | /mcp/instances/{id}/oauth/initiate | Start OAuth |
| GET | /mcp/oauth/callback | OAuth callback (public) |
| GET | /mcp/instances/{id}/tools | List instance tools |
| POST | /mcp/instances/{id}/execute/{tool} | Execute tool |
| GET | /mcp/tools | All user tools (chat) |

---

## Phase mcp-ui: Frontend

### Pages
- `/ui/admin/mcp/page.tsx` - Admin registry management
- `/ui/mcp/page.tsx` - User instances list
- `/ui/mcp/new/page.tsx` - Create instance
- `/ui/mcp/edit/page.tsx` - Edit instance
- `/ui/auth/mcp/callback/page.tsx` - OAuth callback

### Hooks (`crates/bodhi/src/hooks/useMcp.ts`)
- `useMcpServers()`, `useCreateMcpServer()`, `useUpdateMcpServer()`, `useDeleteMcpServer()`
- `useMcpInstances()`, `useCreateMcpInstance()`, `useUpdateMcpInstance()`, `useDeleteMcpInstance()`
- `useSetMcpBearerToken()`, `useSetMcpCustomHeader()`, `useInitiateMcpOAuth()`
- `useMcpUserTools()` - for chat integration

### OAuth Flow (same-window)
1. User clicks "Connect" on OAuth instance
2. `initiate_oauth()` returns authorization_url
3. `window.location.href = authorization_url`
4. User completes OAuth
5. Redirect to /ui/auth/mcp/callback?state=X&code=Y
6. Callback page POSTs to complete_oauth
7. Redirect to /ui/mcp on success

---

## Phase mcp-chat: Chat Integration

### Tool Naming
Format: `mcp_{instance_slug}__{tool_name}`

### Tool Selection UI
- Add MCP section to ToolsetsPopover
- Distinct icon for MCP tools (vs toolset icon)
- Group by server/instance

### Execution
- Parse tool name for `mcp_` prefix
- Extract slug and tool name
- Resolve instance by slug
- Call McpService.execute_tool()
- Handle needs_reauth status

---

## Implementation Order

### PR1: Backend
```
mcp-schema → mcp-objs → mcp-client → mcp-service → mcp-routes
```

### PR2: Frontend
```
mcp-ui → mcp-chat
```

---

## Critical Files to Reference

| Pattern | File |
|---------|------|
| Migration | `crates/services/migrations/0007_toolsets_config.up.sql` |
| Domain types | `crates/objs/src/toolset.rs` |
| Service trait | `crates/services/src/tool_service.rs` |
| DB operations | `crates/services/src/db/service.rs` |
| Encryption | `crates/services/src/db/encryption.rs` |
| Routes | `crates/routes_app/src/routes_toolsets.rs` |
| Hooks | `crates/bodhi/src/hooks/useToolsets.ts` |
| Pages | `crates/bodhi/src/app/ui/toolsets/` |

---

## Verification

### Backend Tests
```bash
cargo test -p services -- mcp
cargo test -p routes_app -- mcp
cargo test -p mcp_client
```

### Integration Test
1. Start app: `make run.app`
2. Login as admin
3. Navigate to /ui/admin/mcp
4. Add public MCP server (e.g., test server)
5. Preview capabilities
6. Switch to regular user
7. Navigate to /ui/mcp
8. Create instance with slug
9. For OAuth: complete auth flow
10. Verify status = ready
11. In chat, verify MCP tools appear
12. Execute MCP tool, verify response

### Manual OAuth Test
1. Add OAuth MCP server with pre-registered credentials
2. Create user instance
3. Click Connect
4. Complete OAuth in same window
5. Verify redirect to /ui/mcp
6. Verify instance status = ready
7. Test token refresh by waiting for expiry
