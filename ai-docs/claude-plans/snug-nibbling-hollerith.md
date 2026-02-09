# Routes App Test Reorganization Plan

## Context

Following a 14-commit migration that added 102 auth tier tests, this plan addresses feedback to:
- Replace `assert_ne!` with `assert_eq!` by crafting valid requests for 2xx responses
- Convert remaining single-file modules (`routes_setup`, `routes_settings`, `routes_api_token`) to folder modules
- Merge auth tests into handler test files (eliminate separate `*_auth_test.rs` files)
- Add `StubQueue` and `NetworkService` for better testability
- Fix role hierarchy test gaps (e.g., settings 403 test missing `resource_manager`)
- Use rstest `#[values]` for cartesian product in rejection tests
- Extract ping/health to `routes_ping.rs` with inline tests
- Delete redundant `test_router_smoke.rs`

---

## Commit 1: Infrastructure — StubQueue + NetworkService

### 1a. StubQueue

**Create** `crates/services/src/test_utils/queue.rs`:
- No-op `QueueProducer` impl: `enqueue()` → `Ok(())`, `queue_length()` → `0`, `queue_status()` → `"idle"`
- `#[derive(Debug, Default)]`

**Modify** `crates/services/src/test_utils/mod.rs` — add `mod queue; pub use queue::*;`

**Note**: Do NOT change `AppServiceStubBuilder` default (handler tests need `MockQueueProducer` with expectations). Only `build_test_router()` switches to `StubQueue`.

### 1b. NetworkService

**Create** `crates/services/src/network_service.rs`:
- `trait NetworkService: Send + Sync + Debug` with `fn get_server_ip(&self) -> Option<String>`
- `DefaultNetworkService` — moves UDP socket logic from `routes_setup.rs:268-286`
- `StubNetworkService { ip: Option<String> }` — returns configurable IP (for test-utils feature gate)

**Modify** these files (follow `tool_service` pattern):
| File | Change |
|------|--------|
| `crates/services/src/lib.rs` | `mod network_service; pub use network_service::*;` |
| `crates/services/src/app_service.rs` | Add `fn network_service(&self) -> Arc<dyn NetworkService>` to trait + `DefaultAppService` field/impl |
| `crates/services/src/test_utils/app.rs` | Add field to `AppServiceStub` + builder default `StubNetworkService { ip: None }` |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | Add field + `get_or_build_network_service()` → `DefaultNetworkService` + wire into `build()` |
| `crates/routes_app/src/routes_setup.rs` | Remove private `get_server_ip()`, use `state.app_service().network_service().get_server_ip()` |

### 1c. Update build_test_router()

**Modify** `crates/routes_app/src/test_utils/router.rs`:
- Wire `StubQueue` for `queue_producer`
- Wire `StubNetworkService { ip: Some("192.168.1.100") }` for `network_service`

**Verify**: `cargo check -p services && cargo check -p routes_app && cargo test -p routes_app`

---

## Commit 2: Extract routes_ping

**Create** `crates/routes_app/src/routes_ping.rs`:
- Move from `routes_setup.rs`: `ping_handler`, `health_handler`, `PingResponse`
- `ENDPOINT_PING` and `ENDPOINT_HEALTH` constants stay in `shared/openapi.rs` (already there)
- Add `#[cfg(test)] mod tests` with:
  - Handler tests: `ping_handler()` returns `{"message":"pong"}`, same for health
  - Router tests via `build_test_router()`: GET /ping → `assert_eq!(StatusCode::OK)`, GET /health → `assert_eq!(StatusCode::OK)`

**Modify**:
- `crates/routes_app/src/routes_setup.rs` — remove `PingResponse`, `ping_handler`, `health_handler`
- `crates/routes_app/src/lib.rs` — add `mod routes_ping; pub use routes_ping::*;`
- `crates/routes_app/src/routes.rs` — imports resolve via crate re-exports (no change needed)

**Verify**: `cargo check -p routes_app && cargo test -p routes_app`

---

## Commit 3: routes_setup → folder module, merge tests

**Structural changes**:
- `src/routes_setup.rs` → `src/routes_setup/mod.rs` (add `#[cfg(test)] mod tests;`)
- Create `src/routes_setup/tests/mod.rs` → `mod setup_test;`
- Move `src/routes_setup_test.rs` → `src/routes_setup/tests/setup_test.rs`
- Merge `tests/routes_setup_auth_test.rs` → into `setup_test.rs`
- Remove `#[cfg(test)] mod routes_setup_test;` from `lib.rs`
- Delete `tests/routes_setup_auth_test.rs`

**Auth test improvements in setup_test.rs**:
- Replace `assert_ne!` with `assert_eq!`:
  - GET /bodhi/v1/info (no auth) → `assert_eq!(StatusCode::OK)` (real SecretService+SettingService)
  - POST /bodhi/v1/logout (no auth) → try for 200, leave failing if needed
  - POST /bodhi/v1/setup (no auth) → send valid `SetupRequest` body with MockAuthService expectations → `assert_eq!(StatusCode::OK)`. This works because build_test_router has SecretServiceStub defaulting to AppStatus::Setup
- Make `test_setup_handler_network_ip_redirect_uris` deterministic: StubNetworkService returns `"192.168.1.100"` deterministically

---

## Commit 4: routes_settings → folder module, merge tests

**Structural changes**:
- `src/routes_settings.rs` → `src/routes_settings/mod.rs` (add `#[cfg(test)] mod tests;`)
- Create `src/routes_settings/tests/mod.rs` → `mod settings_test;`
- Move `src/routes_settings_test.rs` → `src/routes_settings/tests/settings_test.rs`
- Merge `tests/routes_settings_auth_test.rs` → into `settings_test.rs`
- Remove `#[cfg(test)] mod routes_settings_test;` from `lib.rs`
- Delete `tests/routes_settings_auth_test.rs`

**Auth test improvements**:
- **Fix role hierarchy gap**: Add `resource_manager` to 403 rejection (currently missing — Manager < Admin)
- **rstest #[values]** for 403 cartesian product:
  ```rust
  #[values("resource_user", "resource_power_user", "resource_manager")] role: &str,
  #[values(("GET", "/bodhi/v1/settings"), ("PUT", "/bodhi/v1/settings/some_key"), ("DELETE", "/bodhi/v1/settings/some_key"), ("GET", "/bodhi/v1/toolset_types"), ("PUT", "/bodhi/v1/toolset_types/some_type/app-config"), ("DELETE", "/bodhi/v1/toolset_types/some_type/app-config"))] endpoint: (&str, &str),
  ```
- **Allowed test**: Admin role only. Replace `assert_ne!` with:
  - GET /bodhi/v1/settings → `assert_eq!(StatusCode::OK)` (real SettingServiceStub)
  - PUT/DELETE /bodhi/v1/settings/some_key → `assert_eq!(StatusCode::BAD_REQUEST)` or `NOT_FOUND` (proves auth passed)
  - Toolset type endpoints excluded (MockToolService panics) — comment explains

---

## Commit 5: routes_api_token → folder module, merge tests

**Structural changes**:
- `src/routes_api_token.rs` → `src/routes_api_token/mod.rs` (add `#[cfg(test)] mod tests;`)
- Create `src/routes_api_token/tests/mod.rs` → `mod api_token_test;`
- Move `src/routes_api_token_test.rs` → `src/routes_api_token/tests/api_token_test.rs`
- Merge `tests/routes_api_token_auth_test.rs` → into `api_token_test.rs`
- Remove `#[cfg(test)] mod routes_api_token_test;` from `lib.rs`
- Delete `tests/routes_api_token_auth_test.rs`

**Auth test improvements**:
- **Test all eligible roles**: PowerUser, Manager, Admin (not just PowerUser)
- GET /bodhi/v1/tokens → `assert_eq!(StatusCode::OK)` (empty list from real DbService)

---

## Commit 6: routes_models — merge 3 auth test files

**Merge into existing test files** (folder module already exists):
- `tests/routes_models_auth_test.rs` → `src/routes_models/tests/aliases_test.rs`
- `tests/routes_models_metadata_auth_test.rs` → `src/routes_models/tests/metadata_test.rs`
- `tests/routes_models_pull_auth_test.rs` → `src/routes_models/tests/pull_test.rs`
- Delete all 3 from `tests/`

**Auth test improvements**:
- **Read endpoints (user tier)**: Test User, PowerUser, Manager, Admin as eligible roles
  - GET /bodhi/v1/models → `assert_eq!(StatusCode::OK)` (empty list)
  - GET /bodhi/v1/modelfiles → `assert_eq!(StatusCode::OK)` (empty list)
  - GET /bodhi/v1/models/some-id → 404 is acceptable (proves auth passed)
- **Write endpoints (power_user tier)**: Test PowerUser, Manager, Admin
  - POST/PUT /bodhi/v1/models without valid body → 422 or 400 (proves auth passed)
  - DELETE /bodhi/v1/models/some-id → 404 (proves auth passed)
  - **rstest #[values]** for 403: `"resource_user"` × all write endpoints
- **Metadata**: StubQueue makes these fully testable now
  - GET /bodhi/v1/queue → `assert_eq!(StatusCode::OK)` (StubQueue returns "idle")
- **Pull**: GET /bodhi/v1/modelfiles/pull → `assert_eq!(StatusCode::OK)` (empty list from real DbService)

---

## Commit 7: routes_api_models + routes_users + routes_auth — merge auth tests

### routes_api_models
- Merge `tests/routes_api_models_auth_test.rs` → `src/routes_api_models/tests/api_models_test.rs`
- Test PowerUser, Manager, Admin as eligible roles
- GET /bodhi/v1/api-models → `assert_eq!(StatusCode::OK)` (empty list)
- GET /bodhi/v1/api-models/api-formats → `assert_eq!(StatusCode::OK)` (static data)

### routes_users (3 auth test files)
- `tests/routes_users_info_auth_test.rs` → `src/routes_users/tests/user_info_test.rs`
- `tests/routes_users_access_request_auth_test.rs` → `src/routes_users/tests/access_request_test.rs`
- `tests/routes_users_management_auth_test.rs` → `src/routes_users/tests/management_test.rs`
- Manager tier: Test Manager + Admin eligible roles
- GET /bodhi/v1/access-requests/pending → `assert_eq!(StatusCode::OK)` (empty list from real DB)
- User management endpoints call MockAuthService → set `list_users` expectation for `GET /bodhi/v1/users` or leave failing with TODO

### routes_auth
- `tests/routes_auth_auth_test.rs` → `src/routes_auth/tests/request_access_test.rs`
- Optional auth: POST without body → `assert_eq!(StatusCode::BAD_REQUEST)` (proves not auth-blocked)

Delete all 5 auth test files from `tests/`.

---

## Commit 8: routes_toolsets + routes_oai + routes_ollama — merge auth tests + cleanup

### routes_toolsets
- `tests/routes_toolsets_auth_test.rs` → `src/routes_toolsets/tests/toolsets_test.rs`
- All handlers call MockToolService → only 401 rejection tests preserved
- Comment explains: no allowed tests possible without MockToolService expectations

### routes_oai
- `tests/routes_oai_auth_test.rs` → `src/routes_oai/tests/models_test.rs`
- User tier: Test User, PowerUser, Manager, Admin
- GET /v1/models → `assert_eq!(StatusCode::OK)` (real DataService)
- Chat/embeddings endpoints call MockSharedContext → only 401 rejection tests

### routes_ollama
- `tests/routes_ollama_auth_test.rs` → `src/routes_ollama/tests/handlers_test.rs`
- GET /api/tags → `assert_eq!(StatusCode::OK)` (real DataService)
- Chat endpoint calls MockSharedContext → only 401 rejection test

### Cleanup
- Delete `tests/test_router_smoke.rs` (redundant)
- Delete all 3 remaining auth test files from `tests/`
- Verify `tests/` only contains `data/` fixtures
- Final: `cargo test -p routes_app && make test.backend`

---

## Summary

### File Operations
| Type | Count | Details |
|------|-------|---------|
| Create | ~8 | `network_service.rs`, `test_utils/queue.rs`, `routes_ping.rs`, 3 `tests/mod.rs` for folder conversions |
| Move (rename) | 6 | 3 single-file modules → `mod.rs`, 3 sibling test files → `tests/` |
| Delete | 15 | `test_router_smoke.rs` + 13 `*_auth_test.rs` + 1 empty file |
| Modify | ~15 | `lib.rs`, `routes.rs`, `app_service.rs`, `app_service_builder.rs`, `router.rs`, all target test files |

### Endpoints That May Need User Fix (failing TODO tests)
| Endpoint | Issue |
|----------|-------|
| POST /bodhi/v1/logout (no auth) | Exact status uncertain without session |
| GET /bodhi/v1/users | MockAuthService.list_users() panics without expectations |
| PUT/DELETE /bodhi/v1/users/{id} | MockAuthService operations |
| POST /bodhi/v1/access-requests/{id}/approve\|reject | MockAuthService operations |
| All toolset CRUD + execute | MockToolService panics |
| POST /v1/chat/completions, /v1/embeddings, /api/chat | MockSharedContext panics |

### Verification
```bash
cargo check -p services
cargo check -p routes_app
cargo test -p routes_app
make test.backend
```
