# CLAUDE.md - E2E Tests (Playwright)
**Companion docs** (load as needed):
- `E2E.md` -- Test writing conventions, anti-patterns, decision trees

## Purpose

Playwright E2E tests validating **user journeys** through Bodhi App. NAPI bindings embed the Rust server directly into Node.js -- no Docker or external processes. Tests are JavaScript/MJS.

## Core Rules

**Black-box only**: Interact through UI components. Never use `page.evaluate()` + `fetch()` for test assertions. Exception: McpsPage setup helpers (`createAuthHeaderViaApi`, `createOAuthConfigViaApi`, `discoverMcpEndpointsViaApi`, `dynamicRegisterViaApi`) for backend operations with no UI equivalent -- these must include `resp.ok` checks.

**No inline timeouts**: No `page.waitForTimeout()` or `setTimeout()`. Wait for UI state changes via `data-testid`/`data-test-state` attributes with Playwright's built-in waiting. Use `PLAYWRIGHT_TIMEOUT` env var for slow CI.

## Running Tests

All commands from `crates/lib_bodhiserver_napi/`:

| Command | Purpose |
|---------|---------|
| `npm run test:playwright` | All projects, headless (CI default) |
| `npm run test:playwright:standalone` | Standalone project only |
| `npm run test:playwright:multi_tenant` | Multi-tenant project only |
| `npm run test:playwright:headed` | Visible browser |
| `npm run test:playwright:ui` | Playwright UI mode |
| `npm run test:playwright:scheduled` | Local-only token refresh tests |

Sequential execution (`workers: 1`) to avoid port conflicts. `@scheduled` tests excluded by default via `grepInvert`.

## Environment and Credentials

OAuth credentials from `.env.test` (see `.env.test.example`).

### Resource Client
`INTEG_TEST_RESOURCE_CLIENT_ID`/`SECRET` with Direct Access Grants enabled. `user@email.com` as resource admin, `manager@email.com` as resource manager.

### App Client
`INTEG_TEST_APP_CLIENT_ID` -- public OAuth client with redirect URIs for ports 51135 and 55173. Reused across tests so Keycloak remembers consent.

### Test Users

| User | Env Vars | Role |
|------|----------|------|
| `user@email.com` | `INTEG_TEST_USERNAME`, `_ID`, `_PASSWORD` | Resource admin |
| `manager@email.com` | `INTEG_TEST_USER_MANAGER`, `_ID`, `_PASSWORD` | Resource manager |

### Fixed Ports

| Service | Port |
|---------|------|
| Bodhi standalone server (SQLite) | `51135` |
| Bodhi multi-tenant server (PostgreSQL) | `41135` |
| React OAuth test app | `55173` |
| Test MCP OAuth server | `55174` |
| Test MCP OAuth server (DCR) | `55175` |
| Test MCP auth-header server | `55176` |
| Test MCP auth-query server | `55177` |
| Test MCP auth-mixed server | `55178` |

## Architecture Overview

### Directory Layout

| Directory | Contents |
|-----------|----------|
| `specs/` | 32 test files across 13 domain folders |
| `pages/` | 28 page objects extending `BasePage` |
| `fixtures/` | 9 test data modules (classes with static methods) |
| `utils/` | `auth-server-client.mjs`, `bodhi-app-server.mjs`, `browser-with-extension.mjs`, `db-config.mjs`, `mock-openai-server.mjs`, `api-model-helpers.mjs` |
| `scripts/` | `start-shared-server.mjs` |
| `data/` | Test GGUF model refs |

### Dual-Project Architecture

Two Playwright projects run all specs against both deployment modes:
- **`standalone`** (SQLite, port 51135): Runs all specs except `multi-tenant/`
- **`multi_tenant`** (PostgreSQL, port 41135): Runs all specs except `setup/`, `models/`, `request-access/`, `chat/local-models.spec.mjs`

Multi-tenant requires PostgreSQL containers: `docker compose -f docker/docker-compose.test.yml up -d` (app DB on port 64320, session DB on port 54320). Project-aware fixture `sharedServerUrl` auto-selects the correct server URL via `utils/db-config.mjs`.

### Key Mechanisms

- **Path alias**: `@/` maps to `tests-js/`
- **Shared servers**: Auto-started by Playwright `webServer` config (standalone on 51135, multi-tenant on 41135)
- **Auto DB reset**: `@/fixtures.mjs` extends `test` with `autoResetDb` fixture calling `POST /dev/db-reset`
- **Page Object Model**: All pages extend `BasePage` from `@/pages/BasePage.mjs`

## Server Patterns

- **Shared server** (default): Import `test`/`expect` from `@/fixtures.mjs` for auto DB reset
- **Dedicated server**: Import from `@playwright/test`, use `createServerManager()` in `beforeAll`/`afterAll`

## Known Quirks

- **Model selection before API token**: Select model FIRST, then set token (model selection re-renders and clears token input)
- **OAuth flow variants**: With KC session: `waitForTokenExchange()`. Without: `waitForAuthServerRedirect()` + `handleLogin()` + `waitForTokenExchange()`. Access-request: `waitForAccessRequestCallback()`
- **SPA navigation**: Call `waitForSPAReady()` after navigation. Use `page.waitForURL()` for route changes
- **Page object lifecycle**: Create in `beforeEach` (Playwright's `page` is per-test)
- **KC session persistence**: Consent auto-skipped on reused app client
- **Toast handling**: Tech debt (5+ methods in BasePage). Prefer `data-testid` assertions over toast-based verification

## E2E vs server_app Boundary

E2E tests: external Keycloak behavior, UI wiring, browser-dependent flows.
server_app tests: our code's behavior given auth state (via `ExternalTokenSimulator`).

## CI

- `HEADLESS=true` in npm script
- `PLAYWRIGHT_TIMEOUT` env var (default: 120000ms)
- `@scheduled` tests excluded in CI
