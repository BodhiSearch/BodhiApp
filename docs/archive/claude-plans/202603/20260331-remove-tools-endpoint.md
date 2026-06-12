# Remove Legacy Tool Call APIs & Migrate Frontend to MCP Client SDK

## Context

BodhiApp introduced a transparent MCP proxy at `/bodhi/v1/mcps/{id}/mcp` (commit `b7aca718`) that replaces the custom tool-wrapping REST endpoints (`/mcps/{id}/tools/refresh`, `/mcps/{id}/tools/{tool_name}/execute`, `/mcps/fetch-tools`). These custom endpoints were an incorrect, limiting way to invoke MCP methods. The proxy is a transparent HTTP reverse proxy that forwards MCP protocol directly to upstream servers.

This plan removes the legacy tool endpoints, drops `tools_cache` and `tools_filter` from the data model entirely (no server-side tool filtering with transparent proxy), adds `mcp_endpoint` to the API response, and migrates the frontend (chat, playground, creation form) to use `@modelcontextprotocol/sdk` through the proxy.

---

## Layer 1: `services` Crate

### 1.1 DB Migration — Drop `tools_cache` and `tools_filter` columns

**New file**: `crates/services/src/db/sea_migrations/m20250101_000018_drop_mcp_tools_columns.rs`
- `DeriveIden` enum `Mcps` with variants: `Table`, `ToolsCache`, `ToolsFilter`
- `up()`: `manager.alter_table(Table::alter().table(Mcps::Table).drop_column(Mcps::ToolsCache).drop_column(Mcps::ToolsFilter).to_owned())`
- `down()`: no-op (permanent removal)

**Modify**: `crates/services/src/db/sea_migrations/mod.rs` — register migration 18

### 1.2 Entity Changes

**File**: `crates/services/src/mcps/mcp_entity.rs`
- Remove `tools_cache: Option<String>` and `tools_filter: Option<String>` from `Model`
- Remove same fields from `McpWithServerEntity`

### 1.3 API Model Changes

**File**: `crates/services/src/mcps/mcp_objs.rs`
- **`Mcp` response struct**: Remove `tools_cache` and `tools_filter` fields. Add `mcp_endpoint: String`.
- **`impl From<McpWithServerEntity> for Mcp`**: Remove tools deserialization. Add `mcp_endpoint: format!("/bodhi/v1/mcps/{}/mcp", row.id)`.
- **`McpRequest` struct**: Remove `tools_cache` and `tools_filter` fields.
- **Remove**: `McpExecutionRequest` and `McpExecutionResponse` structs (only used by `McpService::execute`).
- **Remove**: `ToolNotAllowed` and `ToolNotFound` variants from `McpError` if only used by execute flow.

### 1.4 Service Trait & Implementation

**File**: `crates/services/src/mcps/mcp_service.rs`
- **McpService trait**: Remove `fetch_tools`, `fetch_tools_for_server`, `execute` method signatures
- **DefaultMcpService impl**: Remove those 3 method implementations (~160 lines, lines 915-1072)
- **`create` method**: Remove `tools_cache_json`/`tools_filter_json` computation and assignment
- **`update` method**: Remove `resolved_cache`/`resolved_filter` computation and assignment
- **`mcp_row_to_with_server` helper**: Remove `tools_cache`/`tools_filter` from struct construction

**File**: `crates/services/src/mcps/auth_scoped.rs`
- Remove `fetch_tools`, `execute`, `fetch_tools_for_server` from `AuthScopedMcpService`

### 1.5 Repository Layer

**File**: `crates/services/src/mcps/mcp_repository.rs`
- Remove `tools_cache: Set(...)` and `tools_filter: Set(...)` from all `ActiveModel` constructions
- Remove those fields from column select and struct mapping in queries

### 1.6 Re-exports

**File**: `crates/services/src/mcps/mod.rs`
- Remove re-exports of `McpExecutionRequest`, `McpExecutionResponse` if they existed
- Keep `McpTool` re-export (still used by `mcp_client` internally)

### 1.7 Services Tests

Update all test files that construct `McpEntity`, `McpWithServerEntity`, or `McpRequest` to remove `tools_cache`/`tools_filter` fields:
- `crates/services/src/mcps/test_helpers.rs`
- `crates/services/src/mcps/test_mcp_service.rs`
- `crates/services/src/mcps/test_mcp_instance_repository.rs`
- `crates/services/src/mcps/test_mcp_repository_isolation.rs`
- `crates/services/src/mcps/test_mcp_proxy_service.rs`
- `crates/services/src/mcps/test_mcp_objs_validation.rs`

**Gate check**: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"`

---

## Layer 2: `routes_app` Crate

### 2.1 Remove Route Handlers

**File**: `crates/routes_app/src/mcps/routes_mcps.rs`
- Remove 6 handler functions + their `#[utoipa::path]` attrs:
  - `mcps_fetch_tools` (lines 188-237)
  - `mcps_list_tools` (lines 244-271)
  - `mcps_refresh_tools` (lines 274-295)
  - `mcps_execute_tool` (lines 298-332)
  - `apps_mcps_refresh_tools` (lines 375-395)
  - `apps_mcps_execute_tool` (lines 397-420)
- Update imports: remove `FetchMcpToolsRequest`, `McpAuth`, `McpExecuteRequest`, `McpExecuteResponse`, `McpToolsResponse`, `ENDPOINT_MCPS_FETCH_TOOLS`, `McpExecutionRequest`

### 2.2 Remove API Schema Types

**File**: `crates/routes_app/src/mcps/mcps_api_schemas.rs`
- Remove: `FetchMcpToolsRequest`, `McpToolsResponse`, `McpExecuteRequest`, `McpExecuteResponse`, `McpAuth`
- Clean up unused imports

### 2.3 Remove Route Registrations

**File**: `crates/routes_app/src/routes.rs`
- **Line 210**: Remove `ENDPOINT_MCPS_FETCH_TOOLS` route from `user_session_apis`
- **Lines 302-309**: Remove `mcps_refresh_tools` and `mcps_execute_tool` from `mcp_exec_apis` (keep proxy route at line 310-312)
- **Lines 343-349**: Remove `apps_mcps_refresh_tools` and `apps_mcps_execute_tool` from `apps_mcp_exec` (keep proxy route at line 351-353)
- Update imports: remove handler function imports

### 2.4 Remove Constants

**File**: `crates/routes_app/src/mcps/mod.rs`
- Remove `ENDPOINT_MCPS_FETCH_TOOLS` constant

### 2.5 OpenAPI Cleanup

**File**: `crates/routes_app/src/shared/openapi.rs`
- Remove path symbols: `__path_mcps_fetch_tools`, `__path_mcps_refresh_tools`, `__path_mcps_execute_tool`, `__path_apps_mcps_refresh_tools`, `__path_apps_mcps_execute_tool`
- Remove schema types: `FetchMcpToolsRequest`, `McpAuth`, `McpTool`, `McpToolsResponse`, `McpExecuteRequest`, `McpExecuteResponse`
- Remove from paths registration

### 2.6 Routes App Tests

**File**: `crates/routes_app/src/mcps/test_mcps.rs`
- **Remove tests**: `test_fetch_mcp_tools_with_inline_auth`, `test_fetch_mcp_tools_with_auth_config_id`, `test_execute_mcp_tool_success`, `test_fetch_mcp_tools_with_oauth_token_id`
- **Update**: all remaining tests constructing `McpRequest`/`McpWithServerEntity` — remove `tools_cache`/`tools_filter` fields
- **Update test router**: remove tool endpoint routes from `test_router_for_crud`

Also update:
- `crates/routes_app/src/mcps/test_mcps_isolation.rs`
- `crates/routes_app/src/mcps/test_oauth_flow.rs`

**Gate check**: `cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED|failures:"`

---

## Layer 3: `server_app` Tests

**File**: `crates/server_app/tests/test_live_mcp.rs`
- Remove `test_mcp_tool_execution_flow` test
- Update `test_mcp_crud_flow`: remove tools_cache/tools_filter from assertions

**File**: `crates/server_app/tests/test_live_mcp_proxy.rs`
- Update `test_mcp_proxy_tools_filter`: remove filter concept, simplify to just verifying proxy returns all upstream tools
- Remove `create_mcp_instance_with_filter` helper if only used by that test

**Gate check**: `cargo test -p server_app 2>&1 | grep -E "test result|FAILED|failures:"`

---

## Layer 4: Regenerate OpenAPI & TypeScript Client

```bash
cargo run --package xtask openapi
make build.ts-client
```

Verify `Mcp` type in `ts-client/src/types/types.gen.ts`:
- No `tools_cache` or `tools_filter` fields
- Has `mcp_endpoint: string` field
- No `McpToolsResponse`, `FetchMcpToolsRequest`, `McpExecuteRequest`, `McpExecuteResponse`

**Gate check**: `make ci.ts-client-check`

---

## Layer 5: Frontend — `crates/bodhi`

### 5.1 Install MCP SDK

```bash
cd crates/bodhi && npm install @modelcontextprotocol/sdk@^1.25.2
```

### 5.2 New: `useMcpClient` Hook

**New file**: `crates/bodhi/src/hooks/mcps/useMcpClient.ts`

Wraps `Client` + `StreamableHTTPClientTransport` from `@modelcontextprotocol/sdk`. Shared by chat and playground.

```typescript
interface UseMcpClientReturn {
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  tools: McpClientTool[];   // from client.listTools()
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  callTool: (name: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
  refreshTools: () => Promise<void>;
}
```

- Takes `endpoint: string | null` (from `Mcp.mcp_endpoint`)
- Constructs full URL: `window.location.origin + endpoint`
- Uses `StreamableHTTPClientTransport` with reconnection options (follow MCP Inspector pattern from `/Users/amir36/Documents/workspace/src/github.com/modelcontextprotocol/inspector/client/src/lib/hooks/useConnection.ts` lines 607-638)
- Client/Transport in `useRef` to avoid React serialization
- Cleanup on unmount

### 5.3 New: `useMcpClients` Hook (plural)

**New file**: `crates/bodhi/src/hooks/mcps/useMcpClients.ts`

Manages multiple MCP connections for chat (multiple MCPs enabled simultaneously).

```typescript
interface UseMcpClientsReturn {
  clients: Map<string, { status, tools, error }>;
  allTools: McpClientTool[];  // flattened
  connectAll: (mcps: Mcp[]) => Promise<void>;
  disconnectAll: () => Promise<void>;
  callTool: (mcpId: string, toolName: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
  isConnecting: boolean;
}
```

Uses refs internally (not per-MCP hooks — React hooks can't be called in loops).

### 5.4 Rewrite `useChat` Hook

**File**: `crates/bodhi/src/hooks/chat/useChat.tsx`

**Remove**:
- `executeMcpToolCall()` function (lines 21-83) — was POSTing to `/mcps/{id}/tools/{toolName}/execute`
- `executeToolCalls()` function (lines 88-109)
- `buildMcpToolsArray()` function (lines 125-157) — was reading from `tools_cache`

**Replace with**:
- New `buildToolsFromMcpClients(enabledMcpTools, mcpTools, mcpSlugs)` — builds OpenAI tools array from live `McpClientTool[]` data. Still encodes names as `mcp__{slug}__{toolName}`.
- New `executeMcpToolCallViaClient(toolCall, mcpSlugToId, callMcpTool)` — decodes tool name, calls `callMcpTool(mcpId, toolName, args)` (provided by `useMcpClients`)
- New `executeToolCallsViaClient(toolCalls, ...)` — parallel execution via `Promise.allSettled()`

**Update `UseChatOptions`**:
```typescript
interface UseChatOptions {
  enabledMcpTools?: Record<string, string[]>;
  mcpTools?: Map<string, McpClientTool[]>;     // live tools from MCP clients
  mcpSlugs?: Map<string, string>;              // mcpId → slug
  callMcpTool?: (mcpId: string, toolName: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
}
```

Agentic loop structure stays the same — only tool building and execution changes.

### 5.5 Update `ChatUI` Component

**File**: `crates/bodhi/src/app/chat/ChatUI.tsx`

- Add `useMcpClients()` hook for persistent MCP sessions
- `useEffect` to `connectAll` when MCPs change, `disconnectAll` on unmount
- Build `mcpTools` and `mcpSlugs` maps from client data
- Pass `mcpTools`, `mcpSlugs`, `callMcpTool` to `useChat` instead of `mcps` objects
- Pass `mcpTools` and connection status to `McpsPopover`

### 5.6 Update `McpsPopover` Component

**File**: `crates/bodhi/src/app/chat/McpsPopover.tsx`

- Add props: `mcpTools: Map<string, McpClientTool[]>`, `mcpConnectionStatus: Map<string, ConnectionStatus>`
- Remove availability check for `tools_cache` (line 29-30) — now just `mcp_server.enabled && mcp.enabled`
- Read tools from `mcpTools.get(mcp.id)` instead of `mcp.tools_cache`
- Show loading spinner per MCP while connecting
- Remove "Tools not yet discovered" / "All tools blocked by filter" unavailable reasons

### 5.7 Update `useMcpSelection` Hook

**File**: `crates/bodhi/src/hooks/mcps/useMcpSelection.ts`

Minimal changes — the hook already works with `Record<string, string[]>`. The source of tool names shifts from `tools_cache` to live MCP client tools. Remove any references to `tools_cache` or `tools_filter` in the availability logic.

### 5.8 Rewrite Playground

**File**: `crates/bodhi/src/app/mcps/playground/page.tsx`

- Replace `useRefreshMcpTools()` and `useExecuteMcpTool()` with `useMcpClient(mcp?.mcp_endpoint)`
- Connect on page load, list tools from `mcpClient.tools`
- Execute via `mcpClient.callTool(toolName, params)`
- Remove whitelisting indicators (opacity-50 for non-whitelisted, "not whitelisted" alert)
- Show connection status (connecting/connected/error) in header
- Remove or simplify refresh button (tools fetched on connect; optionally keep as `mcpClient.refreshTools()`)

### 5.9 Simplify MCP Creation Form

**File**: `crates/bodhi/src/app/mcps/new/page.tsx`

- Remove `<ToolSelection>` component usage and "Tool Selection" wizard step
- Remove `useFetchMcpTools` hook usage and `handleFetchTools` function
- Remove `tools_cache`/`tools_filter` from submit payload
- Remove `canCreate` dependency on `store.toolsFetched` — create enabled as soon as form is valid
- Wizard becomes 3 steps: Server → Basic Info → Auth

**File**: `crates/bodhi/src/stores/mcpFormStore.ts`
- Remove: `fetchedTools`, `selectedTools`, `toolsFetched` state and their setters
- Remove tools from `saveToSession`/`restoreFromSession`

### 5.10 Delete Files

- `crates/bodhi/src/hooks/mcps/useMcpTools.ts` — replaced by useMcpClient
- `crates/bodhi/src/app/mcps/new/ToolSelection.tsx` — removed from wizard

### 5.11 Update Barrel Exports

**File**: `crates/bodhi/src/hooks/mcps/index.ts`
- Remove lines 45-52 (useMcpTools exports)
- Add exports for `useMcpClient`, `useMcpClients` and their types

**File**: `crates/bodhi/src/hooks/mcps/constants.ts`
- Remove `MCPS_FETCH_TOOLS_ENDPOINT`

### 5.12 Keep `lib/mcps.ts` Utilities

`encodeMcpToolName`, `decodeMcpToolName`, `isEncodedMcpToolName` — still needed for LLM tool name encoding in the agentic loop and display in `ToolCallMessage.tsx`.

### 5.13 Update MSW Handlers

**File**: `crates/bodhi/src/test-utils/msw-v2/handlers/mcps.ts`
- Remove: `mockFetchMcpTools`, `mockRefreshMcpTools`, `mockExecuteMcpTool` and their error variants
- Remove from `mcpsHandlers` array
- Update mock MCP fixtures: remove `tools_cache`/`tools_filter` fields, add `mcp_endpoint`

### 5.14 Update Frontend Tests

- **Delete**: `crates/bodhi/src/hooks/mcps/useMcpTools.test.ts`
- **New**: `crates/bodhi/src/hooks/mcps/useMcpClient.test.ts` — mock SDK, test lifecycle
- **Rewrite**: `crates/bodhi/src/app/mcps/playground/page.test.tsx` — mock `useMcpClient`, test connection status + tool execution
- **Update**: any creation form tests — remove ToolSelection references
- **Update**: MCP test fixtures in `crates/bodhi/src/test-fixtures/mcps.ts` — remove `tools_cache`/`tools_filter`, add `mcp_endpoint`

**Gate check**: `cd crates/bodhi && npm test`

---

## Layer 6: E2E Tests — `crates/lib_bodhiserver_napi`

### 6.1 Update Page Object

**File**: `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs`
- Remove tool selection methods: `clickFetchTools`, `expectToolsList`, `expectToolItem`, `toggleTool`, `selectAllTools`, `expectToolsListNotVisible`, `expectToolsEmptyState`, `expectToolsLoadingHidden`
- Update playground methods that reference whitelisting: `expectNotWhitelistedWarning`, `expectNoWhitelistedWarning`
- Keep playground execution methods but update for new flow

### 6.2 Update E2E Test Specs

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-crud.spec.mjs`
- Remove/update: "MCP Server Tool Discovery" test — tool discovery now happens in chat/playground, not during creation
- Update: "MCP Playground" tests — execution goes through MCP client, not our API
- Remove: "Non-Whitelisted Tool Error" test — no more whitelisting
- Update: "Refresh and Disabled States" test — disabled state still applies, refresh concept changes

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs`
- Update: tests that fetch tools during creation — creation no longer has tool step
- Keep: auth config validation tests

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs`
- Remove: references to `tools/refresh` endpoint
- Keep: proxy endpoint access restriction tests

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-mcp-proxy-everything.spec.mjs`
- Keep as-is — this is the black-box MCP Inspector-driven test that validates the proxy

**Gate check**: `make build.ui-rebuild && npm run test:playwright` (from `crates/lib_bodhiserver_napi`)

---

## Layer 7: Documentation Updates

### 7.1 Techdebt Plan

**File**: `ai-docs/claude-plans/202603/20260329-mcp-endpoint-techdebt.md`
- Mark "Deprecate Legacy Tool Endpoints" as **DONE**
- Mark "tools_filter Enforcement via JSON Inspection" as **REMOVED** — design decision to drop tool filtering entirely
- Update "Consolidate /mcp endpoint to /apps/ only" — now blocked because frontend chat/playground use session-cookie route

### 7.2 Crate CLAUDE.md Updates

Update after all changes are complete for modified crates: services, routes_app, bodhi/src.

---

## Verification

### Backend
```bash
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
cargo test -p server_app 2>&1 | grep -E "test result|FAILED"
```

### TypeScript Client
```bash
cargo run --package xtask openapi
make build.ts-client
make ci.ts-client-check
```

### Frontend Unit Tests
```bash
cd crates/bodhi && npm test
```

### E2E Tests
```bash
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

### Manual Smoke Test
```bash
make app.run
```
- Navigate to `/ui/chat`, enable an MCP, verify tools appear in popover from live connection
- Send a message that triggers tool calls, verify agentic loop works
- Navigate to `/ui/mcps/playground`, verify tool listing and execution via MCP client
- Create a new MCP instance — verify no tool selection step

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| SQLite `ALTER TABLE DROP COLUMN` (requires 3.35+) | SeaORM targets modern SQLite; if fails, use raw SQL table recreation |
| Vite bundling of `@modelcontextprotocol/sdk` deep imports | May need `optimizeDeps.include` in `vite.config.ts` |
| MCP SDK in Vitest (node-specific deps) | Fully mock SDK classes in unit tests |
| React strict mode double-mounting effects | Use refs for Client/Transport, cleanup in effect return |
| Stale tool selection in localStorage after MCP tool changes | Gracefully ignore selected tools not in live list |
| Connection drops mid-conversation | Return error message to LLM via `callTool` error handling |
