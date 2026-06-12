# Auth Middleware Crate Refactoring - Q&A Context

## Session Summary

Post-AuthContext migration cleanup of the entire `auth_middleware` crate. The AuthContext enum and extension-based transport (commit `edaa9c97`) is fully implemented. This session addresses the tech debt items identified during that migration: middleware consolidation, control flow cleanup, and naming improvements.

---

## 1. Starting State

Both `auth_middleware` and `inject_optional_auth_info` already inject `AuthContext` into request extensions. The entire plan from `20260217-auth-ctx.md` was executed in commit `edaa9c97` (32 files, 1652+/1773-). The old header-based extractors (`ExtractToken`, `ExtractUserId`, etc.) and `extractors.rs` are deleted.

Remaining issues:
- ~100 lines duplicated between `auth_middleware` and `inject_optional_auth_info`
- `_headers: HeaderMap` parameter naming bug (underscore prefix but used)
- Inconsistency: `auth_middleware` extracts `HeaderMap`, `inject_optional_auth_info` uses `req.headers()`
- Session cleanup logic in `inject_optional_auth_info` duplicated across 2 error branches
- `_impl` unconventional function name in `api_auth_middleware.rs`
- `(is_session, is_oauth)` boolean tuple anti-pattern in `toolset_auth_middleware.rs`
- 80-line nested OAuth validation block in `toolset_auth_middleware.rs`
- Dead error variants `SignatureKey` and `SignatureMismatch` in `AuthError`

---

## 2. Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Middleware consolidation approach | Two thin public wrappers calling shared `resolve_auth_context()` | Call sites unchanged, all logic in one place, eliminates ~100 lines duplication |
| Public function names | `auth_middleware()` + `optional_auth_middleware()` | `inject_optional_auth_info` renamed for consistency |
| Session cleanup | Extract `clear_session_auth_data()` helper + `should_clear_session()` predicate | Dedup the 2 identical 9-line cleanup blocks |
| `_impl` rename | `authorize_request` | More self-documenting |
| Unused `State(_state)` in api_auth | Keep in public fn (Axum requires it), remove from inner `authorize_request` | `from_fn_with_state` injects state, public fn must accept it |
| Toolset auth cleanup scope | Full cleanup: extract sub-functions AND flatten control flow | Most thorough, addresses all nesting |
| Toolset flow detection | Replace `(is_session, is_oauth)` booleans with `ToolsetAuthFlow` enum | Type-safe, self-documenting, eliminates boolean anti-pattern |
| Toolset sub-functions | `extract_toolset_id_from_path()`, `validate_access_request()`, `validate_toolset_approved_list()`, `validate_toolset_configuration()` | Each is a self-contained validation step |
| AuthError consolidation | Remove dead variants only (`SignatureKey`, `SignatureMismatch`) | Other variants have distinct semantics/HTTP status codes, consolidation risks API contract changes |
| `HeaderMap` extractor in auth_middleware | Remove entirely, use `req.headers()` in shared function | Eliminates naming bug, aligns both wrappers, Axum `HeaderMap` extractor clones headers (doesn't consume) |

---

## 3. Call Site Impact

`inject_optional_auth_info` references to update to `optional_auth_middleware`:
- `crates/routes_app/src/routes.rs` lines 40, 104
- `crates/routes_app/src/routes_auth/tests/login_test.rs` lines 6, 338, 446, 602, 680, 752
- `crates/auth_middleware/src/auth_middleware.rs` internal test module line 356, 479
- Documentation files: `crates/routes_app/CLAUDE.md`, `crates/auth_middleware/CLAUDE.md`, `crates/auth_middleware/PACKAGE.md`

---

## 4. Risk Analysis

**Medium risk - Phase 1 (middleware consolidation):**
- `optional_auth_middleware` must preserve exact error-handling semantics
- `RefreshTokenNotFound` → clear session + Anonymous
- `Token(_) | AuthService(_) | InvalidToken(_)` → clear session + Anonymous
- `AppStatusInvalid(_)` → Anonymous (no session cleanup)
- All other errors → Anonymous (no session cleanup)
- The `should_clear_session()` function must match lines 288 and 306-309 of current code exactly

**Low risk - Phase 2 (api_auth rename):** Pure rename, no behavioral change

**Medium risk - Phase 3 (toolset restructure):**
- `validate_access_request` must return `approved: Option<String>` field for downstream approved-list check
- `validate_toolset_configuration` uses `ToolsetError` variants (not `ToolsetAuthError`) via `#[from]` conversion
- `is_type_enabled().await?` propagation chain: `ToolsetError` → `ToolsetAuthError::ToolsetError` → `ApiError`

**Low risk - Phase 4 (dead code removal):** Confirmed `SignatureKey`/`SignatureMismatch` only appear in enum definition, zero construction sites

---

## 5. Tech Debt Status

From `20260217-auth-ctx.md` Phase 8b (skipped):
1. **Single middleware** → Being addressed in this refactoring
2. **Route policy with AuthContext list** → Deferred (future work)
3. **Toolset auth full integration** → Partially addressed (restructuring, not full redesign)
