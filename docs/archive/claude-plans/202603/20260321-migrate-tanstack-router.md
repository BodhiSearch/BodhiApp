# Plan: Migrate Next.js 14 to Vite + TanStack Router

## Context

BodhiApp's frontend (`crates/bodhi/`) uses Next.js 14 with App Router in static export mode (`output: 'export'`). None of Next.js's major features (SSR, SSG, API routes, middleware, server components) are used — it's a pure client-side SPA. The app is embedded at Rust compile time via `include_dir!` and served across Tauri desktop, NAPI bindings, Docker, and public multi-tenant deployments.

Migrating to Vite + TanStack Router eliminates unnecessary framework weight, aligns tooling with actual SPA usage, and leverages TanStack ecosystem consistency (already using TanStack Query v5).

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Router | TanStack Router | Same ecosystem as TanStack Query; built-in file-based routing; type-safe params |
| Routing style | File-based (`@tanstack/router-plugin/vite`) | Minimizes structural migration effort from Next.js App Router |
| Route directory | `src/routes/` | TanStack Router convention; clean separation from components |
| Migration approach | Big bang in-place | Sufficient test coverage; cleanest git history |
| Base path | Keep `/ui` | Minimizes Rust/Tauri/proxy changes |
| Trailing slash | Drop | Vite/TanStack Router default; Rust SPA router handles both |
| Build output dir | `out/` (configure Vite `outDir: 'out'`) | Avoids changes to Rust embedding, build.rs, Tauri config, Makefile |
| Code splitting | Single bundle (split later) | Simpler migration; desktop app doesn't need it yet |
| Font | `@fontsource/inter` | Bundled, works offline, replaces `next/font/google` |
| CDN resources | Bundle locally | Prism CSS imported as node module; no external CDN deps |
| Docs section | Remove entirely | Separate concern; can be re-added later |
| Metadata/PWA | Keep all (react-helmet-async) | App serves publicly, Docker, PWA |
| Dep cleanup | Remove docs-only deps; keep chat deps | react-markdown/prismjs still used in chat |

## Current Codebase State (Post-Refactor)

The codebase was recently refactored (HEAD: `96262fa5e`):
- **TanStack Query already v5** — `@tanstack/react-query` v5 with new API patterns
- **Hooks reorganized into domain subdirectories** — `src/hooks/<domain>/useXxx.ts` with `constants.ts` and `index.ts` per domain
- **Import convention**: `QueryClient`/`QueryClientProvider` imported from `@/hooks/useQuery` (re-exports), NOT directly from `@tanstack/react-query`
- **`ClientProviders.tsx`** is minimal (just `QueryClientProvider`) — clean integration point for `RouterProvider`
- These changes are orthogonal to our migration — hook imports and domain structure are unaffected by the Next.js → Vite swap

## Phase 0: Move Docs Section to Pending (Pre-migration, still Next.js)

Move the entire `/docs` subsystem to `crates/bodhi/pending/` to reduce migration scope. Files are preserved for later reintegration.

**Move operations:**
- `src/app/docs/` → `pending/docs-react/` — React components, pages, tests, utils, config, types, prism-theme.css, `__tests__/` fixtures, `[...slug]/` dynamic route
- `public/doc-images/` → `pending/doc-images/` — 29 documentation images (jpg/jpeg)
- `src/docs/` → `pending/docs-md/` — markdown content files with `_meta.json` navigation configs (intro, install, FAQ, features/*, deployment/*, developer/*)

**Files to modify:**
- `src/hooks/navigation/useNavigation.tsx` — remove "Documentation" nav section from `defaultNavigationItems`
- `next.config.mjs` — remove `createMDX`/`withMDX` wrapper
- `package.json` — remove docs-only deps: `@next/mdx`, `@mdx-js/loader`, `@mdx-js/react`, `@types/mdx`, `gray-matter`, `unified`, `remark-gfm`, `remark-math`, `remark-parse`, `remark-rehype`, `rehype-autolink-headings`, `rehype-prism-plus`, `rehype-slug`, `rehype-stringify`

**Verify:** `npm test` passes after move.

## Phase 1: Swap Build Tooling (Next.js -> Vite)

### 1.1 Install/Remove Dependencies

**Add:**
- `@tanstack/react-router`
- `@tanstack/router-plugin` (devDependency)
- `@fontsource/inter`
- `react-helmet-async`
- `vite` (devDependency, explicit)

**Remove:**
- `next`
- `eslint-config-next`
- `next-router-mock`
- `geist` (only needed for Next.js transpilePackages)

### 1.2 Create New Config Files

**`crates/bodhi/index.html`** (Vite entry point):
```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="theme-color" content="#f69435" />
    <meta name="application-name" content="Bodhi App" />
    <meta name="apple-mobile-web-app-capable" content="yes" />
    <meta name="apple-mobile-web-app-status-bar-style" content="default" />
    <meta name="apple-mobile-web-app-title" content="Bodhi App" />
    <meta property="og:type" content="website" />
    <meta property="og:site_name" content="Bodhi App" />
    <meta property="og:title" content="Bodhi App - Run LLMs Locally" />
    <meta property="og:description" content="..." />
    <meta name="twitter:card" content="summary" />
    <meta name="twitter:title" content="Bodhi App - Run LLMs Locally" />
    <link rel="manifest" href="/ui/manifest.json" />
    <link rel="icon" href="/ui/favicon.ico" />
    <title>Bodhi App</title>
    <script>
      let theme = window.localStorage.getItem('bodhi-ui-theme')
      if (!theme) {
        theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      }
      document.documentElement.classList.add(theme)
    </script>
  </head>
  <body class="min-h-screen bg-background font-sans antialiased">
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

**`crates/bodhi/vite.config.ts`:**
```ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { TanStackRouterVite } from '@tanstack/router-plugin/vite';
import path from 'path';

export default defineConfig({
  plugins: [
    TanStackRouterVite({
      routesDirectory: './src/routes',
      generatedRouteTree: './src/routeTree.gen.ts',
    }),
    react(),
  ],
  base: '/ui/',
  resolve: {
    alias: { '@': path.resolve(__dirname, './src') },
  },
  build: {
    outDir: 'out',       // Match existing Rust include_dir! path
    emptyOutDir: true,
    sourcemap: true,
  },
  server: {
    port: 3000,          // Match existing Rust dev proxy target
    strictPort: true,
  },
});
```

**`crates/bodhi/src/main.tsx`** (app entry point):
```tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider, createRouter } from '@tanstack/react-router';
import { routeTree } from './routeTree.gen';

const router = createRouter({
  routeTree,
  basepath: '/ui',
  defaultPreload: 'intent',
});

declare module '@tanstack/react-router' {
  interface Register { router: typeof router }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
```

**`crates/bodhi/vite-env.d.ts`:**
```ts
/// <reference types="vite/client" />
```

### 1.3 Update Existing Config Files

**`tsconfig.json`:**
- `"jsx": "preserve"` -> `"jsx": "react-jsx"`
- Remove `"plugins": [{"name": "next"}]`
- Remove `next-env.d.ts`, `.next/types/**/*.ts` from `include`
- Add `vite-env.d.ts` to `include`

**`package.json` scripts:**
- `"dev": "vite"` (was `"next dev"`)
- `"build": "vite build"` (was `"next build"`)
- Add `"preview": "vite preview"`

**`vitest.config.ts`:** Stays separate from vite.config.ts. No major changes needed — already uses `@vitejs/plugin-react` and `jsdom`.

**`.eslintrc.json`:** Remove `"next/core-web-vitals"` from extends.

**`.gitignore`:** Remove `/.next/`. Keep `/out/`. Add `routeTree.gen.ts`.

### 1.4 Delete Next.js Files

- `next.config.mjs`
- `next-env.d.ts`
- `.next/` (if exists)
- `src/app/metadata.ts` (static metadata moves to index.html; dynamic titles via react-helmet-async)

## Phase 2: Create Route Tree (`src/routes/`)

### 2.1 Root Layout

**`src/routes/__root.tsx`** — replaces `src/app/layout.tsx`:
```tsx
import { createRootRoute, Outlet } from '@tanstack/react-router';
import '@fontsource/inter/400.css';
import '@fontsource/inter/500.css';
import '@fontsource/inter/600.css';
import '@fontsource/inter/700.css';
import 'prismjs/themes/prism-tomorrow.css';  // Bundled locally
import '@/app/globals.css';
import { HelmetProvider } from 'react-helmet-async';
// ... same providers as current layout.tsx

export const Route = createRootRoute({ component: RootLayout });

function RootLayout() {
  return (
    <HelmetProvider>
      <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
        <ClientProviders>
          <NavigationProvider items={defaultNavigationItems}>
            <div className="flex min-h-screen flex-col" data-testid="root-layout">
              <AppHeader />
              <main className="flex-1" data-testid="app-main">
                <Outlet />
              </main>
              <Toaster />
            </div>
          </NavigationProvider>
        </ClientProviders>
      </ThemeProvider>
    </HelmetProvider>
  );
}
```

### 2.2 Font Migration

- Replace `next/font/google` Inter with `@fontsource/inter` CSS imports in `__root.tsx`
- Update `tailwind.config.ts`: `fontFamily: { sans: ['Inter', ...fontFamily.sans] }` (replace `var(--font-sans)`)
- Remove `fontSans.variable` class from body

### 2.3 Route File Mapping

TanStack Router file-based conventions:
- `__root.tsx` = root layout
- `index.tsx` = page component for that path
- `route.tsx` in a directory = layout route (wraps children with `<Outlet />`)

**Complete mapping (src/app/ -> src/routes/):**

```
src/app/layout.tsx                          -> src/routes/__root.tsx
src/app/page.tsx                            -> src/routes/index.tsx

src/app/auth/callback/page.tsx              -> src/routes/auth/callback/index.tsx
src/app/auth/dashboard/callback/page.tsx    -> src/routes/auth/dashboard/callback/index.tsx

src/app/chat/page.tsx                       -> src/routes/chat/index.tsx
src/app/home/page.tsx                       -> src/routes/home/index.tsx
src/app/login/page.tsx                      -> src/routes/login/index.tsx
src/app/request-access/page.tsx             -> src/routes/request-access/index.tsx
src/app/settings/page.tsx                   -> src/routes/settings/index.tsx
src/app/tokens/page.tsx                     -> src/routes/tokens/index.tsx

src/app/models/page.tsx                     -> src/routes/models/index.tsx
src/app/models/alias/new/page.tsx           -> src/routes/models/alias/new/index.tsx
src/app/models/alias/edit/page.tsx          -> src/routes/models/alias/edit/index.tsx
src/app/models/api/new/page.tsx             -> src/routes/models/api/new/index.tsx
src/app/models/api/edit/page.tsx            -> src/routes/models/api/edit/index.tsx
src/app/models/files/page.tsx               -> src/routes/models/files/index.tsx
src/app/models/files/pull/page.tsx          -> src/routes/models/files/pull/index.tsx

src/app/mcps/page.tsx                       -> src/routes/mcps/index.tsx
src/app/mcps/new/page.tsx                   -> src/routes/mcps/new/index.tsx
src/app/mcps/playground/page.tsx            -> src/routes/mcps/playground/index.tsx
src/app/mcps/oauth/callback/page.tsx        -> src/routes/mcps/oauth/callback/index.tsx
src/app/mcps/servers/page.tsx               -> src/routes/mcps/servers/index.tsx
src/app/mcps/servers/new/page.tsx           -> src/routes/mcps/servers/new/index.tsx
src/app/mcps/servers/view/page.tsx          -> src/routes/mcps/servers/view/index.tsx
src/app/mcps/servers/edit/page.tsx          -> src/routes/mcps/servers/edit/index.tsx

src/app/toolsets/page.tsx                   -> src/routes/toolsets/index.tsx
src/app/toolsets/new/page.tsx               -> src/routes/toolsets/new/index.tsx
src/app/toolsets/edit/page.tsx              -> src/routes/toolsets/edit/index.tsx
src/app/toolsets/admin/page.tsx             -> src/routes/toolsets/admin/index.tsx

src/app/setup/layout.tsx                    -> src/routes/setup/route.tsx  (layout with <Outlet />)
src/app/setup/page.tsx                      -> src/routes/setup/index.tsx
src/app/setup/download-models/page.tsx      -> src/routes/setup/download-models/index.tsx
src/app/setup/toolsets/page.tsx             -> src/routes/setup/toolsets/index.tsx
src/app/setup/api-models/page.tsx           -> src/routes/setup/api-models/index.tsx
src/app/setup/llm-engine/page.tsx           -> src/routes/setup/llm-engine/index.tsx
src/app/setup/browser-extension/page.tsx    -> src/routes/setup/browser-extension/index.tsx
src/app/setup/complete/page.tsx             -> src/routes/setup/complete/index.tsx
src/app/setup/resource-admin/page.tsx       -> src/routes/setup/resource-admin/index.tsx
src/app/setup/tenants/page.tsx              -> src/routes/setup/tenants/index.tsx

src/app/users/page.tsx                      -> src/routes/users/index.tsx
src/app/users/pending/page.tsx              -> src/routes/users/pending/index.tsx
src/app/users/access-requests/page.tsx      -> src/routes/users/access-requests/index.tsx
src/app/apps/access-requests/review/page.tsx -> src/routes/apps/access-requests/review/index.tsx
```

**Non-route components stay in `src/app/`** (or move to `src/components/`). Files like `SetupProgress.tsx`, `BodhiLogo.tsx`, `ChatMessage.tsx`, `McpServerSelector.tsx` etc. are NOT route files — they stay where they are and are imported by route files.

### 2.4 Search Params Routes (need `validateSearch`)

Routes that read query params need Zod schemas:

```tsx
// Example: src/routes/toolsets/edit/index.tsx
import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

export const Route = createFileRoute('/toolsets/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditToolsetPage,
});
```

**Routes needing validateSearch:**
- `/toolsets/edit` — `{ id: string }`
- `/models/alias/edit` — `{ id: string }`
- `/models/api/edit` — `{ id: string }`
- `/mcps/new` — `{ server_id?: string, mode?: string }`
- `/mcps/servers/view` — `{ id: string }`
- `/mcps/servers/edit` — `{ id: string }`
- `/mcps/playground` — `{ id: string }`
- `/mcps/oauth/callback` — `{ code?, state?, error?, error_description? }`
- `/auth/callback` — `z.object({}).passthrough()` (iterates all OAuth params dynamically)
- `/auth/dashboard/callback` — OAuth params
- `/chat` — `{ model?: string }`
- `/login` — `{ error?: string, invite?: string }`
- `/apps/access-requests/review` — `{ id: string }`

## Phase 3: Import Replacements (All Files)

### 3.1 Import Replacement Map

| Next.js Import | TanStack Router Replacement |
|---|---|
| `import { useRouter } from 'next/navigation'` | `import { useNavigate } from '@tanstack/react-router'` |
| `router.push(path)` | `navigate({ to: path })` |
| `router.replace(path)` | `navigate({ to: path, replace: true })` |
| `import { useSearchParams } from 'next/navigation'` | `const { param } = Route.useSearch()` (per-route) |
| `searchParams?.get('id')` | Destructure from `Route.useSearch()` |
| `import { usePathname } from 'next/navigation'` | `import { useLocation } from '@tanstack/react-router'` → `useLocation().pathname` |
| `import Link from 'next/link'` | `import { Link } from '@tanstack/react-router'` |
| `<Link href="/path">` | `<Link to="/path">` |
| `<Link href={x} passHref>` | `<Link to={x}>` (passHref not needed) |
| `import Image from 'next/image'` | Standard `<img>` tag (only 2 files, SVGs, unoptimized) |
| `'use client'` directive | DELETE from all files (everything is client in Vite) |

### 3.2 Files by Category

**54 files using `useRouter`** — Replace with `useNavigate()`.
Key files: `AppInitializer.tsx` (8+ redirect paths), all form submission handlers.

**14 files using `useSearchParams`** — Replace with `Route.useSearch()`.
Special case: `auth/callback/page.tsx` uses `searchParams.forEach()` → use `Object.entries(Route.useSearch())`.

**12 files using `usePathname`** — Replace with `useLocation().pathname`.
Key files: `useNavigation.tsx`, `AppHeader.tsx`, `SetupProvider.tsx`, tab components.

**16 files using `Link`** — Change `href` prop to `to`. Remove `passHref` where used (AuthCard.tsx).

**2 files using `Image`** — Replace with `<img>` (`BodhiLogo.tsx`, `AppBreadcrumb.tsx`). Both are SVGs with explicit dimensions.

### 3.3 Utility Updates

**`src/lib/utils.ts` — `handleSmartRedirect()`:**
Currently takes `router: { push: (href: string) => void }`. Update signature:
```tsx
export function handleSmartRedirect(
  location: string,
  navigate: (opts: { to: string }) => void
): void
```

**`src/lib/constants.ts` — `BASE_PATH`:**
Keep as-is (`'/ui'`). Still needed for `window.location.pathname` stripping.

**Navigation items in `useNavigation.tsx`:**
Remove trailing slashes from all `href` values: `'/chat/'` → `'/chat'`, `'/models/'` → `'/models'`, etc.

### 3.4 Metadata Per-Route

Use `react-helmet-async` for dynamic page titles:
```tsx
import { Helmet } from 'react-helmet-async';
// In route components that need custom titles:
<Helmet><title>Models - Bodhi App</title></Helmet>
```

Static OG/Twitter/PWA metadata stays in `index.html` `<head>`.

## Phase 4: Test Migration (~40 files)

### 4.1 Updated Test Wrapper

**`src/tests/wrapper.tsx`:**
```tsx
import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from '@tanstack/react-router';
// IMPORTANT: Import from @/hooks/useQuery (project convention), not @tanstack/react-query directly
import { QueryClient, QueryClientProvider } from '@/hooks/useQuery';

interface CreateWrapperOptions {
  initialPath?: string;
  search?: Record<string, string>;
}

export const createWrapper = (options: CreateWrapperOptions = {}) => {
  const { initialPath = '/', search } = options;
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, refetchOnMount: false }, mutations: { retry: false } },
  });

  let initialUrl = initialPath;
  if (search) {
    initialUrl = `${initialPath}?${new URLSearchParams(search).toString()}`;
  }

  const history = createMemoryHistory({ initialEntries: [initialUrl] });
  const rootRoute = createRootRoute();
  const catchAllRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '$',
    component: () => null,
  });
  const router = createRouter({
    routeTree: rootRoute.addChildren([catchAllRoute]),
    history,
  });

  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router}>{children}</RouterProvider>
    </QueryClientProvider>
  );

  return { Wrapper, router, history, queryClient };
};
```

### 4.2 Test Migration Patterns

**Pattern: `useRouter().push()` → `navigate()`:**
```tsx
// Before:
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({ useRouter: () => ({ push: pushMock }) }));
expect(pushMock).toHaveBeenCalledWith('/models');

// After:
const { Wrapper, router } = createWrapper({ initialPath: '/' });
const navigateSpy = vi.spyOn(router, 'navigate');
render(<MyPage />, { wrapper: Wrapper });
expect(navigateSpy).toHaveBeenCalledWith(expect.objectContaining({ to: '/models' }));
```

**Pattern: `useSearchParams()`:**
```tsx
// Before:
vi.mock('next/navigation', () => ({
  useSearchParams: () => new URLSearchParams('id=uuid-test'),
}));

// After:
const { Wrapper } = createWrapper({
  initialPath: '/toolsets/edit',
  search: { id: 'uuid-test' },
});
```

**Pattern: `usePathname()`:**
```tsx
// Before:
vi.mock('next/navigation', () => ({ usePathname: () => '/mcps' }));

// After:
const { Wrapper } = createWrapper({ initialPath: '/mcps' });
// No mock needed — router knows the path
```

**Pattern: `next/link` mock:**
```tsx
// Before:
vi.mock('next/link', () => ({ default: ({ children, ...props }) => <a {...props}>{children}</a> }));

// After:
// No mock needed — TanStack Router Link renders <a> inside RouterProvider
```

### 4.3 Migration Order

1. Update `src/tests/wrapper.tsx` with new `createWrapper`
2. Migrate simple push-only test files (~18 files)
3. Migrate search params + pathname test files (~22 files)
4. Remove all `vi.mock('next/navigation')` and `vi.mock('next/link')`
5. Remove `next-router-mock` from devDependencies
6. Update `src/tests/setup.ts` — remove any Next.js-specific setup

### 4.4 Special Test Cases

**`auth/callback/page.tsx`:** Uses `searchParams.forEach()` (iterates all OAuth params). With TanStack Router, `Route.useSearch()` returns a plain object. Production code changes to `Object.entries(search)`. Test passes search via `createWrapper({ search: { code: 'xxx', state: 'yyy' } })`.

**`Suspense` wrappers in auth callbacks:** Next.js requires `<Suspense>` around `useSearchParams()` in client components during static export. TanStack Router does not. Remove unnecessary `<Suspense>` boundaries.

**`mockWindowLocation` utility:** Keep — still needed for external URL redirect tests (OAuth, `window.location.href` assignments).

## Phase 5: Build Pipeline Updates

### 5.0 Vite HMR Through Rust Proxy

**Issue**: Next.js HMR used `/_next/webpack-hmr` under `/ui/`, which the proxy caught. Vite's HMR WebSocket connects to the page's origin root `/`, bypassing the `/ui/*` proxy.

**Fix**: `vite.config.ts` → `server.hmr.clientPort: 3000` tells the HMR client to connect directly to Vite dev server (port 3000) instead of through the Rust proxy (port 1135). The Rust proxy (`routes_proxy.rs`) already has full WebSocket support (raw TCP bidirectional relay) but it only handles `/ui/*` paths.

**Alternative** (if direct connection doesn't work due to CORS or firewall): Add a Rust proxy route for `/__vite_hmr` or configure `server.hmr.path` to put HMR under `/ui/`.

### 5.1 No Rust Changes Needed

**`crates/lib_bodhiserver/src/ui_assets.rs`:** NO CHANGE. Vite configured with `outDir: 'out'` so `include_dir!("$CARGO_MANIFEST_DIR/../bodhi/out")` still works.

**`crates/lib_bodhiserver/build.rs`:** NO CHANGE. `bodhi_dir.join("out")` still valid. `npm run build` now invokes `vite build` instead of `next build` — transparent to Rust.

**`crates/routes_app/src/spa_router.rs`:** NO CHANGE. SPA fallback logic works perfectly with Vite output:
- Exact file match: serves `assets/index-[hash].js`, `assets/index-[hash].css` etc.
- No extension path: SPA fallback serves root `index.html` (client-side router handles route)
- Both `/ui/chat` and `/ui/chat/` fall through to SPA fallback correctly

**`crates/routes_app/src/routes_proxy.rs`:** NO CHANGE. Vite HMR uses WebSocket on same port. The existing proxy already supports WebSocket upgrade with raw TCP relay — protocol-agnostic.

### 5.2 Tauri Config — Minimal Change

**`crates/bodhi/src-tauri/tauri.conf.json`:** NO CHANGE needed. `frontendDist: "../out"` still correct. `devUrl`, `beforeDevCommand`, `beforeBuildCommand` all use `npm run dev`/`npm run build` which are the abstraction layer.

### 5.3 Makefile Updates

```makefile
# Comment change only:
build.ui: ## Build Vite frontend and NAPI bindings
	cd crates/bodhi && npm run build
	cd crates/lib_bodhiserver_napi && npm run build

# No path change needed (still out/):
build.ui-clean: ## Clean UI build artifacts
	rm -rf crates/bodhi/out
	cargo clean -p lib_bodhiserver -p bodhi && rm -rf crates/lib_bodhiserver_napi/app-bindings.*.node

# Update references from Next.js to Vite:
app.run.live: ## Run BodhiApp with live Vite dev server (HMR enabled)
	@echo "==> Starting Vite dev server..."
	@cd crates/bodhi && npm run dev &
	@echo "==> Waiting for Vite dev server at http://localhost:3000/ui/..."
	# ... rest unchanged

app.run.live.stop: ## Stop live dev servers (Vite + Rust)
	@pkill -f "vite" 2>/dev/null || true    # was: pkill -f "next dev"
```

### 5.4 Update Build.rs Comments

`crates/lib_bodhiserver/build.rs` line 33: Update comment from `next.config.js` to `vite.config.ts`.
`crates/lib_bodhiserver/src/ui_assets.rs` line 3: Update doc comment from "Next.js frontend" to "Vite frontend".

## Phase 6: Cleanup

1. Delete `src/app/layout.tsx` (replaced by `src/routes/__root.tsx`)
2. Delete `src/app/metadata.ts` (moved to `index.html`)
3. Remove `'use client'` from ALL `.tsx` files (bulk find-and-remove)
4. Move non-route components out of `src/app/` if desired (optional, can defer)
5. Update `tailwind.config.ts` content paths: ensure `'./src/routes/**/*.{ts,tsx}'` is covered (already covered by `'./src/**/*.{ts,tsx}'`)
6. Update `crates/bodhi/src/CLAUDE.md` — reflect Vite + TanStack Router
7. Run `make format.all`

## Verification

1. `cd crates/bodhi && npm test` — all 67 component tests pass
2. `cd crates/bodhi && npm run build` — Vite build succeeds, `out/` contains `index.html` + `assets/`
3. `cd crates/bodhi && npm run dev` — Vite dev server starts on port 3000, HMR works
4. `cargo check -p lib_bodhiserver` — Rust compiles with embedded Vite output
5. `make build.ui-rebuild` — full UI rebuild cycle works
6. `make test.napi` — Playwright E2E tests pass against embedded UI
7. `make app.run.live` — Dev proxy mode works (browser -> Rust 1135 -> Vite 3000)
8. Spot-check key flows: login, setup, chat, model management, MCP management

## Gotchas

1. **TanStack Router `basepath` behavior**: `navigate({ to: '/chat' })` resolves to `/ui/chat`. Same as Next.js `basePath`. Verify `handleSmartRedirect` still strips `/ui` correctly.
2. **Auth callback dynamic params**: `auth/callback/page.tsx` iterates ALL search params via `searchParams.forEach()`. Must use `z.object({}).passthrough()` in route's `validateSearch` and `Object.entries()` instead.
3. **Vite output structure**: Single `index.html` + `assets/`. No per-route HTML files like Next.js. The Rust SPA router handles this via SPA fallback (already tested at `spa_router.rs:114`).
4. **`routeTree.gen.ts`**: Auto-generated by TanStack Router plugin. Add to `.gitignore`. Generated during `vite build` and `vite dev`.
5. **Test wrapper breaking change**: `createWrapper()` now returns `{ Wrapper, router, history, queryClient }` instead of just a component. All test files must destructure.
6. **`BASE_PATH` in static assets**: Image srcs like `${BASE_PATH}/bodhi-logo/bodhi-logo-60.svg` still resolve correctly — Vite's `base: '/ui/'` serves public assets under `/ui/`.
7. **Navigation items trailing slashes**: Must remove from all `href` values in `defaultNavigationItems`. Path matching `useLocation().pathname` won't include trailing slashes.

## Critical Files

| File | Change |
|------|--------|
| `crates/bodhi/vite.config.ts` | NEW — Vite config with TanStack Router plugin |
| `crates/bodhi/index.html` | NEW — Vite entry point with metadata |
| `crates/bodhi/src/main.tsx` | NEW — React app entry point |
| `crates/bodhi/src/routes/__root.tsx` | NEW — root layout (replaces layout.tsx) |
| `crates/bodhi/src/routes/setup/route.tsx` | NEW — setup layout route |
| `crates/bodhi/src/routes/**/*.tsx` | NEW — all migrated page routes |
| `crates/bodhi/package.json` | MODIFY — deps, scripts |
| `crates/bodhi/tsconfig.json` | MODIFY — jsx mode, includes |
| `crates/bodhi/vitest.config.ts` | MINOR — no major changes |
| `crates/bodhi/tailwind.config.ts` | MODIFY — font family |
| `crates/bodhi/src/tests/wrapper.tsx` | MODIFY — add RouterProvider |
| `crates/bodhi/src/hooks/navigation/useNavigation.tsx` | MODIFY — usePathname -> useLocation, remove docs nav |
| `crates/bodhi/src/components/AppInitializer.tsx` | MODIFY — useRouter -> useNavigate |
| `crates/bodhi/src/lib/utils.ts` | MODIFY — handleSmartRedirect signature |
| `crates/bodhi/src/lib/constants.ts` | MINOR — remove trailing slashes from route constants |
| `Makefile` | MINOR — comments, pkill target |
| ~40 test files | MODIFY — replace vi.mock('next/navigation') |
| `crates/bodhi/next.config.mjs` | DELETE |
| `crates/bodhi/next-env.d.ts` | DELETE |
| `crates/bodhi/src/app/docs/` | MOVE to `pending/docs-react/` |
| `crates/bodhi/public/doc-images/` | MOVE to `pending/doc-images/` |
| `crates/bodhi/src/docs/` | MOVE to `pending/docs-md/` |
| `crates/bodhi/src/app/layout.tsx` | DELETE (replaced by __root.tsx) |
| `crates/bodhi/src/app/metadata.ts` | DELETE (moved to index.html) |
