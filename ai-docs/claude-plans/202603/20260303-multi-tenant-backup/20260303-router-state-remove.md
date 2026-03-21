# Simplify SharedContext: Remove RouterState, Expose Lifecycle via InferenceService

## Context

SharedContext was created to manage llama.cpp process lifecycle and was threaded through the entire bootstrap chain: `build_app_service() → ServeCommand → build_routes() → DefaultRouterState → handlers`. With the InferenceService migration, handlers no longer use SharedContext directly — they go through `auth_scope.inference()`. SharedContext's remaining external consumers (shutdown callback, keep-alive timer, variant listener) all work through `ctx.stop()`, `ctx.set_exec_variant()` etc.

**Goal**: Make SharedContext a pure implementation detail of `StandaloneInferenceService`. Expose lifecycle operations (`stop`, `set_variant`, `set_keep_alive`) on the `InferenceService` trait itself. All external code (listeners, shutdown callbacks) goes through `AppService → InferenceService` instead of holding `SharedContext` directly. Remove `RouterState` (only had `app_service()`) and use `Arc<dyn AppService>` as Axum state.

---

## Phase 1: `services` — Extend InferenceService Trait

**`crates/services/src/inference/inference_service.rs`**
- Add lifecycle methods to `InferenceService` trait:
  ```rust
  async fn stop(&self) -> Result<(), InferenceError>;
  async fn set_variant(&self, variant: &str) -> Result<(), InferenceError>;
  async fn set_keep_alive(&self, secs: i64);
  async fn is_loaded(&self) -> bool;
  ```

**`crates/services/src/inference/noop.rs`** — `NoopInferenceService`:
- `stop()` → `Ok(())`
- `set_variant()` → `Ok(())`
- `set_keep_alive()` → no-op
- `is_loaded()` → `false`

**Test infra**: `MockInferenceService` auto-updated by `mockall::automock`.

### Gate
```bash
cargo test -p services
cargo check -p server_core
```

---

## Phase 2: `server_core` — Implement Lifecycle on InferenceService Impls, Remove RouterState

### 2a. StandaloneInferenceService — add lifecycle + keep-alive timer

**`crates/server_core/src/standalone_inference.rs`**
- Add fields: `keep_alive_secs: RwLock<i64>`, `timer_handle: Mutex<Option<JoinHandle<()>>>`
- Constructor: `new(ctx, keep_alive_secs)` — takes initial value from settings
- `stop()` → `self.ctx.stop().await`, cancel timer
- `set_variant(variant)` → `self.ctx.set_exec_variant(variant).await`
- `set_keep_alive(secs)` → update stored value, restart timer
- `is_loaded()` → `self.ctx.is_loaded().await`
- `forward_local()` → after forwarding, reset keep-alive timer (same semantics as current `ServerKeepAlive`):
  - `keep_alive < 0`: no timer
  - `keep_alive == 0`: stop immediately after request
  - `keep_alive > 0`: start/reset timer

### 2b. MultitenantInferenceService — no-op lifecycle

**`crates/server_core/src/multitenant_inference.rs`**
- `stop()` → `Ok(())`
- `set_variant()` → `Ok(())`
- `set_keep_alive()` → no-op
- `is_loaded()` → `false`

### 2c. Remove RouterState + ModelRouter + NoopSharedContext

**Delete files:**
- `crates/server_core/src/model_router.rs` — dead code (handlers do alias resolution via `auth_scope.data()`)
- `crates/server_core/src/noop_shared_context.rs` — only existed for `build_routes()` ctx param

**`crates/server_core/src/router_state.rs`**
- Delete `RouterState` trait, `DefaultRouterState`, `RouterStateError`
- Keep `LlmEndpoint` re-export (move to `lib.rs` if needed)

**`crates/server_core/src/lib.rs`**
- Remove re-exports: `RouterState`, `DefaultRouterState`, `RouterStateError`, `ModelRouter`, `NoopSharedContext`
- Keep: `SharedContext`, `DefaultSharedContext` (internal to standalone_inference)
- Keep: `StandaloneInferenceService`, `MultitenantInferenceService`

**`crates/server_core/src/test_utils/state.rs`**
- Remove `router_state_stub` fixture
- Add new fixture returning `Arc<dyn AppService>` directly

### Gate
```bash
cargo test -p server_core
```

---

## Phase 3: `auth_middleware` — Replace `State<Arc<dyn RouterState>>`

**`crates/auth_middleware/src/auth_middleware/auth_middleware.rs`**
- `auth_middleware()`: `State(state): State<Arc<dyn RouterState>>` → `State(app_service): State<Arc<dyn AppService>>`
- `optional_auth_middleware()`: same
- Replace `state.app_service()` → direct use of `app_service`

**`crates/auth_middleware/src/api_auth_middleware.rs`**
- `api_auth_middleware()`: change state type (param is unused `_state`)

**`crates/auth_middleware/src/access_request/access_request_middleware.rs`**
- `access_request_auth_middleware()`: change state type, replace `state.app_service()`

**Test files:**
- `test_auth_middleware.rs`, `test_live_auth_middleware.rs`, `test_access_request_middleware.rs`
- Replace `DefaultRouterState::new(MockSharedContext, app_service)` → just `app_service`

### Gate
```bash
cargo test -p auth_middleware
```

---

## Phase 4: `routes_app` — Update build_routes, AuthScope, Tests

### 4a. build_routes()

**`crates/routes_app/src/routes.rs`**
- Remove `ctx: Arc<dyn SharedContext>` parameter
- New signature: `build_routes(app_service: Arc<dyn AppService>, static_router: Option<Router>) -> Router`
- Remove `DefaultRouterState::new()` — use `app_service` directly as state
- Router type: `Router<Arc<dyn AppService>>`

### 4b. AuthScope extractor

**`crates/routes_app/src/shared/auth_scope_extractor.rs`**
- Change `FromRef` bound: `Arc<dyn RouterState>: FromRef<S>` → `Arc<dyn AppService>: FromRef<S>`
- Replace `router_state.app_service()` → direct use

### 4c. Test utilities + ~30 test files

**`crates/routes_app/src/test_utils/router.rs`** — remove ctx param
**`crates/routes_app/src/test_utils/mcp.rs`** — return `Arc<dyn AppService>`
**`crates/routes_app/src/toolsets/routes_toolsets.rs`** — change param type

All test files creating `DefaultRouterState::new(MockSharedContext, app_service)`:
- Replace with `app_service` directly
- Remove imports: `DefaultRouterState`, `MockSharedContext`, `RouterState`

### Gate
```bash
cargo test -p routes_app
```

---

## Phase 5: `server_app` — Rewire Listeners to InferenceService

### 5a. Simplify serve.rs

**`crates/server_app/src/serve.rs`**
- Remove `shared_context: Option<Arc<dyn SharedContext>>` parameter from `get_server_handle()`
- `ShutdownContextCallback` → `ShutdownInferenceCallback`: holds `Arc<dyn AppService>`, calls `app_service.inference_service().stop().await`
- Call `build_routes(app_service, static_router)` directly (no `ctx`)

### 5b. Rewire VariantChangeListener

**`crates/server_app/src/listener_variant.rs`**
- Hold `Arc<dyn InferenceService>` instead of `Arc<dyn SharedContext>`
- On variant change: call `inference_service.set_variant(new_value).await`
- Wiring in serve.rs: `VariantChangeListener::new(app_service.inference_service())`

### 5c. Remove external ServerKeepAlive

**`crates/server_app/src/listener_keep_alive.rs`**
- Delete entire file — keep-alive timer now internal to `StandaloneInferenceService`
- The `SettingsChangeListener` for `BODHI_KEEP_ALIVE_SECS` → wire a simple listener in serve.rs that calls `app_service.inference_service().set_keep_alive(new_value)`

### Gate
```bash
cargo test -p server_app
```

---

## Phase 6: `lib_bodhiserver` — Simplify build_app_service

**`crates/lib_bodhiserver/src/app_service_builder.rs`**
- `build_app_service()` return type: `Result<DefaultAppService, BootstrapError>` (remove `Option<SharedContext>` from tuple)
- `StandaloneInferenceService::new(ctx, keep_alive_secs)` — pass initial value from settings
- SharedContext created and consumed internally — never returned

**`crates/lib_bodhiserver/src/lib.rs`**
- Remove `pub use server_core::SharedContext` re-export (if present)

### Gate
```bash
cargo test -p lib_bodhiserver
```

---

## Phase 7: `bodhi/src-tauri` + `lib_bodhiserver_napi` — Update Callers

**`crates/bodhi/src-tauri/src/native_init.rs`**
- `build_app_service()` returns just `AppService`
- Remove `shared_context` from `NativeCommand::new()` and struct

**`crates/bodhi/src-tauri/src/server_init.rs`**
- Same — remove SharedContext from flow

**`crates/lib_bodhiserver_napi/src/server.rs`**
- No structural changes — `stop()` still sends shutdown signal + awaits join handle
- ShutdownInferenceCallback handles llama cleanup via InferenceService

### Gate
```bash
cargo test -p bodhi --features native
make test.backend
```

---

## Phase 8: TECHDEBT + Documentation

**Create `crates/server_core/TECHDEBT.md`:**
1. Graceful llama.cpp shutdown — currently uses `process.kill()` (SIGKILL) via Drop. Consider SIGTERM + timeout for cleaner shutdown.
2. `BodhiServer::Drop` doesn't send shutdown signal — if dropped without `stop()`, spawned server task leaks until process exit. Add shutdown signal to Drop.

---

## Architecture Summary (Before → After)

**Before:**
```
build_app_service() → (AppService, Option<SharedContext>)
SharedContext passed to → ServeCommand → build_routes() → DefaultRouterState
                       → ShutdownCallback (holds ctx)
                       → ServerKeepAlive (holds ctx)
                       → VariantChangeListener (holds ctx)
Handlers: State<Arc<dyn RouterState>> → state.app_service() or state.forward_request()
```

**After:**
```
build_app_service() → AppService (SharedContext internal to StandaloneInferenceService)
AppService passed to → ServeCommand → build_routes()
                     → ShutdownCallback (holds app_service → inference_service().stop())
                     → VariantChangeListener (holds inference_service → .set_variant())
                     → KeepAlive timer internal to StandaloneInferenceService
Handlers: State<Arc<dyn AppService>> via AuthScope extractor
```

---

## Critical Files

| File | Action |
|---|---|
| `services/src/inference/inference_service.rs` | Add stop/set_variant/set_keep_alive/is_loaded to trait |
| `services/src/inference/noop.rs` | Implement no-op lifecycle methods |
| `server_core/src/standalone_inference.rs` | Implement lifecycle methods + internal keep-alive timer |
| `server_core/src/multitenant_inference.rs` | No-op lifecycle methods |
| `server_core/src/router_state.rs` | **Delete** RouterState, DefaultRouterState |
| `server_core/src/model_router.rs` | **Delete** entire file |
| `server_core/src/noop_shared_context.rs` | **Delete** entire file |
| `server_core/src/lib.rs` | Update re-exports |
| `server_core/src/test_utils/state.rs` | Replace router_state_stub |
| `auth_middleware/src/auth_middleware/auth_middleware.rs` | State type → AppService |
| `auth_middleware/src/api_auth_middleware.rs` | State type → AppService |
| `auth_middleware/src/access_request/access_request_middleware.rs` | State type → AppService |
| `routes_app/src/routes.rs` | Remove ctx param, AppService as state |
| `routes_app/src/shared/auth_scope_extractor.rs` | Update FromRef bound |
| `routes_app/src/test_utils/router.rs` | Update test router builder |
| `routes_app/src/test_utils/mcp.rs` | Update test state builder |
| `server_app/src/serve.rs` | Rewire to InferenceService, remove SharedContext param |
| `server_app/src/listener_keep_alive.rs` | **Delete** (timer internal to InferenceService) |
| `server_app/src/listener_variant.rs` | Rewire to InferenceService |
| `lib_bodhiserver/src/app_service_builder.rs` | Return AppService only |
| `bodhi/src-tauri/src/native_init.rs` | Remove SharedContext from flow |
| `bodhi/src-tauri/src/server_init.rs` | Remove SharedContext from flow |
| ~30 test files | Replace DefaultRouterState with AppService |

## Verification
```bash
cargo test -p services
cargo test -p server_core
cargo test -p auth_middleware
cargo test -p routes_app
cargo test -p server_app
cargo test -p lib_bodhiserver
cargo test -p bodhi --features native
make test.backend
```
