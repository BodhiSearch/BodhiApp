# bodhi/src — CLAUDE.md

**Companion docs** (load as needed):

- `PACKAGE.md` — File index, directory structure, key implementation references
- `TESTING.md` — Test patterns, MSW setup, wrapper utilities

## Purpose

Vite + TanStack Router frontend for BodhiApp. SPA built to `out/` and embedded in Tauri desktop and served by `lib_bodhiserver`. All UI served under `/ui` basepath. Uses TanStack Query v5 for data fetching.

## Architecture Position

**Upstream** (APIs consumed):

- `routes_app` — all HTTP API endpoints
- `@bodhiapp/ts-client` (file:../../ts-client) — generated TypeScript types from OpenAPI spec

**Downstream** (embeds this UI):

- `bodhi/src-tauri` — Tauri desktop app
- `lib_bodhiserver` — embeddable server library

## Critical Rules

### Type Safety with @bodhiapp/ts-client

ALWAYS import request/response types from `@bodhiapp/ts-client`, never define API types locally. After backend API changes:

1. `cargo run --package xtask openapi` — regenerate OpenAPI spec
2. `make build.ts-client` — regenerate and build TypeScript types
3. Types auto-available in frontend via file dependency in package.json

### Basepath Configuration

Three places must stay in sync — all set to `/ui`:

1. `vite.config.ts` — `base: '/ui/'`
2. `src/main.tsx` — `createRouter({ basepath: '/ui' })`
3. `index.html` — asset hrefs prefixed with `/ui/`

`src/lib/constants.ts` defines `BASE_PATH = '/ui'` for cases where the framework does not auto-apply the base path (e.g., `window.location` stripping). Route constants in the same file (e.g., `ROUTE_CHAT = '/chat'`) do NOT include `/ui` — the router prepends it.

### API Client Configuration

- `src/lib/apiClient.ts` — axios instance, `baseURL` is `''` (relative) in prod, `http://localhost:3000` in tests
- `src/components/ClientProviders.tsx` — creates `QueryClient` and wraps app with `QueryClientProvider`
- All API endpoints use `/bodhi/v1/` prefix. Constant `BODHI_API_BASE` defined in `src/hooks/constants.ts`

### Build & Test

- `cd crates/bodhi && npm run dev` — Vite dev server (hot reload, port 3000)
- `cd crates/bodhi && npm test` — run Vitest component tests
- After UI changes, rebuild embedded UI: `make build.ui-rebuild` (see root CLAUDE.md)

### MSW v2 Handler Ordering

MSW handler registration order matters. Sub-path handlers (`/mcps/servers`, `/mcps/auth-configs`) MUST be registered before wildcard `/mcps/:id` handlers to avoid route interception. See `src/test-utils/msw-v2/handlers/mcps.ts`.

## Key Patterns

### TanStack Router (File-Based Routing)

Routes live in `src/routes/` using TanStack Router file conventions. Route files are thin wrappers that import page components from `src/app/` (the old Next.js page components, kept in place).

**Route file pattern** — see `src/routes/login/index.tsx`:

- `createFileRoute('/login/')` defines the route
- `validateSearch` with Zod schema for type-safe query params
- `component` points to the page component from `src/app/`

**Layout routes** — `src/routes/setup/route.tsx` wraps child routes with `Outlet`.

**Root layout** — `src/routes/__root.tsx` provides ThemeProvider, ClientProviders, NavigationProvider, AppHeader.

**Navigation APIs** (replaced Next.js equivalents):

- `useNavigate()` replaces `useRouter().push()` — call as `navigate({ to: ROUTE_CHAT })`
- `useSearch({ strict: false })` replaces `useSearchParams` — returns typed search params
- `useLocation().pathname` replaces `usePathname()`
- `<Link to="/chat">` replaces `<Link href="/chat">`

**Router configuration** — `src/main.tsx`: `trailingSlash: 'always'`, `defaultPreload: 'intent'`.

### Hook Architecture (Domain Subdirectories)

Hooks organized into 12 domain subdirectories under `src/hooks/<domain>/`. Each has `constants.ts` (query key factory + endpoints), `index.ts` (barrel), and CRUD-named hooks (`useList*`, `useGet*`, `useCreate*`, `useUpdate*`, `useDelete*`). Full directory listing in `PACKAGE.md`.

**Query key factory pattern** — see `src/hooks/models/constants.ts` for reference. Keys build hierarchically: `modelKeys.all` → `modelKeys.lists()` → `modelKeys.list(...)` → `modelKeys.detail(id)`.

**Generic hooks** in `src/hooks/useQuery.ts`: `useQuery<T>` (GET) and `useMutationQuery<T, V>` (POST/PUT/DELETE) with `AxiosError<OpenAiApiError>` typing.

### Test Fixtures (Factory Pattern)

`src/test-fixtures/<domain>.ts` — fixture factories using OpenAPI-generated types from `@bodhiapp/ts-client`. Each factory accepts optional `Partial<T>` overrides. Domains: `access-requests`, `apps`, `mcps`, `models`, `tokens`, `toolsets`, `users`.

### Form Pattern

react-hook-form + zod schema + ts-client types. Schemas in `src/schemas/`:

- Zod schema for form validation
- `convertFormToApi()` / `convertApiToForm()` for type conversion between form and API formats
- Types re-exported from `@bodhiapp/ts-client`

### App Initialization Flow

`AppInitializer` (`src/components/AppInitializer.tsx`) checks `/bodhi/v1/info` status and redirects:

- `setup` + standalone -> `/setup`
- `setup` + multi_tenant -> `/setup/tenants`
- `ready` + multi_tenant + no client_id -> `/login`
- `ready` (all other) -> `/chat`
- `resource_admin` -> `/setup/resource-admin`

Authenticated routes check `userInfo.auth_status === 'logged_in'`. Users with `resource_guest` or `resource_anonymous` role are redirected to `/request-access`.

Role hierarchy and utilities in `src/lib/roles.ts`. `Role` type re-exported from `ResourceRole` in `@bodhiapp/ts-client`.

Route constants defined in `src/lib/constants.ts`.

### MCP Server Management

- MCP instances: `src/routes/mcps/` routes, `src/hooks/mcps/useMcpInstances.ts`
- MCP servers (allowlist): `src/routes/mcps/servers/` routes, `src/hooks/mcps/useMcpServers.ts`
- Auth config: `McpAuthType` enum (`public`, `header`, `oauth`). OAuth distinguishes pre-registered vs dynamic via `registration_type` field
- `src/stores/mcpFormStore.ts` uses sessionStorage; `mcpFormStore.reset()` clears it. OAuth callback validates `state` parameter

### Setup Flow

Multi-step onboarding under `src/routes/setup/`: download-models -> toolsets -> api-models -> llm-engine -> browser-extension -> complete. Layout route in `src/routes/setup/route.tsx` wraps with `SetupLayoutComponent`.

## Testing

See `TESTING.md` for MSW setup, wrapper utilities, and test patterns.

**Critical gotcha**: `src/tests/setup.ts` sets `notifyManager.setScheduler((cb) => queueMicrotask(cb))`. Without this, TanStack Query v5's `setTimeout(cb, 0)` scheduling breaks tests by deferring state updates outside `act()` blocks.
