# Security Remediation Plan — BodhiApp

**Status: COMPLETED**
**Commit:** `eb51a738` (squashed — critical wave + high wave + code review refactors)

## Context

A thorough whitebox security assessment by Shannon (2026-03-22) identified 29 vulnerabilities across authentication, authorization, XSS, SSRF, and injection categories. After excluding `/dev/*` endpoints (not active in production) and documenting by-design decisions, this plan addressed the remaining actionable vulnerabilities.

---

## Completed: Documentation Deliverables

- **`deliverables/app-security-notes.md`** — By-design decisions for future security re-scans (6 items)
- **`ai-docs/func-specs/security/security.md`** — Comprehensive known open security issues with risk acceptance reasoning (11 items including PBKDF2, rate limiting, HSTS, CORS, auth-configs, access requests, download jobs, etc.)

---

## Completed: Critical Wave

### SafeReqwest URL Validation (SSRF-VULN-01–10)

**Files:** `crates/services/src/shared_objs/url_validator.rs`, `safe_reqwest.rs`, `test_url_validator.rs`

- Created `validate_outbound_url(url, allow_private_ips)` with configurable private IP enforcement
- Created `SafeReqwest` wrapper around `reqwest::Client` with `allow_private_ips()` builder flag
- **Scheme validation** (http/https only) always enforced — blocks `javascript:`, `file:`, `data:`, `gopher:` etc.
- **Private IP blocklist** configurable per-client — both AI API and MCP services configured with `allow_private_ips()` to support local Ollama / local MCP servers
- `SafeReqwest::build()` returns `Result<SafeReqwest, ReqwestError>` (no panics)
- `DefaultAiApiService::new()` returns `Result<Self>` (propagates build errors)
- Removed `skip_validation()` and `new_for_test()` — `allow_private_ips()` suffices for test mock servers
- **52 unit tests** covering scheme rejection, private IP blocking, IPv6, allow_private_ips mode

**Design decision (post-plan revision):** Private IP blocklist is NOT enforced for AI API or MCP services because users legitimately connect to local Ollama instances and local MCP servers. Scheme validation (http/https only) remains enforced everywhere — this is the primary XSS defense.

### Path Traversal Fix (INJ-VULN-01)

**File:** `crates/services/src/models/hub_service.rs`

- Added `validate_filename()` rejecting `..`, `/`, `\` in filenames
- Added `InvalidFilename` error variant to `HubServiceError`
- Validation called in both `local_file_exists()` and `find_local_file()` before `PathBuf::join(filename)`

### Session Fixation Fix (AUTH-VULN-03)

**File:** `crates/routes_app/src/auth/routes_auth.rs`

- tower-sessions 0.14.0 has `session.cycle_id()` (confirmed in tower-sessions-core source)
- Added `session.cycle_id().await` before storing user data in `auth_callback()`

### Role Ceiling on User Delete (AUTHZ-VULN-06)

**Files:** `crates/routes_app/src/users/routes_users.rs`, `crates/services/src/users/auth_scoped.rs`

- Added `AuthScopedUserService::get_user()` method (wraps `list_users` + filter by ID)
- `users_destroy` handler fetches target user's role via `get_user()`, compares with caller's role using `has_access_to()`
- Managers can only delete users at or below their own role level

### XSS Backend Validation (XSS-VULN-01)

**File:** `crates/services/src/app_access_requests/access_request_objs.rs`

- Added `#[validate(custom(function = "validate_redirect_url_scheme"))]` on `CreateAccessRequest.redirect_url`
- Validator rejects non-http/https schemes — handled automatically by `ValidatedJson` extractor

### Frontend safeNavigate (XSS-VULN-01, XSS-VULN-02)

**Files:** `crates/bodhi/src/lib/safeNavigate.ts`, 4 page components, `utils.ts`

- `safeNavigate(url): boolean` — validates scheme, returns false if blocked
- Callers (review/page.tsx, mcps/new/page.tsx) show destructive toast when navigation blocked
- `handleSmartRedirect` fallback uses `safeNavigate()` (console.error for non-component context)
- **8 unit tests** covering http/https allow, javascript:/data:/vbscript: block, return values

---

## Completed: High Wave

### Secure Cookie Flag (AUTH-VULN-01)

**Files:** `crates/services/src/auth/session_service.rs`, `crates/services/src/settings/setting_service.rs`, `crates/routes_app/src/routes.rs`

- `session_layer()` signature changed to `session_layer(&self, secure: bool)`
- Added `SettingService::is_secure_transport()` method (returns `public_scheme() == "https"`)
- `routes.rs` resolves `is_secure_transport()` and passes to `session_layer()`
- Cookie `Secure=true` when `BODHI_PUBLIC_SCHEME=https` (production behind TLS proxy)

### Session Expiry via Keycloak (AUTH-VULN-06)

**Status:** Already implemented — verified, no code change needed.

The auth middleware (`auth_middleware.rs`) already clears session data on `RefreshTokenNotFound`, `Token(_)`, `AuthService(_)`, and `InvalidToken(_)` errors. When Keycloak refresh token expires, the session is invalidated.

### ValidatedJson for Auth-Config URLs (XSS-VULN-02)

**Files:** `crates/services/src/mcps/mcp_objs.rs`, `crates/routes_app/src/mcps/mcps_api_schemas.rs`, `crates/routes_app/src/mcps/routes_mcps_auth.rs`

- Manual `impl validator::Validate for CreateMcpAuthConfigRequest` — validates OAuth `authorization_endpoint`, `token_endpoint`, `registration_endpoint` are http/https only
- `CreateAuthConfig` derives `Validate` with `#[validate(nested)]` on flattened config field
- Handler changed from `Json(body)` to `ValidatedJson(body)` — automatic validation

### Status Guard for Manager Approve (AUTHZ-VULN-10)

**File:** `crates/routes_app/src/users/routes_users_access_request.rs`, `crates/routes_app/src/users/error.rs`

- Added `AlreadyProcessed` error variant to `UsersRouteError`
- Handler checks `access_request.status != Pending` before allowing approval

### CSP Header on UI Routes

**File:** `crates/routes_app/src/spa_router.rs`

- CSP header added to HTML responses only in `build_response()`
- Policy: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; frame-ancestors 'none'`

---

## Summary — Final Vulnerability Disposition

| Vuln ID | Severity | Disposition | Implementation |
|---------|----------|-------------|----------------|
| AUTH-VULN-01 | High | **Fixed** | Secure cookie from `is_secure_transport()` |
| AUTH-VULN-02 | High | **Documented** | Infra responsibility (rate limiting) |
| AUTH-VULN-03 | Critical | **Fixed** | `session.cycle_id()` in auth_callback |
| AUTH-VULN-04 | Critical | **Excluded** | /dev/* not in production |
| AUTH-VULN-05 | Critical | **Documented** | Existing audience check sufficient |
| AUTH-VULN-06 | High | **Verified** | Already implemented (Keycloak token expiry) |
| AUTHZ-VULN-01,02 | High | **Documented** | By-design: tenant-level auth-configs |
| AUTHZ-VULN-03,04,05 | High | **Documented** | By-design: access request lifecycle |
| AUTHZ-VULN-06 | Critical | **Fixed** | Role ceiling check via `get_user()` |
| AUTHZ-VULN-07,08 | Critical | **Excluded** | /dev/* not in production |
| AUTHZ-VULN-09 | Medium | **Documented** | By-design: tenant-wide downloads |
| AUTHZ-VULN-10 | High | **Fixed** | Status guard (pending only) |
| AUTHZ-VULN-11 | High | **Documented** | Scheme validation sufficient |
| XSS-VULN-01 | Critical | **Fixed** | Validator macro on `redirect_url` + `safeNavigate()` |
| XSS-VULN-02 | High | **Fixed** | `ValidatedJson` with manual `Validate` impl + `safeNavigate()` |
| SSRF-VULN-01–10 | Critical/High | **Mitigated** | `SafeReqwest` with scheme validation (http/https only). Private IPs allowed — local Ollama/MCP servers are a valid use case. |
| INJ-VULN-01 | Critical | **Fixed** | Filename traversal char rejection |
| CSP | Infra | **Fixed** | Basic CSP on HTML UI responses |
| PBKDF2 | Infra | **Documented** | Accepted risk (1000 iters, threat model requires DB + master key) |

---

## Pending Items

### Not Yet Implemented

1. **Filename validation unit tests** — `validate_filename()` was added but dedicated unit tests in the hub_service test file were not created. The function is exercised through integration tests but lacks direct unit test coverage for edge cases.

2. **E2E test extensions** — The plan called for extending existing Playwright E2E tests:
   - Session cookie rotation verification after OAuth callback
   - `javascript:` redirect_url rejection in access request flow
   - Manager cannot delete Admin user
   These E2E extensions were not implemented (only backend unit tests and frontend Vitest tests were added).

### Post-Plan Design Revisions

These decisions were made during implementation and differ from the original plan:

1. **SSRF private IP blocklist relaxed** — Original plan blocked all private IPs/localhost. Revised to `allow_private_ips()` for AI API and MCP services because users need to connect to local Ollama instances and local MCP servers. Scheme validation (http/https) is still enforced everywhere.

2. **SafeReqwest location** — Original plan discussed putting URL validation in `errmeta` crate. Implemented in `services` crate instead (cleaner, no unnecessary dependency bloat on error-only crate).

3. **mcp_client crate unchanged** — Original plan called for URL validation in `mcp_client`. Instead, validation is done in `mcp_service.rs` before calling `mcp_client` methods (services depends on mcp_client, so validation in services covers all call sites).

4. **`skip_validation()` removed** — Originally added for test mock servers. Replaced by `allow_private_ips()` which lets tests connect to localhost while still enforcing scheme validation.

5. **`DefaultAiApiService::new()` returns `Result`** — Changed from `Default` impl with `.expect()` to explicit `new() -> Result<Self>` for proper error propagation.
