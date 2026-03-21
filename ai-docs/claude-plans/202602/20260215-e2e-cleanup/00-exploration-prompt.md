# E2E Test Cleanup: Exploration & Phased Plan

## Objective

BodhiApp's E2E Playwright test suite (in `crates/lib_bodhiserver_napi/tests-js/`) has grown organically and now contains ~25 spec files with tests spanning authorization checks, API validation, multi-user workflows, and UI journeys. Many of these tests exercise concerns that could be tested more reliably and quickly at lower levels of the test pyramid.

Your job is to **explore the existing test landscape thoroughly**, then produce a **phased migration plan** that:
- Offloads authorization/validation tests to `routes_app` (single-endpoint unit tests) or `server_app` (multi-step integration tests)
- Keeps E2E tests focused on genuine end-to-end user journeys that require a real browser
- Replaces the organic `oauth-test-app.html` with a proper Vite+React+Tailwind app
- Explores a cache-bypass approach for testing 3rd-party OAuth token flows in `server_app`
- Results in a lighter, more reliable, more maintainable test suite

**Output**: Write your findings and plan to files in `ai-docs/claude-plans/20260215-e2e-cleanup/`. Start with a context/findings document, then a phased plan document. Each phase should be a self-contained unit of work that ends with running all relevant tests and making a local commit.

---

## Exploration Scope

### What to explore deeply

**1. E2E Test Inventory (crates/lib_bodhiserver_napi/tests-js/specs/)**

For each spec file, understand:
- What is being tested (happy path, authorization, validation, error handling, UI rendering)?
- Does it require a real browser (Keycloak login, UI rendering, redirect behavior) or is it making API calls that could be tested with an HTTP client?
- What services/infrastructure does it need (auth server, LLM model, Exa API key)?
- How many distinct test cases and what's their individual complexity?

**2. routes_app Test Patterns (crates/routes_app/src/)**

Understand the existing test infrastructure:
- How `build_test_router()` and `create_authenticated_session()` work
- The `router.oneshot(request)` pattern for single-endpoint testing
- Existing 401/403 rejection test patterns (`test_*_endpoints_reject_unauthenticated`, `test_*_endpoints_reject_insufficient_role`)
- How mock vs real services are composed via `AppServiceStubBuilder`
- What authorization coverage already exists (look at all `*_test.rs` files across route modules)

**3. server_app Test Patterns (crates/server_app/tests/)**

Understand:
- How `live_server` fixture bootstraps a full server with real Keycloak auth
- The multi-turn request pattern (multiple HTTP calls in one test)
- How OAuth tokens are obtained (password grant via Keycloak) and sessions created
- The `#[serial_test::serial(live)]` constraint and why
- What test utilities exist in `tests/utils/`

**4. OAuth Test App (crates/lib_bodhiserver_napi/tests-js/test-pages/oauth-test-app.html)**

Understand the full flow this HTML app implements:
- Access request creation and review
- PKCE-based OAuth2 authorization code flow
- Token exchange
- State management via sessionStorage

Also read the page object `OAuth2TestAppPage.mjs` to understand how E2E tests interact with it.

**5. Resource Client Creation Patterns**

Explore how resource clients are created across E2E and server_app tests:
- In E2E: Look at `createResourceClient()` and `makeResourceAdmin()` usage in every spec file's `test.beforeAll()`
- In server_app: Look at `setup_minimal_app_service()` in `tests/utils/live_server_utils.rs`
- In Keycloak: What does the `POST /realms/{realm}/bodhi/resources?live_test=true` endpoint do? Can it return an existing client?
- What parameters differ between tests? (redirect_uris use test-specific ports - is this a blocker for sharing?)
- Which tests need custom Keycloak state beyond the standard resource client + admin?

The goal is to determine if a `globalSetup` can create one resource client that 16+ tests reuse, avoiding redundant Keycloak API calls.

**6. Token Validation & Cache Architecture (for later phase)**

Explore how 3rd-party OAuth token validation works:
- `crates/auth_middleware/src/auth_middleware.rs` - How bearer tokens are validated
- `crates/auth_middleware/src/token_service.rs` (or wherever `DefaultTokenService` lives) - The validation pipeline
- How `CacheService` is used to cache token exchange results
- What the cache key format is (token digest?) and what the cached value looks like
- Whether pre-populating the cache with a synthetic exchange result would allow server_app tests to simulate 3rd-party OAuth tokens without hitting Keycloak

This exploration is for a **later phase** - don't propose implementation yet, just document what you find about the cache mechanism and whether the approach is feasible.

---

## Decisions Already Made

These decisions have been made through discussion. Follow them in your plan:

### Test Migration Destinations

| E2E Test Category | Destination | Rationale |
|---|---|---|
| API endpoint 401/403 rejection tests (e.g., toolsets-auth-restrictions API token tests) | `routes_app` | Pure HTTP assertion, no browser needed. routes_app already has this pattern. |
| Validation/error-handling tests that assert on API response bodies | `routes_app` | Single-endpoint tests belong at unit level. |
| OAuth scope-based filtering tests (need real OAuth tokens) | Explore `server_app` via cache bypass | server_app has no browser/Playwright. Need cache pre-population to simulate 3rd-party tokens. Later phase. |
| Multi-user role management flows (session invalidation, re-auth) | Keep in E2E | Genuinely need multiple browser contexts to validate "user sees logged out" |
| UI rendering/navigation tests (toolset config, model forms) | Keep in E2E | Test real browser behavior with real server data |
| Chat tests (chat.spec, chat-agentic, chat-toolsets) | **Out of scope** | Well-structured, test genuine UI. Don't touch. |
| Setup flow (setup-flow.spec.mjs) | Keep in E2E | Genuine 7-step user journey |
| App initializer redirect tests | Remove from E2E | Already covered by AppInitializer.test.tsx Vitest unit tests |
| Canonical redirect tests | Move to `server_app` | HTTP redirect testable with reqwest, no browser needed |
| Network IP setup flow | Keep in E2E | Tests real network behavior that can't be unit tested |
| Token refresh integration (@scheduled) | Keep in E2E | Time-based browser scenario |

### OAuth Test App Replacement

- Replace `oauth-test-app.html` with a **Vite + React + Tailwind CSS** app
- Location: `crates/lib_bodhiserver_napi/test-oauth-app/` (separate package, not shared with E2E test code)
- Support **two flows**: (1) access request + OAuth login, (2) direct OAuth login
- Use React Router for route-based flow selection (`/access-request`, `/oauth-login`)
- Serve via `npx serve` or Vite preview to support client-side routing
- **This happens in a later phase** after test migrations are done

### Phase Ordering

1. **Phase 1**: Explore and inventory all tests. Document what moves where. Investigate resource client reuse feasibility.
2. **Phase 2**: Move 401/403 and validation tests to `routes_app`. Remove from E2E. Run all tests, commit.
3. **Phase 3**: Move canonical redirect and any other HTTP-only tests to `server_app`. Remove app-initializer E2E tests. Run tests, commit.
4. **Phase 4**: Implement resource client reuse via `globalSetup`/`globalTeardown` for E2E tests that share standard config. Run tests, commit.
5. **Phase 5**: Build Vite+React+Tailwind OAuth test app. Update remaining E2E tests to use it. Run tests, commit.
6. **Phase 6**: Explore cache bypass mechanism for 3rd-party OAuth tokens in `server_app`. Propose test utilities. Run tests, commit.
7. **Phase 7**: Final cleanup - remove dead code, update documentation, verify all tests pass.

Each phase should be independently committable and leave all tests passing.

### Resource Client Reuse Optimization

Currently, **16 out of 26 E2E tests** create their own resource client in Keycloak during `test.beforeAll()`. All 16 use the identical pattern: `createResourceClient(serverUrl)` + `makeResourceAdmin(clientId, clientSecret, userId)` with `appStatus: 'ready'`. This is wasteful - each client creation is an HTTP call to Keycloak that could be eliminated.

**Exploration direction**: Investigate whether tests that initialize the server with `appStatus: 'ready'` and don't need custom Keycloak state (no special roles, no resource-admin-less client) can share a pre-created resource client. The server starts fresh each time (new port, new DB), so the Keycloak client is just credentials the server uses - it doesn't need to be unique per test.

**Key questions to answer**:
- Can a resource client be created once in a `globalSetup` and stored for reuse?
- Does the Keycloak `?live_test=true` endpoint support looking up an existing client, or only creating new ones?
- Would the `redirect_uris` registered on the client conflict if different tests use different ports? (Currently each client is registered with `http://localhost:{port}/ui/auth/callback` - this would need to accommodate multiple ports or use a wildcard)
- Which tests genuinely need their own client? (e.g., `list-users.spec.mjs` needs multiple user roles assigned, `oauth2-token-exchange.spec.mjs` creates a public app client, setup-flow tests use `appStatus: 'setup'`)

**Expected outcome**: Identify the subset of tests that can share a client, propose a `globalSetup`/`globalTeardown` pattern, and estimate the time savings. Tests that need custom Keycloak state continue to create their own client.

### Constraints

- `server_app` tests have **no browser/Playwright dependency**. They use `reqwest` HTTP client only.
- `server_app` tests require **real Keycloak** for OAuth (credentials in `.env.test`).
- `server_app` tests run **serially** due to shared llama.cpp binary (`#[serial_test::serial(live)]`).
- `routes_app` tests use `router.oneshot()` - single request/response only.
- The React OAuth test app must be a **separate package** within `crates/lib_bodhiserver_napi/`, not sharing code with E2E tests.
- Chat-related E2E tests (chat.spec, chat-agentic, chat-toolsets) are **out of scope**.

---

## What Your Output Should Look Like

### File 1: `01-exploration-findings.md`

A thorough inventory of:
- Every E2E spec file with test counts and categories
- Which tests are candidates for migration and why
- Which tests must stay in E2E and why
- Existing routes_app and server_app test coverage that would be redundant with migrated tests
- Resource client creation analysis: which tests use identical setup, which need custom state, redirect_uri constraints
- Findings from the cache/token validation architecture exploration

### File 2: `02-phased-plan.md`

A detailed phase-by-phase plan with:
- **Per phase**: What tests move/change, what files are created/modified, what tests to run for validation
- **Per test migration**: Source E2E file, destination module, what the new test should assert, what test utilities are needed
- **For the React app**: Component structure, routes, API integration points, build/serve setup
- **For resource client reuse**: Which tests can share, globalSetup/globalTeardown design, redirect_uri strategy
- **For cache bypass**: Feasibility assessment, proposed test utility API, which E2E tests could move to server_app with this approach
- **Risk assessment** per phase: What could go wrong, how to verify

### File 3 (optional): `03-test-migration-matrix.md`

A comprehensive matrix mapping every E2E test case to its destination (keep/move/remove) with rationale.

---

## Important Notes

- Be thorough in exploration. Read actual test code, not just file names.
- Pay attention to test setup complexity - some E2E tests have elaborate service setup that would need to be replicated in routes_app/server_app.
- When proposing test migrations, ensure the destination already has the necessary test infrastructure (or propose what needs to be added).
- The goal is NOT to eliminate E2E tests, but to right-size them. E2E tests should test user journeys. Lower-level tests should test API contracts and authorization rules.
- Follow existing test patterns in each crate. Don't invent new patterns unless necessary.
- Reference specific file paths and line numbers in your findings.
