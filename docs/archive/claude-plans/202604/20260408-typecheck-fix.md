# Fix typecheck failures in crates/bodhi

## Context

`npm run test:typecheck` (`tsc --noEmit`) produces 200 errors across 50 files. These stem from the recent chat migration to pi-agent-core, ts-client type regeneration (adding `path` to `Mcp`, removing `McpAuth`), and the TanStack Router migration (`trailingSlash: 'always'`). The errors fall into 7 distinct categories.

## Batch 1: tsconfig fixes (2 changes, ~50 errors)

**File:** `crates/bodhi/tsconfig.json`

1. Add `"vitest/globals"` to `compilerOptions.types` array — fixes all `describe`/`it`/`expect`/`beforeEach`/`afterEach`/`beforeAll`/`afterAll` not found errors in test files (~40 errors across 12 test files)
2. Add `"downlevelIteration": true` to `compilerOptions` — fixes all `Map can only be iterated through when using --downlevelIteration` errors (5 errors in `useMcpAgentTools.ts`, `useMcpClients.ts`, `ChatUI.tsx`)

## Batch 2: Trailing slash on route constants (~40 errors)

**File:** `crates/bodhi/src/lib/constants.ts`

Add trailing slashes to all route constants:
- `ROUTE_LOGIN = '/login/'`
- `ROUTE_SETUP = '/setup/'`
- `ROUTE_DEFAULT = '/chat/'` (also `ROUTE_CHAT`)
- `ROUTE_SETUP_TENANTS = '/setup/tenants/'`
- `ROUTE_REQUEST_ACCESS = '/request-access/'`
- `ROUTE_RESOURCE_ADMIN = '/setup/resource-admin/'`
- `ROUTE_MCP_SERVERS = '/mcps/servers/'`
- `ROUTE_SETUP_RESOURCE_ADMIN = '/setup/resource-admin/'`
- `ROUTE_SETUP_DOWNLOAD_MODELS = '/setup/download-models/'`
- `ROUTE_SETUP_API_MODELS = '/setup/api-models/'`
- `ROUTE_SETUP_BROWSER_EXTENSION = '/setup/browser-extension/'`
- `ROUTE_SETUP_COMPLETE = '/setup/complete/'`

This fixes errors in `AppInitializer.tsx`, `useUsers.ts`, `login/index.tsx`, `request-access/index.tsx`, `setup/index.tsx`, `setup/api-models/index.tsx`, `setup/browser-extension/index.tsx`, `setup/complete/index.tsx`, `setup/download-models/index.tsx`, `AliasForm.tsx`, `mcps/servers/new/index.tsx`, `mcps/servers/edit/index.tsx`.

## Batch 3: Inline route string literals needing trailing slash (~20 errors)

Files that use string literals instead of constants — add trailing slash:
- `src/routes/mcps/index.tsx`: `'/mcps/playground'` -> `'/mcps/playground/'`, `'/mcps/new'` -> `'/mcps/new/'`
- `src/routes/mcps/new/index.tsx`: `'/mcps'` -> `'/mcps/'`
- `src/routes/mcps/new/-components/McpServerSelector.tsx`: `'/mcps/servers/new'` -> `'/mcps/servers/new/'`
- `src/routes/mcps/servers/index.tsx`: `${ROUTE_MCP_SERVERS}/view` -> `${ROUTE_MCP_SERVERS}view/` (since ROUTE_MCP_SERVERS will now end with `/`), `${ROUTE_MCP_SERVERS}/new` -> `${ROUTE_MCP_SERVERS}new/`
- `src/routes/mcps/servers/view/index.tsx`: `${ROUTE_MCP_SERVERS}/edit` -> `${ROUTE_MCP_SERVERS}edit/`
- `src/routes/mcps/new/index.tsx`: `${ROUTE_MCP_SERVERS}/view/` -> `${ROUTE_MCP_SERVERS}view/` (ROUTE_MCP_SERVERS already has trailing slash)
- `src/routes/models/index.tsx`: `'/models/api/edit'` -> `'/models/api/edit/'`, `'/models/alias/edit'` -> `'/models/alias/edit/'`, `'/models/alias/new'` -> `'/models/alias/new/'`, `'/models/api/new'` -> `'/models/api/new/'`, `'/chat'` -> `'/chat/'`, `'/models'` -> not needed (covered by constant)
- `src/routes/models/alias/-components/AliasForm.tsx`: `'/models'` -> `'/models/'`
- `src/routes/auth/dashboard/callback/index.tsx`: `'/login'` -> `'/login/'`

**Note:** After ROUTE_MCP_SERVERS gets trailing slash (`/mcps/servers/`), all `${ROUTE_MCP_SERVERS}/xxx` become `${ROUTE_MCP_SERVERS}xxx/` (drop the extra `/`).

## Batch 4: Route search param validation (~15 errors)

Routes that navigate with `search` params to routes that don't declare `validateSearch`:

- `src/routes/mcps/playground/index.tsx`: already has `validateSearch` with `id` -- just needs trailing slash fix (covered above)
- `src/routes/mcps/new/index.tsx`: already has `validateSearch` with `id` -- just needs trailing slash fix
- `src/routes/models/alias/edit/index.tsx`: already has `validateSearch` with `id` -- just needs trailing slash fix
- `src/routes/chat/index.tsx`: already has `validateSearch` with `model` and `id` -- needs trailing slash fix
- `src/routes/mcps/servers/view/index.tsx`: needs `validateSearch: z.object({ id: z.string().optional() })`
- `src/routes/mcps/servers/edit/index.tsx`: needs `validateSearch: z.object({ id: z.string().optional() })`
- `src/routes/models/api/edit/index.tsx`: needs `validateSearch: z.object({ id: z.string().optional() })`
- `src/routes/models/alias/new/index.tsx`: needs `validateSearch: z.object({ repo: z.string().optional(), filename: z.string().optional(), snapshot: z.string().optional() })`
- `src/routes/models/index.tsx` line 133: navigates to `/chat/` with `search: { model }` -- covered by trailing slash fix since `/chat/` already has validateSearch
- `src/routes/auth/dashboard/callback/index.tsx`: needs `validateSearch: z.object({ code: z.string().optional(), state: z.string().optional() })`
- `src/routes/login/index.tsx`: needs `validateSearch` for `error` and `inviteClientId` params
- `src/routes/mcps/oauth/callback/index.tsx`: needs `validateSearch` for `error`, `error_description`, `code`, `state`, `configId`, `mcpId`, `redirectUri` params

After adding `validateSearch`, the `{}` type errors on `useSearch` results will resolve since TypeScript will infer proper types.

## Batch 5: pi-agent-core / pi-ai type fixes (~10 errors)

### 5a. Missing `timestamp` on AgentMessage (~5 errors)
Files: `agentStore.ts:149`, `useBodhiAgent.ts:258`, `useBodhiAgent.test.tsx:241,271`, `chat/index.test.tsx:324`

Add `timestamp: Date.now()` to the fallback AssistantMessage object in:
- `src/stores/agentStore.ts` line ~149 (message restoration)
- `src/hooks/chat/useBodhiAgent.ts` line ~258 (message restoration)

In test files, add `timestamp: Date.now()` to mock AgentMessage objects:
- `src/hooks/chat/useBodhiAgent.test.tsx` lines 241, 271
- `src/routes/chat/index.test.tsx` line 324

### 5b. `vi.fn` type arguments (2 errors)
Files: `useBodhiAgent.test.tsx:8`, `useMcpAgentTools.test.ts:9`

Vitest v2 changed `vi.fn` generic signature. Fix:
- `vi.fn<[string | AgentMessage | AgentMessage[]], Promise<void>>()` -> `vi.fn<(input: string | AgentMessage | AgentMessage[]) => Promise<void>>()`
- `vi.fn<[string, string, Record<string, unknown>], Promise<McpToolCallResult>>()` -> `vi.fn<(mcpId: string, toolName: string, args: Record<string, unknown>) => Promise<McpToolCallResult>>()`

### 5c. `execute` signature mismatch (1 error)
File: `useMcpAgentTools.ts:44`

Change `params: Record<string, unknown>` to `params: unknown` to match the pi-agent-core `AgentTool.execute` signature, then cast inside:
```typescript
execute: async (_toolCallId: string, params: unknown): Promise<AgentToolResult<unknown>> => {
  const typedParams = params as Record<string, unknown>;
  // ... use typedParams
```

## Batch 6: ts-client type alignment (~12 errors)

### 6a. `McpAuth` removed from ts-client (1 error)
File: `src/hooks/mcps/useMcpAuthConfigs.ts:6`

`McpAuth` no longer exists. Replace with `McpAuthType` in both the import (line 6) and re-export (line 148).

### 6b. `isError` typed as `unknown` (2 errors)
Files: `useMcpClient.ts:103`, `useMcpClients.ts:191`

The MCP SDK `CallToolResult.isError` is typed as `unknown`. Cast: `isError: result.isError as boolean`.

### 6c. Missing `path` in test fixture Mcp objects (8 errors)
File: `src/test-fixtures/apps.ts`

Add `path: '/mcp/mcp-instance-1'` (or similar) to every Mcp object fixture (lines 23, 145, 182, 202, 239, 280, 317, 375).

### 6d. Missing `path` in McpApproval.instance (1 error)
File: `src/routes/apps/access-requests/review/index.tsx:257`

Add `path` to the `instance` object: `instance: mcp.instance ? { id: mcp.instance.id, path: mcp.instance.path } : undefined`

## Batch 7: Miscellaneous (5 errors)

### 7a. `style jsx` not supported (1 error)
File: `src/routes/setup/complete/index.tsx:102`

Remove `jsx` attribute: `<style>{...}</style>` or convert to a CSS module / inline styles. Since this is a keyframes animation, convert to a `<style>` tag without `jsx` prop (just `<style>`).

### 7b. `mockChatModel` missing required fields (1 error)
File: `src/routes/chat/index.test.tsx:229`

Add missing `source` and `snapshot` fields to mock model object.

### 7c. `AliasResponse` type mismatch (1 error)
File: `src/routes/models/alias/new/index.tsx:35`

The `initialData` constructed inline is missing fields required by `AliasResponse`. Need to add `id`, `model_params`, `created_at`, `updated_at` fields, or change type to `Partial<AliasResponse>` if the component supports it.

### 7d. `search` on navigate to `/chat` from ModelActions (2 errors)
File: `src/routes/models/-components/ModelActions.tsx:83,125`

These navigate to `/chat` with `search: { model }` but use string type for `to`. Fix: change to `'/chat/'` with trailing slash, which has `validateSearch` defined.

## Verification

After all fixes:
```bash
cd crates/bodhi && npm run test:typecheck
cd crates/bodhi && npm test
```

Then rebuild and run app:
```bash
make build.ui-rebuild
make app.run
```
