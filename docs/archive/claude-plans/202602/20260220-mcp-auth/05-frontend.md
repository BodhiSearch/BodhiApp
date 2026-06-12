# MCP OAuth - Frontend

## Task

✅ **COMPLETED** - Implement OAuth authorization flow UI: Zustand form state store with sessionStorage persistence, OAuth callback page, MCP creation/edit with auth config dropdown and connect/disconnect, MCP server admin pages with inline auth config management, and API hooks.

**Critical Fix**: Resolved MSW v2 handler ordering bug causing 858 frontend test failures. Handler specificity now correctly prioritized.

## Why sessionStorage

OAuth redirects the user to an external authorization server, destroying all React component state. On return to the callback page, form state (selected auth config, tools, server selection) would be lost. sessionStorage preserves this state across the redirect-and-return cycle.

## Files

| File | Purpose | Status |
|------|---------|--------|
| `crates/bodhi/src/stores/mcpFormStore.ts` | Zustand store with session persistence | ✅ No changes |
| `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx` | OAuth callback page | ✅ No changes |
| `crates/bodhi/src/app/ui/mcps/oauth/callback/page.test.tsx` | Callback page tests | ✅ No changes |
| `crates/bodhi/src/app/ui/mcps/new/page.tsx` | MCP create/edit page | ✅ Updated dropdown labels |
| `crates/bodhi/src/app/ui/mcps/new/page.test.tsx` | MCP create/edit tests (130+ cases) | ✅ Updated assertions |
| `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx` | Server view with auth config management | ✅ Updated labels |
| `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx` | Server view tests | ✅ Updated assertions |
| `crates/bodhi/src/app/ui/mcp-servers/new/page.tsx` | Server create with optional auth config | ✅ No changes |
| `crates/bodhi/src/hooks/useMcps.ts` | API hooks (query + mutation) | ✅ **MAJOR UPDATES** |
| `crates/bodhi/src/test-utils/msw-v2/handlers/mcps.ts` | MSW mock handlers and data factories | ✅ **CRITICAL FIX** |

## mcpFormStore.ts (Zustand)

### State Fields
- `selectedAuthConfigId: string | null` - selected auth config ID
- `selectedAuthConfigType: string | null` - auth config type string
- `oauthTokenId: string | null` - token ID after successful exchange
- `isConnected: boolean` - OAuth connection status
- `fetchedTools: McpTool[]` - tools from MCP server
- `selectedTools: Set<string>` - user-selected tool names
- `toolsFetched: boolean` - tools have been fetched

### Actions
- `setSelectedAuthConfig(id, type)` - set auth config selection
- `completeOAuthFlow(tokenId)` - mark OAuth complete (sets isConnected=true)
- `disconnect()` - clear token and connection state
- `setFetchedTools(tools)` / `setSelectedTools(tools)` / `toggleTool(name)` / `selectAllTools()` / `deselectAllTools()` / `setToolsFetched(fetched)` - tool management

### Session Persistence
- `saveToSession(formValues, serverInfo?)` - saves form state + auth state + tools + return URL to sessionStorage key `'mcp_oauth_form_state'`
- `restoreFromSession()` - parses and removes sessionStorage item, returns merged state object
- `clearSession()` - removes sessionStorage item
- `reset()` - full state reset + clearSession

## OAuth Callback Page (`/ui/mcps/oauth/callback`)

URL params from OAuth provider: `code`, `state`, `error`, `error_description`

Flow:
1. If error from provider: show error with "Back to form" button
2. If missing code or state: show error
3. Restore sessionStorage (`OAUTH_FORM_STORAGE_KEY`)
4. Extract `selected_auth_config_id` and `mcp_server_id` from session
5. Call `useOAuthTokenExchange()` mutation
6. On success: store token ID in session, redirect to `return_url` or `/ui/mcps/new/`
7. On failure: show error with "Back to form" button

Key test IDs: `oauth-callback-page`, `oauth-callback-loading`, `oauth-callback-success`, `oauth-callback-error`, `oauth-callback-back`

## MCP Creation/Edit Page (`/ui/mcps/new/`)

### Auth Config Dropdown

✅ **UPDATED** - Options with simplified type badges:
- "Public (No Auth)" - sets auth_type='public'
- Existing configs from server (with type badges: `[Header]`, `[OAuth]`)
- "+ New Auth Config" (admin-only) - redirects to server settings

**Change**: Type badges now show `[OAuth]` instead of `[OAuth Pre-Reg]` / `[OAuth Dynamic]`, matching simplified enum.

Auto-selects first config when server is selected (create mode only).

### OAuth Connect Flow
1. User selects OAuth config, clicks "Connect"
2. `handleOAuthConnect()`: validates, deletes pending token if disconnect was pending, calls `saveToSession()`, calls `useOAuthLogin()`, redirects browser to `authorization_url`
3. User completes authorization at OAuth provider
4. Returns to callback page, token exchanged, redirect back to form
5. Form restores from sessionStorage, shows "Connected" card with disconnect option

### OAuth Disconnect
Lazy deletion: sets `pendingDeleteTokenId`, clears `isConnected`. Token deletion deferred until form submission. If user reconnects, pending delete is cancelled.

### Form Submission
POST `/mcps` (create) or PUT `/mcps/:id` (edit) with: `name`, `slug`, `mcp_server_id`, `description`, `enabled`, `auth_type`, `auth_uuid` (configId for header, tokenId for OAuth), `tools_cache`, `tools_filter`.

### Session Restoration (Post-OAuth)
On mount: `store.restoreFromSession()` populates form fields, server selection, auth config selection + connection status, tool cache + selection. Session data takes priority over API data in edit mode.

Key test IDs: `new-mcp-page`, `mcp-server-combobox`, `auth-config-select`, `auth-config-option-public`, `auth-config-option-{configId}`, `auth-config-option-new`, `auth-config-oauth-connect`, `oauth-connected-card`, `oauth-disconnect-button`, `mcp-fetch-tools-button`, `mcp-create-button`, `mcp-update-button`

## MCP Server View Page (`/ui/mcp-servers/view?id=<serverId>`)

### Auth Configs Section
- Toggle "Add Auth Config" button shows inline form
- Type dropdown: Header | OAuth Pre-Registered | OAuth Dynamic
- Form fields vary by type (header: key/value; OAuth: endpoints, client_id, scopes)
- OAuth Dynamic: auto-triggers `useDiscoverMcp()` on type selection, populates endpoints if successful, shows error with "Switch to Pre-Registered" fallback on failure
- For dynamic: calls `useStandaloneDynamicRegister()` before creating config
- Save calls `useCreateAuthConfig()` (unified endpoint)
- Delete with confirmation dialog, calls `useDeleteAuthConfig()`, cascades token deletion for OAuth

Key test IDs: `auth-configs-section`, `add-auth-config-button`, `auth-config-form`, `auth-config-type-select`, `auth-config-save-button`, `auth-config-row-{configId}`, `auth-config-delete-button-{configId}`

## MCP Server New Page (`/ui/mcp-servers/new`)

Optional "Authentication Configuration" collapsible section. Same auth config form as server view. On save: calls `useCreateMcpServer()` with optional `auth_config` field for atomic creation.

## API Hooks (useMcps.ts)

✅ **MAJOR UPDATES** - Removed type-specific hooks, unified endpoints

### Endpoint Constants (Updated)

**Git diff changes:**
```typescript
-export const MCPS_AUTH_HEADERS_ENDPOINT = `${BODHI_API_BASE}/mcps/auth-headers`;
-export const MCP_SERVERS_ENDPOINT = `${BODHI_API_BASE}/mcp_servers`;
-export const MCP_SERVERS_OAUTH_CONFIGS_ENDPOINT = (serverId: string) =>
-  `${BODHI_API_BASE}/mcp-servers/${serverId}/oauth-configs`;
-export const MCP_AUTH_CONFIGS_ENDPOINT = (serverId: string) =>
-  `${BODHI_API_BASE}/mcp-servers/${serverId}/auth-configs`;
+export const MCP_SERVERS_ENDPOINT = `${BODHI_API_BASE}/mcps/servers`;
+export const MCPS_AUTH_CONFIGS_ENDPOINT = `${BODHI_API_BASE}/mcps/auth-configs`;
```

Retained endpoints:
- `MCPS_OAUTH_DISCOVER_AS_ENDPOINT` = `/bodhi/v1/mcps/oauth/discover-as`
- `MCPS_OAUTH_DISCOVER_MCP_ENDPOINT` = `/bodhi/v1/mcps/oauth/discover-mcp`
- `MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE_ENDPOINT` = `/bodhi/v1/mcps/oauth/dynamic-register`
- `MCPS_OAUTH_TOKENS_ENDPOINT` = `/bodhi/v1/mcps/oauth-tokens`

### Query Hooks (Simplified)

**Removed**:
- ~~`useListAuthHeaders()`~~ - replaced by `useListAuthConfigs()`
- ~~`useAuthHeader()`~~ - replaced by `useGetAuthConfig()`
- ~~`useListOAuthConfigs()`~~ - replaced by `useListAuthConfigs()`
- ~~`useOAuthConfig()`~~ - replaced by `useGetAuthConfig()`

**Updated**:
- `useListAuthConfigs(serverId)` - uses `?mcp_server_id=` query param instead of path nesting
- `useGetAuthConfig(configId)` - removed `serverId` parameter (no longer needed)

**Retained**:
- `useGetOAuthToken(tokenId)` - token metadata

### Mutation Hooks (Simplified)

**Removed**:
- ~~`useCreateAuthHeader()`~~ - replaced by `useCreateAuthConfig()`
- ~~`useUpdateAuthHeader()`~~ - not yet implemented
- ~~`useDeleteAuthHeader()`~~ - replaced by `useDeleteAuthConfig()`
- ~~`useCreateOAuthConfig()`~~ - replaced by `useCreateAuthConfig()`
- ~~`useDynamicRegister()`~~ - server-scoped variant removed

**Retained/Updated**:
- `useDiscoverAs()` - RFC 8414 discovery
- `useDiscoverMcp()` - RFC 9728 + RFC 8414 discovery
- `useStandaloneDynamicRegister()` - standalone DCR (only variant remaining)
- `useOAuthLogin()` - updated path from `/mcp-servers/{id}/oauth-configs/{config_id}/login` to `/mcps/auth-configs/{id}/login`
- `useOAuthTokenExchange()` - updated path similarly
- `useDeleteOAuthToken()` - token deletion
- `useCreateAuthConfig()` - unified create (discriminated union)
- `useDeleteAuthConfig()` - unified delete

## MSW Test Handlers (mcps.ts)

✅ **CRITICAL FIX** - Handler ordering bug resolved (caused 858 test failures)

### MSW v2 Handler Ordering Issue

**Problem**: MSW v2 matches handlers in registration order. Generic patterns like `/mcps/auth-configs` were registered BEFORE specific patterns like `/mcps/auth-configs/{id}/login`, causing all requests to match the generic handler.

**Solution from git diff:**
```typescript
// CORRECT: Specific patterns FIRST, generic patterns LAST
export const mcpHandlers = [
  // ...
  http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/login`, async ({ ...})), // SPECIFIC
  http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/token`, async ({ ...})), // SPECIFIC
  http.post(`${BODHI_API_BASE}/mcps/auth-configs`, async ({ request }) => { ... }), // GENERIC
  http.get(`${BODHI_API_BASE}/mcps/auth-configs`, async ({ request }) => { ... }), // GENERIC
];
```

**Impact**: All 858 frontend component tests now passing. This was the root cause of widespread test failures.

### Mock Data Factories

✅ **UPDATED** - Simplified OAuth config types:
- `mockAuthHeader` - header auth response
- `mockOAuthConfig` - OAuth config response (now unified, uses `registration_type` field)
- ~~`mockDcrOAuthConfig`~~ - merged into `mockOAuthConfig`
- `mockOAuthToken` - OAuth token response
- `mockAuthConfigHeader` - unified auth config response (Header variant)
- `mockAuthConfigOAuth` - unified auth config response (Oauth variant, replaces `OAuthPreReg` and `OAuthDynamic`)

### Handler Factories

✅ **UPDATED** - Functions return MSW handlers with correct ordering:
- `mockListAuthConfigs()` - unified list (header + OAuth)
- `mockCreateAuthConfig()` - discriminated union create
- `mockDeleteAuthConfig()` - unified delete
- `mockDiscoverAs()` - RFC 8414 discovery
- `mockDiscoverMcp()` - RFC 9728 + RFC 8414 discovery
- `mockDynamicRegister()` - standalone DCR (server-scoped variant removed)
- `mockOAuthLogin()` - PKCE login (MUST be registered before generic auth-configs handlers)
- `mockOAuthTokenExchange()` - token exchange (MUST be registered before generic auth-configs handlers)
- `mockGetOAuthToken()` - token metadata
- `mockDeleteOAuthToken()` - token deletion

Error variants available for testing failure scenarios.

## Cross-References

- API endpoints these hooks call: [04-routes-app.md](./04-routes-app.md)
- E2E tests exercising these pages: [06-e2e.md](./06-e2e.md)
