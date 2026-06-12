# objs Crate Review

## Files Reviewed

- `crates/objs/src/token_scope.rs` — `TokenScope` enum (User, PowerUser), `TokenScopeError`, `FromStr`, `from_scope()`, `has_access_to()`, `included_scopes()`, and tests
- `crates/objs/src/user_scope.rs` — `UserScope` enum (User, PowerUser), `UserScopeError`, `FromStr`, `has_access_to()`, `included_scopes()`, and tests

---

## Findings

### Finding 1: `MissingUserScope` variant is now dead code in `UserScopeError`

- **Priority**: Important
- **File**: `crates/objs/src/user_scope.rs`
- **Location**: `UserScopeError::MissingUserScope` (line 41)
- **Issue**: The `MissingUserScope` variant was previously only returned by `UserScope::from_scope()`, which has been removed in this commit. After removal, no code path in the entire codebase produces this variant — `UserScope::FromStr` only returns `InvalidUserScope`. The variant is defined but unreachable. `UserScopeError` is consumed via `#[from]` in `auth_middleware`, so the error type itself is still used, but the `MissingUserScope` variant is dead.
- **Recommendation**: Remove the `MissingUserScope` variant from `UserScopeError`. Its absence is safe because `FromStr` only returns `InvalidUserScope`. If no callers match on `MissingUserScope`, the removal is non-breaking; downstream `#[from] UserScopeError` wrapping in `auth_middleware` continues to work with only `InvalidUserScope`.
- **Rationale**: Dead code in error enums is misleading — it implies there is a code path that can produce "missing user scope" when none exists. It can confuse future developers and should be cleaned up alongside the removal of `from_scope()`.

---

### Finding 2: `TokenScope::from_scope()` is only referenced in its own test module

- **Priority**: Nice-to-have
- **File**: `crates/objs/src/token_scope.rs`
- **Location**: `impl TokenScope { pub fn from_scope(...) }` (lines 58–66)
- **Issue**: A global search across all crates confirms that `TokenScope::from_scope()` is called only from the two test functions inside `token_scope.rs` itself (lines 141 and 151). No production code in `auth_middleware`, `routes_app`, or elsewhere calls this method. In `auth_middleware/token_service/service.rs`, the role is resolved via `r.parse::<UserScope>()` (using `FromStr`), not via `from_scope()`. The method is public API with no non-test callers.
- **Recommendation**: Either remove `from_scope()` and its tests (since the equivalent `UserScope::from_scope()` was already removed), or demote it to `pub(crate)` / make it test-only if future production use is anticipated. If the intent is to keep it for potential future callers of `TokenScope`, add a comment documenting why it differs from `UserScope` (which had its `from_scope()` removed).
- **Rationale**: Keeping public API with no callers increases the maintenance surface. The parallel treatment of `UserScope` (which had `from_scope()` removed) suggests this was an oversight. Consistency is important for the two symmetrical scope types.

---

### Finding 3: `has_access_to()` test case for equal scopes is mislabeled as `false` for `is_greater`

- **Priority**: Important
- **File**: `crates/objs/src/token_scope.rs` and `crates/objs/src/user_scope.rs`
- **Location**: `test_token_scope_ordering_explicit` (lines 88–89), `test_user_scope_ordering_explicit` (lines 77–78)
- **Issue**: The test cases `#[case(TokenScope::PowerUser, TokenScope::PowerUser, false)]` and `#[case(TokenScope::User, TokenScope::User, false)]` with `is_greater = false` are correct as ordering assertions, but the test body validates `has_access_to` only implicitly. The method `has_access_to(&self, required) -> bool` returns `self >= required` — so `PowerUser.has_access_to(PowerUser)` returns `true`. This equality case is not directly exercised by a dedicated test asserting `has_access_to()` returns `true` when the scopes are equal. The ordering test for `>=` is tested correctly at line 97/88, but there is no explicit test that `scope.has_access_to(&scope)` returns `true` for the reflexive case.
- **Recommendation**: Add one explicit `has_access_to` test case demonstrating the reflexive (equal) case returns `true`, e.g. `assert!(TokenScope::User.has_access_to(&TokenScope::User))`. This is security-critical behavior and deserves unambiguous coverage distinct from the ordering property tests.
- **Rationale**: `has_access_to()` is used in `api_auth_middleware` for authorization decisions. The reflexive case (`User` accessing a `User`-gated endpoint) is the common path. While the current ordering tests do implicitly cover this via the `>=` assertion, explicit `has_access_to()` tests for the reflexive case make the security contract self-documenting.

---

### Finding 4: `test_included_scopes_explicit` has an unnecessary invariant assertion for the single-element case

- **Priority**: Nice-to-have
- **File**: `crates/objs/src/token_scope.rs` (lines 123–130), `crates/objs/src/user_scope.rs` (lines 112–119)
- **Location**: `test_included_scopes_explicit` — the `windows(2)` loop
- **Issue**: The `windows(2)` assertion inside `if !included.is_empty()` runs for both `User` (single element) and `PowerUser` (two elements). For `User`, `included.windows(2)` produces an empty iterator, so the `assert!(window[0] > window[1])` body never executes. The invariant is only checked for the `PowerUser` case. This is not a bug, but the conditional guard is misleading — the reader might think the single-element case was carefully considered, when in fact the invariant simply doesn't fire.
- **Recommendation**: No code change required, but the inner invariant could be moved outside the `if !included.is_empty()` guard to clarify intent, or the tests could be split. This is cosmetic.
- **Rationale**: Minor test clarity issue. Not a correctness problem.

---

### Finding 5: Backwards compatibility — removed scope strings correctly fail `FromStr` (positive confirmation)

- **Priority**: N/A — Confirmed correct
- **File**: `crates/objs/src/token_scope.rs` (lines 183–184), `crates/objs/src/user_scope.rs` (lines 150–151)
- **Issue**: None — this is a confirmation of correct behavior.
- **Detail**: `"scope_token_manager"`, `"scope_token_admin"`, `"scope_user_manager"`, and `"scope_user_admin"` are all explicitly included in `test_token_scope_parse_invalid` and `test_user_scope_parse_invalid` respectively. They produce `InvalidTokenScope`/`InvalidUserScope` errors rather than silently mapping to any live variant. This is the correct behavior for security-sensitive scope parsing.

---

### Finding 6: `PartialOrd`/`Ord` ordering derived from declaration order — correct for 2-variant enums

- **Priority**: N/A — Confirmed correct
- **File**: `crates/objs/src/token_scope.rs`, `crates/objs/src/user_scope.rs`
- **Detail**: Both enums declare `User` first and `PowerUser` second. Rust's `#[derive(PartialOrd, Ord)]` assigns ordering by declaration position, so `User < PowerUser`. The `has_access_to()` method uses `self >= required`, so `PowerUser.has_access_to(User)` returns `true` (PowerUser can access User-gated resources) and `User.has_access_to(PowerUser)` returns `false`. This is the correct security model. The ordering tests in both files confirm this explicitly.

---

### Finding 7: Serde conventions are correct

- **Priority**: N/A — Confirmed correct
- **File**: `crates/objs/src/token_scope.rs`, `crates/objs/src/user_scope.rs`
- **Detail**: Both enums use per-variant `#[serde(rename = "scope_*_*")]` overrides rather than relying on `serde(rename_all = "snake_case")` alone. This is the correct pattern because the enum variant names (`User`, `PowerUser`) would serialize as `"user"` / `"power_user"` under `snake_case`, which does not match the wire format. The explicit per-variant rename ensures serde produces `"scope_token_user"` / `"scope_token_power_user"` / `"scope_user_user"` / `"scope_user_power_user"`. The `serde(rename_all = "snake_case")` at the enum level is technically redundant given the per-variant overrides, but it is harmless and may serve as documentation of intent.

---

## Summary

- **Total findings**: 4 actionable (Critical: 0, Important: 2, Nice-to-have: 2) + 3 confirmations of correct behavior

| # | Priority | Description |
|---|----------|-------------|
| 1 | Important | `MissingUserScope` in `UserScopeError` is dead code — `from_scope()` was removed but the error variant was left behind |
| 2 | Nice-to-have | `TokenScope::from_scope()` has no production callers — only referenced in its own tests; `UserScope::from_scope()` was removed consistently but `TokenScope::from_scope()` was not |
| 3 | Important | `has_access_to()` reflexive (equal) case has no dedicated test — covered only implicitly via ordering assertions in `>=` |
| 4 | Nice-to-have | `test_included_scopes_explicit` invariant loop never fires for the single-element case — cosmetic test clarity issue |

The core logic — PartialOrd ordering correctness, serde wire format correctness, backwards incompatibility of removed scope strings, and FromStr parse coverage — is all correct and well-tested.
