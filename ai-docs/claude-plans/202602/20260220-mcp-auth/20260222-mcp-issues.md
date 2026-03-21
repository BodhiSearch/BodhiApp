# Plan: Fix All Issues from MCP OAuth Code Review

## Context

A comprehensive code review of the MCP OAuth 2.1 authentication feature (commits `2a7eb2dc`, `f2c434e9`) identified 3 Critical, 19 Important, and 18 Nice-to-have issues across all layers. This plan fixes ALL of them following the layered development methodology, organized as sequential milestones with gate tests.

Review index: `ai-docs/claude-plans/20260220-mcp-auth/reviews/index.md`

---

## Layer 1: objs crate (I1, I2)

**Gate test**: `cargo test -p objs`

### I1: Create `RegistrationType` enum
- **File**: `crates/objs/src/mcp.rs`
- Add enum with `PreRegistered`, `DynamicRegistration` variants, `#[serde(rename_all = "kebab-case")]`
- Replace `registration_type: String` with `RegistrationType` in `CreateMcpAuthConfigRequest::Oauth`, `McpAuthConfigResponse::Oauth`, `McpOAuthConfig`
- Add `impl Display` for the enum

### I2: URL validation on OAuth endpoints
- **File**: `crates/objs/src/mcp.rs`
- Add validation for `authorization_endpoint` and `token_endpoint` using `url::Url::parse()`
- Follow existing pattern from `validate_mcp_server_url`

---

## Layer 2: services crate (C2, I3, I4, I9, N2, N3, N4, N5, N6, N7)

**Gate test**: `cargo test -p objs -p services`

### C2: Token refresh tests (3 new tests)
- **File**: `crates/services/src/mcp_service/tests.rs`
- Test `resolve_oauth_token` indirectly via `fetch_tools()` (private method)
- Use FrozenTimeService (ts=1735689600) + `expires_in=30` to trigger expiry
- Use mockito for HTTP token refresh endpoint

**Tests**:
1. `test_resolve_oauth_token_expired_with_refresh_success` - expired token + refresh token, mock HTTP returns new tokens, verify DB updated + correct Bearer header passed to MockMcpClient
2. `test_resolve_oauth_token_expired_no_refresh_returns_error` - expired + no refresh, assert `mcp_error-o_auth_token_expired`
3. `test_resolve_oauth_token_expired_refresh_http_failure` - expired + refresh + HTTP 401, assert `mcp_error-o_auth_refresh_failed`

**Pattern**: Follow `test_mcp_service_execute_with_oauth_auth_type` (existing test that covers "not expired" path)

### I3: Fix MCP delete removing shared auth headers
- **File**: `crates/services/src/mcp_service/service.rs:935-937`
- Remove the `McpAuthType::Header` arm from delete method's cleanup logic
- Auth headers are admin-managed shared resources, should only be deleted via explicit `delete_auth_config`

### I4: Prevent orphaned OAuth token rows
- **File**: `crates/services/src/mcp_service/service.rs` (`store_oauth_token` method)
- Before inserting new token, delete existing tokens for same `(config_id, user_id)`
- Alternative: `crates/services/migrations/0012_mcp_oauth.up.sql` add UNIQUE constraint (but migration approach is riskier for existing data)

### I9: Missing DB tests for token update and OAuth bearer
- **File**: `crates/services/src/db/test_mcp_repository.rs`
- Add `test_init_service_update_mcp_oauth_token`: create token, update with new encrypted values, verify decrypted match
- Add `test_init_service_get_decrypted_oauth_bearer`: create config+token, call `get_decrypted_oauth_bearer`, verify `("Authorization", "Bearer <plaintext>")`

### N2: Bounded refresh_locks
- **File**: `crates/services/src/mcp_service/service.rs:225,253-267`
- Replace unbounded `HashMap` with LRU cache or add periodic cleanup

### N3: Disambiguate `get_decrypted_client_secret`
- **File**: `crates/services/src/db/service_mcp.rs:717-738`
- Return `Result<Option<(String, String)>, DbError>` where None = "config has no secret stored" vs error = "config not found"
- Or return a dedicated enum to distinguish the two cases

### N4: Add logging to token refresh flow
- **File**: `crates/services/src/mcp_service/service.rs:428-563`
- INFO: successful refresh, WARN: token expired with no refresh token, DEBUG: token not expired (skipped refresh)

### N5: Add logging to discovery and DCR flows
- **File**: `crates/services/src/mcp_service/service.rs` (discovery/DCR methods around L1308-1450)
- INFO: successful discovery/DCR, WARN: discovery/DCR failure, DEBUG: request details

### N6: Add IF NOT EXISTS to migration indexes
- **File**: `crates/services/migrations/0012_mcp_oauth.up.sql:40-41`
- Change `CREATE INDEX` to `CREATE INDEX IF NOT EXISTS`

### N7: Wrap cascade delete in transaction
- **File**: `crates/services/src/mcp_service/service.rs` (`delete_oauth_config` L1244-1248)
- Wrap token delete + config delete in a DB transaction

---

## Layer 3: routes_app implementation (I5, I6, I7, N8, N9, N10)

**Gate test**: `cargo test -p objs -p services -p routes_app`

### I5: Add TTL to OAuth CSRF state
- **File**: `crates/routes_app/src/routes_mcp/auth_configs.rs:158-166, 226-255`
- In `oauth_login_handler`: store `created_at` timestamp alongside state in session JSON
- In `oauth_token_exchange_handler`: validate state age < 10 minutes, reject with clear error if expired

### I6: Move token exchange HTTP logic into McpService
- **File**: `crates/routes_app/src/routes_mcp/auth_configs.rs:289` and `crates/services/src/mcp_service/service.rs`
- Add `exchange_oauth_token(config_id, code, redirect_uri, code_verifier)` method to McpService trait
- Move reqwest POST logic from handler into service (uses shared `http_client`)
- Handler becomes thin orchestrator: session validation -> service call -> response
- This also fixes N11 (db_service bypass) since service has direct db access

### I7: Validate redirect_uri
- **File**: `crates/routes_app/src/routes_mcp/auth_configs.rs:177, 277`
- Validate `redirect_uri` via `url::Url::parse()` in both handlers
- Reject non-well-formed URLs with descriptive error

### N8: Add ownership check on auth config delete
- **File**: `crates/routes_app/src/routes_mcp/auth_configs.rs:111-118`
- Extract `AuthContext` from request, pass `user_id` to service
- Service validates ownership before deletion (or require Admin role)

### N9: Split McpValidationError into domain-specific variants
- **File**: `crates/routes_app/src/routes_mcp/error.rs`
- Split single `Validation(String)` variant into: `CsrfStateMismatch`, `CsrfStateExpired`, `SessionDataMissing`, `TokenExchangeFailed(String)`, `InvalidUrl(String)`, `InvalidRedirectUri(String)`
- Each variant gets proper `error_type` and `code` via errmeta_derive

### N10: URL validation on discovery inputs
- **File**: `crates/routes_app/src/routes_mcp/oauth_utils.rs` (discovery handlers L38, L85, L134)
- Validate input URLs via `url::Url::parse()` before passing to service

---

## Layer 4: routes_app tests (C1, I8, I10, N12, N13, N14)

**Gate test**: `cargo test -p objs -p services -p routes_app`

### C1: OAuth login/token exchange handler tests (6 new tests)
- **File**: `crates/routes_app/src/routes_mcp/test_auth_configs.rs`

**New session-aware helper** (pattern from `routes_auth/tests/login_test.rs:856-900`):
- `SqliteSessionService::build_session_service(dbfile)` for real session
- `session_store.create(&mut record)` to pre-populate session
- `.with_sqlite_session_service(session_service)` on builder
- `.layer(app_service.session_service().session_layer())` on router
- `header("Cookie", format!("bodhiapp_session_id={}", id))` for cookie

**Login tests**:
1. `test_oauth_login_success` - verify auth URL params (response_type, client_id, redirect_uri, code_challenge_method=S256, state, scope, resource)
2. `test_oauth_login_config_not_found` - 404

**Token exchange tests** (use mockito for token endpoint after I6 moves logic to service):
3. `test_oauth_token_exchange_success` - pre-populated session, mock service calls, verify 200 + store called
4. `test_oauth_token_exchange_state_mismatch` - wrong state -> 400 CSRF error
5. `test_oauth_token_exchange_missing_session` - no session -> 400
6. `test_oauth_token_exchange_http_failure` - token endpoint error -> 400

### I8: OAuth auth config creation tests
- **File**: `crates/routes_app/src/routes_mcp/test_auth_configs.rs`
- Add `test_create_auth_config_oauth_prereg_success`
- Add `test_create_auth_config_oauth_dcr_success`

### I10: Fix delete handler test verification
- **File**: `crates/routes_app/src/routes_mcp/test_oauth_utils.rs:348`
- Add `MockDbService.expect_delete_mcp_oauth_token()` expectation or use real DB integration test

### N12: Extract shared test router helper
- **File**: New helper in `crates/routes_app/src/routes_mcp/` test utils
- Replace 5 duplicate `test_router_for_*` functions with shared builder

### N13: Replace `Utc::now()` with deterministic time in test fixtures
- **Files**: `test_auth_configs.rs`, `test_mcps.rs`, `test_servers.rs`, `test_oauth_utils.rs`
- Use fixed `DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")`

### N14: Test `resource` parameter in auth URL
- Covered by C1 test #1 (`test_oauth_login_success`) which asserts `resource` param

---

## Layer 5: Full backend validation

**Gate test**: `make test.backend`

---

## Layer 6: TypeScript client regeneration

**Command**: `cargo run --package xtask openapi && make build.ts-client`

Required because I1 changes `registration_type` from `String` to enum, affecting OpenAPI schema.

---

## Layer 7: Frontend implementation + tests (C3, I11-I17, N15-N19)

**Gate test**: `cd crates/bodhi && npm run test`

### C3: Replace inline setTimeout (2 files)
- `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx:249` - replace `setTimeout(resolve, 500)` with `waitFor(() => expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument())`
- Also fix L253-254: `textContent.toContain` -> `toHaveTextContent`
- `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx:504` - same replacement

### I11: Deduplicate OAUTH_FORM_STORAGE_KEY
- Export `OAUTH_FORM_STORAGE_KEY` from `crates/bodhi/src/stores/mcpFormStore.ts`
- Import in `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx` instead of re-declaring

### I12: Add try/catch to restoreFromSession
- **File**: `crates/bodhi/src/stores/mcpFormStore.ts:85-90`
- Wrap `JSON.parse(saved)` in try/catch, return null on failure, remove corrupt entry

### I13: Fix view page auto-DCR flag
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx:267`
- Change `enableAutoDcr={true}` to `enableAutoDcr={false}`

### I14: Extract shared DCR submission helper
- Extract `buildDcrAuthConfig` from `new/page.tsx:115-154` and `view/page.tsx:107-161`
- Share between both pages

### I15: Fix skipped DCR test
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx:107-211`
- Fix handler (use `http.post()` directly instead of callback to `mockCreateMcpServer`)
- Remove `{ timeout: 5000 }` from waitFor
- Unskip the test

### I16: Add OAuth auth config creation test for view page
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`
- Add test: select OAuth, fill fields, save, verify creation

### I17: Add custom name preservation test
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`
- Add test: set custom name, switch Header->OAuth, assert name preserved

### N15: Remove dead useDiscoverAs hook
- **File**: `crates/bodhi/src/hooks/useMcps.ts:355-370`

### N16: Remove unused useStandaloneDynamicRegister import
- **File**: `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx:11`

### N17: Extract shared badge mapping
- Extract `authConfigTypeBadge`/`getAuthConfigTypeBadge` from `mcp-servers/page.tsx`, `mcps/new/page.tsx` into shared `mcpUtils.ts`

### N18: Clear session on callback error
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx:87-89`
- Clear sessionStorage data in `onError` handler to prevent stale retry

### N19: Remove non-existent import
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx:5`
- Remove `mockCreateMcpServerError` import (doesn't exist)

---

## Layer 8: E2E tests + Documentation

**Gate test**: `make build.ui-rebuild && make test.napi`

### CRITICAL: E2E Testing Philosophy Update

**E2E tests are BLACK BOX tests.** They must NOT call APIs directly via `page.evaluate()` + `fetch()`. All test interactions must go through UI components only.

#### 8a: Update CLAUDE.md documentation
- **Remove** from `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md` the section:
  > **API-based test setup**: E2E tests use `page.evaluate()` + `fetch()` to create auth configs...
- **Add** to `tests-js/CLAUDE.md`:
  > **Strict black-box testing**: E2E tests MUST interact only through UI components. Never use `page.evaluate()` + `fetch()` to call APIs directly. All test data setup must go through the application's UI flow. If a prerequisite step requires complex setup, create it through the UI navigation path, not through API calls.
  > **No inline timeouts**: Do not use `setTimeout` or fixed waits in tests (exception: LLM inference calls that take longer). Instead, wait for UI element updates using `data-testid`, `data-test-state`, or other `data-test*` attributes to detect internal state changes.

#### 8b: Migrate existing page.evaluate patterns to UI interactions
- **File**: `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs:174-244`
- Replace `createAuthHeaderViaApi`, `createOAuthConfigViaApi`, `discoverMcpEndpointsViaApi`, `dynamicRegisterViaApi` methods
- Create equivalent UI-flow methods that navigate to the server view page, click "Add Auth Config", fill the form, and save
- Also improve I18: since we're removing page.evaluate API calls, the resp.ok check issue is solved by elimination

#### 8c: Fix I19 - Header auth tests external dependency
- **File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs`
- Create mock header-auth MCP server (simple Express with bearer validation) instead of depending on external Tavily API

#### 8d: Fix N20 - DCR setup duplication
- **File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-dcr.spec.mjs`
- Extract shared DCR setup into helper/fixture

#### 8e: E2E debugging workflow
When fixing E2E tests:
1. Load playwright skills (`.claude/skills/` if available)
2. Run `make app.clean-run` to start server on `localhost:1135`
3. Use Claude in Chrome to navigate to app
4. Login using credentials from `crates/lib_bodhiserver_napi/tests-js/.env.test`
5. Manually reproduce E2E steps to identify where tests fail
6. Verify all tests avoid timeouts (except LLM inference) - use `data-test-state` / `data-test*` attributes

---

## Layer 9: Documentation updates

### Update crate CLAUDE.md files
- `crates/routes_app/CLAUDE.md`: Add McpValidationError variant docs (after N9 split)
- `crates/bodhi/src/CLAUDE.md`: Add auto-DCR behavior docs (new page = silent fallback, view page = error + retry)
- `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md`: Update E2E testing rules (no page.evaluate, no timeouts)
- `tests-js/E2E.md`: Add step naming convention documentation

---

## Execution Summary

| Layer | Issues | Gate Test | Sub-agent |
|-------|--------|-----------|-----------|
| 1 | I1, I2 | `cargo test -p objs` | Agent 1 |
| 2 | C2, I3, I4, I9, N2-N7 | `cargo test -p objs -p services` | Agent 2 |
| 3 | I5, I6, I7, N8, N9, N10 | `cargo test -p objs -p services -p routes_app` | Agent 3 |
| 4 | C1, I8, I10, N12, N13 | `cargo test -p objs -p services -p routes_app` | Agent 4 |
| 5 | -- | `make test.backend` | Gate only |
| 6 | -- | `make build.ts-client` | Gate only |
| 7 | C3, I11-I17, N15-N19 | `cd crates/bodhi && npm run test` | Agent 5 |
| 8 | I18, I19, N20, docs | `make build.ui-rebuild && make test.napi` | Agent 6 |
| 9 | docs | Manual review | Agent 7 |

**Total**: ~42 issues across 9 layers, each gated by cumulative test pass.
