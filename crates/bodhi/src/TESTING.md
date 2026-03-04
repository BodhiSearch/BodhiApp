# bodhi/src — TESTING.md

## Test Infrastructure

### Setup File (`src/tests/setup.ts`)

- Sets `apiClient.defaults.baseURL = 'http://localhost:3000'`
- Mocks: `matchMedia`, `ResizeObserver`, pointer events (`hasPointerCapture`, `setPointerCapture`, `releasePointerCapture`), `scrollIntoView`
- Suppresses console errors for expected HTTP error messages (`Request failed with status code`, `Network Error`)

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

Domain-specific mock handlers: `access-requests.ts`, `app-access-requests.ts`, `api-models.ts`, `auth.ts`, `chat-completions.ts`, `info.ts`, `mcps.ts`, `models.ts`, `modelfiles.ts`, `setup.ts`, `settings.ts`, `tokens.ts`, `toolsets.ts`, `user.ts`

**IMPORTANT**: Handler registration order matters for MCPs — sub-path handlers (`/mcps/servers`, `/mcps/auth-configs`) must come before wildcard `/mcps/:id` handlers.

### Test Fixtures

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

### Page Tests

- Co-located: `page.test.tsx` alongside `page.tsx`
- Test routing, data loading, user interactions
- Use MSW to mock backend responses
