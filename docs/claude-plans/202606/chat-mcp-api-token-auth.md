# Chat MCP connections honor the API Token

## Context

In `/ui/chat/` the user can switch inference to an **API Token** (Settings → API Token toggle +
input). Today that token is used **only for model inference** — MCP tool connections always go
through the **session cookie**, regardless of the token. So a chat driven by a scoped API token can
still reach MCP servers the token isn't granted, and the MCP layer ignores the token entirely.

This change makes the chat's MCP connections **follow the API-token state**: when token mode is on,
MCPs connect with `Authorization: Bearer <token>` (so the backend's per-token MCP grant enforcement
applies); when off, they connect via the session cookie. On committing a token, the chat introspects
it via `GET /bodhi/v1/user`, **removes selected MCPs the token can't reach**, and reconnects the rest
under the token. Switching the token off reconnects the (remaining) selection via the session.

## Key finding (resolved during analysis)

**The MCP connect/proxy endpoint is already unified** — no URL switching and no backend change are
needed for connecting:

- `mcp_proxy_path(id)` always returns `/bodhi/v1/apps/mcps/{id}/mcp`
  (`crates/services/src/mcps/mcp_objs.rs:11`), and that is the **only** proxy route registered
  (`crates/routes_app/src/routes.rs:411`). Both the session list (`/bodhi/v1/mcps`) and the apps list
  (`/bodhi/v1/apps/mcps`) stamp that **same** `path` onto every `Mcp`.
- So the chat **already connects through `/apps/mcps/{id}/mcp` with the session cookie today**
  (`useMcpClients.ts`, `credentials:'include'`). That endpoint already accepts session, API-token,
  and external-app auth, and runs `auth_scope.access_policy().ensure_mcp_connect(id)` — which is
  `Unrestricted` for a session and grant-enforced (`403 McpForbidden`) for an API token.

Therefore **session vs. API-token differs only in the auth header and the discovery/list endpoint**,
not the connect path. This is a **frontend-only** change (plus tests). The earlier idea of "support
session on `/apps/mcps`" is already true for connecting.

## Locked decisions (from interview)

1. **Prune is destructive** — a selected MCP the token can't reach is **removed** from the chat's MCP
   selection (`mcpSelectionStore.removeMcp`); it does not auto-return when the token is switched off.
2. **Add-picker source** — in token mode the "add an MCP server" picker lists from
   `/bodhi/v1/apps/mcps` (with the Bearer), so only token-reachable servers are offered.
3. **Scope = chat only** — the MCP Playground (`useMcpClient`, single connection) stays on session
   auth for now; the same seam can be reused later.

## Current architecture (the seams)

- **Token state** — `crates/bodhi/src/stores/chatSettingsStore.ts`: `api_token` + `api_token_enabled`
  in sessionStorage; setters `setApiToken` / `setApiTokenEnabled`. Inference reads it in
  `stores/agentStore.ts` (`createBodhiStreamFn` sets `Authorization: Bearer` + `x-api-key`).
- **Token input** — `routes/chat/-components/settings/ParametersPane.tsx`: `#api-token-enabled`
  switch + `#api-token` input (`data-testid="api-token-input"`); currently commits on **change**.
- **MCP selection** — `stores/mcpSelectionStore.ts`: `enabledTools: Record<mcpId, toolName[]>` in
  localStorage; `addMcp` / `removeMcp` / `setEnabledTools`.
- **MCP catalog** — `hooks/chat/useChatMcp.ts` uses `useListMcps()` (`/bodhi/v1/mcps`, session) →
  `mcps`; `addedMcps = mcps ∩ enabledTools` drives connections via a `useEffect`.
- **MCP connections** — `hooks/mcps/useMcpClients.ts`: `connectAll(mcps)` diffs by **`mcp.path`**
  (connect/disconnect when the path for an id changes); transport built with
  `credentialFetch = (url, init) => fetch(url, { ...init, credentials:'include' })` — **no
  Authorization seam**. `disconnectAll()` tears everything down.
- **Introspection** — `hooks/users/useUsers.ts` `useGetUser()` → `GET /bodhi/v1/user`. For a token it
  returns `UserResponse` with `auth_status:'api_token'` + `TokenInfo { role, models, mcps }`;
  `mcps: ResourceAccess` = `{ type:'all', list } | { type:'specific', list, ids }`
  (`@bodhiapp/ts-client`). `apiClient` (`lib/apiClient.ts`) is plain axios with no auth interceptor —
  a one-off `Authorization: Bearer` header can be passed per request.
- **Backend (unchanged)** — `/apps/mcps/{id}/mcp` enforces the grant (`ensure_mcp_connect` → 403);
  `GET /apps/mcps` already grant-filters via `policy.mcp_listable` (`routes_app/src/mcps/`).

## Design

A small **auth descriptor** flows from the chat settings into the MCP layer:

```
type McpAuth = { mode: 'session' } | { mode: 'token'; token: string };
```

Derived in `useChatMcp` from `chatSettingsStore`: `token` mode iff `api_token_enabled && committedToken`.

### 1. Bearer-aware transport + reconnect-on-auth-change (`useMcpClients.ts`)
- `connectAll(mcps, auth)` gains the `auth` arg. The per-connection `fetch` injects the header in
  token mode:
  `(url, init) => fetch(url, { ...init, headers: { ...init?.headers, Authorization: 'Bearer '+token }, credentials: 'omit' })`;
  session mode keeps `credentials:'include'` (today's behavior).
- Because the **path is identical** across modes, fold an `authKey` into the connection identity used
  by the diff: `identity = `${mcp.path}::${auth.mode==='token' ? 't:'+token : 's'}``. Track
  `connectedEndpointsRef` by this identity so an auth change (toggle, or a new token value on blur)
  diffs → disconnect old + reconnect new. This implements "disconnect connected MCPs, reconnect under
  the token" and the reverse, reusing the existing diff machinery.

### 2. Commit-on-blur (`ParametersPane.tsx`)
- The token field keeps a local draft; **on blur** it commits to `chatSettingsStore` (the "active"
  token). Only the committed value feeds `authKey` + introspection, so reconnect/introspection fire
  once per blur, not per keystroke. (Toggling the switch also commits.)

### 3. Introspect + prune (new `hooks/chat/useApiTokenMcpAccess.ts`)
- When token mode is active and the committed token changes: `apiClient.get('/bodhi/v1/user', { headers: { Authorization: 'Bearer '+token } })`.
  - **Invalid/expired** (`auth_status !== 'api_token'`, or 401): surface an error (toast/inline),
    disconnect token-mode MCPs, and **do not** prune (only a valid grant prunes). Keep selection.
  - **Valid**: read `mcps`. Predicate `canConnect(id) = mcps.type==='all' || mcps.ids.includes(id)`.
    For each `mcpId` in `mcpSelectionStore.enabledTools`, if `!canConnect(mcpId)` →
    `mcpSelectionStore.removeMcp(mcpId)` (destructive, per decision). Surviving selection reconnects
    under the token via the effect in §1.

### 4. Token-filtered add-picker (`useChatMcp.ts` + new `hooks/mcps/useListAppsMcps.ts`)
- New `useListAppsMcps(token)` → `GET /bodhi/v1/apps/mcps` with the Bearer (backend already
  grant-filters). In token mode `useChatMcp` sources its catalog from this; in session mode it keeps
  `useListMcps()`. Both return `Mcp` objects with the same `/apps/mcps/{id}/mcp` `path`, so
  `connectAll` is unaffected. `McpServersPane`'s "available to add" then only offers reachable
  servers. (Listable-but-not-connectable is a corner case under `list_mcps=on`+`specific`; the §3
  prune + the backend 403 cover it.)

### Reconnect flow summary
- **Enable token + blur** → commit token → introspect → prune disallowed → `authKey` flips to
  `t:<token>` → `connectAll` disconnects session conns + reconnects survivors with Bearer; picker
  switches to `/apps/mcps`.
- **Change token + blur** → new `authKey` → reconnect under the new token; re-introspect + re-prune.
- **Disable token** → `authKey` → `'s'` → reconnect remaining selection via session; picker back to
  `/bodhi/v1/mcps`. (Removed MCPs stay removed.)

## Phases (frontend-only; commit per phase)

1. **Transport auth seam** — `useMcpClients.connectAll(mcps, auth)`: Bearer injection + `authKey` in
   the connection identity; `disconnectAll` unchanged. Unit-test: token mode sets the header; an auth
   change reconnects.
2. **Apps-list hook** — `useListAppsMcps(token)` (+ constants). Unit-test the request/headers.
3. **Introspect + prune** — `useApiTokenMcpAccess`; wire into `useChatMcp`; commit-on-blur in
   `ParametersPane`. Unit-test prune for `all` / `specific` / invalid-token.
4. **Wire catalog + auth into `useChatMcp`** — source catalog per mode, pass `auth` to `connectAll`.
   Unit/integration-test the mode switch reconnect + picker source.
5. **E2E + polish** — extend the chat-MCP E2E (below); optional small UI hint that MCP access is
   token-scoped.

## Testing

- **Unit (vitest, MSW)**:
  - `useMcpClients`: mock the transport/`fetch`; assert `Authorization: Bearer` present in token mode
    and absent in session mode; assert auth change → disconnect+reconnect (identity diff).
  - `useApiTokenMcpAccess`: MSW `/bodhi/v1/user` returning `api_token` with `mcps:{type:'specific',ids:[A]}`
    → `removeMcp(B)` called, `A` kept; `type:'all'` → no removal; `logged_out`/401 → error, no prune.
  - `useListAppsMcps`: hits `/bodhi/v1/apps/mcps` with the Bearer header.
  - `ParametersPane`: blur commits the token (store setter called once on blur, not per keystroke).
- **E2E (Playwright, `tests-js/specs/chat/chat-mcps.spec.mjs` or a new token-scoped spec)** — reuses
  the Everything MCP server + the token grant infra from `tokens/api-tokens.spec.mjs`:
  1. Create **two** MCP instances; create an API token granting **only one** (specific MCP grant).
  2. In chat, select both MCPs (session) and confirm both connect.
  3. Settings → enable API Token, paste the token, **blur**. Assert: the **non-granted** MCP is
     removed from the selection; the granted MCP reconnects; the add-picker no longer offers the
     non-granted server. Send a message that calls the granted MCP tool successfully under the token.
  4. Disable the token → the remaining selection reconnects via session.
  - Page objects: extend `ChatSettingsPage` (blur the token field) + `ChatPage`/`McpServersPane`
    selectors for connection status + removal. Black-box only (per `feedback_blackbox_e2e`); set
    `reducedMotion:'reduce'` if any rail/transition is involved; throw in `beforeAll` on missing env.

## Verification

1. Frontend unit: `cd crates/bodhi && npm test` (new MCP-auth + introspection + ParametersPane tests).
2. Chrome (`make app.run.live`): with a scoped token, switch modes in chat settings; confirm the
   network tab shows `Authorization: Bearer` on `/apps/mcps/{id}/mcp` in token mode and cookie-only in
   session mode; confirm the disallowed MCP disappears and tool calls work under the token.
3. E2E: `make build.dev-server` then `make test.e2e` (the chat-MCP token-scoped flow).
4. Gates before each commit: `make format`, unit tests, E2E when touched.

## Critical files

- `crates/bodhi/src/hooks/mcps/useMcpClients.ts` (transport auth + reconnect identity),
  new `hooks/mcps/useListAppsMcps.ts` (+ `hooks/mcps/constants.ts`).
- `crates/bodhi/src/hooks/chat/useChatMcp.ts` (auth descriptor + catalog source),
  new `hooks/chat/useApiTokenMcpAccess.ts` (introspect + prune).
- `crates/bodhi/src/routes/chat/-components/settings/ParametersPane.tsx` (commit-on-blur),
  `routes/chat/-components/settings/McpServersPane.tsx` (token-scoped picker).
- `crates/bodhi/src/stores/chatSettingsStore.ts` (committed token), `stores/mcpSelectionStore.ts`
  (reused `removeMcp`).
- Tests: `hooks/mcps/*.test.ts`, `hooks/chat/*.test.ts`, `routes/chat/-components/settings/*.test.tsx`,
  `lib_bodhiserver/tests-js/specs/chat/chat-mcps.spec.mjs` (+ `pages/ChatSettingsPage.mjs`).

## Notes / non-goals

- **No backend change** — the proxy already accepts session + Bearer and enforces token grants. (If a
  future review wants the add-picker's `/apps/mcps` to also accept a session cookie for symmetry,
  that's a separate, security-reviewed change; not needed here.)
- Backend token-grant types are being refactored in parallel (App Tokens / `ResourceGrants`); this
  plan only depends on the **stable** `GET /bodhi/v1/user` (`ResourceAccess`) and `/apps/mcps`
  contracts — rebase on latest before implementing.
- MCP Playground (`useMcpClient`) is out of scope; reuse this seam later if desired.
