# Plan: Restore Missing Auth Tier Tests for 3 Routes Modules

## Summary

Replace stub comments with real auth tier test implementations in 3 files within the `routes_app` crate. All three files are already inside `#[cfg(test)]` module trees, so no additional conditional compilation is needed on the new test functions.

## Context

Existing auth tier tests in other modules (e.g., `routes_api_token`, `routes_settings`, `routes_models`) follow a consistent pattern using:
- `build_test_router()` - full router with real in-memory services
- `create_authenticated_session()` - creates session with specified roles
- `session_request()` - builds request with session cookie
- `unauth_request()` - builds request without auth

All are in `crate::test_utils`.

---

## File 1: `crates/routes_app/src/routes_api_models/tests/api_models_test.rs`

**Location**: Replace line 1376 (`// Auth tier tests merged (stub for plan completion)`)

**Auth tier**: PowerUser (`ResourceRole::PowerUser`)

### Test 1: `test_api_model_endpoints_reject_unauthenticated`
- Pattern: `#[rstest]` with `#[case]` for 9 endpoints
- Endpoints: GET/POST /bodhi/v1/api-models, GET/PUT/DELETE /bodhi/v1/api-models/some_id, POST sync-models, POST test, POST fetch-models, GET api-formats
- Assert: `StatusCode::UNAUTHORIZED`

### Test 2: `test_api_model_endpoints_reject_insufficient_role`
- Pattern: `#[rstest]` with `#[values]` for roles x endpoints (cartesian product)
- Roles: `"resource_user"` only
- Endpoints: same 9 as above (via `#[values(...)]` tuple)
- Assert: `StatusCode::FORBIDDEN`

### Test 3: `test_api_model_list_endpoints_allow_power_user_and_above`
- Pattern: `#[rstest]` with `#[values]` for roles x safe endpoints
- Roles: `"resource_power_user"`, `"resource_manager"`, `"resource_admin"`
- Safe endpoints: GET /bodhi/v1/api-models (returns 200 empty list), GET /bodhi/v1/api-models/api-formats (returns 200 static data)
- Assert: `StatusCode::OK`

### Imports needed (inside function bodies):
- `crate::test_utils::{build_test_router, unauth_request, create_authenticated_session, session_request}`
- `tower::ServiceExt` (already imported at file level? No - file uses its own test_router. Need to add `use tower::ServiceExt;` inside functions or at file level)

Looking at the file: `tower::ServiceExt` is NOT in the file-level imports. The existing tests use a custom `test_router()` helper. The auth tier tests use `build_test_router()` which returns a `Router`, and `.oneshot()` requires `ServiceExt`. Must import `tower::ServiceExt` in each auth test function body.

---

## File 2: `crates/routes_app/src/routes_oai/tests/models_test.rs`

**Location**: Replace line 309 (`// Auth tier tests merged (stub for plan completion)`)

**Auth tier**: User (`ResourceRole::User`)

### Test 1: `test_oai_endpoints_reject_unauthenticated`
- 4 endpoints: GET /v1/models, GET /v1/models/some_model, POST /v1/chat/completions, POST /v1/embeddings
- Assert: `StatusCode::UNAUTHORIZED`

### Test 2: `test_oai_models_list_allows_all_roles`
- Roles: all 4 (`resource_user`, `resource_power_user`, `resource_manager`, `resource_admin`)
- Safe endpoint: GET /v1/models only (others use MockSharedContext which panics)
- Assert: `StatusCode::OK`

### Imports:
File already has `StatusCode`, `tower::ServiceExt`. Auth test functions need `crate::test_utils::{build_test_router, unauth_request, create_authenticated_session, session_request}` inside function bodies.

---

## File 3: `crates/routes_app/src/routes_ollama/tests/handlers_test.rs`

**Location**: Replace line 88 (`// Auth tier tests merged (stub for plan completion)`) -- OUTSIDE the `#[cfg(test)] mod test { ... }` block but still in a test-only module tree.

**Auth tier**: User (`ResourceRole::User`)

### Test 1: `test_ollama_endpoints_reject_unauthenticated`
- 3 endpoints: GET /api/tags, POST /api/show, POST /api/chat
- Assert: `StatusCode::UNAUTHORIZED`

### Test 2: `test_ollama_tags_allows_all_roles`
- Roles: all 4
- Safe endpoint: GET /api/tags only
- Assert: `StatusCode::OK`

### Imports:
Since these tests are at file scope (outside `mod test`), they need all imports. Place imports inside each function body to follow the canonical pattern.

---

## Exact Code Changes

### File 1: api_models_test.rs - Replace line 1376

```rust
// Auth tier tests (merged from tests/routes_api_models_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::list_api_models("GET", "/bodhi/v1/api-models")]
#[case::create_api_model("POST", "/bodhi/v1/api-models")]
#[case::get_api_model("GET", "/bodhi/v1/api-models/some_id")]
#[case::update_api_model("PUT", "/bodhi/v1/api-models/some_id")]
#[case::delete_api_model("DELETE", "/bodhi/v1/api-models/some_id")]
#[case::sync_models("POST", "/bodhi/v1/api-models/some_id/sync-models")]
#[case::test_api_model("POST", "/bodhi/v1/api-models/test")]
#[case::fetch_models("POST", "/bodhi/v1/api-models/fetch-models")]
#[case::get_api_formats("GET", "/bodhi/v1/api-models/api-formats")]
#[tokio::test]
async fn test_api_model_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  use tower::ServiceExt;
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_api_model_endpoints_reject_insufficient_role(
  #[values("resource_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/api-models"),
    ("POST", "/bodhi/v1/api-models"),
    ("GET", "/bodhi/v1/api-models/some_id"),
    ("PUT", "/bodhi/v1/api-models/some_id"),
    ("DELETE", "/bodhi/v1/api-models/some_id"),
    ("POST", "/bodhi/v1/api-models/some_id/sync-models"),
    ("POST", "/bodhi/v1/api-models/test"),
    ("POST", "/bodhi/v1/api-models/fetch-models"),
    ("GET", "/bodhi/v1/api-models/api-formats")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_api_model_list_endpoints_allow_power_user_and_above(
  #[values("resource_power_user", "resource_manager", "resource_admin")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/api-models"),
    ("GET", "/bodhi/v1/api-models/api-formats")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
```

### File 2: models_test.rs (OAI) - Replace line 309

```rust
// Auth tier tests (merged from tests/routes_oai_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::list_models("GET", "/v1/models")]
#[case::get_model("GET", "/v1/models/some_model")]
#[case::chat_completions("POST", "/v1/chat/completions")]
#[case::embeddings("POST", "/v1/embeddings")]
#[tokio::test]
async fn test_oai_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_oai_models_list_allows_all_roles(
  #[values("resource_user", "resource_power_user", "resource_manager", "resource_admin")] role: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let response = router.oneshot(session_request("GET", "/v1/models", &cookie)).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
```

### File 3: handlers_test.rs (Ollama) - Replace line 88

```rust
// Auth tier tests (merged from tests/routes_ollama_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::list_tags("GET", "/api/tags")]
#[case::show_model("POST", "/api/show")]
#[case::chat("POST", "/api/chat")]
#[tokio::test]
async fn test_ollama_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use axum::http::StatusCode;
  use crate::test_utils::{build_test_router, unauth_request};
  use tower::ServiceExt;
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_ollama_tags_allows_all_roles(
  #[values("resource_user", "resource_power_user", "resource_manager", "resource_admin")] role: &str,
) -> anyhow::Result<()> {
  use axum::http::StatusCode;
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let response = router.oneshot(session_request("GET", "/api/tags", &cookie)).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
```

**Note for File 3**: The ollama file needs extra imports (`StatusCode`, `tower::ServiceExt`, `anyhow_trace`, `rstest`) since the tests are outside the inner `mod test`. The `anyhow_trace` and `rstest` macros need to be imported. Since this file is a module in the test tree, we need to import these at the top of the function or add use statements. The safest approach is imports inside each function body plus `use` at file scope for the proc macros (`anyhow_trace`, `rstest`, `tokio`).

Actually, looking more carefully: `anyhow_trace` and `rstest` are proc macro attributes - they must be resolvable at the item level. Since the file already imports them inside `mod test` (lines 4, 11), the auth tests outside `mod test` won't have access. We need file-level imports for the proc macros.

We need to add at file level (before the existing `#[cfg(test)] mod test`):
```rust
use anyhow_trace::anyhow_trace;
use rstest::rstest;
```

But wait - these are already used inside `mod test` with their own imports. We can add them at the file scope level before the auth tests. Since the whole file is already `#[cfg(test)]` (via the parent module), this is fine.

Actually the cleanest approach: add file-level use statements after the existing `mod test` block closing brace. The `use` statements can go right before the auth test functions.

---

## Verification

After all 3 edits:
```bash
cargo test -p routes_app --lib 2>&1 | tail -20
```

Expected: All new tests pass. The tests use `build_test_router()` which creates real in-memory services, so:
- Unauthenticated requests should get 401
- Insufficient roles should get 403
- Sufficient roles hitting safe list endpoints should get 200 (empty lists)

## Risk Assessment

- **Low risk**: The patterns are well-established in other modules (api_token, settings, models)
- **Potential issue**: The `api-formats` endpoint might need a specific role check - if it returns something other than 200, we adjust
- **Potential issue**: Ollama file-level imports for proc macros - if compilation fails, we adjust placement
