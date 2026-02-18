---
name: E2E test documentation
overview: Rewrite tests-js/CLAUDE.md (fix stale claims, add missing sections) and create tests-js/E2E.md as a detailed test-writing reference using SKILL.md-style progressive disclosure.
todos:
  - id: rewrite-claude-md
    content: "Rewrite CLAUDE.md: fix stale addRedirectUri claim, add @scheduled/autoResetDb/realm-admin/path-alias sections, add architecture overview and server patterns, restructure with progressive disclosure links to E2E.md"
    status: completed
  - id: create-e2e-md
    content: "Create E2E.md: test philosophy, new spec process (TDD-style with test.step), anti-patterns (MCP example with corrected version), page object conventions, fixture conventions, server decision tree, flakiness mitigations, CI considerations, import patterns"
    status: completed
isProject: false
---

# E2E Test Documentation (CLAUDE.md + E2E.md)

## Goal

Rewrite [tests-js/CLAUDE.md](crates/lib_bodhiserver_napi/tests-js/CLAUDE.md) to fix stale information and add missing sections, and create [tests-js/E2E.md](crates/lib_bodhiserver_napi/tests-js/E2E.md) as a detailed test-writing reference. Uses SKILL.md-style progressive disclosure: CLAUDE.md is the concise entry point, E2E.md is the deep-dive loaded when writing tests.

## Source-of-Truth Validation Results

Validated every claim in the current CLAUDE.md against source code. Key findings:

### Stale/Incorrect (MUST FIX)

- **addRedirectUri() claim**: CLAUDE.md says "redirect URI for the test app is also registered dynamically in `test.beforeAll` via `authClient.addRedirectUri()`" -- `addRedirectUri()` exists in auth-server-client.mjs but is **never called** in any spec. Redirect URIs are pre-configured in Keycloak. Remove this claim.

### Verified and Retained

- Resource client needs Direct Access Grants -- verified via `grant_type: 'password'` in `getResourceUserToken()`
- [user@email.com](mailto:user@email.com) as resource admin, [manager@email.com](mailto:manager@email.com) as resource manager -- verified in test assertions and env vars
- App client redirect URIs for 51135/55173 -- verified in playwright.config.mjs and test-helpers.mjs
- handleConsent() NOT needed -- verified: defined in OAuthSection.mjs but never called
- KC session auto-redirect behavior -- verified across toolsets and oauth specs
- Fixed ports 51135/55173 -- verified in config and scripts
- workers: 1, fullyParallel: false -- verified in playwright.config.mjs
- E2E vs server_app boundary -- verified: ExternalTokenSimulator exists in server_app tests

### Documented but Violated (KEEP as guidance, note violations)

- Chat settings model-before-token order -- correct advice but violated in several api-tokens.spec.mjs tests (lines 76-82, 287-292, 306-307, etc.)

### Missing from Current CLAUDE.md (MUST ADD)

- `@scheduled` tag and `test:playwright:scheduled` command (used in token-refresh-integration.spec.mjs)
- `autoResetDb` fixture from `@/fixtures.mjs` (resets DB before each test)
- `@/` path alias mapping to `tests-js/`
- Realm admin credentials (`INTEG_TEST_REALM_ADMIN`, `INTEG_TEST_REALM_ADMIN_PASS`)
- Architecture overview (directory layout, shared server, page objects, fixtures)
- Server patterns (shared vs dedicated decision tree)
- Page object lifecycle (must be created in `beforeEach`)
- SPA navigation timing patterns
- Flakiness sources and mitigations

---

## Subfolder CLAUDE.md Decision: NOT Needed

Explored whether `pages/`, `specs/`, `fixtures/`, `utils/` need their own CLAUDE.md files.

**Decision: No subfolder CLAUDE.md files.** Reasons:

- The project already has 21 CLAUDE.md files; many existing subfolder ones (server_core/test_utils, objs/test_utils, services/test_utils) are extremely verbose and repetitive
- The SKILL.md progressive disclosure pattern uses **topic-based files in the same directory** (not subfolder CLAUDE.md files)
- `pages/` (28 files) -- conventions fit cleanly as a section in E2E.md
- `fixtures/` (8 files) -- same, covered by E2E.md fixture section
- `specs/` (25 files across 11 domains) -- anti-patterns and canonical examples in E2E.md
- `utils/` (4 files) -- brief utility listing in CLAUDE.md

**Progressive disclosure structure:**

```
tests-js/CLAUDE.md   -- concise orientation, links to E2E.md
tests-js/E2E.md      -- detailed test-writing reference
```

---

## CLAUDE.md Structure (~120 lines)

### 1. Header and Purpose

- Playwright E2E tests validating user journeys through Bodhi App
- NAPI bindings embed Rust server in Node.js; tests written in JavaScript/MJS
- Tests are user journeys, NOT unit tests
- "For detailed test writing guide, see [E2E.md](E2E.md)"

### 2. Running Tests

- Commands: `npm run test:playwright`, `test:playwright:headed`, `test:playwright:ui`
- Sequential: `workers: 1`, `fullyParallel: false`
- `@scheduled` tag: local-only tests for token refresh debugging, excluded by default via `grepInvert: /@scheduled/`, run with `npm run test:playwright:scheduled`

### 3. Environment and Credentials

- `.env.test` loaded from `.env.test.example` template
- **Resource client** (`INTEG_TEST_RESOURCE_CLIENT_ID/SECRET`): Direct Access Grants enabled, [user@email.com](mailto:user@email.com) as resource admin, [manager@email.com](mailto:manager@email.com) as resource manager
- **App client** (`INTEG_TEST_APP_CLIENT_ID`): Public OAuth client with redirect URIs pre-configured in Keycloak for ports 51135 and 55173
- **Realm admin** (`INTEG_TEST_REALM_ADMIN/PASS`): Used by `AuthServerTestClient` for dynamic resource client creation and role assignment in dedicated-server tests
- **Test users table** (retain existing)
- **Fixed ports table** (retain existing: Bodhi 51135, OAuth test app 55173)

### 4. Architecture Overview

- **Directory layout**: `specs/` (25 test files), `pages/` (28 page objects), `fixtures/` (8 data modules), `utils/` (4 helpers), `scripts/` (server startup)
- **Path alias**: `@/` maps to `tests-js/` (e.g., `import { test } from '@/fixtures.mjs'`)
- **Shared server**: Auto-started by Playwright webServer config on port 51135 via `scripts/start-shared-server.mjs`
- **Auto DB reset**: `@/fixtures.mjs` extends Playwright test with `autoResetDb` fixture that calls `POST /dev/db-reset` before each test
- **Page Object Model**: All page objects extend `BasePage` from `@/pages/BasePage.mjs`

### 5. Server Patterns

- **Shared server** (default): For app state=ready with [user@email.com](mailto:user@email.com) as admin. Import from `@/fixtures.mjs` (gets auto DB reset).
- **Dedicated server**: For non-ready app state (setup), multi-user scenarios, custom config (e.g., overridden HF_HOME for model-metadata tests). Import from `@playwright/test` directly. Use `createServerManager()` in beforeAll/afterAll.
- Link to E2E.md Server Decision Tree for full criteria.

### 6. Known Quirks

- **Model before token**: Select model FIRST, then set API token in chat settings. Model selection re-render clears token input. (Retain existing code example.)
- **OAuth flow variants**: Three patterns depending on KC session state. (Retain existing code examples.)
- **SPA navigation**: Always `waitForSPAReady()` after navigation; use `page.waitForURL()` for route changes.
- **Page object lifecycle**: Create in `beforeEach`, NOT `beforeAll` -- Playwright page is per-test.
- **KC session persistence**: Consent auto-skipped on reused app client; `waitForAuthServerRedirect()` skipped when KC session exists.
- **Toast handling**: Tech debt -- 5+ methods in BasePage with fallbacks. Use `waitForToast` for critical assertions only.
- **Flakiness sources**: Toast timing, KC redirect timing, LLM model loading variability. Never add inline timeouts.

### 7. E2E vs server_app Boundary

- Retain existing section (verified accurate)

### 8. CI Considerations

- `HEADLESS=true` set in `test:playwright` npm script
- Env vars from GitHub repo vars/secrets
- `PLAYWRIGHT_TIMEOUT` env var for slower CI (default: 120000ms)
- `@scheduled` tests excluded in CI

---

## E2E.md Structure (~350 lines)

### 1. Test Philosophy

- Tests are user journeys, not unit tests
- Each test tells a story: login -> perform actions -> verify results
- 1-4 tests per `test.describe`, each a meaningful journey
- Journey boundary: same user + building state = same test; different user or needs fresh state = new test
- Do NOT combine unrelated flows for efficiency; each test must make sense as a standalone story

### 2. Writing a New E2E Spec

TDD-style process:

1. Define user journey as `test.step()` blocks (skeleton first)
2. Build Page Object methods as needed
3. Add Fixture file for reusable test data
4. Fill in assertions within each step
5. Run and iterate

Canonical examples (reference by path):

- Shared-server journey: [api-tokens.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/tokens/api-tokens.spec.mjs)
- Multi-phase with test.step: [toolsets-auth-restrictions.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs)
- Dedicated server + custom config: [model-metadata.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/models/model-metadata.spec.mjs)
- Multi-user with browser contexts: [multi-user-request-approval-flow.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/request-access/multi-user-request-approval-flow.spec.mjs)

### 3. Test Structure Standards

- `test.step()` for all major phases (standardized going forward)
- Import rules: `@/fixtures.mjs` for shared server, `@playwright/test` for dedicated
- `beforeAll`: auth config, server setup (expensive, shared across tests)
- `beforeEach`: page object creation (per-test, due to Playwright page lifecycle)
- `afterAll`: server cleanup for dedicated servers
- `try/finally`: browser context cleanup in multi-user tests

### 4. Anti-Patterns

Using [mcps-crud.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-crud.spec.mjs) as reference:

- **Fragmented tests**: 6 small tests that should be 1-2 journeys
- **Repeated login**: `performOAuthLogin()` called 6x instead of once per journey
- **No test.step()**: Flat bodies without phase markers
- **No fixture file**: Inline test data
- **Overlapping assertions**: "displays list" and "shows button" are sub-steps, not tests
- **Unit-test naming**: Describes UI elements, not user goals

Include a corrected version showing how the MCP spec should be structured as 1-2 journey tests with test.step.

### 5. Page Object Conventions

- Extend `BasePage` (`@/pages/BasePage.mjs`): provides `navigate()`, `waitForSPAReady()`, `waitForToast()`, `clickTestId()`, `fillTestId()`, `expectVisible()`
- Constructor: `(page, baseUrl)` matching Playwright page lifecycle
- Selectors: `selectors = {}` property with `data-testid` selectors (not CSS classes)
- Dynamic selectors: functions like `mcpRow: (id) => \`[data-testid="mcp-row-${id}"]`
- Method naming: `navigateTo*`, `expect*`, `click*`, `fill*`, `waitFor*`
- Composite pages: For complex UIs, compose section objects (e.g., `OAuthTestApp` with `config`, `oauth`, `accessCallback`, `rest`, `chat` sections)
- Responsive elements: `data-testid` may have mobile/desktop duplicates; use `.last()` for desktop viewport

### 6. Fixture Conventions

- Classes with static methods in `fixtures/` directory
- `Date.now()` + random suffix for unique data within a journey
- Factory methods: `createComprehensiveLifecycleData()`, `getTestTokenNames()`, `getErrorScenarios()`
- Canonical examples: [localModelFixtures.mjs](crates/lib_bodhiserver_napi/tests-js/fixtures/localModelFixtures.mjs) (factory pattern), [tokenFixtures.mjs](crates/lib_bodhiserver_napi/tests-js/fixtures/tokenFixtures.mjs) (static data + utilities like clipboard mock)

### 7. Server Configuration Decision Tree

```
Need custom server config (HF_HOME, auth, port)?
  YES -> Dedicated server
Need non-ready app state (setup flow)?
  YES -> Dedicated server
Need multiple users with separate roles?
  YES -> Dedicated server with dynamic resource client
         (uses createAuthServerTestClient + createResourceClient + makeResourceAdmin)
Otherwise:
  -> Shared server (port 51135)
     Import from @/fixtures.mjs
```

For dedicated servers: `createServerManager()` from `@/utils/bodhi-app-server.mjs`, start in `beforeAll`, stop in `afterAll`.

### 8. Flakiness Patterns and Mitigations

- **Toast timing**: Use `waitForToast` for critical assertions, `waitForToastOptional` for non-critical. Toast methods in BasePage are tech debt.
- **KC redirect timing**: Use `waitForAuthServerRedirect` / `waitForTokenExchange` -- never fixed waits. KC auto-redirects with cached session.
- **LLM model loading**: Default 120s timeout from config. Never add inline timeouts.
- **SPA hydration**: Always `waitForSPAReady()` after navigation. Use `page.waitForURL()` for route changes.

### 9. CI and @scheduled Tests

- `HEADLESS=true` in npm script
- Env vars from GitHub repo vars/secrets
- `PLAYWRIGHT_TIMEOUT` for slower CI
- `@scheduled` tests: local-only for token refresh debugging (long waits for token expiration). Run with `npm run test:playwright:scheduled`.

### 10. Import Patterns Quick Reference

```js
// Shared server (auto DB reset):
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL } from '@/test-helpers.mjs';

// Dedicated server (manage own state):
import { expect, test } from '@playwright/test';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
```

### 11. Utility Reference

- `auth-server-client.mjs`: Keycloak config, test credentials, `AuthServerTestClient` for dynamic client creation and role management
- `bodhi-app-server.mjs`: `BodhiAppServer` lifecycle (start/stop) for dedicated-server specs
- `browser-with-extension.mjs`: Chromium with Bodhi extension loaded for extension tests
- `mock-openai-server.mjs`: Mock OpenAI API for API model tests without real API keys

