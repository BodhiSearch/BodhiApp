# Multi-Tenant: DataService + InferenceService Rearchitecture

## Context

BodhiApp's multi-tenant mode incorrectly disables OAI chat completions, embeddings, and model listing routes. These routes should be available — only local GGUF model operations should be restricted. Additionally:
- `DataService` hardcodes `tenant_id=""` everywhere — not tenant-aware
- No `user_id` scoping on aliases — API models should be per-user
- `forward_request` lives on `RouterState` tied to `SharedContext` — needs to become a service
- `AiApiService::forward_request` also hardcodes `tenant_id=""`

## Decision Summary

| Topic | Decision |
|---|---|
| Route disabling | No routes disabled for multi-tenancy; remove all `is_multi_tenant` guards |
| User alias ops in multi-tenant | Return `DataServiceError::Unsupported` (not 404) |
| API alias scoping | Strictly per-user (user A cannot see user B's aliases) |
| api_models auth | Lower to user role, session-only (no tokens/exchanged JWT) |
| DataService trait | Add `tenant_id`+`user_id` params to all methods |
| DataService impl | Separate: `LocalDataService` (standalone) + `MultiTenantDataService` (API-only) |
| AuthScopedDataService | Yes, following existing auth-scoped pattern |
| Hardcoded `""` tenant_id | Remove everywhere; always use actual IDs |
| InferenceService | Single trait in services; `StandaloneInferenceService` + `MultitenantInferenceService` impls in server_core |
| InferenceService methods | `forward_local()` + `forward_remote()` — caller decides routing |
| RouterState | Keep as thin shell (only `app_service()`), remove `forward_request()` |
| OAI handlers | Add `AuthScope` extractor alongside existing params |
| DB schema | Add `user_id NOT NULL` to both tables; modify migrations in place |
| Pull/queue/metadata routes | Defer to TECHDEBT.md |
| Standalone user_id/tenant_id | Always use actual IDs from auth context (never `""`) |

---

## Execution Model

Each phase is implemented by a dedicated sub-agent. Every phase follows this workflow:

1. **Code** — Make all code changes for the phase
2. **Test** — Run `cargo test -p <crate>` for the target crate
3. **Fix** — Iterate on test failures until all pass
4. **Upstream test** — Run cumulative tests for all upstream+current crates changed so far
5. **Commit** — `git add` changed files + `git commit` with descriptive message
6. **Summary** — Return structured summary to be passed as context to the next phase's agent

---

## Phase 1: DB Schema + Entity Changes (`services`)

### Agent prompt context
> You are implementing Phase 1 of a multi-tenant rearchitecture. Add `user_id` column to `api_model_aliases` and `user_aliases` tables.

### Changes

**`crates/services/src/db/sea_migrations/m20250101_000002_api_model_aliases.rs`**
- Add `UserId` to `DeriveIden` enum
- Add `user_id STRING NOT NULL` column to table creation
- Change unique prefix index from `(tenant_id, prefix)` to `(tenant_id, user_id, prefix)`
- Add `idx_api_model_aliases_user_id` index

**`crates/services/src/db/sea_migrations/m20250101_000007_user_aliases.rs`**
- Add `UserId` to `DeriveIden` enum
- Add `user_id STRING NOT NULL` column to table creation
- Change unique alias index from `(tenant_id, alias)` to `(tenant_id, user_id, alias)`
- Add `idx_user_aliases_user_id` index

**`crates/services/src/models/api_model_alias_entity.rs`**
- Add `pub user_id: String` to `Model`
- Add `user_id` to `ApiAliasView` partial model

**`crates/services/src/models/user_alias_entity.rs`**
- Add `pub user_id: String` to `Model`

**RLS note**: `m20250101_000014_rls.rs` — no changes needed. RLS filters on `tenant_id` only; `user_id` is app-layer.

### Gate checks
```bash
cargo test -p services
```

### Commit message
`multi-tenant: add user_id column to api_model_aliases and user_aliases`

### Summary to pass forward
> Phase 1 complete. Both `api_model_aliases` and `user_aliases` tables now have `user_id NOT NULL` column. Entity models updated. Unique constraints are now `(tenant_id, user_id, prefix)` and `(tenant_id, user_id, alias)` respectively. RLS unchanged (tenant_id only).

---

## Phase 2: Repository Trait Changes (`services`)

### Agent prompt context
> Phase 1 added `user_id` column to both alias tables and entity models. Now add `user_id: &str` parameter to all repository trait methods and their implementations.

### Changes

**`crates/services/src/models/api_alias_repository.rs`** — `ApiAliasRepository` trait:
- Add `user_id: &str` to: `create_api_model_alias`, `get_api_model_alias`, `update_api_model_alias`, `delete_api_model_alias`, `list_api_model_aliases`, `get_api_key_for_alias`, `check_prefix_exists`
- `update_api_model_cache` unchanged (internal op by primary key)
- Impl on `DefaultDbService`: add `.filter(Column::UserId.eq(user_id))` to all queries, set `user_id` on creates

**`crates/services/src/models/user_alias_repository.rs`** — `UserAliasRepository` trait:
- Add `user_id: &str` to ALL methods: `create_user_alias`, `get_user_alias_by_id`, `get_user_alias_by_name`, `update_user_alias`, `delete_user_alias`, `list_user_aliases`
- Impl on `DefaultDbService`: add `user_id` filter to all queries, set `user_id` on creates

**Callers to fix** (compilation will break — fix all call sites with `""` placeholder for now):
- `crates/services/src/models/data_service.rs` — `LocalDataService` passes `""` for user_id temporarily
- `crates/services/src/ai_apis/ai_api_service.rs` — `get_api_config` passes `""` for user_id temporarily
- Any test files using repository methods directly — add `""` user_id param
- `crates/routes_app/src/api_models/routes_api_models.rs` — add `""` user_id temporarily

### Gate checks
```bash
cargo test -p services
cargo check -p server_core -p auth_middleware -p routes_app -p server_app
```

### Commit message
`multi-tenant: add user_id parameter to all alias repository methods`

### Summary to pass forward
> Phase 2 complete. Both `ApiAliasRepository` and `UserAliasRepository` traits now require `user_id: &str` on all methods (except `update_api_model_cache`). Implementations filter by `user_id`. Callers temporarily pass `""` — to be fixed in later phases. All tests pass.

---

## Phase 3: DataService Trait + Implementations (`services`)

### Agent prompt context
> Phases 1-2 added user_id to DB schema and repository methods. Now update DataService trait to accept `tenant_id` + `user_id`, create `MultiTenantDataService`, and add `Unsupported` error variant.

### Changes

**`crates/services/src/models/data_service.rs`**

Add `Unsupported` variant to `DataServiceError`:
```rust
#[error("operation not supported in current deployment mode")]
#[error_meta(error_type = ErrorType::BadRequest)]
Unsupported,
```

Update `DataService` trait — add `tenant_id` and `user_id` to all methods:
```rust
async fn list_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<Alias>>;
async fn find_alias(&self, tenant_id: &str, user_id: &str, alias: &str) -> Option<Alias>;
async fn find_user_alias(&self, tenant_id: &str, user_id: &str, alias: &str) -> Option<UserAlias>;
async fn get_user_alias_by_id(&self, tenant_id: &str, user_id: &str, id: &str) -> Option<UserAlias>;
async fn save_alias(&self, tenant_id: &str, user_id: &str, alias: &UserAlias) -> Result<()>;
async fn copy_alias(&self, tenant_id: &str, user_id: &str, id: &str, new_alias: &str) -> Result<UserAlias>;
async fn delete_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<()>;
```

Update `LocalDataService` impl — replace all hardcoded `""` with the `tenant_id`/`user_id` parameters.

**New: `crates/services/src/models/multi_tenant_data_service.rs`**
- `list_aliases` → only API aliases via `db_service.list_api_model_aliases(tenant_id, user_id)`
- `find_alias` → only API aliases via `db_service.list_api_model_aliases(tenant_id, user_id)` + `supports_model()`
- `find_user_alias`, `get_user_alias_by_id` → `None`
- `save_alias`, `copy_alias`, `delete_alias` → `Err(DataServiceError::Unsupported)`

Wire: `crates/services/src/models/mod.rs` — add `mod multi_tenant_data_service; pub use ...`

**Fix downstream callers** (compilation will break):
- `crates/server_core/src/model_router.rs` — `DataService::find_alias` now needs tenant_id + user_id. Pass `""` temporarily.
- `crates/server_core/src/router_state.rs` — `model_router()` usage of DataService
- `crates/routes_app/src/models/routes_models.rs` — all `data_service().*` calls
- `crates/routes_app/src/oai/routes_oai_models.rs` — `data_service().list_aliases()` / `find_alias()`
- `crates/routes_app/src/ollama/routes_ollama.rs` — same
- Test files across crates

### Gate checks
```bash
cargo test -p services
cargo check -p server_core -p auth_middleware -p routes_app
```

### Commit message
`multi-tenant: add tenant_id+user_id to DataService trait, create MultiTenantDataService`

### Summary to pass forward
> Phase 3 complete. DataService trait now requires `tenant_id` + `user_id` on all methods. `LocalDataService` passes them through to repositories. New `MultiTenantDataService` returns only API aliases and `Unsupported` for user alias mutations. `DataServiceError::Unsupported` variant added. Downstream callers temporarily use `""` placeholders.

---

## Phase 4: AuthScopedDataService (`services`)

### Agent prompt context
> Phases 1-3 added user_id to schema, repositories, and DataService trait. Now create `AuthScopedDataService` wrapper following the existing auth-scoped pattern (see `auth_scoped_tokens.rs`).

### Changes

**New: `crates/services/src/app_service/auth_scoped_data.rs`**

Following pattern from `auth_scoped_tokens.rs`:
```rust
pub struct AuthScopedDataService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}
```
Methods delegate to `app_service.data_service().*` injecting `tenant_id` and `user_id` from `auth_context`. Use `require_tenant_id()` and `require_user_id()` for methods that need auth, with appropriate error handling.

**`crates/services/src/app_service/auth_scoped.rs`** — add accessor:
```rust
pub fn data(&self) -> AuthScopedDataService {
  AuthScopedDataService::new(self.app_service.clone(), self.auth_context.clone())
}
```

Wire: `crates/services/src/app_service/mod.rs` — add `mod auth_scoped_data; pub use ...`

### Gate checks
```bash
cargo test -p services
```

### Commit message
`multi-tenant: add AuthScopedDataService wrapper`

### Summary to pass forward
> Phase 4 complete. `AuthScopedDataService` wraps `DataService` injecting `tenant_id` + `user_id` from `AuthContext`. Available via `auth_scope.data()` on `AuthScopedAppService`. Pattern matches existing `auth_scoped_tokens.rs`.

---

## Phase 5: InferenceService Trait + AiApiService Fix (`services`)

### Agent prompt context
> Phases 1-4 completed DataService rearchitecture. Now create `InferenceService` trait in services and fix `AiApiService` hardcoded tenant_id.

### Changes

**New: `crates/services/src/inference/mod.rs`**
**New: `crates/services/src/inference/inference_service.rs`**
**New: `crates/services/src/inference/error.rs`**

Move `LlmEndpoint` from `server_core::router_state` to `services::inference`.

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait InferenceService: Send + Sync + std::fmt::Debug {
  async fn forward_local(
    &self, endpoint: LlmEndpoint, request: Value, alias: Alias,
  ) -> Result<Response, InferenceError>;

  async fn forward_remote(
    &self, endpoint: LlmEndpoint, request: Value,
    api_alias: &ApiAlias, api_key: Option<String>,
  ) -> Result<Response, InferenceError>;
}
```

`InferenceError` variants: `Unsupported`, `ModelNotFound(String)`, transparent wraps for `ContextError` and `AiApiServiceError`.

**`crates/services/src/app_service/app_service.rs`** — add to `AppService` trait + `DefaultAppService`:
```rust
fn inference_service(&self) -> Arc<dyn InferenceService>;
```

**`crates/services/src/app_service/auth_scoped.rs`** — add passthrough:
```rust
pub fn inference(&self) -> Arc<dyn InferenceService> {
  self.app_service.inference_service()
}
```

**`crates/services/src/ai_apis/ai_api_service.rs`** — change `forward_request` to accept pre-resolved config:
```rust
async fn forward_request(
  &self, api_path: &str, api_alias: &ApiAlias,
  api_key: Option<String>, request: Value,
) -> Result<Response>;
```
Remove `get_api_config()` private method (hardcoded `""` tenant_id). The prefix stripping and HTTP forwarding logic stays.

**Fix callers**: `server_core/src/router_state.rs` — `DefaultRouterState::forward_request` must resolve alias+key before calling `ai_api_service.forward_request(...)`. Temporarily pass resolved values from model_router result.

**Test infrastructure**: Update `AppServiceStubBuilder` and `MockAppService` to include `inference_service` field. Use `MockInferenceService` for tests.

### Gate checks
```bash
cargo test -p services
cargo check -p server_core -p routes_app
```

### Commit message
`multi-tenant: add InferenceService trait, fix AiApiService hardcoded tenant_id`

### Summary to pass forward
> Phase 5 complete. `InferenceService` trait defined in `services::inference` with `forward_local()` and `forward_remote()` methods. `LlmEndpoint` moved from server_core to services. `AiApiService::forward_request` now accepts pre-resolved `ApiAlias` + `api_key` instead of looking up by id with hardcoded `""` tenant. `AppService` trait has new `inference_service()` accessor. `InferenceError` enum defined.

---

## Phase 6: InferenceService Impls + RouterState Simplification (`server_core`)

### Agent prompt context
> Phases 1-5 completed all services-layer changes. `InferenceService` trait defined in services. Now create implementations in server_core and simplify `RouterState`.

### Changes

**New: `crates/server_core/src/standalone_inference.rs`**

`StandaloneInferenceService` wraps `SharedContext`:
```rust
pub struct StandaloneInferenceService {
  ctx: Arc<dyn SharedContext>,
}
```
- `forward_local` → delegates to `self.ctx.forward_request(endpoint, request, alias)`
- `forward_remote` → HTTP proxy to `api_alias.base_url + endpoint.api_path()` with prefix stripping, api_key header, and axum→reqwest response conversion (move logic from current `DefaultRouterState::forward_request` lines 115-148)

**New: `crates/server_core/src/multitenant_inference.rs`**

`MultitenantInferenceService`:
- `forward_local` → `Err(InferenceError::Unsupported)`
- `forward_remote` → same proxy logic (share via helper function or trait default)

**`crates/server_core/src/router_state.rs`**

Remove `forward_request()` from `RouterState` trait:
```rust
pub trait RouterState: std::fmt::Debug + Send + Sync {
  fn app_service(&self) -> Arc<dyn AppService>;
}
```

`DefaultRouterState` keeps `ctx` for `stop()` method. Remove `model_router()` and `ai_api_service()` helpers. Simplify `RouterStateError`.

Re-export `LlmEndpoint` from services for backwards compat.

**`crates/server_core/src/model_router.rs`** — remove or keep as dead code (routing now in handlers via DataService + Alias match). If removing, clean up `mod.rs` and re-exports.

### Gate checks
```bash
cargo test -p services
cargo test -p server_core
```

### Commit message
`multi-tenant: add InferenceService impls, simplify RouterState`

### Summary to pass forward
> Phase 6 complete. `StandaloneInferenceService` wraps SharedContext for local inference + HTTP proxy for remote. `MultitenantInferenceService` returns Unsupported for local, proxies remote. `RouterState` simplified to thin shell (`app_service()` only). `forward_request` removed from trait. `ModelRouter` removed/deprecated.

---

## Phase 7: Route Registration + API Models Auth (`routes_app`)

### Agent prompt context
> Phases 1-6 completed services and server_core changes. Now update route registration and api_models handlers.

### Changes

**`crates/routes_app/src/routes.rs`**
- Remove `let is_multi_tenant = app_service.setting_service().is_multi_tenant().await;`
- Remove both `if !is_multi_tenant` blocks (lines 117-138 for user LLM routes, lines 316-355 for power user LLM routes)
- Merge all previously-guarded routes unconditionally into their groups
- Move api_models routes from `power_user_apis`/`power_user_session_apis` to `user_session_apis` (auth: `ResourceRole::User, None, None`)

**`crates/routes_app/src/api_models/routes_api_models.rs`**
- All handlers: add `let user_id = auth_scope.require_user_id()?;`
- Pass `user_id` to all `db.*(tenant_id, user_id, ...)` repository calls
- `spawn_cache_refresh`: capture `user_id` as owned `String` before spawning

### Gate checks
```bash
cargo test -p services
cargo test -p server_core
cargo test -p auth_middleware
cargo test -p routes_app
```

### Commit message
`multi-tenant: remove route guards, add user_id to api_models, lower auth to user role`

### Summary to pass forward
> Phase 7 complete. All `is_multi_tenant` route guards removed — all routes always registered. api_models routes moved to user role session-only auth. All api_models handlers now pass `user_id` to repository calls.

---

## Phase 8: OAI + Ollama + Models Handler Updates (`routes_app`)

### Agent prompt context
> Phases 1-7 completed. Now migrate OAI, Ollama, and models route handlers to use `AuthScope`, `AuthScopedDataService`, and `InferenceService`.

### Changes

**`crates/routes_app/src/oai/routes_oai_chat.rs`**
- `chat_completions_handler`: replace `State(state)` with `auth_scope: AuthScope`
- Resolve alias via `auth_scope.data().find_alias(model)`
- Match on alias variant: `Alias::User|Model` → `inference.forward_local()`, `Alias::Api` → resolve api_key + `inference.forward_remote()`
- Same for `embeddings_handler`

**`crates/routes_app/src/oai/routes_oai_models.rs`**
- `oai_models_handler`: replace `state.app_service().data_service().list_aliases()` with `auth_scope.data().list_aliases()`
- `oai_model_handler`: replace `state.app_service().data_service().find_alias()` with `auth_scope.data().find_alias()`
- Remove `State(state)` from both

**`crates/routes_app/src/ollama/routes_ollama.rs`**
- Replace `State(state)` with `auth_scope: AuthScope` in all handlers
- Use `auth_scope.data()` for alias operations
- Use `auth_scope.inference()` for chat forwarding

**`crates/routes_app/src/models/routes_models.rs`**
- Replace all `auth_scope.data_service().*` calls with `auth_scope.data().*`
- Applies to: `models_index`, `models_show`, `models_create`, `models_update`, `models_destroy`, `models_copy`

### Gate checks
```bash
cargo test -p services
cargo test -p server_core
cargo test -p auth_middleware
cargo test -p routes_app
```

### Commit message
`multi-tenant: migrate OAI/Ollama/models handlers to AuthScope + InferenceService`

### Summary to pass forward
> Phase 8 complete. All OAI, Ollama, and models handlers now use `AuthScope` instead of `State(state)`. Model resolution goes through `AuthScopedDataService` (tenant+user scoped). Inference forwarding uses `InferenceService`. No handlers reference `RouterState::forward_request` anymore.

---

## Phase 9: Bootstrap Wiring (`server_app` + `lib_bodhiserver` + `bodhi/src-tauri`)

### Agent prompt context
> Phases 1-8 completed all trait/handler changes. Now wire the correct DataService and InferenceService implementations during application bootstrap based on deployment mode.

### Changes

**`crates/server_app/src/`** (bootstrap code)
- Check `setting_service.is_multi_tenant()`
- Standalone: `LocalDataService` + `StandaloneInferenceService { ctx }`
- Multi-tenant: `MultiTenantDataService { db_service }` + `MultitenantInferenceService`
- Pass to `DefaultAppService::new(...)` (new `inference_service` field)

**`crates/lib_bodhiserver/src/`** — same wiring pattern

**`crates/bodhi/src-tauri/`** — same wiring pattern (always standalone)

`DefaultRouterState::new(ctx, app_service)` — `ctx` still needed for `stop()`.

### Gate checks
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

### Commit message
`multi-tenant: wire InferenceService + DataService impls in bootstrap`

### Summary to pass forward
> Phase 9 complete. Application bootstrap selects correct DataService and InferenceService implementations based on deployment mode. All backend tests pass.

---

## Phase 10: TypeScript Client + Frontend + TECHDEBT

### Agent prompt context
> Phases 1-9 completed all backend changes. Now regenerate TypeScript types, fix frontend compilation, and add TECHDEBT entries.

### Changes

```bash
cargo run --package xtask openapi
make build.ts-client
```

- Fix any frontend compilation errors from API type changes
- `cd crates/bodhi && npm test` — fix any test failures

**TECHDEBT additions** to `ai-docs/claude-plans/20260303-multi-tenant/TECHDEBT.md`:
1. Model pull/metadata refresh/queue status routes — return Unsupported in multi-tenant
2. Rename `LocalDataService` → `StandaloneModelService`, `MultiTenantDataService` → `MultitenantModelService`
3. Update D10 decision — routes always registered, behavior varies by impl

### Gate checks
```bash
make test.backend
make build.ts-client
cd crates/bodhi && npm test
```

### Commit message
`multi-tenant: regenerate TS types, update TECHDEBT`

### Summary to pass forward
> Phase 10 complete. TypeScript client regenerated. Frontend compiles. TECHDEBT updated. Full rearchitecture complete.

---

## Critical Files Summary

| File | Change |
|---|---|
| `services/src/db/sea_migrations/m20250101_000002_api_model_aliases.rs` | Add user_id column |
| `services/src/db/sea_migrations/m20250101_000007_user_aliases.rs` | Add user_id column |
| `services/src/models/api_model_alias_entity.rs` | Add user_id field |
| `services/src/models/user_alias_entity.rs` | Add user_id field |
| `services/src/models/api_alias_repository.rs` | Add user_id param to all methods |
| `services/src/models/user_alias_repository.rs` | Add user_id param to all methods |
| `services/src/models/data_service.rs` | Add tenant_id+user_id, add Unsupported variant |
| **NEW** `services/src/models/multi_tenant_data_service.rs` | API-only DataService impl |
| **NEW** `services/src/app_service/auth_scoped_data.rs` | AuthScopedDataService |
| `services/src/app_service/auth_scoped.rs` | Add `data()` + `inference()` accessors |
| `services/src/app_service/app_service.rs` | Add `inference_service()` to trait |
| **NEW** `services/src/inference/mod.rs` | InferenceService trait + LlmEndpoint |
| **NEW** `services/src/inference/error.rs` | InferenceError enum |
| `services/src/ai_apis/ai_api_service.rs` | Change forward_request to accept resolved config |
| **NEW** `server_core/src/standalone_inference.rs` | StandaloneInferenceService |
| **NEW** `server_core/src/multitenant_inference.rs` | MultitenantInferenceService |
| `server_core/src/router_state.rs` | Remove forward_request, thin shell |
| `routes_app/src/routes.rs` | Remove is_multi_tenant guards, move api_models auth |
| `routes_app/src/oai/routes_oai_chat.rs` | Add AuthScope, use InferenceService |
| `routes_app/src/oai/routes_oai_models.rs` | Use AuthScopedDataService |
| `routes_app/src/ollama/routes_ollama.rs` | Add AuthScope, use InferenceService |
| `routes_app/src/models/routes_models.rs` | Use AuthScopedDataService |
| `routes_app/src/api_models/routes_api_models.rs` | Add user_id, lower auth to user role |
