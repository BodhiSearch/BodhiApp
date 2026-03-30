# MCP Proxy Endpoint — Tech Debt

Tracking future work items from the MCP-compliant stateful proxy endpoint implementation.

## `BODHI_INSTANCE_MODE=single|cluster`

Add setting to distinguish single-process deployment (native app, single Docker container) from multi-instance cluster (Kubernetes, replicated containers behind LB).

Current stateful MCP proxy uses `LocalSessionManager` (in-memory sessions per process). For cluster:
- Requires sticky sessions (LB affinity by `Mcp-Session-Id` header) OR distributed `SessionManager`
- If sticky sessions guaranteed (e.g., K8s ingress session affinity), `LocalSessionManager` works as-is
- Otherwise, implement Redis-backed `SessionManager` (rmcp's `SessionManager` trait is pluggable)

## Cluster-Specific Constraints

These constraints exist because the current implementation uses in-memory state:

- **Session affinity**: MCP sessions are stateful. If client requests hit different instances, sessions won't be found (404). Need LB sticky sessions or distributed session state.
- **Upstream connection per instance**: Each session has a persistent upstream connection on the instance that created it. Cannot be migrated between instances without reconnecting.
- **Notification routing**: Upstream notifications are delivered to the instance holding the upstream connection. If client reconnects to a different instance, notifications are lost.
- **Event ID caching**: `LocalSessionManager` caches event IDs in memory for stream resumption. In cluster, cache is per-instance. Resumption only works on the same instance.
- **Session cleanup**: If an instance crashes, its sessions are lost. Clients must re-initialize. For graceful shutdown, sessions should drain before termination.

## Connection Pooling

Pool persistent upstream MCP connections per server. Reuse across sessions to avoid repeated connection + handshake overhead. Must handle connection health checks, eviction, and per-MCP-instance auth isolation.

## CacheService for Cluster

`MokaCacheService` is in-memory per-process. For cluster deployment:
- Option A: Redis-backed CacheService
- Option B: PostgreSQL-backed cache (already have Postgres in multi-tenant)
- Option C: Make CacheService trait pluggable with configurable backend based on `BODHI_INSTANCE_MODE`
- Add configurable TTL to server info cache entries

## Deprecate Legacy Tool Endpoints

These endpoints are superseded by the MCP protocol proxy:
- `POST /bodhi/v1/mcps/{id}/tools/{tool_name}/execute` -> MCP `tools/call`
- `GET /bodhi/v1/mcps/{id}/tools` -> MCP `tools/list`
- `POST /bodhi/v1/mcps/{id}/tools/refresh` -> MCP `initialize`
- Same for `/apps/` variants

Frontend should migrate to using the MCP protocol endpoint for tool invocation. Keep legacy endpoints functional during deprecation period.

## Resource/Prompt Access Control

Currently `tools_filter` controls which tools are exposed. Resources, prompts, and completions are transparently proxied with no filtering.

Future: Add allow/disallow boolean toggles per MCP instance for `resources`, `prompts`, `completions`. These control whether the proxy forwards these operation categories at all. Fine-grained resource/prompt filtering (by name/URI pattern) deferred until concrete use case.

## Notification Forwarding

The current `McpProxyHandler` does not implement `UpstreamNotificationForwarder` (a `ClientHandler` that bridges upstream notifications to the downstream SSE stream). The `PersistentMcpConnection` connects with a default handler that doesn't forward notifications.

To implement: capture `context.peer` (downstream `Peer<RoleServer>`) during `initialize`, pass it to a custom `ClientHandler` on the upstream connection that forwards `on_resource_updated`, `on_tool_list_changed`, `on_prompt_list_changed`, etc. to the downstream client. Requires changes to `connect_persistent` to accept a notification handler.

## E2E Tests: Rewrite as Black-Box Tests

The current MCP proxy E2E tests in `specs/mcps/mcps-mcp-proxy.spec.mjs` are **white-box** — they use `page.evaluate()` and `page.context().request` to make raw HTTP/MCP protocol calls directly, bypassing the UI entirely. This violates the project's black-box E2E testing convention.

### What needs to change

The test app (React OAuth test app at port 55173, or a new dedicated MCP client test app) should be enhanced to exercise the `/mcp` endpoint through its UI. The E2E tests should then interact with that test app UI via Playwright, not make direct API/protocol calls.

### Methods in McpsPage that are white-box (to be replaced)

These methods in `pages/McpsPage.mjs` use `page.evaluate()` or `page.context().request` — they should be replaced with UI-driven equivalents once the test app supports MCP proxy:

- `createMcpServerViaApi()` — uses `page.evaluate()` + `fetch()`. Should use the existing UI flow via `createMcpServer()` instead.
- `createMcpInstanceViaApi()` — uses `page.evaluate()` + `fetch()`. Should use the existing UI flow via `createMcpInstance()` instead.
- `initializeMcpSession()` — uses `page.context().request.fetch()` to POST MCP `initialize`. Needs a test app that acts as an MCP client.
- `#getCookieHeader()` — uses `page.context().storageState()` to extract cookies for manual injection. Only needed because of the `page.context().request` approach.
- `mcpRequest()` — uses `page.context().request.fetch()` to send MCP JSON-RPC requests with session ID.
- `mcpNotify()` — uses `page.context().request.fetch()` to send MCP notifications.
- `mcpDeleteSession()` — uses `page.context().request.fetch()` to send DELETE with session ID.

### Direct page.evaluate in spec file

`mcps-mcp-proxy.spec.mjs` also has a direct `page.evaluate()` call in the "delete session invalidates it" test to verify that a deleted session returns 4xx. This should also go through a test app UI.

### Browser Accept header limitation

A key finding: browser `fetch()` inside `page.evaluate()` **ignores custom Accept headers** and sends Chromium defaults (`text/html,application/xhtml+xml,...`). This is why the methods were switched to `page.context().request.fetch()`. When rewriting as black-box tests via a test app, this limitation won't apply since the test app itself (not the browser's fetch) will set the correct MCP headers.

### Approach

1. Enhance the React test app (or create a new MCP client test app) to:
   - Connect to a configured MCP server URL via the `/mcp` proxy endpoint
   - Display MCP session status, tools list, resource list, prompt list
   - Allow executing tools, reading resources, getting prompts through UI
   - Show server push notifications in the UI
2. Rewrite `mcps-mcp-proxy.spec.mjs` to interact with the test app UI
3. Remove all `page.evaluate()`/`page.context().request` MCP protocol methods from `McpsPage`
