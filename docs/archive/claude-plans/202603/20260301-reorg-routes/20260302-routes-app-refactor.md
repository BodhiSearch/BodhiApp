# Plan: routes_app Crate Consistency Refactoring

## Context

The `services` crate was recently reorganized by domain — `objs` eliminated, DB repositories/entities distributed to domain folders, types co-located with services. The reviews-index.md findings (105 items) have been largely completed. The `routes_app` crate now needs the same treatment: consistent naming, file organization, error handling, validation, and a user-scoped service facade that pushes business logic from routes into services (enabling future Cloudflare Workers deployment).

No backwards compatibility or data migration required — no production release exists.

## Architecture Decisions (from user interview)

| Decision | Choice |
|---|---|
| User-scoped service | `state.app_service().for_auth(ctx).mcps().list()` — domain sub-services |
| Admin operations | Same `for_auth()` path, services check role internally |
| Scoped service type | Concrete structs wrapping `Arc<dyn XService>` + AuthContext |
| Scoped service coverage | **Every domain uniformly** — even thin ones (settings, ping) |
| Scoped service location | `services/src/app_service/` module (alongside AppService) |
| AuthContext location | Move to `services` crate; `auth_middleware` imports from it |
| AuthContext + Anonymous | Keep `AuthContext::Anonymous` variant. Add `require_user_id() -> Result<&str, ApiError>` (403 for Anonymous). Services requiring user call `require_user_id()?`, others like /ping ignore and process. |
| Auth middleware | Keep `api_auth_middleware` as defense-in-depth |
| Validation | Hand-rolled `ValidatedJson<T>` extractor (~30 lines), returns `ApiError` envelope |
| Persistence validation | Services layer only |
| Folder naming | Drop `routes_` prefix: `tokens/`, `mcps/`, `settings/` |
| Handler naming | Rails-style, no suffix: `tokens_index`, `tokens_create`, `tokens_destroy` |
| Error handling | One error enum per domain in `error.rs` |
| Schema file | `<domain>_api_schemas.rs` per domain module |
| Shared DTOs | Owned by concept domain; `api_dto.rs` eliminated |
| Endpoint constants | All in `shared/openapi.rs` |
| Test files | Sibling `test_<domain>_<concern>.rs` (Pattern A default, Pattern B for auth from mod.rs) |
| OAI/Ollama | Same treatment as app routes |
| Eager/lazy fetch | Lazy — ExternalApp access_request fetched on demand |
| Token forwarding | Auth service special case; facade reads AppInstanceService for client creds |

---

## Current State (verified 2026-03-02)

### Services crate — post-reorg, reviews-index.md largely complete
- Domain modules: `auth/`, `tokens/`, `mcps/`, `toolsets/`, `models/`, `users/`, `settings/`, `apps/`, `app_access_requests/`, `ai_apis/`
- Each domain has `error.rs` (except `tokens/` — Finding #24 not yet fixed)
- `token.rs` orphan at crate root: **FIXED** (moved to `shared_objs/token.rs` + `tokens/token_objs.rs`)
- `test_utils/objs.rs` → `test_utils/fixtures.rs`: **FIXED**
- `use super::*` eliminated from all services test files: **FIXED**
- ErrorType has PartialEq/Clone/Eq derives: **FIXED**

### Remaining db/ backward-compat shims (pre-req for this plan)
- `db/objs.rs` still exists (contains `ApiKeyUpdate` only)
- `db/mod.rs` re-exports via `pub use objs::*`
- `lib.rs` declares `pub mod db;` (not `pub use db::*;`) — so types only reachable via `services::db::*` path
- **auth_middleware**: 24 occurrences of `services::db::*` paths (`DbService`, `DbError`, `DefaultTimeService`)
- **routes_app**: 8+ occurrences of `services::db::*` paths (`DbError`, `DbService`, `TimeService`, `ApiKeyUpdate`)

### AuthContext (current — `auth_middleware/src/auth_context.rs`)
```rust
pub enum AuthContext {
  Anonymous,
  Session { user_id: String, username: String, role: Option<ResourceRole>, token: String },
  ApiToken { user_id: String, role: TokenScope, token: String },
  ExternalApp { user_id: String, role: Option<UserScope>, token: String, external_app_token: String, app_client_id: String, access_request_id: Option<String> },
}
```
- `user_id()` → `Option<&str>` (None for Anonymous)
- 30+ handler call sites use `.user_id().expect("requires auth middleware")`
- Username only on Session; ApiToken/ExternalApp lack it
- Role types differ per variant, unified via `app_role() -> Option<AppRole>`

### Routes_app current state (what needs changing)
- 12 domain folders all prefixed `routes_*` (e.g., `routes_api_token/`, `routes_mcp/`)
- 3 standalone files (`routes_ping.rs`, `routes_dev.rs`, `routes_proxy.rs`)
- `api_dto.rs` at root with shared paginated response types
- `shared/` with `constants.rs` (API tags), `openapi.rs`, `pagination.rs`, `utils.rs`, `common.rs`
- All imports already migrated from `objs::` to `services::`
- `use super::*` still present in some test modules (routes_ping, routes_dev, routes_toolsets/types)

---

## Phase 0: Pre-requisites

### 0.1 Fix `services::db::*` stale import paths

**Why**: Downstream crates use `services::db::DbService`, `services::db::DefaultTimeService`, etc. These need to use direct paths (`services::DbService`, etc.) before we can clean up the db module and before route refactoring begins.

**Steps**:
1. Add `pub use db::*;` to `services/src/lib.rs` (makes `services::DbService` etc. reachable)
2. Update all 24 occurrences in `auth_middleware` — change `services::db::DbService` → `services::DbService`, `services::db::DefaultTimeService` → `services::DefaultTimeService`, etc.
3. Update all 8+ occurrences in `routes_app` — same treatment
4. Move `ApiKeyUpdate` from `db/objs.rs` to `models/model_objs.rs` (where it conceptually belongs — API model alias updates)
5. Delete `db/objs.rs`, remove `pub use objs::*` from `db/mod.rs`
6. Optionally: change `pub mod db` → `mod db` in `lib.rs` with `pub use db::*;` re-export (matches other domain modules)

**Files**:
- `crates/services/src/lib.rs`
- `crates/services/src/db/mod.rs`, `crates/services/src/db/objs.rs`
- `crates/services/src/models/model_objs.rs`
- `crates/auth_middleware/src/access_request_auth_middleware/middleware.rs`
- `crates/auth_middleware/src/token_service/tests.rs`
- `crates/auth_middleware/src/auth_middleware/tests.rs`
- `crates/auth_middleware/tests/test_live_auth_middleware.rs`
- `crates/routes_app/src/routes_models/pull.rs`
- `crates/routes_app/src/routes_mcp/mcps.rs`
- `crates/routes_app/src/routes_api_models/api_models.rs`, `types.rs`, `test_types.rs`
- `crates/routes_app/src/routes_toolsets/toolsets.rs`, `types.rs`
- `crates/routes_app/src/routes_apps/test_access_request.rs`
- `crates/routes_app/src/routes_auth/test_login_initiate.rs`
- `crates/routes_app/src/routes_users/test_access_request_admin.rs`

**Test gate**: `cargo test -p services && cargo test -p auth_middleware && cargo test -p routes_app`

### 0.2 Create `TokenServiceError` in `tokens/error.rs`

**Why**: Only remaining reviews-index.md finding (#24). `tokens/` module currently returns raw `DbError`.

**Steps**:
- Create `crates/services/src/tokens/error.rs` with `TokenServiceError` enum
- Update `token_service.rs` and `token_repository.rs` to use it
- Add `pub use error::*` to `tokens/mod.rs`

**Test gate**: `cargo test -p services`

---

## Phase 1: Infrastructure

### 1.1 Move AuthContext to Services + add require_user_id()

**Why**: `AuthScopedAppService` needs AuthContext in same crate as AppService.

**Design**: Keep AuthContext enum structure unchanged (Anonymous, Session, ApiToken, ExternalApp). Add `require_user_id() -> Result<&str, ApiError>` that returns 403 for Anonymous. This replaces all `.user_id().expect(...)` calls with proper error handling.

**Steps**:
1. Move `AuthContext` from `auth_middleware/src/auth_context.rs` to `crates/services/src/auth/auth_context.rs`
2. Add `require_user_id()` method — returns `Ok(&str)` for authenticated variants, `Err(ApiError)` with 403 for Anonymous
3. Update `crates/services/src/auth/mod.rs` — add module + re-exports
4. Update `crates/auth_middleware/src/auth_context.rs` — replace definition with `pub use services::AuthContext;`
5. Keep `RequestAuthContextExt` and test factory functions in auth_middleware (they depend on axum types)
6. Update all 30+ handler call sites: `.user_id().expect(...)` → `.require_user_id()?`

**Files**:
- `crates/services/src/auth/auth_context.rs` (new — moved from auth_middleware)
- `crates/services/src/auth/mod.rs`
- `crates/auth_middleware/src/auth_context.rs` (becomes re-export shim)
- `crates/auth_middleware/src/lib.rs`
- All routes_app handler files using `.user_id().expect(...)`

**Test gate**: `cargo test -p services && cargo test -p auth_middleware && cargo test -p routes_app`

### 1.2 Expand app_service.rs → Module Directory + AuthScopedAppService

**Why**: The `for_auth(ctx)` pattern needs a home alongside AppService. Every domain gets a uniform scoped service.

**Steps**:
- Rename `crates/services/src/app_service.rs` → `crates/services/src/app_service/app_service.rs`
- Create `crates/services/src/app_service/mod.rs` — `mod app_service; mod auth_scoped; pub use app_service::*; pub use auth_scoped::*;`
- Create `crates/services/src/app_service/auth_scoped.rs` — `AuthScopedAppService` struct

```rust
pub struct AuthScopedAppService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedAppService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self { ... }
  pub fn require_user_id(&self) -> Result<&str, ApiError> { self.auth_context.require_user_id() }
  pub fn auth_context(&self) -> &AuthContext { &self.auth_context }
  // Domain sub-service accessors added incrementally per Phase 3 module migrations
}
```

- Add default method on `AppService` trait:
  ```rust
  fn for_auth(&self, auth_context: AuthContext) -> AuthScopedAppService;
  ```

**Note**: Domain sub-services (`AuthScopedTokenService`, `AuthScopedSettingsService`, etc.) are built incrementally — each Phase 3 module migration creates its scoped service file in `app_service/`. All domains get one, even thin ones.

**Test gate**: `cargo test -p services`

### 1.3 Create ValidatedJson Extractor

**Why**: Replace inconsistent `WithRejection<Json<T>, JsonRejectionError>` + manual `validate()` calls.

**Steps**:
- Create `crates/routes_app/src/shared/validated_json.rs` — `ValidatedJson<T>` implementing `FromRequest`
  - Calls `Json::<T>::from_request()` then `value.validate()`
  - Maps both `JsonRejection` and `ValidationErrors` to `ApiError` envelope
- Update `crates/routes_app/src/shared/mod.rs` — add `mod validated_json; pub use validated_json::*;`

**Test gate**: `cargo test -p routes_app`

### 1.4 Consolidate Endpoint Constants

**Why**: Currently scattered across `routes_mcp/mod.rs` (7), `routes_oai/mod.rs` (3), `routes_ollama/mod.rs` (3), `routes_apps/handlers.rs` (5), plus hardcoded strings in `routes.rs`.

**Steps**:
- Move all 18 scattered constants to `shared/openapi.rs`
- Remove from source locations, eliminate hardcoded path strings in `routes.rs`

**Test gate**: `cargo test -p routes_app`

---

## Phase 2: Reference Implementation — `tokens/`

Refactor `routes_api_token/` → `tokens/` as the canonical example.

### Current → Target

```
routes_api_token/                    tokens/
  mod.rs                               mod.rs              — declarations + re-exports only
  route_api_token.rs (all inline)  →   error.rs            — TokenRouteError
                                       routes_tokens.rs    — tokens_index, tokens_create, tokens_update
                                       tokens_api_schemas.rs — request/response types
  test_api_token_crud.rs           →   test_tokens_crud.rs
  test_api_token_security.rs       →   test_tokens_security.rs
  test_api_token_auth.rs           →   test_tokens_auth.rs (Pattern B from mod.rs)
```

### Steps

1. Create `AuthScopedTokenService` in `services/src/app_service/auth_scoped_tokens.rs`
   - Wraps `Arc<dyn AppService>` + AuthContext
   - Methods: `index()`, `create()`, `update()` — pre-inject user_id via `require_user_id()?`, validate privilege escalation
   - Add `fn tokens(&self) -> AuthScopedTokenService` on `AuthScopedAppService`

2. Create `tokens/` directory with new file layout per target above

3. Rename handlers: `list_tokens_handler` → `tokens_index`, `create_token_handler` → `tokens_create`, `update_token_handler` → `tokens_update`

4. Keep existing `operation_id` values in `#[utoipa::path]` annotations (preserves TypeScript client)

5. Use `ValidatedJson<CreateApiTokenRequest>` replacing `WithRejection<Json<...>, JsonRejectionError>`

6. Wire `for_auth(ctx).tokens()` in handlers

7. Move `PaginatedApiTokenResponse` from `api_dto.rs` to `tokens/tokens_api_schemas.rs`

8. Update `lib.rs`, `routes.rs`, `shared/openapi.rs`

**Error code**: `api_token_error-*` → `token_route_error-*` (no production release, rename freely)

**Test gate**: `cargo test -p services && cargo test -p routes_app`

---

## Phase 3: Domain-by-Domain Migration

Apply `tokens/` pattern to each module. Each gets its own commit after `cargo test -p routes_app`.

### Migration Order

| # | Current → Target | Key Changes |
|---|---|---|
| 1 | `routes_api_token/` → `tokens/` | Phase 2 reference |
| 2 | `routes_settings/` → `settings/` | Extract inline error+types |
| 3 | `routes_setup/` → `setup/` | Extract inline error+types |
| 4 | `routes_toolsets/` → `toolsets/` | Rename files, consolidate error |
| 5 | `routes_mcp/` → `mcps/` | Largest, 4 handler files → split |
| 6 | `routes_models/` → `models/` | Consolidate 4 error enums → 1 |
| 7 | `routes_users/` → `users/` | Consolidate 2 error enums → 1 |
| 8 | `routes_apps/` → `apps/` | Rename handlers+files |
| 9 | `routes_auth/` → `auth/` | Consolidate LoginError+LogoutError |
| 10 | `routes_api_models/` → `api_models/` | Extract error enum |
| 11 | `routes_oai/` → `oai/` | Rename, same treatment |
| 12 | `routes_ollama/` → `ollama/` | Rename, add error.rs |

### Target File Layout (per domain module)

```
<domain>/
  mod.rs                      — declarations + pub use re-exports only
  error.rs                    — single <Domain>RouteError enum
  routes_<domain>.rs          — handler functions (< 500 lines)
  routes_<domain>_<feat>.rs   — split handler file (if > 500 lines)
  <domain>_api_schemas.rs     — request/response types
  test_<domain>_<concern>.rs  — test files
```

### Handler Naming Convention

Standard CRUD (Rails-style, no `_handler` suffix):
- `<domain>_index` — list (GET collection)
- `<domain>_show` — get one (GET item)
- `<domain>_create` — create (POST)
- `<domain>_update` — update (PUT/PATCH)
- `<domain>_destroy` — delete (DELETE)

Non-CRUD use descriptive names:
- `toolsets_execute`, `mcps_fetch_tools`, `mcps_execute_tool`
- `auth_initiate`, `auth_callback`, `auth_logout`
- `mcp_oauth_login`, `mcp_oauth_token_exchange`
- `users_change_role`, `user_access_approve`

### Migration Checklist (per module)

1. Create `AuthScoped<Domain>Service` in `services/src/app_service/` (**every domain**, even thin ones)
2. Create new folder with domain name
3. Create `error.rs` — single consolidated error enum
4. Create `<domain>_api_schemas.rs` — request/response types with `#[derive(Validate)]` where applicable
5. Create `routes_<domain>.rs` — renamed handlers using Rails-style naming
6. Create `mod.rs` — declarations + re-exports only
7. Use `ValidatedJson<T>` for requests
8. Wire `for_auth(ctx).<domain>()` in handlers — use `require_user_id()?` (403 if anonymous)
9. Move/rename test files to `test_<domain>_<concern>.rs`
10. Move domain-specific DTOs from `api_dto.rs`
11. Update `lib.rs`, `routes.rs`, `shared/openapi.rs`
12. Replace `use super::*` with explicit imports in test files
13. `cargo test -p routes_app` — gate check
14. Commit

### Standalone Files (unchanged)

- `routes_ping.rs` — system-level, too small for folder
- `routes_dev.rs` — dev-only
- `routes_proxy.rs` — pure utility

---

## Phase 4: Final Cleanup

1. Delete `api_dto.rs` — all types distributed to domain `_api_schemas.rs` files
2. Clean `lib.rs` — update all module declarations
3. Verify `routes.rs` — all handler references use new names
4. Verify `shared/openapi.rs` — all `__path_*` imports updated
5. Update CLAUDE.md / PACKAGE.md for `routes_app`, `services`, `auth_middleware`

---

## Key Files

| File | Role |
|---|---|
| `services/src/auth/auth_context.rs` | **New**: AuthContext moved from auth_middleware + `require_user_id()` |
| `services/src/app_service/` | Expand from single file to module: `app_service.rs`, `auth_scoped.rs`, `auth_scoped_*.rs` |
| `auth_middleware/src/auth_context.rs` | Becomes re-export shim + test utilities |
| `routes_app/src/shared/validated_json.rs` | **New**: ValidatedJson extractor |
| `routes_app/src/shared/openapi.rs` | All endpoint constants consolidated here |
| `routes_app/src/lib.rs` | Module declarations updated per migration |
| `routes_app/src/routes.rs` | Handler references updated per migration |
| `routes_app/src/api_dto.rs` | Eliminated in Phase 4 |

## Implementation Mechanism: Sub-Agent Pipeline

Each work unit is executed by a specialized sub-agent in isolation. The orchestrator (main agent) launches sub-agents sequentially, each completing a full cycle before the next begins.

### Sub-Agent Lifecycle

Each sub-agent receives a self-contained task description and executes this pipeline:

```
1. Code implement    — write/modify source files
2. Test implement    — write/update test files
3. Test run          — cargo test -p <crate(s)>
4. Test pass         — fix failures, re-run until green
5. Code format       — cargo fmt --all
6. Lint check        — cargo clippy -p <crate(s)>
7. Local commit      — git add + commit with descriptive message
8. Return            — report status to orchestrator
```

If a sub-agent cannot achieve green tests after reasonable effort, it reports the failure and the orchestrator decides whether to fix forward or adjust the plan.

### Work Unit Breakdown

Each row is one sub-agent invocation. Sub-agents run sequentially (each depends on the previous commit).

| # | Work Unit | Crates Tested | Description |
|---|---|---|---|
| **Phase 0** | | | |
| 0.1 | db:: stale import paths | services, auth_middleware, routes_app | Fix `services::db::*` → `services::*`, move `ApiKeyUpdate`, delete `db/objs.rs` |
| 0.2 | TokenServiceError | services | Create `tokens/error.rs` with `TokenServiceError` |
| **Phase 1** | | | |
| 1.1 | Move AuthContext to services | services, auth_middleware, routes_app | Move enum to `auth/auth_context.rs`, add `require_user_id()`, update 30+ handler call sites |
| 1.2 | AuthScopedAppService | services | Expand `app_service.rs` → module, create `auth_scoped.rs`, add `for_auth()` |
| 1.3 | ValidatedJson extractor | routes_app | Create `shared/validated_json.rs` |
| 1.4 | Endpoint constants | routes_app | Consolidate scattered constants to `shared/openapi.rs` |
| **Phase 2** | | | |
| 2.1 | Reference: tokens/ | services, routes_app | Full migration of `routes_api_token/` → `tokens/` with `AuthScopedTokenService` |
| **Phase 3** (one sub-agent per module) | | | |
| 3.1 | settings/ | services, routes_app | `routes_settings/` → `settings/` |
| 3.2 | setup/ | services, routes_app | `routes_setup/` → `setup/` |
| 3.3 | toolsets/ | services, routes_app | `routes_toolsets/` → `toolsets/` |
| 3.4 | mcps/ | services, routes_app | `routes_mcp/` → `mcps/` (largest) |
| 3.5 | models/ | services, routes_app | `routes_models/` → `models/` |
| 3.6 | users/ | services, routes_app | `routes_users/` → `users/` |
| 3.7 | apps/ | services, routes_app | `routes_apps/` → `apps/` |
| 3.8 | auth/ | services, routes_app | `routes_auth/` → `auth/` |
| 3.9 | api_models/ | services, routes_app | `routes_api_models/` → `api_models/` |
| 3.10 | oai/ | services, routes_app | `routes_oai/` → `oai/` |
| 3.11 | ollama/ | services, routes_app | `routes_ollama/` → `ollama/` |
| **Phase 4** | | | |
| 4.1 | Final cleanup | all backend | Delete `api_dto.rs`, clean `lib.rs`/`routes.rs`/`openapi.rs`, run `make test.backend` |
| 4.2 | Documentation | — | Update CLAUDE.md/PACKAGE.md for routes_app, services, auth_middleware |

**Total: ~19 sub-agent invocations**

### Sub-Agent Prompt Template

Each sub-agent receives:
1. **This plan file** — for architectural context and conventions
2. **Task-specific instructions** — which work unit to execute, files to modify, target structure
3. **Previous commit SHA** — to verify starting point
4. **Test gate command** — exact `cargo test` command(s) to run
5. **Reference example** — for Phase 3 modules, point to the completed `tokens/` as the canonical example

### Orchestrator Responsibilities

The main agent:
- Launches sub-agents sequentially
- Verifies each sub-agent's commit exists and tests pass
- If a sub-agent reports failure, diagnoses and either retries or adjusts the plan
- After Phase 3 completion, runs full backend validation (`make test.backend`)
- After Phase 4, runs full verification suite (OpenAPI regen, ts-client build, UI tests)

---

## Verification

After each module migration:
```bash
cargo test -p routes_app
```

After all migrations:
```bash
make test.backend
cargo run --package xtask openapi
make build.ts-client
cd crates/bodhi && npm run test
```

## Risks & Mitigation

| Risk | Mitigation |
|---|---|
| Error code changes | No production release; rename freely. Or use `#[error_meta(code = "...")]` per-variant. |
| OpenAPI operation_id changes | Keep explicit `operation_id = "..."` in `#[utoipa::path]` unchanged. Only `__path_*` symbols change. |
| Large merge conflicts | Per-module commits, each passes cargo test. |
| Re-export breakage | Flat `pub use module::*` in `lib.rs` preserves symbol accessibility. |
| AuthContext move breakage | Phase 1.1 touches auth_middleware + all routes_app handlers. Do as single focused commit with comprehensive test gate. |
| Anonymous 403 regressions | `require_user_id()` returns clear 403 with error code. Defense-in-depth: auth middleware still runs upstream. |
