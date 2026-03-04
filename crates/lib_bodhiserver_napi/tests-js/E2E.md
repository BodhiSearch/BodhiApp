# E2E.md - Test Writing Reference

For setup, credentials, and architecture, see [CLAUDE.md](CLAUDE.md).

## Test Philosophy

Tests are **user journeys**, not unit tests. Each test tells a story: login, perform actions, verify results. Aim for 1-4 tests per `test.describe`. Same user building on accumulated state = same test. Different user/role/fresh DB = new test.

## Writing a New Spec

1. Define journey as `test.step()` blocks with descriptive phase names
2. Build Page Object methods as needed (extend `BasePage`)
3. Add Fixture class with static factory methods for test data
4. Fill in assertions within steps
5. Run and iterate

### Canonical Examples

| Scenario | Spec File |
|----------|-----------|
| Shared-server lifecycle | `specs/tokens/api-tokens.spec.mjs` |
| Multi-phase with `test.step` | `specs/toolsets/toolsets-auth-restrictions.spec.mjs` |
| Dedicated server + custom config | `specs/models/model-metadata.spec.mjs` |
| Multi-user browser contexts | `specs/request-access/multi-user-request-approval-flow.spec.mjs` |
| Multi-step OAuth MCP | `specs/mcps/mcps-oauth-auth.spec.mjs` |

## Structure Standards

### test.step() for Major Phases
All tests must use `test.step()`. Step names describe **user goals**, not implementation. For multi-phase tests prefix with `'Phase N:'`. See `specs/mcps/mcps-crud.spec.mjs` for corrected pattern.

### Import Rules
- **Shared server**: `import { expect, test } from '@/fixtures.mjs'` -- gets `autoResetDb` + `sharedServerUrl` fixtures
- **Dedicated server**: `import { expect, test } from '@playwright/test'` + `createServerManager()`

### Setup Hooks

| Hook | Use For |
|------|---------|
| `beforeAll` | Auth config, dedicated server startup |
| `beforeEach` | Page object creation (MUST be per-test) |
| `afterAll` | Dedicated server cleanup |
| `try/finally` | Browser context cleanup in multi-user tests |

## Anti-Pattern Summary

Avoid these patterns (previously found in `specs/mcps/mcps-crud.spec.mjs`, since corrected):
- Fragmented tests (6 small tests that should be 1-2 journeys)
- Repeated login per test instead of once per journey
- No `test.step()` phase markers
- Hardcoded test data instead of fixture classes
- Unit-test naming ("shows Add button") instead of user goals ("MCP Server CRUD Lifecycle")

## Page Object Conventions

- Extend `BasePage` from `@/pages/BasePage.mjs`
- Constructor: `(page, baseUrl)` always
- Selectors in `selectors = {}` property using `data-testid`, never CSS classes
- Dynamic selectors as functions: `mcpRow: id => \`[data-testid="mcp-row-${id}"]\``
- Method prefixes: `navigateTo*`, `expect*`, `click*`, `fill*`, `waitFor*`, `get*`
- Composite page objects for complex UIs (e.g., `OAuthTestApp` composes `ConfigSection`, `OAuthSection`, etc.)
- Responsive duplicates: use `.last()` for desktop viewport

## Fixture Conventions

- Classes with static factory methods in `fixtures/` directory
- Use `Date.now()` + random suffix for unique data
- Canonical examples: `localModelFixtures.mjs`, `tokenFixtures.mjs`, `ChatFixtures.mjs`

## Server Decision Tree

```
Custom config (HF_HOME, auth, port)? -> Dedicated server
Non-ready app state (setup flow)?    -> Dedicated server
Multiple users with separate roles?  -> Dedicated server (+ dynamic resource client)
Otherwise                            -> Shared server (port 51135, import @/fixtures.mjs)
```

Dedicated server pattern: see `specs/models/model-metadata.spec.mjs` for `createServerManager()` usage with `beforeAll`/`afterAll`.

## Flakiness Patterns

### Toast Timing
Most common flakiness source. Prefer `data-testid`/`data-test-state` assertions over toast-based verification when possible.
- Critical: `waitForToast('message')`
- Non-critical: `waitForToastOptional('message')`
- Extract data: `waitForToastAndExtractId(pattern)`

### Keycloak Redirects
Use appropriate wait helpers, never fixed timeouts:
- No KC session: `waitForAuthServerRedirect(authUrl)`
- After KC login/auto-redirect: `waitForTokenExchange(appUrl)`
- Access-request flows: `waitForAccessRequestCallback(appUrl)`

### SPA Navigation
Call `waitForSPAReady()` after any navigation. Use `page.waitForURL()` for route changes, not `page.goto()` + immediate assertion.

### LLM Model Loading
Default timeout (120s, configurable via `PLAYWRIGHT_TIMEOUT`) handles variable load times. Never add inline timeouts.

## Utility Reference

| Utility | Key Exports |
|---------|-------------|
| `auth-server-client.mjs` | `getAuthServerConfig()`, `getTestCredentials()`, `getPreConfiguredResourceClient()`, `getPreConfiguredAppClient()`, `AuthServerTestClient` |
| `bodhi-app-server.mjs` | `createServerManager()` |
| `browser-with-extension.mjs` | `BrowserWithExtension` |
| `mock-openai-server.mjs` | `MockOpenAIServer` |
| `test-helpers.mjs` | `SHARED_STATIC_SERVER_URL`, `loadBindings()`, `createTestServer()`, `resetDatabase()`, `randomPort()` |
| `fixtures.mjs` | `test` (extended with `sharedServerUrl` + `autoResetDb`), `expect` |
| `utils/db-config.mjs` | `getDbConfig(projectName)`, `getServerUrl(projectName)` |
