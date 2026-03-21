# Plan: Add tests for routes_apps access request handlers

## Context

The `approve_access_request_handler` and `deny_access_request_handler` were changed to return `Json<AccessRequestActionResponse>` instead of `StatusCode::OK`. No tests exist for the `routes_apps` module. We need tests verifying the new JSON response body and security/authorization behavior.

## Security analysis

The review/approve/deny endpoints are in the `user_session_apis` group with `api_auth_middleware(ResourceRole::User, None, None, ...)`:
- Session-only auth (API tokens and OAuth blocked by `None` scope params)
- Any role (`User` minimum) is sufficient - by design for user-approval flow
- No handler-level ownership check needed - any authenticated user can review/approve/deny as the resource owner

The create/status endpoints are public (no auth).

## Files to create/modify

1. **Create** `crates/routes_app/src/routes_apps/tests/mod.rs`
2. **Create** `crates/routes_app/src/routes_apps/tests/access_request_test.rs`
3. **Modify** `crates/routes_app/src/routes_apps/mod.rs` - add `#[cfg(test)] mod tests;`
4. **Modify** `crates/routes_app/src/routes_users/tests/access_request_test.rs` - add cases to existing parameterized auth test

## Changes

### 1. Add unauthenticated rejection cases to existing test

In `routes_users/tests/access_request_test.rs`, add cases to `test_access_request_endpoints_reject_unauthenticated`:

```rust
#[case::app_review("GET", "/bodhi/v1/access-requests/test-id/review")]
#[case::app_approve("PUT", "/bodhi/v1/access-requests/test-id/approve")]
#[case::app_deny("POST", "/bodhi/v1/access-requests/test-id/deny")]
```

These endpoints share the same unauth behavior (401) with the existing cases.

Note: Cannot add to `reject_insufficient_role` - different auth tier (User min vs Manager min).

### 2. Functional tests in routes_apps/tests/access_request_test.rs

Use `#[rstest]` with `#[case]` for approve/deny scenarios.

**Test setup**: Build full router with real DB services + mocked AuthService via `build_routes()` + `create_authenticated_session()` + `session_request_with_body()`.

```rust
// Setup pattern (shared across tests):
// 1. AppServiceStubBuilder with .with_db_service().await + .with_session_service().await + .with_secret_service()
// 2. get_db_service() to insert test data (AppAccessRequestRow, ToolsetRow)
// 3. Build real DefaultToolService + DefaultAccessRequestService backed by same DB
// 4. MockAuthService for approve (register_access_request_consent)
// 5. build_routes() for full router, create_authenticated_session() for auth
```

**Approve handler - success cases** (parameterized with `#[case]`):
```rust
#[case::popup_flow("popup", None, false)]    // no redirect_url in response
#[case::redirect_flow("redirect", Some("https://app.com/cb"), true)]  // redirect_url present
```
Assert: 200, body `status` = "approved", `flow_type` matches, `redirect_url` presence matches

**Approve handler - error cases** (parameterized with `#[case]`):
```rust
#[case::instance_not_found("nonexistent-id", 403, "app_access_request_error-tool_instance_not_owned")]
#[case::instance_not_enabled(disabled_instance_id, 400, "app_access_request_error-tool_instance_not_configured")]
```

**Deny handler - success cases** (parameterized with `#[case]`):
```rust
#[case::popup_flow("popup", None, false)]
#[case::redirect_flow("redirect", Some("https://app.com/cb"), true)]
```
Assert: 200, body `status` = "denied", `flow_type` matches, `redirect_url` presence matches

## Key utilities to reuse

| Utility | Location |
|---------|----------|
| `AppServiceStubBuilder` | `services::test_utils::app` |
| `FrozenTimeService` | `services::test_utils` |
| `DefaultToolService` | `services::tool_service` |
| `DefaultAccessRequestService` | `services::access_request_service` |
| `MockAuthService` | `services::MockAuthService` |
| `MockExaService` | `services::exa_service::MockExaService` |
| `build_routes()` | `crate::build_routes` |
| `create_authenticated_session()` | `crate::test_utils` |
| `session_request_with_body()` | `crate::test_utils` |
| `session_request()` | `crate::test_utils` |
| `unauth_request()` | `crate::test_utils` |
| `ResponseTestExt::json()` | `server_core::test_utils` |
| `AppAccessRequestRow` | `services::db` |
| `ToolsetRow` | `services::db` |
| `RegisterAccessRequestConsentResponse` | `services` |

## Verification

```bash
cargo test -p routes_app -- routes_apps::tests::access_request_test
cargo test -p routes_app -- routes_users::tests::access_request_test::test_access_request_endpoints_reject_unauthenticated
```
