# bodhi/src — CLAUDE.md

**Companion docs** (load as needed):

- `PACKAGE.md` — File index, directory structure, key implementation references
- `TESTING.md` — Test patterns, MSW setup, wrapper utilities

## Purpose

Next.js 14 frontend for BodhiApp. Static-exported (`output: 'export'`) React app embedded in Tauri desktop and served by `lib_bodhiserver`. Uses App Router with `/ui` route group for main application and `/docs` for documentation.

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

### API Client Configuration

- `src/lib/apiClient.ts` — axios instance, `baseURL` is `''` (relative) in prod, `http://localhost:3000` in tests
- `src/lib/queryClient.ts` — separate file, re-exports `QueryClient` from react-query
- All API endpoints use `/bodhi/v1/` prefix. Constant `BODHI_API_BASE` defined in `src/hooks/useQuery.ts:19`

### Build Workflow

IMPORTANT: After UI changes, rebuild embedded UI before testing:

- `make build.ui-clean` then `make build.ui` — or `make build.ui-rebuild`
- For active development: `cd crates/bodhi && npm run dev` (hot reload)
- `cd crates/bodhi && npm test` — run Vitest component tests

### MSW v2 Handler Ordering

MSW handler registration order matters. Sub-path handlers (`/mcps/servers`, `/mcps/auth-configs`) MUST be registered before wildcard `/mcps/:id` handlers to avoid route interception. See `src/test-utils/msw-v2/handlers/mcps.ts`.

## Key Patterns

### Hook Architecture

- Generic hooks in `src/hooks/useQuery.ts` — `useQuery<T>`, `useMutation<TData, TVariables>` wrapping react-query with `AxiosError<OpenAiApiError>` error typing
- Domain hooks (useModels, useMcps, useToolsets, useApiModels, etc.) compose the generic hooks with endpoint-specific types
- Chat system: `use-chat.tsx` (orchestration) → `use-chat-completions.ts` (streaming SSE) → `use-chat-db.tsx` (localStorage persistence) → `use-chat-settings.tsx` (configuration)

### Form Pattern

react-hook-form + zod schema + ts-client types. See `src/schemas/alias.ts` for canonical example:

- Zod schema for form validation
- `convertFormToApi()` / `convertApiToForm()` for type conversion between form and API formats
- Types re-exported from `@bodhiapp/ts-client`

### App Initialization Flow

`AppInitializer` (`src/components/AppInitializer.tsx`) checks `/bodhi/v1/info` status and redirects:

- `setup` → `/ui/setup`
- `ready` → `/ui/setup/download-models` (first visit) or `/ui/chat` (returning)
- `resource-admin` → `/ui/setup/resource-admin`

Route constants defined in `src/lib/constants.ts`.

### MCP Server Management

- MCP instances: `src/app/ui/mcps/` pages, `src/hooks/useMcps.ts`
- MCP servers (allowlist): `src/app/ui/mcp-servers/` pages
- Auth config: `McpAuthType` enum (`public`, `header`, `oauth`). OAuth distinguishes pre-registered vs dynamic via `registration_type` field
- Auto-DCR behavior differs: new page (`mcp-servers/new/page.tsx`) uses `enableAutoDcr={true}` (silent fallback), view page (`mcp-servers/view/page.tsx`) uses `enableAutoDcr={false}` (shows errors)
- `src/stores/mcpFormStore.ts` uses sessionStorage; `mcpFormStore.reset()` clears it. OAuth callback validates `state` parameter

### Navigation

`use-navigation.tsx` provides `NavigationProvider` with `defaultNavigationItems`. `AppHeader.tsx` renders header. Sidebar system in `src/components/ui/sidebar.tsx`.

### Setup Flow

Multi-step onboarding under `/ui/setup/`: download-models → toolsets → api-models → llm-engine → browser-extension → complete. `SetupProvider` manages step state.

## Testing Rules

- Do NOT add inline timeouts in component tests — rely on defaults or fix root cause
- Use `data-testid` attributes with `getByTestId` for element selection
- Test wrapper: `src/tests/wrapper.tsx` — `createWrapper()` provides QueryClientProvider
- MSW v2 with `openapi-msw` for type-safe mocking. Setup in `src/test-utils/msw-v2/setup.ts`
- `src/tests/setup.ts` configures test environment (sets baseURL, mocks matchMedia/ResizeObserver/pointer events)
