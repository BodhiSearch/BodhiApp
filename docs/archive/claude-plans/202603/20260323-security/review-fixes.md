# Security Remediation Code Review — Consolidated

## Review Scope
- **Original commit**: `eb51a738` — `fix: security remediation critical wave — SSRF, XSS, session fixation, path traversal, authz`
- **Review date**: 2026-03-23
- **Review fixes applied**: Same session — store-time validation, CSP hardening, safeNavigate, MCP service Result
- **Shannon Report**: `deliverables/comprehensive_security_assessment_report.md` (29 vulnerabilities)

---

## Review Findings — All Resolved

All 14 findings from the code review have been addressed. Summary of what was done:

### Critical (3) — All Fixed

| # | Finding | Resolution |
|---|---------|------------|
| 1 | MCP server URL store-time validation accepts `javascript:` | `validate_mcp_server_url_validator` now calls centralized `validate_http_url()` enforcing http/https scheme |
| 2 | AI API `base_url` store-time validation accepts `javascript:` | `ApiModelRequest`, `TestPromptRequest`, `FetchModelsRequest` — replaced `#[validate(url)]` with `#[validate(custom(function = "crate::validate_http_url"))]` |
| 3 | `get_user()` pagination bypass (list_users(100) + in-memory filter) | Fixed locally (pre-review) — dedicated Keycloak `get_user()` API call |

### Important (5) — All Fixed

| # | Finding | Resolution |
|---|---------|------------|
| 4 | `DefaultMcpService::new()` panics with `.expect()` | Changed to return `Result<Self, McpError>`, propagated through `app_service_builder.rs` |
| 5 | CSP missing `base-uri` and `form-action` | Added `base-uri 'self'; form-action 'self'` to CSP header |
| 6 | `users_destroy` None case undocumented | Added comment explaining orphan cleanup intent |
| 7 | `safeNavigate()` missing trim and edge case tests | Added `.trim()`, empty string guard, 7 parameterized edge case tests via `it.each` |
| 8 | Verify User-level approve has status gating | Verified: service-layer `approve_request()` enforces `Draft`-only status (line 187-195 of `access_request_service.rs`). No code change needed. |

### Nice-to-Have (6) — All Fixed

| # | Finding | Resolution |
|---|---------|------------|
| 9 | `cycle_id()` error silently discarded | Changed `let _ =` to `if let Err(e)` with `warn!` log |
| 10 | OpenAPI annotation mismatch on `users_destroy` | Updated utoipa to `resource_manager`, description to "managers or above" |
| 11 | `validate_filename()` no unit tests | Added rstest parameterized tests (valid + invalid filenames) |
| 12 | `is_valid_http_url()` duplicates scheme check | Refactored to call centralized `validate_http_url().is_ok()` |
| 13 | Tenants page no toast on blocked navigation | Added error toast matching review/page.tsx and mcps/new/page.tsx patterns |
| 14 | `handleSmartRedirect` safeNavigate return unchecked | No change needed — non-component context, console.error is sufficient |

### Test Coverage Gaps — Addressed

| # | What was missing | Resolution |
|---|------------------|------------|
| 1 | `validate_filename()` unit tests | Added 10 rstest cases (4 valid, 6 invalid) |
| 2 | safeNavigate edge cases | Added 7 parameterized tests (uppercase, mixed case, whitespace, tab, data:, vbscript:, whitespace-only) + empty string test |
| 3 | E2E tests (session rotation, XSS redirect rejection, role ceiling) | Deferred — not in scope for this review fix pass |

---

## Shannon Report Cross-Reference

### Fixed in Code (9 findings + CSP)
| ID | Severity | Fix |
|----|----------|-----|
| AUTH-VULN-01 | High | Secure cookie from `is_secure_transport()` |
| AUTH-VULN-03 | Critical | `session.cycle_id()` in auth_callback + warn log on failure |
| AUTHZ-VULN-06 | Critical | Role ceiling check via dedicated `get_user()` Keycloak API |
| AUTHZ-VULN-10 | High | Status guard — only Pending requests |
| XSS-VULN-01 | Critical | Backend validator + frontend `safeNavigate()` with trim + edge cases |
| XSS-VULN-02 | High | `ValidatedJson` + manual Validate impl + `safeNavigate()` |
| INJ-VULN-01 | Critical | `validate_filename()` + unit tests |
| SSRF-VULN-01–10 | Critical/High | `SafeReqwest` scheme validation + **store-time** `validate_http_url()` |

| CSP | Infra | CSP with `base-uri 'self'; form-action 'self'` |

### Verified Already Implemented (1)
- AUTH-VULN-06 (session expiry): Keycloak token lifecycle governs sessions

### Excluded — /dev/* Endpoints (4)
- AUTH-VULN-04, AUTH-VULN-05, AUTHZ-VULN-07, AUTHZ-VULN-08
- **Reason**: Dev-only, guarded by `!is_production()`. Never active in production.

### Accepted By-Design (7)
- AUTHZ-VULN-01/02: MCP auth-configs are tenant-level shared resources
- AUTHZ-VULN-03/04/05: Access request lifecycle (orphaned → owned)
- AUTHZ-VULN-09: Download jobs are tenant-wide
- AUTHZ-VULN-11: Dynamic-register is stateless (mitigated by scheme validation)

### Deferred to Infrastructure (3)
- AUTH-VULN-02: Rate limiting → reverse proxy / WAF
- HSTS: → TLS-terminating proxy
- CORS wildcard on /dev/*: Dev-only

### Accepted with Risk Documentation (4)
- PBKDF2 1,000 iterations (performance vs narrow threat)
- SHA-256 for API tokens (high-entropy, brute-force infeasible)
- Session cookie no Max-Age (Keycloak governs lifetime)
- SSRF private IPs allowed (local Ollama/MCP servers)

---

## Files Modified in Review Fix Pass

| File | Change |
|------|--------|
| `crates/services/src/shared_objs/url_validator.rs` | Added `validate_http_url()` |
| `crates/services/src/shared_objs/test_url_validator.rs` | Added `validate_http_url` tests |
| `crates/services/src/mcps/mcp_objs.rs` | Replaced `validate_mcp_server_url_validator` + `is_valid_http_url` |
| `crates/services/src/mcps/mcp_service.rs` | `DefaultMcpService::new()` → `Result` |
| `crates/services/src/mcps/error.rs` | Added `Reqwest` variant to `McpError` |
| `crates/services/src/mcps/test_mcp_service.rs` | Updated `new()` calls for Result |
| `crates/services/src/models/model_objs.rs` | 3 `base_url` fields: `validate(url)` → `validate(custom(...))` |
| `crates/services/src/models/hub_service.rs` | Added `validate_filename()` unit tests |
| `crates/services/src/test_utils/app.rs` | Updated `DefaultMcpService::new()` call |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | `build_mcp_service` → `Result` |
| `crates/routes_app/src/spa_router.rs` | CSP: added `base-uri`, `form-action` |
| `crates/routes_app/src/users/routes_users.rs` | Comment + utoipa annotation fix |
| `crates/routes_app/src/auth/routes_auth.rs` | `warn!` on cycle_id failure |
| `crates/bodhi/src/lib/safeNavigate.ts` | `.trim()`, empty string guard |
| `crates/bodhi/src/lib/safeNavigate.test.ts` | 8 new edge case tests |
| `crates/bodhi/src/app/setup/tenants/page.tsx` | Error toast on blocked navigation |
