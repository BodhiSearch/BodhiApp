# Plan: Security Review Fixes — Store-Time Validation, CSP, SafeNavigate, MCP Result

## Context

Security remediation commit `eb51a738` addressed 29 vulnerabilities from Shannon's whitebox assessment. A thorough code review (`ai-docs/claude-plans/202603/20260323-security/index.md`) identified 14 remaining issues: 3 Critical, 5 Important, 6 Nice-to-have. This plan implements the Critical and Important fixes.

The #1 gap: `javascript:` and other dangerous URI schemes can be **stored** in the database via MCP server URLs and AI API base_urls. They're caught at request-time by `SafeReqwest`, but store-time rejection is missing. The user confirmed this is high-critical priority.

---

## Phase 1: Services Crate — Store-Time URL Validation

### 1a. Add shared validator functions in `url_validator.rs`

**File**: `crates/services/src/shared_objs/url_validator.rs`

Add two reusable validator-crate-compatible functions at the end (before the `#[cfg(test)]` block):

```rust
/// Validator function for URL fields that must use http/https scheme.
/// Allows private IPs (for local Ollama, MCP servers).
/// Use with `#[validate(custom(function = "validate_http_url"))]`
pub fn validate_http_url(url: &str) -> Result<(), validator::ValidationError> {
  validate_outbound_url(url, true)
    .map(|_| ())
    .map_err(|_| validator::ValidationError::new("invalid_url_scheme"))
}
```

This is a thin wrapper around `validate_outbound_url` that returns `validator::ValidationError` for use with the `#[validate(custom(...))]` derive macro. Uses `allow_private_ips=true` because both MCP servers and AI APIs legitimately target localhost.

### 1b. Replace `validate_mcp_server_url_validator` in `mcp_objs.rs`

**File**: `crates/services/src/mcps/mcp_objs.rs`

Replace the body of `validate_mcp_server_url_validator` (line 629-633) to call `validate_http_url`:

```rust
fn validate_mcp_server_url_validator(url: &str) -> Result<(), validator::ValidationError> {
  crate::validate_http_url(url)
}
```

This enforces http/https scheme at store-time for MCP server URLs.

### 1c. Add scheme validation to AI API base_url fields

**File**: `crates/services/src/models/model_objs.rs`

Three structs need updating:

1. **`ApiModelRequest.base_url`** (line 1104): Change from `#[validate(url)]` to:
   ```rust
   #[validate(custom(function = "crate::validate_http_url"))]
   ```

2. **`TestPromptRequest.base_url`** (around line 1171): Same change.

3. **`FetchModelsRequest.base_url`** (around line 1230): Same change.

All three currently accept `javascript:` URLs via syntactic-only `#[validate(url)]`.

### 1d. Refactor `is_valid_http_url` in `mcp_objs.rs` (optional DRY cleanup)

**File**: `crates/services/src/mcps/mcp_objs.rs`

The `is_valid_http_url()` function (line 415-419) is used in the manual `Validate` impl for `CreateMcpAuthConfigRequest`. Replace its body to call the centralized function:

```rust
fn is_valid_http_url(url: &str) -> bool {
  crate::validate_http_url(url).is_ok()
}
```

### 1e. `DefaultMcpService::new()` — return Result instead of panicking

**File**: `crates/services/src/mcps/mcp_service.rs` (around line 246-262)

Change `new()` signature to return `Result<Self, McpError>`:
```rust
pub fn new(
  db_service: Arc<dyn DbService>,
  mcp_client: Arc<dyn McpClient>,
  time_service: Arc<dyn TimeService>,
) -> Result<Self, McpError> {
  let http_client = SafeReqwest::builder()
    .allow_private_ips()
    .build()
    .map_err(|e| McpError::InternalError(e.to_string()))?;
  Ok(Self { db_service, mcp_client, time_service, http_client, refresh_locks: ... })
}
```

Need to add an appropriate error variant to `McpError` if `InternalError` doesn't exist (check first — may need `McpError::HttpClientBuild(String)` or similar).

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs` (around line 362-369)

Update `build_mcp_service` to return `Result` and propagate:
```rust
fn build_mcp_service(...) -> std::result::Result<Arc<dyn McpService>, BootstrapError> {
  let mcp_client = Arc::new(mcp_client::DefaultMcpClient::new());
  Ok(Arc::new(DefaultMcpService::new(db_service, mcp_client, time_service)
    .map_err(|e| BootstrapError::UnexpectedError(services::AppError::code(&e), e.to_string()))?))
}
```

Update the call site in `build()` (around line 163) to use `?`.

### 1f. Add tests for store-time validation

**File**: `crates/services/src/shared_objs/test_url_validator.rs`

Add test for `validate_http_url`:
```rust
#[rstest]
#[case("http://example.com")]
#[case("https://example.com")]
#[case("http://localhost:11434")]
fn test_validate_http_url_accepts_valid(#[case] url: &str) {
  assert!(validate_http_url(url).is_ok());
}

#[rstest]
#[case("javascript:alert(1)")]
#[case("file:///etc/passwd")]
#[case("data:text/html,<script>")]
fn test_validate_http_url_rejects_bad_schemes(#[case] url: &str) {
  assert!(validate_http_url(url).is_err());
}
```

**Verify**: `cargo test -p services`

---

## Phase 2: Routes App — CSP, Comments, Observability

### 2a. CSP: Add `base-uri` and `form-action` directives

**File**: `crates/routes_app/src/spa_router.rs` (line 90)

Change the CSP string to:
```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'
```

### 2b. Add comment for users_destroy None case

**File**: `crates/routes_app/src/users/routes_users.rs` (line 154)

After `if let Some(target_user) = target_user {`, add a comment before the closing:
```rust
  // target_user is None: user not found in Keycloak — proceed with deletion
  // as orphan cleanup (by-design: stale local records should be removable)
```

### 2c. Update OpenAPI annotation on users_destroy

**File**: `crates/routes_app/src/users/routes_users.rs` (line 140)

Change:
```rust
security(("session_auth" = ["resource_admin"]))
```
To:
```rust
security(("session_auth" = ["resource_manager"]))
```

And update the description from "Only admins can remove users" to "Only managers or above can remove users."

### 2d. Add warn! log for cycle_id failure

**File**: `crates/routes_app/src/auth/routes_auth.rs` (line 288-289)

Change:
```rust
let _ = session.cycle_id().await;
```
To:
```rust
if let Err(e) = session.cycle_id().await {
  warn!("Failed to rotate session ID after OAuth callback: {}", e);
}
```

### 2e. Add `validate_filename()` unit tests

**File**: `crates/services/src/models/hub_service.rs`

Add a `#[cfg(test)]` module with rstest cases:
```rust
#[cfg(test)]
mod test_validate_filename {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case("model.gguf")]
  #[case("my-model-v2.Q4_K_M.gguf")]
  #[case("file.bin")]
  fn test_allows_valid_filenames(#[case] name: &str) {
    assert!(DefaultHubService::validate_filename(name).is_ok());
  }

  #[rstest]
  #[case("../etc/passwd")]
  #[case("..")]
  #[case("foo/../bar")]
  #[case("path/file")]
  #[case("path\\file")]
  fn test_rejects_invalid_filenames(#[case] name: &str) {
    assert!(DefaultHubService::validate_filename(name).is_err());
  }
}
```

Note: `validate_filename` is a private method — may need `pub(crate)` or test within same module scope.

**Verify**: `cargo test -p routes_app && cargo test -p services`

---

## Phase 3: Frontend — safeNavigate Hardening

### 3a. Add `.trim()` normalization to safeNavigate

**File**: `crates/bodhi/src/lib/safeNavigate.ts`

Change line 12 from:
```typescript
const parsed = new URL(url, window.location.origin);
```
To:
```typescript
const trimmed = url.trim();
if (!trimmed) {
  console.error('Blocked navigation to empty URL');
  return false;
}
const parsed = new URL(trimmed, window.location.origin);
```

And change line 17 from `window.location.href = url;` to `window.location.href = trimmed;`.

### 3b. Add edge case tests (parameterized)

**File**: `crates/bodhi/src/lib/safeNavigate.test.ts`

Use `it.each` for parameterized testing:
```typescript
it.each([
  ['JAVASCRIPT:alert(1)', 'uppercase scheme'],
  ['jAvAsCrIpT:alert(1)', 'mixed case scheme'],
  ['  javascript:alert(1)', 'leading whitespace'],
  ['\tjavascript:alert(1)', 'tab prefix'],
  ['data:text/html,<script>alert(1)</script>', 'data: URI'],
  ['vbscript:MsgBox(1)', 'vbscript: URI'],
  ['', 'empty string'],
  ['   ', 'whitespace only'],
])('blocks dangerous URL: %s (%s)', (url) => {
  expect(safeNavigate(url)).toBe(false);
});
```

### 3c. Add error toast in tenants page

**File**: `crates/bodhi/src/app/setup/tenants/page.tsx`

Change:
```typescript
safeNavigate(location);
```
To:
```typescript
if (!safeNavigate(location)) {
  toast({
    title: 'Invalid redirect URL',
    description: `URL must use http:// or https:// scheme`,
    variant: 'destructive',
    duration: 5000,
  });
}
```

Import `toast` from `@/hooks/use-toast` if not already imported.

**Verify**: `cd crates/bodhi && npm test`

---

## Phase 4: Full Backend Validation

```bash
make test.backend
```

---

## Phase 5: Regenerate TypeScript Client (if API changes)

The store-time validation changes modify validation behavior but not API schemas, so `make build.ts-client` is likely unnecessary. But if `cargo run --package xtask openapi` shows changes (due to the utoipa annotation fix), then regenerate:

```bash
cargo run --package xtask openapi && cd ts-client && npm run generate
```

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/services/src/shared_objs/url_validator.rs` | Add `validate_http_url()` shared validator |
| `crates/services/src/shared_objs/test_url_validator.rs` | Add tests for `validate_http_url` |
| `crates/services/src/mcps/mcp_objs.rs` | Replace `validate_mcp_server_url_validator` body, refactor `is_valid_http_url` |
| `crates/services/src/mcps/mcp_service.rs` | `DefaultMcpService::new()` → `Result` |
| `crates/services/src/mcps/error.rs` | Possibly add error variant for HTTP client build |
| `crates/services/src/models/model_objs.rs` | Replace `#[validate(url)]` with custom validator on 3 structs |
| `crates/services/src/models/hub_service.rs` | Add `validate_filename()` unit tests |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | `build_mcp_service` → `Result` |
| `crates/routes_app/src/spa_router.rs` | CSP: add `base-uri`, `form-action` |
| `crates/routes_app/src/users/routes_users.rs` | Comment on None case, fix utoipa annotation |
| `crates/routes_app/src/auth/routes_auth.rs` | `warn!` on cycle_id failure |
| `crates/bodhi/src/lib/safeNavigate.ts` | Add `.trim()`, empty string guard |
| `crates/bodhi/src/lib/safeNavigate.test.ts` | Add edge case tests |
| `crates/bodhi/src/app/setup/tenants/page.tsx` | Add error toast on blocked navigation |

## Verification

1. `cargo test -p services` — validates store-time URL validation, filename tests
2. `cargo test -p routes_app` — validates CSP, users_destroy, cycle_id changes
3. `make test.backend` — full backend validation
4. `cd crates/bodhi && npm test` — frontend safeNavigate tests
5. OpenAPI check: `cargo run --package xtask openapi` — verify utoipa annotation change propagates
