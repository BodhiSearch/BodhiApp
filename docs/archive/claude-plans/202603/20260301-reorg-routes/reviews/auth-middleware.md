# auth_middleware Crate Review

Review of changes in `HEAD~2..HEAD` for `crates/auth_middleware/src/`.

## Summary

Two commits introduce:
1. `AuthContext` moved to `services` crate; `auth_context.rs` becomes a re-export shim.
2. `MiddlewareError` replaces `ApiError` as the return type for all 4 middleware functions.
3. `client_id` field added to all `AuthContext` variants (including `Anonymous`).
4. `AuthError::TowerSession` simplified from explicit code/transparent to auto-derived code with `#[error("{0}")]`.
5. Test factory methods moved from `auth_middleware::test_utils` to `services::test_utils` (re-exported via `AuthContext` impl block).
6. Import paths updated from `services::db::*` to `services::*` (re-export).

## Findings

| Priority | File | Location | Issue | Recommendation |
|----------|------|----------|-------|----------------|
| P2 | `crates/auth_middleware/src/auth_middleware/middleware.rs` | `auth_middleware()` lines 145-150 | Redundant `get_instance()` call. `instance_client_id` is fetched unconditionally before the bearer-vs-session branch, but the bearer token path (`token_service.validate_bearer_token`) does its own `get_instance()` call internally (in `token_service/service.rs` lines 103-108 for API tokens, and line 358 for external token exchange). This doubles the DB call for bearer token requests. | Defer `get_instance()` to the session-only branch, or pass the already-fetched `client_id` into the token service to avoid the redundant lookup. |
| P2 | `crates/auth_middleware/src/auth_middleware/middleware.rs` | `optional_auth_middleware()` line 265 | `client_id: instance_client_id.unwrap_or_default()` silently produces an empty string `""` if the instance lookup failed (`.ok().flatten()` on line 224-229). This creates a `Session` variant with an empty `client_id` rather than surfacing an error or falling back to Anonymous. Downstream code calling `require_client_id()` would succeed but return `""`, which may cause subtle bugs. | Either return `MiddlewareError` when instance is missing in the authenticated session path (matching `auth_middleware`'s strict behavior), or fall back to `AuthContext::Anonymous` when `instance_client_id` is `None`. |
| P3 | `crates/auth_middleware/src/middleware_error.rs` | `From<T> for MiddlewareError` line 18-27 | The `value.borrow()` call on line 19 is unnecessary. `value` is already owned `T`; calling `.borrow()` returns `&T`, which is then used for field extraction. The `use std::borrow::Borrow` import exists solely for this. While functionally correct, it adds unnecessary indirection and an unused-looking import. | Remove the `borrow()` call and the `use std::borrow::Borrow` import. Access fields directly on `&value` (take a reference). |
| P3 | `crates/auth_middleware/src/middleware_error.rs` | `IntoResponse` impl line 43 | `Response::builder().status(self.status).body(...).unwrap()` will panic if `self.status` is not a valid HTTP status code. While `AppError::status()` should always return valid codes, a malformed `ErrorType` mapping could cause a panic in production middleware. | Use `.map_err()` or provide a fallback 500 response instead of `unwrap()`. |
