---
name: MCP Thin Slice Plan
overview: "Implement a thin slice of MCP server support: unauthenticated MCP servers only, two-level admin URL allowlist + user instances, CRUD with tool discovery, per-request tool execution (no connection pool), tested against real deepwiki MCP server, plus frontend add-MCP page with admin inline enable."
todos:
  - id: phase1-types-db
    content: "Phase 1: Domain types in objs + DB migration + mcp_repository + mcp_client crate"
    status: completed
  - id: phase2-service-routes
    content: "Phase 2: McpService + CRUD routes + unit tests in routes_app"
    status: completed
  - id: phase3-server-app
    content: "Phase 3: server_app integration tests for CRUD"
    status: completed
  - id: phase4-execute
    content: "Phase 4: Tool execution endpoint + real deepwiki MCP tests"
    status: completed
  - id: phase5-frontend
    content: "Phase 5: Frontend add-MCP page with admin inline enable + tool fetch"
    status: completed
  - id: phase6-playwright
    content: "Phase 6: Playwright UI tests for MCP CRUD and tool execution"
    status: completed
isProject: false
---

# MCP Servers - Thin Slice Implementation Plan

## Scope

Implement end-to-end MCP server support for **unauthenticated (public) MCP servers only**. Two-level control: admin URL allowlist (`mcp_servers`) + user instances (`mcps`). Per-request connections (no pool). Test-driven against real deepwiki MCP (`https://mcp.deepwiki.com/mcp`). Frontend add-MCP page with admin inline enable.

## Architecture: Two-Level Control (Parallel to Toolsets)

```
Level 1: mcp_servers (admin URL allowlist)     ~  app_toolset_configs
         keyed by URL, admin enables/disables
         GET/PUT/DELETE /bodhi/v1/mcp_servers

Level 2: mcps (user instances)                 ~  toolsets
         per-user, references allowed URL
         CRUD /bodhi/v1/mcps
         tool execution /bodhi/v1/mcps/{id}/tools/{tool}/execute
```

## Key Design Decisions

- **Transport**: Streamable HTTP only (via rmcp crate)
- **Connection management**: Per-request create/destroy (no pool)
- **Tool naming**: `mcp__{slug}__{tool_name}` (parallel to `toolset__{slug}__{method}`)
- **Tools filter**: Whitelist stored as JSON array; empty = block all; seeded on first fetch
- **Tools cache**: JSON column on mcps table (persists tool schemas across restarts)
- **Admin inline enable**: When user enters URL not in allowlist AND has admin/manager role, popup to single-click enable
- **mcp_servers.id**: UUID/TEXT primary key (consistent with all other IDs in the app)
- **mcps -> mcp_servers FK**: `mcps.mcp_server_id` references `mcp_servers.id`, not URL directly; URL resolved via join
- **URL matching**: Exact match only, no normalization (no auto-trailing-slash). `https://mcp.deepwiki.com/mcp` != `https://mcp.deepwiki.com/mcp/`

---

## Phase 1: Foundation (Domain Types + DB + mcp_client Crate)

### 1a. Domain Types ([crates/objs/src/mcp.rs](crates/objs/src/mcp.rs))

New types parallel to [crates/objs/src/toolsets.rs](crates/objs/src/toolsets.rs):

- `McpServer` - admin URL allowlist entry (id, url, enabled, updated_by, timestamps)
- `Mcp` - user instance (id, mcp_server_id, slug, name, description, enabled, tools_cache, tools_filter, timestamps; URL derived from joined McpServer)
- `McpTool` - cached tool schema (name, description, input_schema)
- `McpExecutionRequest` / `McpExecutionResponse` - parallel to `ToolsetExecutionRequest/Response`
- Validation functions: `validate_mcp_slug()`, `validate_mcp_description()` (reuse toolset regex/limits)

### 1b. DB Migration ([crates/services/migrations/0010_mcp_servers.up.sql](crates/services/migrations/0010_mcp_servers.up.sql))

`**mcp_servers` table** (admin URL allowlist):

```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,                   -- UUID as TEXT (consistent with other tables)
    url TEXT NOT NULL,                     -- exact match, no normalization
    enabled INTEGER NOT NULL DEFAULT 0,
    updated_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_mcp_servers_url ON mcp_servers(url);
```

`**mcps` table** (user instances):

```sql
CREATE TABLE mcps (
    id TEXT PRIMARY KEY,                   -- UUID as TEXT
    user_id TEXT NOT NULL,
    mcp_server_id TEXT NOT NULL,           -- FK to mcp_servers.id (link, not URL copy)
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    tools_cache TEXT,                      -- JSON array of tool schemas
    tools_filter TEXT,                     -- JSON array of whitelisted tool names
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, slug COLLATE NOCASE)
);
CREATE INDEX idx_mcps_user_id ON mcps(user_id);
CREATE INDEX idx_mcps_mcp_server_id ON mcps(mcp_server_id);
```

**URL matching rule**: URLs are stored and compared exactly as provided. No trailing slash normalization, no case folding, no query string stripping. `https://mcp.deepwiki.com/mcp` and `https://mcp.deepwiki.com/mcp/` are two distinct entries.

### 1c. Repository Layer ([crates/services/src/db/mcp_repository.rs](crates/services/src/db/mcp_repository.rs))

Add to `DbService` trait (pattern from [crates/services/src/db/toolset_repository.rs](crates/services/src/db/toolset_repository.rs)):

- `set_mcp_server_enabled(url, enabled, updated_by)` - upsert mcp_servers (generates UUID on insert)
- `get_mcp_server_by_url(url)` - exact match lookup, check if URL is allowed
- `get_mcp_server(id)` - lookup by UUID
- `list_mcp_servers()` - list all allowlist entries
- `create_mcp(row)` / `get_mcp(user_id, id)` / `get_mcp_with_url(user_id, id)` (JOIN) / `list_mcps(user_id)` (JOIN) / `update_mcp(row)` / `delete_mcp(user_id, id)`

Note: `get_mcp_with_url` and `list_mcps` JOIN with `mcp_servers` to include the URL in the result, since `mcps` stores `mcp_server_id` not `url`.

### 1d. mcp_client Crate ([crates/mcp_client/](crates/mcp_client/))

New workspace member. Minimal rmcp wrapper, **no connection pool**.

```toml
# Cargo.toml
[dependencies]
rmcp = { version = "0.14", features = [
    "client",
    "transport-streamable-http-client",
    "transport-streamable-http-client-reqwest",
    "reqwest",
] }
```

**McpClient trait**:

```rust
#[async_trait]
pub trait McpClient: Debug + Send + Sync {
    async fn fetch_tools(&self, url: &str) -> Result<Vec<McpTool>, McpClientError>;
    async fn call_tool(&self, url: &str, tool_name: &str, args: Value) -> Result<Value, McpClientError>;
}
```

**DefaultMcpClient**: Per-request connection pattern:

1. Create `StreamableHttpClientTransport` with URL
2. `create_client_info().serve(transport).await`
3. Call `list_tools()` or `call_tool()`
4. `client.cancel().await` (disconnect)

---

## Phase 2: McpService + CRUD Routes + Tests

### 2a. McpService ([crates/services/src/mcp_service/](crates/services/src/mcp_service/))

**McpService trait** (parallel to [ToolService](crates/services/src/tool_service/service.rs)):

- `list(user_id)` -> `Vec<Mcp>`
- `get(user_id, id)` -> `Option<Mcp>`
- `create(user_id, name, slug, url, description, enabled)` -> `Mcp`
  - Validates slug, description
  - Looks up `mcp_server_id` from `mcp_servers` by exact URL match
  - Validates mcp_server exists and is enabled
  - Stores `mcp_server_id` in mcps row (not URL)
- `update(user_id, id, ...)` -> `Mcp`
- `delete(user_id, id)`
- `is_url_enabled(url)` -> `bool`
- `set_mcp_server_enabled(url, enabled, updated_by)` -> `McpServer`
- `list_mcp_servers()` -> `Vec<McpServer>`
- `fetch_tools(user_id, id)` -> `Vec<McpTool>` (connects to MCP, fetches, caches, seeds filter)
- `execute(user_id, id, tool_name, request)` -> `McpExecutionResponse`

**McpError enum** (parallel to [ToolsetError](crates/services/src/tool_service/error.rs)):

- `McpNotFound`, `McpUrlNotAllowed`, `McpDisabled`, `ToolNotAllowed`, `ToolNotFound`
- `SlugExists`, `InvalidSlug`, `InvalidDescription`
- `ConnectionFailed`, `ExecutionFailed`
- `DbError`

### 2b. Wire into AppService

Add `mcp_service()` to [AppService trait](crates/services/src/app_service.rs) and `DefaultAppService`.
Add `default_mcp_service()` to [AppServiceStubBuilder](crates/services/src/test_utils/app.rs).

### 2c. CRUD Routes ([crates/routes_app/src/routes_mcps/](crates/routes_app/src/routes_mcps/))

Add endpoint constants in [openapi.rs](crates/routes_app/src/shared/openapi.rs):

```rust
make_ui_endpoint!(ENDPOINT_MCPS, "mcps");
make_ui_endpoint!(ENDPOINT_MCP_SERVERS, "mcp_servers");
```

**Handlers**:

- `POST /bodhi/v1/mcps` - `create_mcp_handler` (session auth, user role; request body has `url`, service resolves to `mcp_server_id`)
- `GET /bodhi/v1/mcps` - `list_mcps_handler`
- `GET /bodhi/v1/mcps/{id}` - `get_mcp_handler`
- `PUT /bodhi/v1/mcps/{id}` - `update_mcp_handler`
- `DELETE /bodhi/v1/mcps/{id}` - `delete_mcp_handler`
- `GET /bodhi/v1/mcp_servers` - `list_mcp_servers_handler` (with optional `?url=` query filter)
- `PUT /bodhi/v1/mcp_servers` - `enable_mcp_server_handler` (admin/manager only)
- `DELETE /bodhi/v1/mcp_servers` - `disable_mcp_server_handler` (admin/manager only)

Register in [routes.rs](crates/routes_app/src/routes.rs) under `user_session_apis` (CRUD) and admin-protected router (mcp_servers enable/disable).

### 2d. Unit Tests (routes_app)

Test `POST /mcps` and `GET /mcp_servers?url=<>` with `MockMcpService`. Follow [toolsets_test.rs](crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs) patterns.

---

## Phase 3: server_app Integration Tests for CRUD

Add integration tests in [crates/server_app/tests/](crates/server_app/tests/):

1. Wire `DefaultMcpService` in [live_server_utils.rs](crates/server_app/tests/utils/live_server_utils.rs)
2. Test flow:
  - Enable MCP server URL: `PUT /bodhi/v1/mcp_servers` with `{ url: "https://mcp.deepwiki.com/mcp" }` -> returns `McpServer` with UUID `id`
  - Create MCP instance: `POST /bodhi/v1/mcps` with `{ name, slug, url: "https://mcp.deepwiki.com/mcp", enabled }` (service resolves URL to `mcp_server_id` internally)
  - List MCPs: `GET /bodhi/v1/mcps` -> assert instance present with URL from joined mcp_servers
  - Get MCP: `GET /bodhi/v1/mcps/{id}` -> assert fields including resolved URL

---

## Phase 4: Tool Execution Endpoint + Real MCP Tests

### 4a. Execution Routes

Add to `routes_mcps`:

- `GET /bodhi/v1/mcps/{id}/tools` - list cached tools
- `POST /bodhi/v1/mcps/{id}/tools/refresh` - connect, fetch, update cache (does NOT reset filter)
- `POST /bodhi/v1/mcps/{id}/tools/{tool_name}/execute` - execute tool

Execution flow:

1. Load MCP instance from DB (JOIN with mcp_servers to get URL)
2. Validate mcp_server is still enabled
3. Validate tool_name is in `tools_filter` whitelist
4. Connect to MCP server (per-request, via McpClient, using URL from mcp_servers)
5. `call_tool(url, tool_name, params)`
6. Disconnect
7. Return result

### 4b. routes_app Tests (Real deepwiki MCP)

In routes_app tests:

1. Insert MCP instance directly via `DbService` (bypass CRUD validation for test setup)
2. Set `tools_filter` to include `read_wiki_structure`
3. Call `POST /bodhi/v1/mcps/{id}/tools/read_wiki_structure/execute` with `{ params: { repo_name: "BodhiSearch/BodhiApp" } }`
4. Assert response contains wiki structure data

### 4c. server_app Integration Tests (Full Flow)

1. Enable URL -> Create MCP -> Fetch tools (refresh) -> Execute tool
2. Uses real deepwiki MCP at `https://mcp.deepwiki.com/mcp`
3. Validates: `read_wiki_structure`, `read_wiki_contents`, `ask_question`

---

## Phase 5: Frontend (Add MCP Page)

### 5a. Navigation

Add to [use-navigation.tsx](crates/bodhi/src/hooks/use-navigation.tsx) under Settings:

```typescript
{ title: 'MCP Servers', href: '/ui/mcp/', description: 'MCP server connections' }
```

### 5b. API Hooks

New [crates/bodhi/src/hooks/useMcpServers.ts](crates/bodhi/src/hooks/useMcpServers.ts):

- `useMcpServers()` - GET /mcps
- `useCreateMcp()` - POST /mcps
- `useMcpServerCheck(url)` - GET /mcp_servers?url=<>
- `useEnableMcpServer()` - PUT /mcp_servers
- `useFetchMcpTools(id)` - POST /mcps/{id}/tools/refresh

### 5c. Add MCP Page ([crates/bodhi/src/app/ui/mcp/new/page.tsx](crates/bodhi/src/app/ui/mcp/new/page.tsx))

Form fields: name, URL, slug (auto-generated from URL domain), description (optional), enabled (default true).

**URL check flow**:

1. User enters URL -> debounced `GET /mcp_servers?url=<encoded>`
2. If allowed (enabled=true): green checkmark, proceed
3. If not allowed AND user is admin/manager: show inline popup "This URL is not registered. Enable it?" with single-click "Enable" button
4. On enable click: `PUT /mcp_servers` with `{ url, enabled: true }` -> re-check -> proceed
5. If not allowed AND user is regular user: show "Contact admin" message

**Tool fetch flow**:

1. "Fetch Tools" button -> `POST /mcps` (create first) or `POST /mcps/{id}/tools/refresh`
2. Display tool list with checkboxes (all checked by default)
3. User deselects tools to restrict
4. "Save" updates `tools_filter`

### 5d. MCP List Page ([crates/bodhi/src/app/ui/mcp/page.tsx](crates/bodhi/src/app/ui/mcp/page.tsx))

Table of user's MCP instances: name, URL, status, tool count, actions (edit, delete).

---

## Phase 6: Playwright UI Tests

### 6a. Test Setup

Add MCP test page objects and test files in [crates/lib_bodhiserver_napi/js-tests/](crates/lib_bodhiserver_napi/js-tests/). Follow existing Playwright test patterns.

### 6b. Test Scenarios

**MCP Server Admin (manager/admin role)**:

- Navigate to add MCP page
- Enter URL of a non-registered MCP server
- Verify inline "Enable" popup appears for admin user
- Click Enable, verify URL is registered
- Fill in name, slug, description
- Click "Fetch Tools", verify tool list appears with checkboxes
- Deselect a tool, click Save
- Verify MCP instance appears in list page

**MCP CRUD (regular user)**:

- Navigate to add MCP page
- Enter URL of a pre-registered (enabled) MCP server
- Verify green checkmark (URL allowed)
- Fill form, save
- Navigate to list page, verify entry
- Edit MCP, change description, save
- Delete MCP, verify removed from list

**MCP Tool Execution (via API in test)**:

- Create MCP instance via API
- Refresh tools via API
- Execute `read_wiki_structure` tool against deepwiki MCP
- Assert valid response

---

## Files to Create/Modify (Summary)

### New Files

- `crates/objs/src/mcp.rs`
- `crates/services/migrations/0010_mcp_servers.up.sql` + `.down.sql`
- `crates/services/src/db/mcp_repository.rs`
- `crates/services/src/mcp_service/mod.rs`, `service.rs`, `error.rs`
- `crates/mcp_client/Cargo.toml`, `src/lib.rs`
- `crates/routes_app/src/routes_mcps/mod.rs`, `mcps.rs`, `types.rs`, `error.rs`, `tests/`
- `crates/server_app/tests/test_live_mcp.rs`
- `crates/bodhi/src/app/ui/mcp/` (pages: list, new, edit)
- `crates/bodhi/src/hooks/useMcpServers.ts`
- `crates/lib_bodhiserver_napi/js-tests/tests/mcp/` (Playwright test files + page objects)

### Modified Files

- `Cargo.toml` (workspace: add mcp_client member + rmcp dep)
- `crates/objs/src/lib.rs` (add mcp module)
- `crates/services/src/lib.rs` (add mcp_service module + export)
- `crates/services/src/db/mod.rs` (add mcp_repository)
- `crates/services/src/db/objs.rs` (add McpServerRow, McpRow)
- `crates/services/src/app_service.rs` (add mcp_service accessor)
- `crates/services/src/test_utils/app.rs` (add default_mcp_service)
- `crates/routes_app/src/routes.rs` (register MCP routes)
- `crates/routes_app/src/shared/openapi.rs` (add MCP endpoints)
- `crates/server_app/tests/utils/live_server_utils.rs` (wire McpService)
- `crates/bodhi/src/hooks/use-navigation.tsx` (add nav item)

---

## Test Target: deepwiki MCP

URL: `https://mcp.deepwiki.com/mcp` (Streamable HTTP, no auth required)

Tools available:

- `read_wiki_structure` - get documentation topics for a repo
- `read_wiki_contents` - view documentation about a repo
- `ask_question` - ask AI-powered questions about a repo

Test with: `{ repo_name: "BodhiSearch/BodhiApp" }` or similar public repo.