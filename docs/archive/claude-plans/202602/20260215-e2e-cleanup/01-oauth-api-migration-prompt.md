# OAuth API Test Migration: Exploration & Phased Plan

## Objective

The `ExternalTokenSimulator` cache bypass approach has been proven in a POC (commit `27774045`, test file `crates/server_app/tests/test_external_token_poc.rs`). It allows `server_app` tests to simulate 3rd-party OAuth token flows without browser-based Keycloak flows by seeding the token validation cache directly.

Your job is to **explore the remaining E2E tests that use OAuth tokens for API calls**, understand what each test actually validates, and produce a **phased migration plan** that moves API-level OAuth tests to `server_app` while keeping browser-dependent tests in E2E.

**Output**: Write your findings and plan to files in `ai-docs/claude-plans/20260215-e2e-cleanup/`. Start with a findings document, then a phased plan document. Each phase should be a self-contained unit of work that ends with running all relevant tests and making a local commit.

---

## Background: What the POC Proved

Read these files to understand the proven cache bypass mechanism:

- `crates/server_app/tests/utils/external_token.rs` — `ExternalTokenSimulator` utility
- `crates/server_app/tests/test_external_token_poc.rs` — 3 passing POC tests
- `ai-docs/claude-plans/20260215-e2e-cleanup/20260215-e2e-test-migration-to-routes-app.md` — Phase 4 implementation details

The POC demonstrated:
1. OAuth token with `scope_user_user` → 200 OK on scope-accepting endpoints
2. OAuth token without required scope → 401 on scope-accepting endpoints
3. OAuth token → 401 on session-only endpoints

The simulator creates JWTs with `build_token()` (RSA-signed test keys) and seeds the `MokaCacheService` with a `CachedExchangeResult{token, azp}`. No Keycloak, no browser, sub-second execution.

---

## Exploration Scope

### 1. E2E OAuth Token Tests — What's Left?

Thoroughly read the remaining E2E tests that use OAuth tokens to make API calls. The primary candidates are in:

- `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`
- `crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`

For each test, understand:
- What is the **actual assertion**? (HTTP status code? Response body structure? Specific field values?)
- What **setup** does it need beyond auth? (Toolset configuration? Exa API key? Specific DB state?)
- Does it require a **browser** for reasons beyond obtaining an OAuth token? (e.g., Keycloak consent screen rejection, redirect error handling)
- Could the same assertion be made with `router.oneshot()` + `ExternalTokenSimulator`?

Also scan these directories for any other OAuth-token-based API tests you might find:
- `crates/lib_bodhiserver_napi/tests-js/specs/auth/`
- `crates/lib_bodhiserver_napi/tests-js/specs/tokens/`
- `crates/lib_bodhiserver_napi/tests-js/specs/request-access/`

### 2. What Each Test Actually Needs from the Server

Some tests don't just check auth — they also need real data in the database (toolsets, toolset types, API keys). Understand:

- How are toolsets created in the E2E tests? (Session auth + API calls? Direct DB setup?)
- What does the toolset list endpoint actually return? Read the handler code to understand the response shape.
- Which tests call `POST /bodhi/v1/toolsets/{id}/execute/{method}` and what does that execute? (Exa API? Mock?)
- What role does `INTEG_TEST_EXA_API_KEY` play? Which tests need it?

### 3. Existing server_app Test Infrastructure

Understand what's already available in `server_app` tests:

- `crates/server_app/tests/utils/live_server_utils.rs` — How live tests bootstrap services
- `crates/server_app/tests/utils/external_token.rs` — The ExternalTokenSimulator POC
- `crates/server_app/tests/utils/tool_call.rs` — Existing tool call test utilities
- `crates/server_app/tests/test_live_agentic_chat_with_exa.rs` — Existing live test that uses real Exa API

Pay attention to the distinction between:
- **Live tests** (full TCP server, real Keycloak, `#[serial_test::serial(live)]`) — existing pattern
- **Router tests** (in-memory `router.oneshot()`, no TCP, no Keycloak) — what the POC uses

Consider which pattern is more appropriate for each migrated test.

### 4. routes_app Test Infrastructure

Also explore `crates/routes_app/src/test_utils/router.rs` to understand:
- `build_test_router()` — what services it wires (and that it uses `MockToolService` by default)
- The POC's custom router setup with `DefaultToolService` — is this pattern worth extracting?
- `create_authenticated_session()` — for tests that need session auth alongside OAuth

### 5. Auth Middleware Route Configuration

Read `crates/routes_app/src/routes.rs` to understand the auth tiers for each toolset endpoint:
- Which endpoints accept OAuth tokens? (What `TokenScope`/`UserScope` configuration?)
- Which are session-only? (Both `TokenScope` and `UserScope` set to `None`)
- How does scope filtering work in the toolset list handler?

This is critical for understanding what the cache bypass can and cannot simulate.

### 6. Tests That Should Stay in E2E

Some OAuth tests genuinely need a browser. Identify these and explain why:
- Tests that exercise Keycloak's consent screen or error pages
- Tests that validate redirect flows (authorization code exchange)
- Tests where the browser's cookie/session behavior is part of what's being tested
- Tests that validate the full OAuth2 authorization code flow end-to-end

The goal is to be precise about the boundary: what can move, what must stay, and why.

---

## Context: Prior Work

These documents provide context on the broader E2E cleanup effort:

- `ai-docs/claude-plans/20260215-e2e-cleanup/20260215-e2e-test-migration-to-routes-app.md` — Completed 4-phase plan (Phases 1-4 all done)
- `ai-docs/claude-plans/20260215-e2e-cleanup/00-exploration-ctx.md` — Original exploration findings
- `ai-docs/claude-plans/20260215-e2e-cleanup/00-exploration-prompt.md` — Original exploration prompt

### Decisions Already Made

- `server_app` tests have **no browser/Playwright dependency**. They use `reqwest` or `router.oneshot()`.
- Each migration phase should be independently committable with all tests passing.
- Prefer real services over mocks in `server_app` tests (the POC test 1 uses `DefaultToolService` backed by real SQLite).
- Live Exa API calls in tests are acceptable (the existing `test_live_agentic_chat_with_exa.rs` does this).

### Constraints

- `ExternalTokenSimulator` bypasses the auth server entirely — it cannot test scenarios where Keycloak itself rejects a request (e.g., `invalid_scope` error from Keycloak's authorization endpoint).
- The simulator seeds `MokaCacheService` which is the default in-memory cache from `AppServiceStubBuilder`. Live server tests using `setup_minimal_app_service()` also have a cache service — verify it's the same type.
- Tests using `router.oneshot()` are single-request only. Multi-request sequences (e.g., "create toolset via session, then access via OAuth") require either multiple `oneshot()` calls on cloned routers or a live server.

---

## What Your Output Should Look Like

### File 1: Exploration Findings

A thorough analysis of:
- Every remaining E2E test that uses OAuth tokens, with migration feasibility assessment
- What data setup each test requires beyond authentication
- Gap analysis: what the `ExternalTokenSimulator` can handle vs. what needs real Keycloak
- Existing test infrastructure inventory (what can be reused, what needs to be built)
- Any tricky edge cases or gotchas discovered during exploration

### File 2: Phased Migration Plan

A detailed phase-by-phase plan with:
- **Per phase**: Which tests move, what files are created/modified, verification commands
- **Per test migration**: Source E2E test, destination file, what the new test asserts, what setup is needed
- **Test utility design**: What helpers or builders need to be created for the migrated tests
- **What stays in E2E**: Explicit list of tests that cannot move and why
- **Risk assessment**: What could go wrong, how to verify nothing is lost

---

## Important Notes

- Be thorough in exploration. Read actual test code and handler code, not just file names.
- Pay attention to what each test's `beforeAll` sets up — this is often the hardest part to replicate.
- Some E2E tests may be partially migratable (the API assertion can move, but the setup flow stays as a separate E2E test).
- The goal is to keep E2E tests focused on genuine browser-dependent user journeys while moving pure API contract tests closer to the code they test.
- Reference specific file paths and line numbers in your findings.
- Follow existing patterns in `server_app` and `routes_app` tests. Don't invent new patterns unless necessary.
