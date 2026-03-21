# Plan: Fix Next.js → Vite + TanStack Router Migration Issues

## Context

BodhiApp migrated its frontend from Next.js 14 to Vite + TanStack Router (commit `bdc20c80b`). The migration is functionally complete but a thorough review found 25 actionable issues: missing PWA assets, broken route schemas, type-unsafe search params, dead dependencies, a chat URL feature request, and various cleanup items. Review files are at `ai-docs/claude-plans/20260321-migrate-tanstack-v5/`.

## Scope Boundaries
- **In scope**: Migration fixes, dead dep removal, inline code comment updates, PWA restoration, chat URL feature
- **Out of scope**: *.md doc updates (CLAUDE.md etc.), dark theme, new/edit URL unification, library upgrades, font bundling, build.rs rerun-if-changed

---

## Phase 1: PWA Restoration
**Findings**: #1, #2

Restore PWA assets that were removed in prior commits. The `index.html` already references them.

| Action | File |
|--------|------|
| Restore from git (`bc2bc8f67^`) | `crates/bodhi/public/manifest.json` — update paths for `/ui/` base (`scope`, `start_url`, icon `src` all need `/ui/` prefix) |
| Restore from git (`bc2bc8f67^`) | `crates/bodhi/public/icon-192x192.png` |
| Restore from git (`bc2bc8f67^`) | `crates/bodhi/public/icon-256x256.png` |
| Restore from git (`bc2bc8f67^`) | `crates/bodhi/public/icon-384x384.png` |
| Restore from git (`bc2bc8f67^`) | `crates/bodhi/public/icon-512x512.png` |
| Move `src/app/favicon.ico` → `public/favicon.ico` | Vite serves `public/` at base path |

**Verify**: `cd crates/bodhi && npm run build` — confirm `out/manifest.json`, `out/favicon.ico`, `out/icon-*.png` exist.

---

## Phase 2: Build Config Cleanup
**Findings**: #12, #13, #14, #16, #17

| Action | File | Detail |
|--------|------|--------|
| Remove 7 dead deps | `package.json` | `react-helmet-async`, `jest`, `@types/jest`, `vite-tsconfig-paths`, `happy-dom`, `@vitest/coverage-v8`, `dotenv` |
| Move `@types/prismjs` | `package.json` | From `dependencies` → `devDependencies` |
| Fix content paths | `tailwind.config.ts` | Replace `['./pages/**', './components/**', './app/**', './src/**']` with `['./src/**/*.{ts,tsx}', './index.html']` |
| Add ESLint ignore | `.eslintrc.json` | Add `"src/routeTree.gen.ts"` to `ignorePatterns` |
| Disable sourcemaps | `vite.config.ts` | Change `sourcemap: true` → `sourcemap: false` |

**Verify**: `cd crates/bodhi && npm install && npm run build && npm run lint`

---

## Phase 3: Route Schema Fixes
**Findings**: #3 (Critical), #9, #10, #11, #18, plus #4 prep

Fix `validateSearch` schemas to match what page components actually read.

| Route File | Current Schema | Fix |
|------------|---------------|-----|
| `routes/mcps/new/index.tsx` | `{ server_id, mode }` (WRONG) | `{ id: z.string().optional() }` |
| `routes/models/alias/new/index.tsx` | `{}.passthrough()` | `{ repo, filename, snapshot }` all optional strings |
| `routes/mcps/oauth/callback/index.tsx` | `{}.passthrough()` | `{ code, state, error, error_description }` all optional strings |
| `routes/auth/dashboard/callback/index.tsx` | `{}.passthrough()` | `{ code, state }` all optional strings |
| `routes/models/files/pull/index.tsx` | `{ repo, filename }` (unused) | Remove `validateSearch` entirely |
| `routes/chat/index.tsx` | `{ model }` | Add `id: z.string().optional()` for Phase 5 |

**Verify**: `cd crates/bodhi && npm test`

---

## Phase 4: useSearch Type Safety + navigate Fixes
**Findings**: #5, #6

**Pattern**: In each page file, replace:
```ts
const search = useSearch({ strict: false }) as Record<string, string | undefined>;
```
with typed search using route import. The exact approach depends on whether importing `Route` from the route file creates circular deps. TanStack Router supports this — route files import page components, page files import `Route` for types only.

**13 files to update** (skip `auth/callback/page.tsx` — it uses `.passthrough()` + `Object.entries()` intentionally):

`login/page.tsx`, `auth/dashboard/callback/page.tsx`, `mcps/oauth/callback/page.tsx`, `chat/page.tsx`, `mcps/new/page.tsx`, `mcps/servers/view/page.tsx`, `mcps/servers/edit/page.tsx`, `mcps/playground/page.tsx`, `toolsets/edit/page.tsx`, `apps/access-requests/review/page.tsx`, `models/api/edit/page.tsx`, `models/alias/new/page.tsx`, `models/alias/edit/page.tsx`

**Also fix 2 broken navigate calls** (confirmed bug — produces `/edit?id=xyz/` instead of `/edit/?id=xyz`):
- `mcps/servers/page.tsx:151` — `navigate({ to: \`${ROUTE_MCP_SERVERS}/edit\`, search: { id: server.id } })`
- `mcps/servers/edit/page.tsx:58` — `navigate({ to: \`${ROUTE_MCP_SERVERS}/view\`, search: { id: serverId } })`

**Update corresponding tests**: Change `useSearch` mock return values to match new typed patterns. The mock setup changes from mocking `useSearch` to return a plain object, to potentially adjusting how the mock is structured. Navigate assertions for the 2 fixed calls also need updating.

**Verify**: `cd crates/bodhi && npm test`

---

## Phase 5: Chat URL Feature
**Finding**: #4 (Bug/Feature)

When selecting a previous chat, update URL to `/ui/chat/?id=chatId`. When creating new chat, clear `id`.

**Architecture**: Keep `ChatDBProvider` URL-unaware. Add a `ChatUrlSync` component rendered inside `ChatDBProvider` context in `page.tsx`.

**Files**:

1. **`src/app/chat/page.tsx`** — `ChatWithHistory` component:
   - Read `id` from typed search (already available after Phase 4)
   - Add `<ChatUrlSync chatIdFromUrl={search.id} />` inside the `ChatDBProvider` tree
   - Define `ChatUrlSync` inline or in a separate file:
     - On mount: if `chatIdFromUrl` exists and chat exists in localStorage, call `setCurrentChatId(chatIdFromUrl)`
     - On `currentChatId` change: call `navigate({ to: '/chat', search: currentChatId ? { id: currentChatId } : {}, replace: true })`
     - Use `useRef` guard to avoid infinite loops (navigate → re-render → effect → navigate)
     - Preserve `model` param if present

2. **`src/app/chat/page.test.tsx`** — Add tests:
   - URL with `?id=existingChatId` loads that chat
   - URL with `?id=nonExistentId` falls back gracefully
   - Selecting a chat from history updates the navigate mock
   - New chat button updates navigate mock (clears or updates id)

**Verify**: `cd crates/bodhi && npm test`

---

## Phase 6: handleSmartRedirect Fix + Tests
**Findings**: #7, #8

1. **`src/lib/utils.ts`** — `handleSmartRedirect`:
   - Widen navigate type: `(opts: { to: string; search?: Record<string, string>; hash?: string }) => void`
   - For relative paths: parse with `new URL(location, window.location.origin)`, extract pathname/search/hash separately
   - For absolute same-origin URLs: same parsing, pass `{ to, search, hash }` to navigate
   - Strip `BASE_PATH` from pathname as before

2. **`src/lib/utils.test.ts`** (NEW) — 8 test cases:
   - Relative `/login` → `navigate({ to: '/login' })`
   - Relative `/ui/chat` → strips BASE_PATH → `navigate({ to: '/chat' })`
   - Relative `/ui` → `navigate({ to: '/' })`
   - Same-origin `http://localhost/ui/chat` → `navigate({ to: '/chat' })`
   - Same-origin with query `http://localhost/ui/chat?model=x` → `navigate({ to: '/chat', search: { model: 'x' } })`
   - Same-origin with hash → includes `hash` in navigate
   - External URL → `window.location.href`
   - Malformed URL → `window.location.href`

**Verify**: `cd crates/bodhi && npm test`

---

## Phase 7: Cleanup
**Findings**: #19, #20, #21, #22, #23, #24, #25

| Action | File | Detail |
|--------|------|--------|
| Remove `<Suspense>` wrapper | `auth/callback/page.tsx:109-131` | Replace with direct `<AuthCallbackContent />` export |
| Remove `<Suspense>` wrapper | `auth/dashboard/callback/page.tsx:62-68` | Replace with direct `<DashboardCallbackContent />` export |
| Update inline comment | `src/lib/constants.ts:1-2` | "Next.js basePath" → "Vite base path" |
| Update inline comment | `src-tauri/Makefile:12` | "Build Next.js frontend" → "Build Vite frontend" |
| Update inline comments | `routes_app/src/routes.rs`, `routes_app/src/routes_proxy.rs` | "Next.js dev server" → "Vite dev server" (if any exist — exploration found them referenced in backend-e2e review) |
| Rename mock variable | `useNavigation.test.tsx:8` | `mockUsePathname` → `mockPathname` + update all refs |
| Remove duplicate | `login/page.test.tsx` | Remove duplicate `navigateMock.mockClear()` in both `beforeEach` blocks |
| Standardize Link mock | Test files using `<Link search={...}>` | Add `search` prop handling to Link mock in files where page uses `<Link search={}>` |
| Add index.html check | `lib_bodhiserver/build.rs:105-113` | Add `out/index.html` existence check in `validate_frontend_assets()` |

**Verify**: `cd crates/bodhi && npm test` + `cargo check -p lib_bodhiserver`

---

## Phase 8: E2E Popover Consistency
**Finding**: #15

1. **Investigate first**: Run E2E tests to see if Escape-based close is currently working
2. **`ChatPage.mjs:570-573`** — Change `closeToolsetsPopover` from `keyboard.press('Escape')` to clicking the trigger (matching `closeMcpsPopover` pattern)
3. Leave other Escape usages (`McpsPage`, `UsersManagementPage`, `ModelsListPage`, `AllUsersPage`) alone — those close dropdown menus/modals, not Radix Popovers

**Verify**: `make build.ui-rebuild && make test.napi`

---

## Final Verification
```bash
make test.backend && make test.ui && make test.napi
```

---

## Implementation Note

This plan is designed to be executed in a **new chat session** with fresh context. Copy this plan file path to the new session:
```
ai-docs/claude-plans/bubbly-sleeping-leaf.md
```
Along with the review index for reference:
```
ai-docs/claude-plans/20260321-migrate-tanstack-v5/index.md
```
