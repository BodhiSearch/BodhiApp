# E2E.md - Test Writing Reference

Detailed reference for writing and reviewing Playwright E2E tests in this suite. For setup, credentials, and quick orientation, see [CLAUDE.md](CLAUDE.md).

## Test Philosophy

Tests in this suite are **user journeys**, not unit tests.

- Each test tells a story: a user logs in, performs a sequence of actions, and sees expected results.
- Aim for **1-4 tests per `test.describe`**, each a meaningful, standalone journey.
- **Journey boundary**: same user building on accumulated state = same test. Different user, different role, or needs a fresh DB state = new test.
- Do NOT combine unrelated flows just for efficiency. Each test must make sense as a story.
- Do NOT fragment a single flow into many small tests that each repeat login and setup.

## Writing a New E2E Spec

Follow this TDD-style process:

1. **Skeleton first**: Define the user journey as `test.step()` blocks with descriptive phase names.
2. **Build Page Object methods** as you need them (extend `BasePage`).
3. **Add a Fixture file** for reusable test data if the feature has constants, factory data, or helper utilities.
4. **Fill in assertions** within each step.
5. **Run and iterate**.

### Canonical Examples

Reference these specs as models for different test scenarios:

| Scenario                         | Spec File                                                        | Pattern                                                                |
| -------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Shared-server journey            | `specs/tokens/api-tokens.spec.mjs`                               | Single login, full lifecycle (create/verify/use/deactivate/reactivate) |
| Multi-phase with `test.step`     | `specs/toolsets/toolsets-auth-restrictions.spec.mjs`             | Phases for login, OAuth config, access request, API verification       |
| Dedicated server + custom config | `specs/models/model-metadata.spec.mjs`                           | Custom HF_HOME, `createServerManager`, `test.step` per model           |
| Multi-user with browser contexts | `specs/request-access/multi-user-request-approval-flow.spec.mjs` | Separate browser contexts per user, `try/finally` cleanup              |
| Multi-step OAuth MCP flow        | `specs/mcps/mcps-oauth-auth.spec.mjs`                            | OAuth MCP creation, 3rd-party access request, REST API verification    |

## Test Structure Standards

### Use `test.step()` for Major Phases

All new tests should use `test.step()` for major phases. This integrates with Playwright's trace viewer and HTML reports, making failures easier to locate.

```js
test('MCP Server CRUD Lifecycle', async ({ page }) => {
  const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
  const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);

  await test.step('Login and verify empty list', async () => {
    await loginPage.performOAuthLogin('/ui/chat/');
    await mcpsPage.navigateToMcpsList();
    await mcpsPage.expectEmptyState();
  });

  await test.step('Create MCP server with admin enable', async () => {
    await mcpsPage.createMcpWithAdminEnable(MCP_URL, 'DeepWiki', 'deepwiki', 'DeepWiki MCP');
    await mcpsPage.expectToolsSection();
  });

  await test.step('Verify MCP appears in list', async () => {
    await mcpsPage.clickDone();
    await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
    const row = await mcpsPage.getMcpRowByName('DeepWiki');
    await expect(row).toBeVisible();
  });

  // ... more steps
});
```

### Import Rules

```js
// Shared server (auto DB reset before each test):
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL } from '@/test-helpers.mjs';

// Dedicated server (manage own state):
import { expect, test } from '@playwright/test';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
```

Importing from `@/fixtures.mjs` gives you the `autoResetDb` fixture that calls `POST /dev/db-reset` before each test. Dedicated-server tests import from `@playwright/test` directly because the DB reset targets port 51135 (the shared server), not the dedicated server.

### Setup and Teardown

| Hook          | Use For                                                                                 |
| ------------- | --------------------------------------------------------------------------------------- |
| `beforeAll`   | Auth config (`getAuthServerConfig()`, `getTestCredentials()`), dedicated server startup |
| `beforeEach`  | Page object creation (MUST be per-test -- Playwright's `page` is per-test)              |
| `afterAll`    | Dedicated server cleanup (`serverManager.stopServer()`)                                 |
| `try/finally` | Browser context cleanup in multi-user tests                                             |

## Strict Black-Box Testing

E2E tests are **black-box** tests. They must interact with the application exclusively through the UI.

**Rule**: Never call APIs directly via `page.evaluate()` + `fetch()` as the primary test interaction. All test assertions must observe results through UI components (visible elements, navigation, `data-testid` attributes).

**Exception -- setup-only helpers**: When a backend operation has no UI equivalent (e.g., creating an auth config, discovering OAuth endpoints, DCR registration), `page.evaluate()` fetch helpers in McpsPage are acceptable *for test setup only*. They must always include `if (!resp.ok) throw new Error(...)` so failures are surfaced clearly. The assertions that follow must still use UI interactions.

**Anti-pattern** (do not do this):
```js
// WRONG: Using fetch to assert results instead of UI
const resp = await page.evaluate(async ({ baseUrl }) => {
  const r = await fetch(`${baseUrl}/bodhi/v1/mcps`);
  return r.json();
}, { baseUrl });
expect(resp.mcps.length).toBe(1); // asserting via API, not UI
```

**Correct pattern** (verify through UI):
```js
// RIGHT: Navigate to list and verify via UI element
await mcpsPage.navigateToMcpsList();
const row = await mcpsPage.getMcpRowByName(instanceName);
await expect(row).toBeVisible();
```

## step() Naming Convention

Use `test.step()` for all major phases. Step names must describe the **user goal** or **phase outcome**, not the implementation:

| Good step name                                  | Bad step name                                |
| ----------------------------------------------- | -------------------------------------------- |
| `'Login and verify empty list'`                 | `'Call performOAuthLogin'`                   |
| `'Create MCP server with auth header'`          | `'POST to /bodhi/v1/mcps/auth-configs'`      |
| `'Verify auth works via playground execution'`  | `'Click execute button and check response'`  |
| `'Phase 2: Configure external app OAuth form'`  | `'Fill form fields'`                         |

For multi-phase tests (e.g., OAuth access request flows), prefix with `'Phase N:'` to indicate sequence:

```js
await test.step('Phase 1: Login and create MCP instance', async () => { ... });
await test.step('Phase 2: Configure external app OAuth form', async () => { ... });
await test.step('Phase 3: Submit access request and approve', async () => { ... });
await test.step('Phase 4: Verify MCP access via REST API', async () => { ... });
```

## Anti-Patterns

The following shows common anti-patterns (previously present in `specs/mcps/mcps-crud.spec.mjs`, since corrected). Avoid this structure in new specs.

### What Is Wrong

```js
// ANTI-PATTERN: 6 fragmented tests with repeated login
test('displays MCP servers list page (empty)', async () => {
  await loginPage.performOAuthLogin('/ui/chat/'); // login #1
  await mcpsPage.navigateToMcpsList();
  await mcpsPage.expectEmptyState();
});

test('shows Add MCP Server button on list page', async () => {
  await loginPage.performOAuthLogin('/ui/chat/'); // login #2
  await mcpsPage.navigateToMcpsList();
  await expect(mcpsPage.page.locator(mcpsPage.selectors.newButton)).toBeVisible();
});

test('navigates to new MCP page', async () => {
  await loginPage.performOAuthLogin('/ui/chat/'); // login #3
  // ... same navigation repeated
});
// ... 3 more tests, each logging in again
```

**Problems**:

- **Fragmented tests**: 6 small tests that should be 1-2 journeys
- **Repeated login**: `performOAuthLogin()` called 6 times instead of once per journey
- **No `test.step()`**: Flat test bodies without phase markers
- **No fixture file**: Test data hardcoded inline (`'https://mcp.deepwiki.com/mcp'`, `'DeepWiki'`)
- **Overlapping assertions**: "displays list page" and "shows add button" are sub-steps of the same flow, not independent tests
- **Unit-test naming**: Names describe UI elements ("shows Add button") instead of user goals

### How It Should Look

```js
import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL } from '@/test-helpers.mjs';

test.describe('MCP Server Management', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test('MCP Server CRUD Lifecycle', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const testData = McpFixtures.createLifecycleData();

    await test.step('Login and verify empty MCP list', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.navigateToMcpsList();
      await mcpsPage.expectMcpsListPage();
      await mcpsPage.expectEmptyState();
    });

    await test.step('Create MCP server with admin enable flow', async () => {
      await mcpsPage.clickNewMcp();
      await mcpsPage.expectNewMcpPage();
      await mcpsPage.createMcpWithAdminEnable(testData.url, testData.name, testData.slug, testData.description);
      await mcpsPage.expectToolsSection();
    });

    await test.step('Verify MCP appears in list', async () => {
      await mcpsPage.clickDone();
      await mcpsPage.page.waitForURL(/\/ui\/mcps(?!\/new)/);
      await mcpsPage.expectMcpsListPage();
      const row = await mcpsPage.getMcpRowByName(testData.name);
      await expect(row).toBeVisible();
    });

    await test.step('Delete MCP server with confirmation', async () => {
      const mcpId = await mcpsPage.getMcpUuidByName(testData.name);
      expect(mcpId).toBeTruthy();
      await mcpsPage.clickDeleteById(mcpId);
      await expect(mcpsPage.page.getByText('Delete MCP Server')).toBeVisible();
      await mcpsPage.confirmDelete();
      await mcpsPage.expectEmptyState();
    });
  });

  test('MCP Server Tool Discovery', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const mcpsPage = new McpsPage(page, SHARED_SERVER_URL);
    const testData = McpFixtures.createToolDiscoveryData();

    await test.step('Login and create MCP server', async () => {
      await loginPage.performOAuthLogin('/ui/chat/');
      await mcpsPage.createMcpWithAdminEnable(testData.url, testData.name, testData.slug);
    });

    await test.step('Fetch and verify tools', async () => {
      await mcpsPage.clickFetchTools();
      await mcpsPage.expectToolsList();
      await mcpsPage.expectToolItem('read_wiki_structure');
    });
  });
});
```

**Key differences**: single login per journey, `test.step()` for phases, fixture-driven test data, 2 journey tests instead of 6 fragmented ones, names describe user goals.

## Page Object Conventions

### Inheritance and Construction

All page objects extend `BasePage` from `@/pages/BasePage.mjs`, which provides:

- `navigate(path)`, `waitForSPAReady()`, `clickTestId(testId)`, `fillTestId(testId, value)`
- `waitForToast(message)`, `waitForToastOptional(message)`, `waitForToastAndExtractId(pattern)`
- `expectCurrentPath(path)`, `getCurrentPath()`, `takeScreenshot(name)`

Constructor signature is always `(page, baseUrl)` to match Playwright's per-test page lifecycle.

### Selectors

Define selectors in a `selectors = {}` property using `data-testid` attributes, never CSS classes:

```js
selectors = {
  pageContainer: '[data-testid="mcps-page"]',
  newButton: '[data-testid="mcp-new-button"]',
  mcpRow: id => `[data-testid="mcp-row-${id}"]`,
  mcpRowByName: name => `[data-test-mcp-name="${name}"]`,
};
```

Use dynamic selector functions for elements with IDs or names.

### Method Naming

| Prefix        | Purpose                    | Example                  |
| ------------- | -------------------------- | ------------------------ |
| `navigateTo*` | Navigate to a page         | `navigateToMcpsList()`   |
| `expect*`     | Assert visibility/state    | `expectEmptyState()`     |
| `click*`      | Click an element           | `clickNewMcp()`          |
| `fill*`       | Fill an input              | `fillUrl(url)`           |
| `waitFor*`    | Wait for a condition       | `waitForFormReady()`     |
| `get*`        | Extract data from the page | `getMcpUuidByName(name)` |

### Composite Page Objects

For complex UIs, compose section objects. Example: `OAuthTestApp` composes `ConfigSection`, `OAuthSection`, `AccessCallbackSection`, `DashboardPage`, `ChatPage`, and `RESTPage`:

```js
const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);
await app.config.configureOAuthForm({ ... });
await app.oauth.waitForTokenExchange(url);
await app.rest.sendRequest({ method: 'GET', url: '/bodhi/v1/user' });
```

### Responsive Elements

Some `data-testid` elements have mobile and desktop duplicates due to responsive design. Use `.last()` for desktop viewport:

```js
const previewButton = page.locator(`[data-testid="modelfiles-preview-button-${key}"]`).last();
```

## Fixture Conventions

### Structure

Fixtures are classes with static methods in the `fixtures/` directory. Each fixture module targets a specific domain.

```js
export class McpFixtures {
  static createLifecycleData() {
    const timestamp = Date.now();
    return {
      url: 'https://mcp.deepwiki.com/mcp',
      name: `DeepWiki-${timestamp}`,
      slug: `deepwiki-${timestamp}`,
      description: 'DeepWiki MCP Server',
    };
  }

  static createToolDiscoveryData() {
    return { ... };
  }
}
```

### Conventions

- Use `Date.now()` + random suffix for unique data within a journey to avoid collisions.
- Factory methods for different scenarios: `createComprehensiveLifecycleData()`, `createValidationData()`, `getTestTokenNames()`.
- Utility methods for test-specific setup (e.g., `TokenFixtures.mockClipboard(page)`).

### Canonical Examples

- `fixtures/localModelFixtures.mjs`: Factory pattern with `QWEN_MODEL` constant and lifecycle/validation/context test data builders.
- `fixtures/tokenFixtures.mjs`: Static data (`getTestTokenNames()`, `getInvalidTokens()`) plus utilities (clipboard mock, error patterns).
- `fixtures/ChatFixtures.mjs`: Scenarios, edge cases, streaming prompts, and responsive viewport configs.

## Server Configuration Decision Tree

```
Need custom server config (HF_HOME, auth, port)?
  YES -> Dedicated server
Need non-ready app state (e.g., setup flow)?
  YES -> Dedicated server
Need multiple users with separate roles?
  YES -> Dedicated server with dynamic resource client
         (createAuthServerTestClient + createResourceClient + makeResourceAdmin)
Otherwise:
  -> Shared server (port 51135)
     Import from @/fixtures.mjs
```

### Dedicated Server Setup Pattern

```js
import { createAuthServerTestClient, getAuthServerConfig } from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { expect, test } from '@playwright/test'; // NOT @/fixtures.mjs

test.describe('Feature Needing Custom Server', () => {
  let serverManager;
  let baseUrl;

  test.beforeAll(async () => {
    const authServerConfig = getAuthServerConfig();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    const authClient = createAuthServerTestClient(authServerConfig);
    const resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.userId);

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) await serverManager.stopServer();
  });

  test('...', async ({ page }) => {
    /* use baseUrl */
  });
});
```

## Flakiness Patterns and Mitigations

### Toast Timing

Toasts are the most common flakiness source. BasePage has 5+ toast methods with fallbacks -- this is tech debt.

- **Critical assertion** (test fails without it): `await this.waitForToast('message')`
- **Non-critical confirmation**: `await this.waitForToastOptional('message')`
- **Extract data from toast**: `await this.waitForToastAndExtractId(pattern)`
- **Dismiss before next action**: `await this.dismissAllToasts()`

Do not add more toast method complexity. Prefer `data-testid` and `data-test-state` assertions over toast-based verification when possible.

### Keycloak Redirect Timing

KC may auto-redirect instantly (cached session) or take seconds (cold start). Always use the appropriate wait helper:

- `waitForAuthServerRedirect(authUrl)` -- when no KC session exists
- `waitForTokenExchange(appUrl)` -- after KC login or auto-redirect
- `waitForAccessRequestCallback(appUrl)` -- for access-request flows

Never use fixed `page.waitForTimeout()` for KC flows.

### LLM Model Loading

Chat tests with local models have variable loading times. The default 120s timeout from `playwright.config.mjs` (configurable via `PLAYWRIGHT_TIMEOUT`) handles this. Never add inline timeouts to tests or components.

### SPA Navigation

After any navigation, call `waitForSPAReady()` before interacting with elements. For route changes within the SPA, use `page.waitForURL()` instead of `page.goto()` + immediate assertion.

## CI and @scheduled Tests

- `HEADLESS=true` is set in the `test:playwright` npm script.
- Environment variables are loaded from GitHub repo vars/secrets into `.env.test`.
- `PLAYWRIGHT_TIMEOUT` env var can be increased for slower CI infrastructure (default: 120000ms).
- `@scheduled` tests are local-only, used for debugging token refresh (which requires waiting for real token expiration). Run with `npm run test:playwright:scheduled`.

## Utility Reference

| Utility                      | Purpose                                                                                                                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `auth-server-client.mjs`     | `getAuthServerConfig()`, `getTestCredentials()`, `getPreConfiguredResourceClient()`, `getPreConfiguredAppClient()`, `AuthServerTestClient` for dynamic client creation and role assignment |
| `bodhi-app-server.mjs`       | `createServerManager()` for dedicated-server lifecycle (start/stop)                                                                                                                        |
| `browser-with-extension.mjs` | `BrowserWithExtension` for Chromium with Bodhi extension loaded                                                                                                                            |
| `mock-openai-server.mjs`     | `MockOpenAIServer` for API model tests without real API keys                                                                                                                               |
| `test-helpers.mjs`           | `SHARED_SERVER_URL`, `SHARED_STATIC_SERVER_URL`, `loadBindings()`, `createTestServer()`, `resetDatabase()`, `randomPort()`, `sleep()`, `waitForServer()`                                   |
