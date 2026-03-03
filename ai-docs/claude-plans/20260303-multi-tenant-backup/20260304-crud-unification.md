# CRUD Unification — 2.5-Type Pattern Implementation Plan

## Context

The codebase has an explosion of types per domain (up to 7 types for simple CRUD). Route handlers contain business logic (ID generation, timestamps, field mapping, side effects) that belongs in services. Some auth-scoped services and handlers bypass domain services and call DbService directly. This plan establishes a uniform CRUD pattern across all domains, reducing types and centralizing logic.

Reference docs:
- `ai-docs/claude-plans/20260303-multi-tenant/crud-unification-plan.md` — Design decisions
- `ai-docs/claude-plans/20260303-multi-tenant/db-service-report.md` — DB access audit
- `ai-docs/claude-plans/20260303-multi-tenant/TECHDEBT.md` — Deferred items

## The 2.5-Type Pattern

Per domain, 3 types max (2 for domains without secrets):

| Type | Name | Location |
|------|------|----------|
| DB Entity | `<entity>::Model` | `services/src/<domain>/` |
| Input Form | `<Domain>Form` | `services/src/<domain>/<domain>_schemas.rs` |
| Output | `<Domain>` | `services/src/<domain>/<domain>_schemas.rs` |

Rules:
- Form: `#[derive(Deserialize, Validate, ToSchema)]`, used as `Json<DomainForm>` in handlers
- Output: `#[derive(Serialize, ToSchema)]`, returned as `Json<Domain>` from handlers
- `*Row` types eliminated. Create/Update share same Form (PUT). ID from path on update.
- Asymmetric CRUD (Token): separate forms
- Service validates Form internally, generates ULID + timestamps, returns composed output
- Auth-scoped wrappers kept. Forms + `ApiKeyUpdate` + `ApiKey` live in services crate.

## Execution Model

Each phase is executed by a **specialized sub-agent** in sequence:

```
Main Agent
  ├─ Spawn Agent Phase N with: task description + cumulative summary from phases 1..N-1
  ├─ Agent: implements code → runs tests → gate check → local commit → returns summary
  ├─ Main Agent: consolidates summary, appends to cumulative context
  └─ Spawn Agent Phase N+1 with updated cumulative summary
```

Each agent receives:
1. **Task/Goal**: What to implement in this phase
2. **References**: Files to read, patterns to follow
3. **Cumulative Summary**: What previous agents changed (type renames, file moves, new patterns)
4. **Gate Check**: `cargo test -p services -p routes_app -p server_app` must pass
5. **Commit**: Create a local commit with descriptive message
6. **Return**: Summary of all changes made (files created/moved/deleted, types added/removed, patterns established)

---

## Phase 1: Fix Existing Bypasses

**Goal**: Fix code that bypasses existing domain services. Validates the pattern before creating new services.

**Sub-tasks**:
1. `crates/services/src/app_service/auth_scoped_data.rs:112` — `update_alias()` should use `data_service()` not `db_service()`. Verify `DataService` trait has the method.
2. `crates/services/src/app_service/auth_scoped_mcps.rs:205` — `delete_oauth_token()` should use `mcp_service()` not `db_service()`. Verify `McpService` trait has the method.
3. `crates/routes_app/src/users/routes_users_access_request.rs` — All 6 `auth_scope.db()` calls should route through `AccessRequestService`. May need `AuthScopedAccessRequestService` + `.access_requests()` accessor on `AuthScopedAppService`.

**References**:
- `crates/services/src/app_service/auth_scoped_mcps.rs` — pattern for other auth-scoped services
- `crates/services/src/app_service/auth_scoped_tokens.rs` — reference for thin wrapper pattern
- `crates/services/src/app_access_requests/` — existing AccessRequestService

**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: fix service bypasses in auth_scoped_data, auth_scoped_mcps, access request handlers`

---

## Phase 2: ApiModel (Reference Implementation)

**Goal**: Implement the full 2.5-type pattern on ApiModel as the reference. All subsequent phases follow patterns established here.

**Sub-tasks**:

### 2a. Module restructure
- Create `crates/services/src/models/api/` directory
- Move `api_model_alias_entity.rs` → `models/api/api_alias_entity.rs`
- Move `api_alias_repository.rs` (from `crates/services/src/db/`) → `models/api/api_alias_repository.rs`
- Create `models/shared.rs` — move shared types from `model_objs.rs`: `Alias` enum, `AliasSource`, `ModelAlias`, `Repo`, `HubFile`, `JsonVec`, `OAIRequestParams`, `DownloadStatus`
- Update all `mod.rs` declarations and re-exports

### 2b. Type consolidation
- Merge `ApiKeyUpdateAction` (routes_app) into `ApiKeyUpdate` (services) with serde annotations: `#[serde(tag = "action", content = "value", rename_all = "lowercase")]`. Keep → validation error on create. Set(Option<String>).
- Move `ApiKey` wrapper from routes_app to services (custom deserializer, length validation)
- Create `api_model_schemas.rs`:
  - `ApiModelForm`: `api_format`, `base_url`, `api_key: ApiKeyUpdate`, `models`, `prefix`, `forward_all_with_prefix`. Derives `Deserialize, Validate, ToSchema`.
  - `ApiModel` (output): `id`, `api_format`, `base_url`, `has_api_key: bool`, `models`, `prefix`, `forward_all_with_prefix`, `created_at`, `updated_at`. Derives `Serialize, ToSchema`.

### 2c. Service layer
- Create `api_model_service.rs`: `ApiModelService` trait with `create`, `update`, `delete`, `get`, `list`, `test_prompt`, `fetch_models`, `sync_cache`
- Create `error.rs`: `ApiModelServiceError`
- Service validates Form, generates ULID + timestamps via TimeService, calls repository, returns `ApiModel`
- Create `AuthScopedApiModelService`, add `.api_models()` to `AuthScopedAppService`
- Register in `AppService` trait + `DefaultAppService`

### 2d. Handler refactor
- `crates/routes_app/src/api_models/routes_api_models.rs` — replace `auth_scope.db()` with `auth_scope.api_models().*`
- Handlers become: extract `Json<ApiModelForm>` → call service → return `Json<ApiModel>`
- Remove `spawn_cache_refresh` (TECHDEBT item)
- Delete from routes_app: `CreateApiModelRequest`, `UpdateApiModelRequest`, `ApiModelResponse`, `PaginatedApiModelResponse`, `ApiKey`, `ApiKeyUpdateAction`, `mask_api_key()`

### 2e. Test updates
- Update handler tests to use new Form types
- Update OpenAPI spec: `cargo run --package xtask openapi`

**References**:
- `crates/services/src/tokens/token_service.rs` — existing service pattern (create_token handles ULID + hash)
- `crates/services/src/mcps/mcp_service.rs` — service with Arc<dyn AppService> internally
- `crates/routes_app/src/api_models/` — current handler implementations to refactor

**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: implement 2.5-type pattern for ApiModel domain`

---

## Phase 3: Download

**Goal**: Apply pattern to simplest domain (no secrets, no derived fields).

**Sub-tasks**:
- Create `crates/services/src/models/download/` module
- Move `download_request_entity.rs` and `download_repository.rs`
- Add `#[serde(skip_serializing)]` on entity `tenant_id` — entity IS the response (no separate output type needed)
- Create `NewDownloadForm` in `download_schemas.rs` (just `repo`, `filename`)
- Create `DownloadService` trait: `create`, `get`, `list`, `update_status`
- Create `AuthScopedDownloadService`
- Refactor `routes_models_pull.rs`, delete `DownloadRequestResponse` from routes_app

**References**: Phase 2 patterns + cumulative summary from agent
**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: implement 2.5-type pattern for Download domain`

---

## Phase 4: Toolset

**Goal**: Apply pattern to domain with secrets (similar to ApiModel).

**Sub-tasks**:
- Drop `ToolsetRow` — use `toolset_entity::Model` directly
- Create `ToolsetForm` in `toolset_schemas.rs`
- Existing `Toolset` output already has `has_api_key` — keep as output type
- Create/enhance `ToolsetService`, `AuthScopedToolService`
- Refactor toolset route handlers

**References**: Phase 2 patterns (ApiModel has same secret handling)
**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: implement 2.5-type pattern for Toolset domain`

---

## Phase 5: MCP

**Goal**: Apply pattern to most complex domain (nested objects, sub-entities, OAuth).

**Sub-tasks**:
- Drop `McpRow`, `McpServerRow`, `McpWithServerRow`
- Merge `CreateMcpRequest`/`UpdateMcpRequest` → `McpForm`
- Drop `McpResponse` — use `Mcp` directly (change date fields from String to DateTime<Utc>)
- Apply same to MCP Server (`McpServerForm`), Auth sub-entities
- Service returns composed `Mcp` with nested `McpServerInfo`
- Refactor all MCP route handlers

**References**: Phase 2 patterns + MCP-specific nested composition
**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: implement 2.5-type pattern for MCP domain`

---

## Phase 6: Token

**Goal**: Apply pattern to asymmetric CRUD domain.

**Sub-tasks**:
- Drop `ApiTokenRow` — use `ApiToken`/`entity::Model` directly
- Keep separate `CreateTokenForm` / `UpdateTokenForm` (scope is immutable, status doesn't exist at creation)
- `TokenCreated { token: String }` replaces `ApiTokenResponse`
- `TokenDetail` stays as output type
- Refactor token route handlers

**References**: Phase 2 patterns, noting asymmetric exception
**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: implement 2.5-type pattern for Token domain`

---

## Phase 7: UserAlias

**Goal**: Apply pattern to hybrid domain (file-based YAML + DB).

**Sub-tasks**:
- Create `crates/services/src/models/user/` module
- Move user alias entity + repository
- Create `UserModelService` (hybrid file + DB)
- Refactor user alias route handlers

**References**: Phase 2 patterns + DataService (existing hybrid service)
**Gate**:
1. `cargo test -p services -p routes_app -p server_app`
2. `cargo run --package xtask openapi && make build.ts-client`
3. `cd crates/bodhi && npm run test:all` — fix any TS type breakage

**Commit**: `refactor: implement 2.5-type pattern for UserAlias domain`

---

## Final Verification

After all phases complete:
1. `cargo test -p services -p routes_app -p server_app` — all tests pass
2. `cargo run --package xtask openapi` — regenerate OpenAPI spec
3. `make build.ts-client` — TypeScript client regenerates
4. `grep -r "auth_scope.db()" crates/routes_app/src/` — no direct DB access in handlers (except routes_dev.rs)
5. `grep -r "Row {" crates/services/src/` — no remaining Row types
