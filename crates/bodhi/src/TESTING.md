# bodhi/src — TESTING.md

## Test Infrastructure

### Setup File (`src/tests/setup.ts`)

- Sets `apiClient.defaults.baseURL = 'http://localhost:3000'`
- Sets `notifyManager.setScheduler((cb) => queueMicrotask(cb))` — required for TanStack Query v5 state updates to flush within `act()` blocks
- Mocks: `matchMedia`, `ResizeObserver`, pointer events (`hasPointerCapture`, `setPointerCapture`, `releasePointerCapture`), `scrollIntoView`, `useMediaQuery`
- Suppresses console errors for expected HTTP error messages (`Request failed with status code`, `Network Error`)

### Vitest Config (`vitest.config.ts`)

- Environment: `jsdom`
- Setup file: `src/tests/setup.ts`
- Aliases `framer-motion` to `src/tests/mocks/framer-motion.tsx`

### Test Wrapper (`src/tests/wrapper.tsx`)

- `createWrapper()` — returns a React component wrapping children in `QueryClientProvider` with `retry: false`
- `mockWindowLocation(href)` — mocks `window.location` for tests that check URL behavior

### MSW v2 Setup (`src/test-utils/msw-v2/setup.ts`)

- `server` — MSW `setupServer()` instance
- `setupMswV2()` — call in `describe` block; sets up `beforeAll/afterEach/afterAll` lifecycle
- `typedHttp` — OpenAPI-typed HTTP handler creator via `openapi-msw` (preferred for type safety)
- `http`, `HttpResponse` — re-exported from MSW v2 for standard handlers
- `createTypedResponse<T>(status, data)` — helper for consistent response creation
- Re-exports `components`, `paths` types from `@bodhiapp/ts-client`

### MSW Handler Files (`src/test-utils/msw-v2/handlers/`)

Domain-specific mock handlers: `api-models.ts`, `apps.ts`, `auth.ts`, `chat-completions.ts`, `info.ts`, `mcp-protocol.ts`, `mcps.ts`, `models.ts`, `modelfiles.ts`, `setup.ts`, `settings.ts`, `tenants.ts`, `tokens.ts`, `user-access-requests.ts`, `user.ts`

**MCP protocol handlers** (`mcp-protocol.ts`): `createMcpProtocolHandlers(config)` simulates an MCP Streamable HTTP server at the JSON-RPC level. Handles `initialize`, `notifications/initialized`, `tools/list`, `tools/call`, plus GET (405) and DELETE (202) for session management. Allows the real `useMcpClient` hook and MCP SDK to run end-to-end in tests. Config accepts `endpoint`, `tools`, `serverName`, `toolCallHandler`.

**IMPORTANT**: Handler registration order matters for MCPs — sub-path handlers (`/mcps/servers`, `/mcps/auth-configs`) must come before wildcard `/mcps/:id` handlers.

### Test Fixtures (`src/test-fixtures/`)

Factory functions using OpenAPI-generated types. Each factory accepts `Partial<T>` overrides:

- `access-requests.ts`, `apps.ts`, `mcps.ts`, `models.ts`, `tokens.ts`, `users.ts`

### Other Test Utilities

- `src/test-utils/fixtures/chat.ts` — chat message fixtures
- `src/test-utils/api-model-test-utils.ts` — API model test helpers
- `src/test-utils/mock-user.ts` — mock user data
- `src/tests/mocks/framer-motion.tsx` — framer-motion mock

## Test Conventions

### Component Tests

- Co-located with source: `ComponentName.test.tsx` alongside `ComponentName.tsx`
- Use `data-testid` attributes for element selection (prefer over CSS selectors)
- Do NOT add inline timeouts — fix root cause or rely on Vitest defaults
- Use `createWrapper()` for hooks that need QueryClientProvider

### Hook Tests

- Use `renderHook()` from `@testing-library/react` with `wrapper: createWrapper()`
- Mock API calls with MSW handlers, not axios mocks
- Hook test files co-located in domain directories (e.g., `src/hooks/models/useModels.test.ts`)

### Page Tests

- Co-located in `src/app/` alongside page components: `page.test.tsx` next to `page.tsx`
- Test routing, data loading, user interactions
- Use MSW to mock backend responses
