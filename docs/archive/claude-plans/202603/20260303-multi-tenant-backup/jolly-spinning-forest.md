# CRUD Layer Refactor: Validation to Routes, Services Return Entities

## Context

The previous CRUD uniformity plan (20260304) standardized entity aliases, WithRejection, and column renames. This plan **reverses the architectural direction**: instead of making routes_app thin and moving logic to services, we keep domain logic in routes_app and make services focused executors.

### Layer Responsibilities

```
repository     = DB access (SeaORM queries, transactions)
services       = repository + non-DB operations + transaction boundaries → returns Entity types
auth_scoped    = thin tenant_id/user_id injection → returns Entity types
routes         = input validation, authorization, Entity→Response conversion, I/O handling
```

### Key Rules

- **routes_app MUST only interact with auth_scoped services** — never directly with domain services or db_service
- If an auth_scoped service method doesn't exist for what a route needs, add it to auth_scoped service
- **Services return Entity types** — Entity types are `pub` re-exported from services
- **AuthScoped returns Entity types** — thin injector, passes through service return types
- **Route handlers convert Entity→Response** via `.into()` before returning

### Key Architectural Decisions

| Decision | Choice |
|----------|--------|
| Form rename | `*Form` -> `*Request` (e.g., `McpForm` -> `McpRequest`) |
| Output rename | Keep current names (no `*Response` suffix rename) |
| ValidatedJson | Re-introduced, standalone extractor (no WithRejection wrapper needed) |
| ValidatedJson error | Returns `ValidationRejection`, auto-converts via `From` to `ApiError` |
| Field validation | Routes via ValidatedJson (`form.validate()`) |
| DB validation | DB constraints raise errors, bubbled up as domain errors |
| Business rule validation in services | Keep for: UserAlias file existence check. Remove for: MCP URL uniqueness (DB constraint). ApiModel API key check -> routes |
| Service validation | Services do NOT call `form.validate()`, assume validated input |
| Authorization | Two-layer: middleware = endpoint access, route handler = operation-specific checks |
| AuthScoped services | Thin injector: inject tenant_id/user_id, return Entity types |
| ExternalApp filtering | Keep in route handlers (not AuthScoped) |
| Entity exposure | Entity types are `pub` from services. Services return entities. Routes convert to Response via `.into()` |
| Serialize on Request | Keep `#[derive(Serialize)]` for test convenience |
| *_api_schemas.rs files | Keep for HTTP-only types (query params, pagination) after domain types move |
| Session coordination | Keep in routes_app (session_service is axum-specific) |
| OAuth flows | Skip MCP OAuth migration, add to TECHDEBT.md |
| OAI/Ollama conversion | Keep in routes_app, add to TECHDEBT.md (Ollama dropping soon) |
| Setup/Auth URL construction | Keep in routes_app, add to TECHDEBT.md |

### Exception: Token domain (Phase 1 — already committed)
Phase 1 was committed with services returning Response types. This is left as-is — not reverted. All subsequent phases (2-7) follow the Entity-return pattern.

---

## Execution Model

Each phase is executed by a specialized sub-agent with:
1. **Prescriptive section**: Exact changes per domain
2. **Reference**: CRUD Convention (below) + work done by previous agents
3. **Gate check**: `cargo test -p services -p routes_app -p server_app`
4. **TS regeneration**: `cargo run --package xtask openapi && cd ts-client && npm run generate && npm run build`
5. **UI fix**: `cd crates/bodhi && npm run build && npm run test` — fix any import/type errors from renames
6. **Local commit** after passing gate
7. **Stale object cleanup pass** (see Sub-Agent Instructions below)
8. **Summary** sent back to main agent for next phase

Main agent passes cumulative context to each sub-agent.

---

## CRUD Convention Reference (Updated)

### Entity Layer (pub from services)
- `pub type <Domain>Entity = Model;` — **re-exported from services** so routes_app can use for `.into()` conversion
- Standard fields: `id` (ULID), `tenant_id`, `user_id`, `created_at`/`updated_at`

### Request Type (in services/*_objs.rs)
- Renamed from `*Form` -> `*Request`
- `#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]`
- Field-level validation via `#[validate(...)]` annotations
- Excludes: `id`, `tenant_id`, `user_id`, `created_at`, `updated_at`

### Response Type (in services/*_objs.rs)
- Keep current names (no rename in this pass)
- HTTP-compliant, excludes tenant_id, user_id, encrypted fields
- Secret fields -> `has_<secret>: bool`
- `impl From<Entity> for ResponseType` conversion defined in services

### Service Layer
- NO `form.validate()` calls — input assumed validated
- **Returns Entity types** (not Response types)
- Business invariants that require service deps stay (e.g., UserAlias file existence)
- DB constraints handle uniqueness/FK violations -> mapped to domain errors

### AuthScoped Service
- Injects `tenant_id`/`user_id` from `AuthContext` only
- No validation, no authorization
- **Returns Entity types** (passes through service return types)
- **routes_app MUST use auth_scoped services exclusively** — never bypass to domain services directly

### Route Handlers
- `ValidatedJson<DomainRequest>` for JSON body extraction (standalone, no WithRejection wrapper)
- ValidatedJson calls `form.validate()` -> returns `ValidationRejection` on failure
- Route handler does operation-specific authorization checks
- **Route handler converts Entity→Response** via `.into()` before returning
- Route handler coordinates multi-service interactions
- Route handler does NOT call `require_tenant_id()`/`require_user_id()` (AuthScoped handles)

### ValidatedJson Extractor (already implemented in Phase 0)
Location: `crates/routes_app/src/shared/validated_json.rs`
Handler pattern: `ValidatedJson(form): ValidatedJson<DomainRequest>`
`ValidationRejection` implements `IntoResponse` (400 Bad Request with structured error body).

---

## Sub-Agent Instructions

Each sub-agent MUST follow this workflow:

1. **Read** all files listed in prescriptive section before making changes
2. **Apply** prescriptive changes (renames, moves, service return type changes)
3. **Gate check**: `cargo test -p services -p routes_app -p server_app`
4. **TS regeneration**: `cargo run --package xtask openapi && cd ts-client && npm run generate && npm run build`
5. **UI fix**: `cd crates/bodhi && npm run test` — fix broken imports/types from renames
6. **Gate check again** (full: `cargo test -p services -p routes_app -p server_app`)
7. **Local commit** with prescribed commit message
8. **Stale object cleanup pass**:
   - After committing, take a holistic look at ALL changed files
   - Search for stale/unused structs, types, functions, imports that can be deleted
   - Delete them, then run: gate check + TS generate + UI build + test
   - If all passes, `git commit --amend` to include cleanup
   - If cleanup breaks things, `git checkout` the cleanup changes (the domain commit is already safe)
9. **Report back**: changes made, stale objects found/removed, issues, decisions needed

---

## Phase 0: Infrastructure — ValidatedJson + TECHDEBT updates

### Sub-Agent 0: ValidatedJson extractor + TECHDEBT

**Create ValidatedJson extractor:**
- Create `crates/routes_app/src/shared/validated_json.rs`
- Implement `ValidatedJson<T>` extractor that:
  1. Deserializes JSON via `Json<T>`
  2. Calls `T::validate()` (from `validator::Validate`)
  3. Returns `ValidationRejection` on either failure
- Create `ValidationRejection` enum with `JsonRejection` and `Validation(ValidationErrors)` variants
- Implement `IntoResponse` for `ValidationRejection` -> 400 with error body
- Implement `From<ValidationRejection> for ApiError` so handlers can use `?`
- Add to `shared/mod.rs` re-exports
- Write unit tests for ValidatedJson (valid input passes, invalid field returns 400, malformed JSON returns 400)

**Update TECHDEBT.md:**
- Add: MCP OAuth flow duplication (skip migration, known anomaly)
- Add: OAI/Ollama model conversion logic (keep in routes, Ollama dropping soon)
- Add: Setup/Auth redirect URL construction (HTTP-layer, complex)
- Add: SessionService tower-sessions coupling in services crate (framework leak)
- Remove: old items about migrating types to services (reversed decision)

**Gate check**: `cargo test -p routes_app`

**Commit**: `feat: re-introduce ValidatedJson extractor, update TECHDEBT.md`

---

## Phase 1: Token Domain (simplest, split forms already)

### Sub-Agent 1: Token

**services changes (`crates/services/src/tokens/`):**

1. **Rename forms** (`token_objs.rs`):
   - `CreateTokenForm` -> `CreateTokenRequest`
   - `UpdateTokenForm` -> `UpdateTokenRequest`

2. **Service returns Response types** (`token_service.rs`):
   - `create_token(tenant_id, user_id, form)` currently returns `Result<(String, TokenEntity)>`
   - Change to return `Result<TokenCreated, TokenServiceError>` where `TokenCreated` already exists (has `token: String`)
   - But also need the created detail. Create `TokenCreateResult { token: String, detail: TokenDetail }` or have `TokenCreated` include detail fields
   - `list_api_tokens()` returns `Result<(Vec<TokenEntity>, usize)>` -> change to `Result<PaginatedTokenResponse, _>` (already exists)
   - `get_api_token_by_id()` returns `Result<Option<TokenEntity>>` -> `Result<Option<TokenDetail>, _>`
   - `update_token()` returns `Result<TokenEntity>` -> `Result<TokenDetail, _>`
   - Add `impl From<TokenEntity> for TokenDetail` conversion (may already exist)

3. **Remove form.validate()** from service create/update methods

4. **Make TokenEntity non-public**: remove from `mod.rs` re-exports (if possible without breaking AuthScoped). If AuthScoped needs it internally, keep entity pub(crate).

**routes_app changes (`crates/routes_app/src/tokens/`):**

1. **Update handler signatures** to use ValidatedJson:
   - `tokens_create`: `WithRejection<ValidatedJson<CreateTokenRequest>, JsonRejectionError>`
   - `tokens_update`: `WithRejection<ValidatedJson<UpdateTokenRequest>, JsonRejectionError>`

2. **Token scope privilege check stays in handler** (`tokens_create`):
   - Check `auth_scope.auth_context()` role vs requested scope
   - This is the two-layer auth model: middleware checks endpoint access, handler checks operation params

3. **Update imports**: `CreateTokenForm` -> `CreateTokenRequest`, etc.

**Auth-scoped changes (`auth_scoped_tokens.rs`):**
- Update method signatures to use `*Request` types
- Return Response types (from underlying service which now returns them)

**Test updates**: Update all test files in both crates for renamed types.

**TS regeneration + UI fix**: Regenerate, fix `CreateTokenForm`/`UpdateTokenForm` references in bodhi/src/.

**Gate check**: `cargo test -p services -p routes_app -p server_app`

**Commit**: `refactor: Token domain — Form->Request, ValidatedJson, services return Response`

---

## Phase 2: Settings + Setup Domain

### Sub-Agent 2: Settings + Setup

**services changes (`crates/services/src/settings/`):**

1. **Move types FROM routes_app TO services**:
   - `UpdateSettingRequest` from `routes_app/src/settings/settings_api_schemas.rs` -> `services/src/settings/setting_objs.rs` (rename to `UpdateSettingRequest` — already correct name, just moving)

2. **Move constants**:
   - `EDIT_SETTINGS_ALLOWED` from `routes_settings.rs` -> `setting_service.rs` or `setting_objs.rs`
   - `LLM_SETTINGS` from `routes_settings.rs` -> same location

3. **SettingService changes**:
   - Remove any validation from service
   - `set_setting_value()` should just set, not validate allowlist
   - Return `SettingInfo` (already does)

**routes_app changes (`crates/routes_app/src/settings/`):**

1. **settings_update handler**: Validate key in EDIT_SETTINGS_ALLOWED (import from services), check multi-tenant LLM block, then call service
2. **Update ValidatedJson** for `UpdateSettingRequest`
3. **settings_api_schemas.rs**: Remove `UpdateSettingRequest` (moved to services). Keep any HTTP-only types.

**Setup domain** (`crates/routes_app/src/setup/`):
- `SetupRequest`, `SetupResponse`, `AppInfo` — these stay in routes_app since they're HTTP/setup-flow specific and involve URL construction logic
- No changes needed for setup in this phase

**Gate check**: `cargo test -p services -p routes_app -p server_app`

**Commit**: `refactor: Settings domain — move types/constants to services, validation in routes`

---

## Phase 3: Toolset Domain

### Sub-Agent 3: Toolset

**services changes (`crates/services/src/toolsets/`):**

1. **Rename**: `ToolsetForm` -> `ToolsetRequest` (in `toolset_objs.rs`)

2. **Service keeps returning Entity types** (`tool_service.rs`):
   - Verify services return `ToolsetEntity` (or whatever current return type is)
   - Ensure `impl From<ToolsetEntity> for Toolset` exists in services for routes to use
   - Entity types must be `pub` re-exported from services

3. **Remove form.validate()** from service create/update

4. **ToolsetExecutionRequest**, **ToolsetExecutionResponse** — already in services, no rename needed

**routes_app changes (`crates/routes_app/src/toolsets/`):**

1. **Update handlers** to use `ValidatedJson<ToolsetRequest>`
2. **ExternalApp filtering** stays in `toolsets_index` handler
3. **Route handlers convert Entity→Response** via `.into()` (e.g., `entity.into()` -> `Toolset`)
4. **Update imports**

**Auth-scoped changes**: Update types, return Entity types (pass through from service).

**Gate check + TS regen + UI fix**

**Commit**: `refactor: Toolset domain — Form->Request, ValidatedJson, Entity->Response in routes`

---

## Phase 4: ApiModel Domain

### Sub-Agent 4: ApiModel

**services changes (`crates/services/src/models/`):**

1. **Rename**: `ApiModelForm` -> `ApiModelRequest` (in `model_objs.rs`)

2. **Service returns Entity types** (`api_model_service.rs`):
   - Verify services return `ApiModelEntity` (or check current return types)
   - If some methods already return Response types (e.g., `ApiModelOutput`), keep Entity returns and add `impl From<ApiModelEntity> for ApiModelOutput` for routes to use
   - Entity types must be `pub` re-exported from services

3. **Remove form.validate()** from service

4. **Move types from routes_app**: `TestCreds`, `TestPromptRequest`, `FetchModelsRequest`, `TestPromptResponse`, `FetchModelsResponse`, `ApiFormatsResponse` from `api_models_api_schemas.rs` -> `services/src/models/model_objs.rs` or new `api_model_objs.rs`

5. **ApiModel API key validation**: Move explicit api_key presence check to routes (per user decision)

**routes_app changes:**

1. **Update handlers** to use `ValidatedJson<ApiModelRequest>`
2. **Route handlers convert Entity→Response** via `.into()` (e.g., `ApiModelOutput::from(entity)`)
3. **api_models_test, api_models_fetch_models**: Use ValidatedJson for their request types too
4. **api_models_api_schemas.rs**: Remove moved types, keep HTTP-only types

**Gate check + TS regen + UI fix**

**Commit**: `refactor: ApiModel domain — Form->Request, move types, Entity->Response in routes`

---

## Phase 5: UserAlias + Download Domain

### Sub-Agent 5: UserAlias + Download

**services changes:**

1. **Rename**: `UserAliasForm` -> `UserAliasRequest` (in `model_objs.rs`)

2. **Service returns Entity types**:
   - Verify services return `UserAliasEntity` / `DownloadEntity`
   - Ensure `impl From<Entity> for Response` exists for routes to use
   - Keep file existence check in service (per user decision)

3. **Remove form.validate()** from UserAlias service methods

4. **Download service**:
   - `DownloadRequestForm` -> `DownloadRequest` (if exists as a form)
   - Already has background task spawning in routes — keep as-is

5. **Move types from routes_app models_api_schemas.rs**:
   - `CopyAliasRequest`, `RefreshSource`, `RefreshRequest`, `RefreshResponse` -> services
   - `UserAliasResponse`, `ModelAliasResponse`, `ApiAliasResponse`, `AliasResponse` -> services
   - Keep `LocalModelResponse`, `PaginatedLocalModelResponse` in routes_app (presentation)

**routes_app changes:**

1. **Update handlers** to use ValidatedJson where applicable
2. **Route handlers convert Entity→Response** via `.into()` before returning
3. **Update imports**

**Gate check + TS regen + UI fix**

**Commit**: `refactor: UserAlias + Download — Form->Request, move types, Entity->Response in routes`

---

## Phase 6: MCP Domain (largest)

### Sub-Agent 6: MCP

**services changes (`crates/services/src/mcps/`):**

1. **Rename**:
   - `McpForm` -> `McpRequest`
   - `McpServerForm` -> `McpServerRequest`

2. **Service returns Entity types**:
   - Verify services return Entity types (`McpEntity`, `McpServerEntity`, etc.)
   - Ensure `impl From<Entity> for Response` exists for each (e.g., `McpEntity -> Mcp`)
   - Entity types must be `pub` re-exported from services

3. **Remove form.validate()** from all service create/update methods

4. **MCP URL uniqueness**: Remove explicit uniqueness check in service — DB constraint handles it, map DB error to domain error

5. **Atomic server+auth config creation**: `create_mcp_server()` handles optional auth config in one transaction (per previous decision)

6. **Move types from routes_app mcps_api_schemas.rs**:
   - `McpAuth`, `FetchMcpToolsRequest`, `McpToolsResponse`, `McpExecuteRequest`, `McpExecuteResponse`, `OAuthTokenResponse` -> services
   - Keep `McpServerQuery`, `CreateAuthConfig`, `AuthConfigsQuery` in routes_app (HTTP-only)

**routes_app changes:**

1. **Update all MCP handlers** to use ValidatedJson
2. **Route handlers convert Entity→Response** via `.into()`
3. **ExternalApp filtering** stays in `mcps_index` handler
4. **mcps_create validation**: `mcp_server_id` required check stays in handler (operation-specific)
5. **MCP OAuth handlers**: Skip migration per decision (add to TECHDEBT)
6. **Update imports**

**Auth-scoped changes**: Return Entity types (pass through from service).

**Gate check + TS regen + UI fix**

**Commit**: `refactor: MCP domain — Form->Request, ValidatedJson, Entity->Response in routes`

---

## Phase 7: Users + Access Requests Domain

### Sub-Agent 7: Users + Access Requests

**services changes:**

1. **Move types from routes_app users_api_schemas.rs**:
   - `ChangeRoleRequest` -> services (domain form for role assignment)
   - `UserAccessStatusResponse` -> services (domain output)
   - `ApproveUserAccessRequest` -> services (domain form)
   - `UserAccessRequest` (the paginated item) -> services
   - Keep `TokenType`, `RoleSource`, `TokenInfo`, `ListUsersParams`, `UserResponse` in routes_app (HTTP-specific)

2. **Move types from routes_app apps_api_schemas.rs**:
   - Skip app access request review/enrichment (deferred per user decision)
   - `CreateAccessRequestBody` -> services as `CreateAccessRequest`
   - `ApproveAccessRequestBody` -> services as `ApproveAccessRequest`
   - Keep response types in routes_app for now (complex workflow, deferred)

3. **Service returns Entity types**: Verify services return Entity types, ensure `From<Entity> for Response` exists

4. **Access request approval privilege validation**: Keep in route handler (routes own authorization logic)

**routes_app changes:**

1. **Route handlers convert Entity→Response** via `.into()`
2. **users_change_role**: Authorization check stays in handler. Session clearing stays in handler.
3. **users_approve_access_request**: Privilege validation stays in handler.
4. **Update imports for moved types**

**Gate check + TS regen + UI fix**

**Commit**: `refactor: Users + Access Requests — move types, Entity->Response in routes`

---

## Phase 8: Final Cleanup + Documentation

### Sub-Agent 8: Cleanup + Docs

1. **Remove stale form.validate() calls** in any remaining service methods
2. **Verify Entity types are pub** from services (needed by routes for `.into()` conversion)
3. **Verify routes_app never bypasses auth_scoped services** — search for direct service calls
4. **Verify route handlers do Entity→Response conversion** — no direct entity returns from handlers
4. **Run full test suite**: `make test.backend`
5. **Final TS regeneration**: `cargo run --package xtask openapi && make build.ts-client`
6. **UI build + test**: `cd crates/bodhi && npm run build && npm run test`
7. **Update documentation**:
   - `crates/CLAUDE.md`: Update CRUD Convention Reference with new patterns
   - `crates/routes_app/CLAUDE.md`: Document ValidatedJson, two-layer auth model, auth_scoped-only rule
   - `crates/services/CLAUDE.md`: Document that services don't validate, return Response types
   - `crates/routes_app/TECHDEBT.md`: Updated with all deferred items

**Commit**: `docs: update CLAUDE.md files for CRUD layer refactor`

---

## Verification

After all phases:
1. `cargo test -p services -p routes_app -p server_app` — all pass
2. `cargo run --package xtask openapi` — regenerate without errors
3. `cd ts-client && npm run generate && npm run build` — TS client compiles
4. `cd crates/bodhi && npm run build && npm run test` — UI builds and tests pass
5. `make test.backend` — full backend green
6. `make test.napi` — E2E tests pass (after `make build.ui-rebuild`)

---

## Sub-Agent Prompt Template

Each sub-agent receives:
1. **Cumulative summary** from prior phases (what changed, any surprises)
2. **CRUD Convention Reference** (from this plan)
3. **Prescriptive section** (exact files, types, changes)
4. **Gate check + TS regen + UI fix commands**
5. **Commit message**

### Sub-Agent Workflow (MUST follow in order)

1. **Read** all files listed in prescriptive section before making changes
2. **Interface audit** (BEFORE making changes):
   - Review the repository layer (DB queries), domain service trait, and auth_scoped service for the domain being refactored
   - Compare method signatures across all three layers — look for: duplicate methods doing the same thing, inconsistent parameter ordering, methods that exist on one layer but not another, unused methods, methods with confusing names
   - Review how route handlers call auth_scoped services — identify any handlers that bypass auth_scoped and call domain services directly
   - Prepare a mini-plan of interface cleanups to include alongside the prescriptive changes
3. **Apply** prescriptive changes (renames, moves, service return type changes) AND interface cleanups from step 2
4. **Gate check**: `cargo test -p services -p routes_app -p server_app`
5. **TS regeneration**: `cargo run --package xtask openapi && cd ts-client && npm run generate && npm run build`
6. **UI fix**: `cd crates/bodhi && npm run test` — fix broken imports/types from renames
7. **Gate check again** (full: `cargo test -p services -p routes_app -p server_app`)
8. **Local commit** with prescribed commit message
9. **Stale object cleanup pass**:
   - After committing, take a holistic look at ALL changed files in the domain
   - Search for stale/unused structs, types, functions, imports, dead code that can be deleted
   - Delete them, then run: gate check + TS generate + UI build + test
   - If all passes, `git commit --amend` to include cleanup in the same commit
   - If cleanup breaks things, `git checkout` the cleanup changes — the domain commit is already safe
10. **Report back**: changes made, interface cleanups applied, stale objects found/removed, issues, decisions needed

### Key Rules for Sub-Agents

- **No backwards compatibility concerns**: There is no production deployment. Make clean, direct changes. No fallback shims, no re-exports for compat, no data migration worries. Rename/delete/restructure freely.
- **routes_app MUST only call auth_scoped services** (via `auth_scope.<domain>()`) — never domain services directly. If a needed method doesn't exist on auth_scoped service, add it (delegating to the underlying domain service with tenant_id/user_id injection).
- **Services return Entity types, auth_scoped returns Entity types**: Route handlers convert to Response via `.into()`.
- **Entity types must be pub from services**: So routes_app can import and use `From<Entity> for Response`.
- **Interface consistency**: When reviewing a domain, ensure repository -> service -> auth_scoped -> route handler chain has consistent naming, no duplicates, no dead methods. Clean up as part of the refactor.
- **Exception: Token domain** (Phase 1 already committed with services returning Response types — left as-is).
