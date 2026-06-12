# Testing — Navigation Hub & Philosophy

The single entry point for how BodhiApp tests. This is a hub: it states the philosophy and the cross-layer rules that must live in-repo, then links to the canonical per-layer docs — it does **not** duplicate them. The live test suite is the source of truth; when this doc and the suite disagree, the suite wins.

---

## (a) The Optimized E2E Philosophy

**ONE user journey = ONE `test()` made of many `test.step()` blocks.** Each step is a setup → action → assertion cycle that builds on the server, session, and DB state accumulated by the previous steps. Same user continuing to build state = same test. A different user/role or a fresh DB = a new test.

**Why this matters.** Spinning up `bodhiserver_dev` (and logging into Keycloak) is expensive. A shared server plus batched steps reuses **one session and one DB** across many assertions instead of paying full setup per assertion. Fragmenting a CRUD flow into N tiny `test()`s re-pays login + setup N times and is the single biggest source of slow, flaky suites.

**Anti-patterns** (call these out in review):
- Fragmented N-tiny-tests CRUD that should be 1-2 journeys.
- Re-logging-in per test instead of once per journey.
- Missing `'Phase N:'` step markers on multi-phase journeys.
- Hardcoded test data instead of fixture classes; unit-test naming ("shows Add button") instead of user-goal naming ("MCP Server CRUD Lifecycle").

**The batched exemplar** — `crates/lib_bodhiserver/tests-js/specs/mcps/mcps-crud.spec.mjs:16` — one `test()`, one login, then steps that accumulate state:

```js
// crates/lib_bodhiserver/tests-js/specs/mcps/mcps-crud.spec.mjs:16
test('MCP Server and Instance CRUD Lifecycle', async ({ page, sharedServerUrl }) => {
  const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
  const mcpsPage = new McpsPage(page, sharedServerUrl);
  const serverData = McpFixtures.createServerData();
  const instanceData = McpFixtures.createLifecycleData();

  await test.step('Login', async () => {
    await loginPage.performOAuthLogin('/ui/chat/');
  });

  await test.step('Create MCP server (admin)', async () => {
    await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
    const row = await mcpsPage.page
      .locator(`[data-test-server-name="${serverData.name}"]`).first();
    await expect(row).toBeVisible();
  });

  await test.step('Create MCP instance', async () => {
    await mcpsPage.createMcpInstance(serverData.name, instanceData.name,
      instanceData.slug, instanceData.description);
    await mcpsPage.expectMcpsListPage();
    const row = await mcpsPage.getMcpRowByName(instanceData.name);
    await expect(row).toBeVisible();
  });

  await test.step('Delete MCP instance', async () => {
    const mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
    expect(mcpId).toBeTruthy();
    await mcpsPage.clickDeleteById(mcpId);
    await mcpsPage.confirmDelete();
  });
});
```

Login once. Create-server → create-instance → delete-instance each builds on the prior DB state, all inside one `test()` against the shared server. Use this shape; `crates/lib_bodhiserver/tests-js/E2E.md` lists more canonical exemplars per scenario.

---

## (b) Stack Overview

| Layer | Stack | Where |
|-------|-------|-------|
| **E2E** | Playwright driving a real browser against the prebuilt `bodhiserver_dev` binary + live Vite (`/ui/*` proxied) | `crates/lib_bodhiserver/tests-js/` |
| **Components / hooks / pages** | Vitest (jsdom) + MSW v2 (OpenAPI-typed handlers) | `crates/bodhi/src/` (`*.test.tsx` co-located) |
| **Backend unit** | Rust `rstest` + `#[tokio::test]` + `#[anyhow_trace]`, sibling `test_*.rs` files | per crate (`services`, `routes_app`, …) |
| **Backend integration boundary** | `routes_app` = single request via `tower::oneshot()`; `server_app` = multi-turn real HTTP, `#[serial_test::serial(live)]` | `crates/routes_app/`, `crates/server_app/tests/` |

The integration boundary is a hard rule: single request/response → `routes_app` route-level test (`router.oneshot()`); multi-turn workflows (OAuth flow, tool-calling sequences, session persistence across requests) → `server_app` live integration test (e.g. `crates/server_app/tests/test_oauth_external_token.rs`).

---

## (c) Server Decision Tree (E2E)

```
Custom config (HF_HOME, auth, port)?     -> Dedicated server (createServerManager)
Non-ready app state (setup flow)?        -> Dedicated server
Multiple users with separate roles?      -> Dedicated server (+ dynamic resource client)
Otherwise                                -> Shared server (import @/fixtures.mjs)
```

- **Shared server** (default): import `{ test, expect } from '@/fixtures.mjs'` to get the `sharedServerUrl` + `autoResetDb` fixtures. Ports: **51135** standalone (SQLite), **41135** multi-tenant (PostgreSQL). The same specs run under both Playwright projects; the project-aware `sharedServerUrl` selects the right URL.
- **Dedicated server**: import from `@playwright/test` and use `createServerManager()` (`crates/lib_bodhiserver/tests-js/utils/bodhi-app-server.mjs:64`) in `beforeAll`/`afterAll`. Use this when you need custom env, a non-ready app state, or multiple browser contexts with separate roles.

Full port table, dual-project mechanics, and credential setup: `crates/lib_bodhiserver/tests-js/CLAUDE.md`.

---

## (d) Cross-Layer Rules (codified here so they live in-repo)

- **Black-box E2E.** Drive the UI; never use `page.evaluate()` / `page.context().request` for **assertions**. The only documented exception is `McpsPage` setup helpers (`createAuthHeaderViaApi`, `createOAuthConfigViaApi`, `discoverMcpEndpointsViaApi`, `dynamicRegisterViaApi`) for backend operations with no UI equivalent — and these **must** include `resp.ok` checks.
- **Never `test.skip()` for missing env.** If required credentials/env are absent, **throw** in `beforeAll` so the failure is loud — a silently skipped test is a false green.
- **Cover all layers.** A feature touching multiple crates needs tests at every layer it spans: backend unit, `server_app` integration (multi-turn), and Playwright E2E — not just one.
- **Stable selectors.** Use `data-testid` and `data-test-state` attributes, never CSS classes. Dynamic selectors as functions, e.g. `` mcpRow: id => `[data-testid="mcp-row-${id}"]` ``.
- **No inline timeouts.** No `page.waitForTimeout()` / `setTimeout()`. Wait on `data-testid` / `data-test-state` changes via Playwright's built-in waiting; tune slow CI with `PLAYWRIGHT_TIMEOUT`. (Frontend Vitest: same rule — no inline timeouts.)
- **Frozen time in backend tests.** Never call `Utc::now()`; all timestamps go through `TimeService`, and tests use `FrozenTimeService` (fixed 2025-01-01T00:00:00Z, supplied by the `test_db_service` fixture).
- **Multi-tenant isolation.** Tenant-scoped tables get isolation tests using `TEST_TENANT_ID` / `TEST_TENANT_B_ID` / `TEST_USER_ID` (`crates/services/src/test_utils/db.rs:6`), run across `#[values("sqlite", "postgres")]`. Reference pattern: `crates/services/src/mcps/test_mcp_repository_isolation.rs`.
- **Real services over mocks (backend routes).** Prefer `build_test_router()` (`crates/routes_app/src/test_utils/router.rs:26`) with real DB/session/data/hub services; mock only external boundaries (HTTP calls, specific error paths, LLM inference). See `crates/routes_app/TESTING.md`.

---

## (e) Canonical Sources (do not duplicate)

| Topic | Doc |
|-------|-----|
| E2E philosophy, journey structure, anti-patterns, decision trees | `crates/lib_bodhiserver/tests-js/E2E.md` |
| E2E infra: ports, dual-project, credentials, server patterns, quirks | `crates/lib_bodhiserver/tests-js/CLAUDE.md` |
| Frontend: Vitest setup, MSW v2 handlers, wrappers, fixtures | `crates/bodhi/src/TESTING.md` |
| Route-level Rust tests: `build_test_router()`, mocking strategy, auth injection | `crates/routes_app/TESTING.md` |
| Test-util infrastructure: `TestDbService`, `AppServiceStub`, auth factories, `FrozenTimeService` | `crates/services/src/test_utils/CLAUDE.md` |
| Cross-crate testing conventions (rstest, sibling `test_*.rs`, test boundaries) | `crates/CLAUDE.md` (Shared Testing Conventions) |
| Test-utils packaging (feature-flag pattern) | `docs/conventions/test-utils-packaging.md` |
| Skill: writing/migrating `services` tests | `.claude/skills/test-services/SKILL.md` |
| Skill: writing/migrating `routes_app` tests | `.claude/skills/test-routes-app/SKILL.md` |
