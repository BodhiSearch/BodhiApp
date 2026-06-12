# auth_middleware Crate — Code Review
## Scope: HEAD commit (scope removal / access request flow changes)
## Date: 2026-02-27
## Reviewer: Claude (claude-sonnet-4-6)

---

## Summary

This review covers the changes to the `auth_middleware` crate that remove the `ResourceScope` intermediate type, move role derivation from JWT scope claims into DB-sourced `approved_role`, and introduce `CachedExchangeResult` enrichment with `role` and `access_request_id`. The changes affect the token validation pipeline, caching layer, and authorization middleware.

Overall assessment: **APPROVED with minor findings**. The security-critical path (role derived from DB, not JWT claims) is implemented correctly. Several low-priority code quality issues exist but none are blocking.

---

## Security Checklist Results

| Check | Result | Notes |
|---|---|---|
| Role derived ONLY from `approved_role`, never from JWT scope claims | PASS | Lines 308-315 of service.rs: `validated_record.approved_role` only |
| `scope_user_*` NOT forwarded in exchange scopes | PASS | Lines 185-195: filter only passes `scope_access_request:*` |
| Cache construction uses DB-sourced role | PASS | `CachedExchangeResult.role` set from `validated_record.approved_role` |
| `ExternalApp.role` is `None` when no `validated_record` | PASS | Lines 308-315: else branch returns `None` |
| Constant-time comparison for API token hash | PASS | Lines 89-92: `constant_time_eq::constant_time_eq` used |
| Access request pre-validation: status=approved check | PASS | Lines 211-221 |
| Access request pre-validation: azp match | PASS | Lines 223-232 |
| Access request pre-validation: user_id match | PASS | Lines 235-253 |
| Post-exchange: `access_request_id` claim consistency check | PASS | Lines 272-306 |

---

## Findings

### P1 — Critical / Security

*No P1 findings.*

---

### P2 — High / Correctness

#### P2-1: Redundant `access_request_scopes` variable is dead code
**File:** `crates/auth_middleware/src/token_service/service.rs`, lines 185–195

```rust
let mut scopes: Vec<&str> = claims
    .scope
    .split_whitespace()
    .filter(|s| s.starts_with("scope_access_request:"))
    .collect();

let access_request_scopes: Vec<&str> = scopes
    .iter()
    .filter(|s| s.starts_with("scope_access_request:"))
    .copied()
    .collect();
```

`scopes` is already filtered to only contain `scope_access_request:*` entries (the `.filter()` on line 188). The second variable `access_request_scopes` then re-filters the same collection with the identical predicate — producing an identical `Vec<&str>`. Only `access_request_scopes` is used after this point (in the `validated_record` block and later in the `tracing::error!` call on line 292). `scopes` is reused on line 259 as the mutable variable that `openid/email/profile/roles` are added to for the token exchange call.

The current code works correctly because both variables contain the same items, but it is confusing and may mislead future maintainers into thinking they differ. The `access_request_scopes` variable should be removed or `scopes` should be renamed and defined after the initial filter to avoid the apparent redundancy.

**Impact:** No functional issue, but the misleading duplication could cause a maintainer to accidentally break the scope forwarding logic if they mistakenly merge them, or fail to understand which variable drives the exchange scope list.

**Suggested fix:**
```rust
// Build the scope_access_request:* list for validation
let access_request_scopes: Vec<&str> = claims
    .scope
    .split_whitespace()
    .filter(|s| s.starts_with("scope_access_request:"))
    .collect();

// Start the exchange scope list from access_request_scopes, then add standard scopes
let mut exchange_scopes = access_request_scopes.clone();
exchange_scopes.extend(["openid", "email", "profile", "roles"]);
```

---

### P3 — Medium / Code Quality

#### P3-1: `scopes` declared `mut` but only one mutation occurs in a non-obvious location
**File:** `crates/auth_middleware/src/token_service/service.rs`, lines 185, 259

The variable `scopes` is declared `mut` on line 185 but only mutated on line 259 with `.extend(...)`. The long block of access-request validation code between those two lines (74 lines) makes it non-obvious that `scopes` is the exchange-scope accumulator. The variable name `scopes` is also ambiguous — it is the exchange scopes list, not the full scope list from the token.

**Suggested fix:** Rename to `exchange_scopes` and initialize after the validation block, as shown in the P2-1 fix above.

---

#### P3-2: `tracing::error!` for missing `access_request_id` claim should use `warn!`
**File:** `crates/auth_middleware/src/token_service/service.rs`, lines 291-295

```rust
tracing::error!(
    scope = %access_request_scopes[0],
    record_id = %validated_record.id,
    "KC did not return access_request_id claim despite valid scope"
);
```

This is a Keycloak configuration issue (KC returning a token without `access_request_id`), not an internal application error. Using `error!` will trigger alert rules in most monitoring systems. The condition is handled correctly (returns `Err`), so `warn!` is more appropriate here. The adjacent `tracing::warn!` on lines 275-279 (for the mismatch case) correctly uses `warn!`.

---

#### P3-3: `access_request_scopes[0]` unchecked index in error branch
**File:** `crates/auth_middleware/src/token_service/service.rs`, line 292

```rust
"KC did not return access_request_id claim despite valid scope"
// ...
scope = %access_request_scopes[0],
```

This is inside the `if let Some(ref validated_record) = validated_record` block, which is only entered when `access_request_scopes.first()` returned `Some`. So `access_request_scopes` is guaranteed non-empty here. However, the direct index `[0]` is less idiomatic than `.first().unwrap_or(...)` or using the already-bound `access_request_scope` variable from the outer `if let Some(&access_request_scope)`. The `access_request_scope` binding is still in scope here and could be used directly.

**Suggested fix:** Use the in-scope `access_request_scope` binding instead of re-indexing:
```rust
scope = %access_request_scope,
```

---

#### P3-4: Cache key uses a 12-hex-char digest (48-bit) — acceptable but worth documenting
**File:** `crates/auth_middleware/src/token_service/service.rs`, lines 112-114

```rust
let token_digest = format!("{:x}", hasher.finalize())[0..12].to_string();
```

The full SHA-256 is computed but only 12 hex characters (6 bytes / 48 bits) are used as the cache key. This is a cache key, not a security identifier, so collision resistance is not a security requirement here — a cache miss causes a re-exchange, not a security bypass. However, this decision is not documented. A comment noting "12 chars = 48 bits of SHA-256, sufficient for cache key collision avoidance (cache misses are safe)" would clarify that this truncation is intentional.

---

### P4 — Low / Style / Test Coverage

#### P4-1: No test for `ExternalApp` cache hit returning correct `role` from cache
**File:** `crates/auth_middleware/src/token_service/tests.rs`

`test_validate_bearer_token_with_access_request_scope_success` verifies the non-cached path. There is no test that:
1. Calls `validate_bearer_token` a first time (populates cache).
2. Calls `validate_bearer_token` a second time with the same token.
3. Asserts that the second call returns the same `AuthContext::ExternalApp { role: Some(...) }` from cache without invoking `exchange_app_token` again.

The JTI-forgery test (`test_external_client_token_cache_security_prevents_jti_forgery`) tests cache isolation but uses a no-access-request token, so `role` is `None`. There is no cache-hit test with `role: Some(...)` to confirm the round-trip through `CachedExchangeResult.role` deserialization produces the correct `UserScope`.

**Impact:** The cache serialization/deserialization of `role: Option<String>` → `parse::<UserScope>()` is not exercised by any unit test.

---

#### P4-2: `AuthServerTestClient` scope parameter is always forwarded as-is
**File:** `crates/auth_middleware/src/test_utils/auth_server_test_client.rs`, lines 60-69

The `scope_string` is computed via `scopes.join(" ")` but the `scope` form parameter is only added when `!scopes.is_empty()`. This means calling with an empty `scopes` slice sends no `scope` parameter, which may differ from calling with `&["openid"]`. This is a minor inconsistency in the test helper, not a production issue.

---

#### P4-3: Manager/Admin test cases removed from `api_auth_middleware` tests
**File:** `crates/auth_middleware/src/api_auth_middleware.rs` (inline tests)

Per the commit description, Manager and Admin test cases were removed from the `api_auth_middleware` tests. The existing tests cover `User` and `PowerUser` for session, token, and user-scope variants. Adding cases for `Manager` and `Admin` roles (both session role and `ResourceRole` checks) would complete coverage of the full `ResourceRole` hierarchy. This is a coverage gap, not a regression.

---

## Architecture Observations (Non-blocking)

### Clean elimination of `ResourceScope`
The removal of `ResourceScope` as an intermediate type simplifies the token validation pipeline. Previously `validate_bearer_token` returned a tuple containing `ResourceScope` that had to be converted into `AuthContext` by a separate `build_auth_context_from_bearer()` helper. The new design returns `AuthContext` directly, which is cleaner and reduces the opportunity for errors at the conversion boundary.

### `CachedExchangeResult` correctly mirrors `AuthContext::ExternalApp` role fields
The cache struct mirrors exactly the two fields needed to reconstruct `AuthContext::ExternalApp` from cache: `role: Option<String>` (stored as string, parsed to `UserScope` on read) and `access_request_id: Option<String>`. The parse-on-read pattern is correct: a cache corruption that produces an unparseable role string results in `role: None`, which is treated as unauthenticated — a safe degradation.

### Access request validation is defense-in-depth across two layers
1. `token_service/service.rs`: pre-exchange validation (status, azp, user_id, access_request_id claim).
2. `access_request_auth_middleware`: per-request entity validation (toolset/MCP instance in approved list, re-checks status/app_client_id/user_id from DB).

This layering is correct. An attacker cannot use a legitimate token from one access request to access resources approved in a different access request.

### `optional_auth_middleware` correctly handles `ExternalApp` with `role: None`
The middleware inserts `AuthContext::ExternalApp { role: None, .. }` for external tokens without an approved access request. Downstream `api_auth_middleware` returns `MissingAuth` (401) for this case (line 97-98 of `api_auth_middleware.rs`). This is the correct behavior — the token is structurally valid but not authorized.

---

## Summary Table

| ID | Priority | Area | Description |
|---|---|---|---|
| P2-1 | High | Code correctness | Redundant `access_request_scopes` re-filter on already-filtered `scopes` vec |
| P3-1 | Medium | Readability | `scopes` variable name and `mut` placement is confusing |
| P3-2 | Medium | Observability | `tracing::error!` should be `tracing::warn!` for missing KC claim |
| P3-3 | Medium | Readability | Use in-scope binding instead of `[0]` index in error log |
| P3-4 | Low | Documentation | 12-char cache key truncation is undocumented |
| P4-1 | Low | Test coverage | No cache-hit test with `role: Some(...)` to exercise cache round-trip |
| P4-2 | Low | Test quality | Minor inconsistency in `AuthServerTestClient` empty-scope handling |
| P4-3 | Low | Test coverage | Manager/Admin role cases absent from `api_auth_middleware` tests |
