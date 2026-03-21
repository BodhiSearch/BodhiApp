# MCP OAuth Review - Issue Index

## Summary
- Total findings: 56 (across 7 reports)
- Positive findings (no action): 14
- Actionable findings: 42
- **Critical: 3** | **Important: 19** | **Nice-to-have: 20**
- False positive retracted: 1 (services-review Finding 3 -- `resolve_oauth_token` correctly uses token ID, not config ID)

## Cross-Cutting Analysis

### Type Consistency: objs <-> OpenAPI <-> ts-client <-> frontend
- **PASS**: All frontend types sourced from `@bodhiapp/ts-client` (ui-impl Finding 15)
- **GAP**: `registration_type` is unvalidated `String` everywhere (objs Finding 1)
- **MINOR**: Duplicate UI-local type aliases (`AuthConfigType`, `OAuthRegistrationType`) in 2 files (ui-impl Finding 8)

### Error Chain: service -> AppError -> ApiError -> HTTP -> UI
- **GAP**: `McpValidationError` is a single-variant catch-all -- all OAuth errors produce the same error code `mcp_validation_error-validation` (routes_app Finding 5). Frontend cannot distinguish CSRF mismatch from session expiry from token exchange failure.
- **OK**: Error enums in services follow errmeta_derive pattern correctly (services Finding 9)

### OAuth 2.1 Security Summary
| Control | Status | Notes |
|---------|--------|-------|
| PKCE S256 | PASS | Correct implementation (routes_app Finding 9) |
| State CSRF | PASS with gap | Correct validation but **no TTL** (routes_app Finding 1) |
| Redirect URI | GAP | Not validated against allowlist (routes_app Finding 3) |
| Token encryption at rest | PASS | AES-256-GCM with per-field salt/nonce (services Finding 11) |
| Secrets not in API | PASS | Boolean flags only (objs Finding 6) |
| PBKDF2 iterations | LOW | 1000 iterations vs OWASP 600K+ recommended (services Finding 1) |
| Token refresh concurrency | OK with leak | Per-config Mutex correct but HashMap grows unbounded (services Finding 2) |
| Session-only auth | PASS | All OAuth endpoints behind session middleware (routes_app Finding 12) |

### AS Metadata (`discover-as`) Resolution
- **Backend**: Endpoint exists and tested (backend-tests: AS discovery success + failure)
- **Frontend**: `useDiscoverAs` hook is exported but **never called** from any component
- **Recommendation**: Remove dead `useDiscoverAs` hook and related types from frontend (ui-impl Finding 1)

### Documentation Gaps
- `routes_app/CLAUDE.md`: Missing error enum documentation for `McpValidationError`
- `services/CLAUDE.md`: Already documents MCP auth patterns adequately
- `bodhi/src/CLAUDE.md`: Missing documentation for auto-DCR behavior difference (new vs view page)
- `tests-js/E2E.md`: Missing step naming convention documentation

---

## Critical Issues (Must Fix)

| # | Layer | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| C1 | backend-tests | `routes_app/src/routes_mcp/auth_configs.rs` | `oauth_login_handler` (L140), `oauth_token_exchange_handler` (L216) | OAuth login and token exchange handlers have ZERO test coverage | Add 6 tests: login success (verify URL params), login missing config, token exchange success (mock HTTP + session), state mismatch (CSRF), missing session data, HTTP failure | backend-tests Finding 1 |
| C2 | backend-tests | `services/src/mcp_service/service.rs` | `resolve_oauth_token` (L428-563) | Token refresh logic has ZERO test coverage; proactive 60s-before-expiry path untested | Add 4 service tests: not-expired returns token, expired+refresh succeeds, expired+no-refresh errors, expired+refresh-HTTP-failure errors | backend-tests Finding 2 |
| C3 | ui-tests | `mcp-servers/new/page.test.tsx` L249, `view/page.test.tsx` L504 | `await new Promise(resolve => setTimeout(resolve, 500))` | Inline setTimeout in tests violates project rule; hides actual issue; flaky on slow CI | Replace with `waitFor(() => expect(element).toHaveTextContent('Pre-Registered'))` | ui-tests Finding 1 |

---

## Important Issues (Should Fix)

| # | Layer | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| I1 | objs | `objs/src/mcp.rs` | `CreateMcpAuthConfigRequest::Oauth` L272 | `registration_type` is unvalidated free-form String; arbitrary values accepted | Create `RegistrationType` enum with `PreRegistered`/`DynamicRegistration` variants, `#[serde(rename_all = "kebab-case")]` | objs Finding 1 |
| I2 | objs | `objs/src/mcp.rs` | `CreateMcpAuthConfigRequest::Oauth` L265-266 | No URL validation on `authorization_endpoint` and `token_endpoint` | Add `url::Url::parse()` validation (like existing `validate_mcp_server_url`) | objs Finding 2 |
| I3 | services | `services/src/mcp_service/service.rs` | `delete` method L935-937 | MCP instance delete also deletes shared admin-managed auth_header configs; inconsistent with `update` which preserves them | Remove `McpAuthType::Header` arm from delete cleanup; auth headers deleted only via explicit `delete_auth_config` | services Finding 4 |
| I4 | services | `services/migrations/0012_mcp_oauth.up.sql` | `mcp_oauth_tokens` table L24-38 | No unique constraint on `(mcp_oauth_config_id, created_by)`; orphaned token rows accumulate on re-authorization | Delete existing tokens for same config+user before inserting new one in `store_oauth_token` | services Finding 5 |
| I5 | routes_app | `routes_app/src/routes_mcp/auth_configs.rs` | `oauth_login_handler` L158-166, `oauth_token_exchange_handler` L226-255 | OAuth CSRF state parameter has no TTL; state valid until session expires (hours/days) | Store `created_at` timestamp alongside state in session; validate age < 10min in token exchange | routes_app Finding 1 |
| I6 | routes_app | `routes_app/src/routes_mcp/auth_configs.rs` | L289 | Token exchange creates new `reqwest::Client` instead of using shared one from McpService | Move token exchange logic into McpService (consolidates all OAuth HTTP calls behind service abstraction) | routes_app Finding 2 |
| I7 | routes_app | `routes_app/src/routes_mcp/auth_configs.rs` | `oauth_login_handler` L177, `oauth_token_exchange_handler` L277 | `redirect_uri` accepted without validation or allowlisting | Validate redirect_uri is well-formed URL via `url::Url::parse()` at minimum | routes_app Finding 3 |
| I8 | backend-tests | `routes_app/src/routes_mcp/test_auth_configs.rs` | Only `test_create_auth_config_header_success` L50 | No test creates OAuth auth config via standalone `POST /mcps/auth-configs` | Add `test_create_auth_config_oauth_prereg_success` and `test_create_auth_config_oauth_dcr_success` | backend-tests Finding 5 |
| I9 | backend-tests | `services/src/db/test_mcp_repository.rs` | End of file L572 | Missing DB tests for `update_mcp_oauth_token` and `get_decrypted_oauth_bearer` | Add 2 tests: token update roundtrip, oauth bearer decrypt returns correct tuple | backend-tests Finding 6 |
| I10 | backend-tests | `routes_app/src/routes_mcp/test_oauth_utils.rs` | `test_delete_oauth_token_handler_success` L348 | Delete handler test doesn't verify deletion actually occurred (no mock expectation) | Use `MockDbService` with `.expect_delete_mcp_oauth_token()` or real DB integration test | backend-tests Finding 8 |
| I11 | ui-impl | `stores/mcpFormStore.ts` L31, `mcps/oauth/callback/page.tsx` L13 | `OAUTH_FORM_STORAGE_KEY` constant | Storage key `'mcp_oauth_form_state'` duplicated in 2 files; divergence breaks OAuth redirect silently | Export from `mcpFormStore.ts`, import in callback page | ui-impl Finding 2 |
| I12 | ui-impl | `stores/mcpFormStore.ts` | `restoreFromSession` L85-90 | `JSON.parse(saved)` has no try/catch; corrupt sessionStorage crashes MCP new/edit page | Wrap in try/catch, return null on failure, remove corrupt entry | ui-impl Finding 3 |
| I13 | ui-impl | `mcp-servers/view/page.tsx` | L267 `enableAutoDcr={true}` | View page passes `enableAutoDcr={true}`, contradicting spec (new=silent fallback, view=error+retry) | Change to `enableAutoDcr={false}`; verify error display path in AuthConfigForm works | ui-impl Finding 5 |
| I14 | ui-impl | `mcp-servers/new/page.tsx` L115-154, `view/page.tsx` L107-161 | `handleSubmit` / `handleCreateSubmit` | DCR submission logic (~30 lines) duplicated between new and view server pages | Extract shared `buildDcrAuthConfig` helper function | ui-impl Finding 9 |
| I15 | ui-tests | `mcp-servers/new/page.test.tsx` | L107-211 `.skip` test | Skipped test for "creates MCP server with OAuth DCR config" has broken handler call and inline timeout | Fix handler (use `http.post()` directly), remove timeout, investigate mutation chain | ui-tests Finding 4 |
| I16 | ui-tests | `mcp-servers/view/page.test.tsx` | After L266 | No test for creating OAuth auth config via view page inline form (only header tested) | Add test: select OAuth, fill fields, save, verify creation | ui-tests Finding 5 |
| I17 | ui-tests | `mcp-servers/view/page.test.tsx` | After L562 | No test verifies custom name preservation on type switch | Add test: set custom name, switch Header->OAuth, assert name preserved | ui-tests Finding 2 |
| I18 | e2e | `tests-js/pages/McpsPage.mjs` | L174-244 API helpers | `page.evaluate()` API calls don't check `resp.ok`; silent failures cause cryptic downstream errors | Add `if (!resp.ok) throw new Error(...)` in each fetch callback | e2e Finding 6 |
| I19 | e2e | `tests-js/specs/mcps/mcps-header-auth.spec.mjs` | All 3 tests | All header auth tests depend on external Tavily API; flaky in CI | Create mock header-auth MCP server (simple Express with bearer validation) | e2e Finding 7 |

---

## Nice-to-Have (Future)

| # | Layer | File | Location | Issue | Report |
|---|-------|------|----------|-------|--------|
| N1 | services | `services/src/db/encryption.rs` | `PBKDF2_ITERATIONS` L13 | 1000 iterations vs OWASP 600K+ recommended (master key is machine-generated, so practical risk low) | services Finding 1 |
| N2 | services | `services/src/mcp_service/service.rs` | `refresh_locks` L225, `get_refresh_lock` L253-267 | HashMap grows unboundedly; consider LRU cache or weak refs | services Finding 2 |
| N3 | services | `services/src/db/service_mcp.rs` | `get_decrypted_client_secret` L717-738 | Returns None for both "config not found" and "no secret"; ambiguous | services Finding 6 |
| N4 | services | `services/src/mcp_service/service.rs` | `resolve_oauth_token` L428-563 | No logging in token refresh flow | services Finding 7 |
| N5 | services | `services/src/mcp_service/service.rs` | `discover_oauth_metadata` L1308-1450 | No logging in discovery and DCR flows | services Finding 8 |
| N6 | services | `services/migrations/0012_mcp_oauth.up.sql` | L40-41 indexes | Missing `IF NOT EXISTS` on CREATE INDEX statements | services Finding 12 |
| N7 | services | `services/src/mcp_service/service.rs` | `delete_oauth_config` L1244-1248 | Cascade delete of tokens + config not wrapped in transaction | services Finding 13 |
| N8 | routes_app | `routes_app/src/routes_mcp/auth_configs.rs` | `delete_auth_config_handler` L111-118 | No ownership or role check on delete; any authenticated user can delete any config | routes_app Finding 4 |
| N9 | routes_app | `routes_app/src/routes_mcp/error.rs` | `McpValidationError` (9 lines) | Single-variant catch-all; all errors produce same code; frontend can't distinguish | routes_app Finding 5 |
| N10 | routes_app | `routes_app/src/routes_mcp/oauth_utils.rs` | Discovery handlers L38, L85, L134 | No URL format validation on discovery/DCR inputs (inconsistent with login handler) | routes_app Finding 7 |
| N11 | routes_app | `routes_app/src/routes_mcp/auth_configs.rs` | L257-261 | Token exchange accesses `db_service` directly instead of through `mcp_service` | routes_app Finding 8 |
| N12 | backend-tests | Multiple test files | Router setup functions | 5 duplicate `test_router_for_*` functions across test files | backend-tests Finding 3 |
| N13 | backend-tests | Multiple test files | Fixture timestamps | `Utc::now()` used directly in test fixtures instead of deterministic time | backend-tests Finding 4 |
| N14 | backend-tests | `routes_app/src/routes_mcp/auth_configs.rs` | L185-192 | No test for `resource` parameter in OAuth authorization URL | backend-tests Finding 10 |
| N15 | ui-impl | `hooks/useMcps.ts` | L355-370 `useDiscoverAs` | Dead code: hook exported but never called from any component | ui-impl Finding 1 |
| N16 | ui-impl | `mcp-servers/components/AuthConfigForm.tsx` | L11 | Unused import `useStandaloneDynamicRegister` | ui-impl Finding 4 |
| N17 | ui-impl | `mcp-servers/page.tsx` L39-44, `mcps/new/page.tsx` L143-152 | `authConfigTypeBadge` / `getAuthConfigTypeBadge` | 3 copies of auth type badge mapping logic; extract to `mcpUtils.ts` | ui-impl Findings 6,7 |
| N18 | ui-impl | `mcps/oauth/callback/page.tsx` | `onError` L87-89 | Session data not cleared on token exchange error; stale data may confuse retry | ui-impl Finding 11 |
| N19 | ui-tests | `mcp-servers/new/page.test.tsx` | L5 | Unused import `mockCreateMcpServerError` (doesn't exist in handlers) | ui-tests Finding 6 |
| N20 | e2e | `tests-js/specs/mcps/mcps-oauth-dcr.spec.mjs` | All 3 tests L42-232 | DCR setup (discover -> register -> create config) duplicated ~30 lines x3 | e2e Finding 5 |

---

## Retracted Findings

| # | Report | Original Priority | Reason |
|---|--------|-------------------|--------|
| R1 | services Finding 3 | Critical | **FALSE POSITIVE**: `resolve_oauth_token(user_id, auth_uuid)` is correct. `auth_uuid` stores the token ID (set by frontend from `OAuthTokenResponse.id`), not the config ID. The `get_mcp_oauth_token(user_id, auth_uuid)` query `WHERE id = ?` correctly matches the token primary key. Verified by tracing: callback page stores `response.data.id` -> `oauth_token_id` -> MCP create sends as `auth_uuid`. |

---

## Fix Order (Layered Development Methodology)

When applying fixes, follow this order per the project's upstream-to-downstream layered approach:

### Phase 1: objs crate (I1, I2)
```
cargo test -p objs
```

### Phase 2: services crate (I3, I4)
```
cargo test -p objs -p services
```

### Phase 3: routes_app crate (I5, I6, I7)
```
cargo test -p objs -p services -p routes_app
```

### Phase 4: Backend tests (C1, C2, I8, I9, I10)
```
make test.backend
```

### Phase 5: Regenerate TypeScript types
```
make build.ts-client
```
(Only needed if objs/routes_app type changes affect OpenAPI schema -- I1 will change types)

### Phase 6: Frontend implementation (I11, I12, I13, I14)
```
cd crates/bodhi && npm test
```

### Phase 7: Frontend tests (C3, I15, I16, I17)
```
cd crates/bodhi && npm test
```

### Phase 8: E2E tests (I18, I19)
```
make build.ui-rebuild && make test.napi
```

### Phase 9: Documentation updates
- Update `routes_app/CLAUDE.md` with error enum docs
- Update `bodhi/src/CLAUDE.md` with auto-DCR behavior docs
- Update `tests-js/E2E.md` with step naming convention

---

## Quick Reference: Issue Clusters

**Security fixes** (address together): I5 (state TTL), I7 (redirect_uri validation), N8 (delete ownership)

**Token exchange consolidation** (address together): I6 (move to McpService), N11 (db_service bypass) -- both resolved by moving token exchange into McpService

**Test infrastructure** (address together): N12 (router setup), N13 (Utc::now), backend-tests Finding 11 (shared factories) -- extract to `test_utils/mcp.rs`

**Frontend deduplication** (address together): I11 (storage key), I14 (DCR logic), N17 (badge functions), ui-impl Finding 8 (type aliases)

**Dead code cleanup** (address together): N15 (useDiscoverAs), N16 (unused import), N19 (non-existent import)
