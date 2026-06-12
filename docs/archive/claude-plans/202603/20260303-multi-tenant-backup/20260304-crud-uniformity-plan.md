# CRUD Uniformity — Implementation Plan

## Context

The CRUD stack across domains uses inconsistent patterns: mixed entity type alias naming (`*Row`, `*Model`, no alias), some routes use plain `Json<>` instead of `WithRejection`, validation happens in different layers, MCP entities misuse `created_by` as both audit and ownership column, and `ValidatedJson` creates confusion by duplicating validation. This refactor standardizes everything to a single uniform architecture, executed domain-by-domain with specialized sub-agents.

## Key Decisions (from interview)

| Decision | Choice |
|----------|--------|
| JSON extraction | `WithRejection<Json<T>, JsonRejectionError>` everywhere |
| `ValidatedJson` | **Delete completely** — causes confusion |
| Validation layer | Service layer only. `form.validate()?` in service create/update. Remove `.validate()` from route handlers. |
| Entity type aliases | All use `pub type <Domain>Entity = Model;` pattern. Add + migrate usages. |
| `routes_app` require_* calls | Remove from handlers. Use scoped services. Remove logging that depends on user_id. |
| MCP instance `created_by` | Rename to `user_id` (it IS user-scoped) |
| MCP OAuth token `created_by` | Rename to `user_id` (user-scoped) |
| MCP auth headers/OAuth configs `created_by` | **Remove entirely** (parent server tracks audit) |
| MCP server `created_by`/`updated_by` | Keep (audit, exposed in API) |
| `access_requests` table | Rename to `user_access_requests` |
| `AppAccessRequestRow` | Rename to `AppAccessRequest` |
| Migrations | Modify in place (no production release) |
| Stale structs | Audit during implementation — rename, document, or delete |

---

## Execution Model

Each phase is executed by a specialized sub-agent with:
1. **Prescriptive section**: Known issues to fix (specific files, lines, changes)
2. **Exploratory section**: CRUD conventions to check for unknown violations
3. **Gate check**: `cargo test -p services -p routes_app -p server_app`
4. **Local commit** after passing gate

Main agent receives a concise summary from each sub-agent and passes cumulative context to the next.

---

## CRUD Convention Reference (for all sub-agents)

### Entity Layer
- Every SeaORM entity module: `pub type <Domain>Entity = Model;`
- Standard fields: `id` (ULID), `tenant_id`, `user_id` (for user-scoped), `created_at`/`updated_at`
- Encrypted fields: `encrypted_<col>`, `<col>_salt`, `<col>_nonce`
- No `#[serde(skip_serializing)]` tricks for API hiding — use output types

### Domain Output Type
- Separate struct from entity. Contains only API-safe fields.
- Excludes: `tenant_id`, `user_id` (exception: Token exposes `user_id`)
- Secret fields → `has_<secret>: bool`
- `impl From<Entity> for OutputType` conversion

### Form (Input)
- Single form for create/update (exception: Token uses split forms)
- `#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]`
- Excludes: `id`, `tenant_id`, `user_id`, `created_at`, `updated_at`
- All field-level validation via `#[validate(...)]` annotations

### Service Layer
- `form.validate()?` as FIRST step of create/update
- All timestamps via `TimeService` (never `Utc::now()`)
- New IDs: `ulid::Ulid::new().to_string()`

### Auth-Scoped Service
- Injects `tenant_id`/`user_id` from `AuthContext`
- Route handlers NEVER call `require_tenant_id()`/`require_user_id()`

### Route Handlers
- `WithRejection<Json<Form>, JsonRejectionError>` for all JSON body extractors
- No `.validate()` calls
- No `require_tenant_id()`/`require_user_id()` calls
- Use scoped services: `auth_scope.<domain>().create(form)`

---

## Phase 1: Toolset + MCP

### Sub-Agent 1A: Toolset Domain

**Prescriptive fixes:**

1. **Entity alias** (`crates/services/src/toolsets/toolset_entity.rs`):
   - Add `pub type ToolsetEntity = Model;`
   - Global find-replace: all `toolset_entity::Model` usages → `ToolsetEntity`
   - Update `crates/services/src/toolsets/mod.rs` re-exports

2. **WithRejection** (`crates/routes_app/src/toolsets/routes_toolsets.rs`):
   - `toolsets_create` (line ~130): `Json(form): Json<ToolsetForm>` → `WithRejection(Json(form), _): WithRejection<Json<ToolsetForm>, JsonRejectionError>`
   - `toolsets_update` (line ~190): same
   - Add imports: `use axum_extra::extract::WithRejection;` and `use crate::JsonRejectionError;`

**Exploratory checks:**
- Verify no `.validate()` in route handlers (should be clean)
- Verify no `require_tenant_id()`/`require_user_id()` in handlers (should be clean — uses `auth_scope.tools()`)
- Verify `tool_service.rs` calls `form.validate()` in create (line ~333) and update (line ~408)
- Check for stale/unused structs in `toolset_objs.rs`
- Verify `entity_to_output()` properly converts entity → `Toolset` output type
- Verify no `Utc::now()` usage (should use TimeService)

**Files to read**: `toolset_entity.rs`, `toolset_objs.rs`, `tool_service.rs`, `auth_scoped_tools.rs`, `routes_toolsets.rs`, `toolsets/mod.rs`

### Sub-Agent 1B: MCP Domain (largest change)

**Prescriptive fixes:**

**Column renames — modify migrations in place:**
1. `mcps` table: column `created_by` → `user_id` (migration: `m20250101_000009_mcp_servers.rs`)
   - Also update unique index on `(tenant_id, LOWER(created_by), LOWER(slug))` → `(tenant_id, LOWER(user_id), LOWER(slug))`
   - Also update index on `created_by` → `user_id`
2. `mcp_oauth_tokens` table: column `created_by` → `user_id` (migration: `m20250101_000011_mcp_oauth.rs`)
3. `mcp_auth_headers` table: **remove** `created_by` column (migration: `m20250101_000010_mcp_auth_headers.rs`)
4. `mcp_oauth_configs` table: **remove** `created_by` column (migration: `m20250101_000011_mcp_oauth.rs`)

**Entity files:**
- `mcp_entity.rs` (line 11): `created_by: String` → `user_id: String`. SeaORM Column enum auto-derives from field name.
- `mcp_oauth_token_entity.rs` (line 20): `created_by: String` → `user_id: String`
- `mcp_auth_header_entity.rs` (line 17): remove `created_by` field entirely. Also update `McpAuthHeaderView` (line 48) — remove `created_by`.
- `mcp_oauth_config_entity.rs` (line 28): remove `created_by` field entirely. Also update `McpOAuthConfigView` (line 78) — remove `created_by`.

**Domain types** (`mcp_objs.rs`):
- `McpWithServerRow` (line 50): `created_by` → `user_id`
- `McpAuthHeader` (line 142): remove `created_by`
- `McpOAuthConfig` (line 176): remove `created_by`
- `McpOAuthToken` (line 198): `created_by` → `user_id`
- `McpAuthConfigResponse` Header variant (line 308): remove `created_by`
- `McpAuthConfigResponse` OAuth variant (line 332): remove `created_by`
- `McpAuthConfigResponse::created_by()` method (line 355): remove entirely
- `McpServer` output (line 29): keep `created_by`/`updated_by` (audit, API-visible)

**Entity type alias renames** (global find-replace):
- `McpRow` → `McpEntity` (in `mcp_entity.rs`)
- `McpServerRow` → `McpServerEntity` (in `mcp_server_entity.rs`)
- `McpAuthHeaderRow` → `McpAuthHeaderEntity` (in `mcp_auth_header_entity.rs`)
- `McpOAuthConfigRow` → `McpOAuthConfigEntity` (in `mcp_oauth_config_entity.rs`)
- `McpOAuthTokenRow` → `McpOAuthTokenEntity` (in `mcp_oauth_token_entity.rs`)
- `McpWithServerRow` → `McpWithServerEntity` (investigate if this is a query result — if so, rename anyway for consistency)

**Validate derive:**
- Add `#[derive(Validate)]` to `McpForm` (line ~414). Wire field validators:
  - `name`: `#[validate(custom(function = "validate_mcp_instance_name_validator"))]`
  - `slug`: `#[validate(custom(function = "validate_mcp_slug_validator"))]`
  - `description`: optional field validator for max length
- Add `#[derive(Validate)]` to `McpServerForm` (line ~448). Wire field validators:
  - `name`: custom validator
  - `url`: custom validator
  - `description`: optional field validator for max length
- Create wrapper functions returning `Result<(), validator::ValidationError>` (follow pattern in `toolset_objs.rs:141-146`)
- **OAuth URL validation stays manual** in service layer (parameterized field_name issue)

**Service** (`mcp_service.rs`):
- Add `form.validate().map_err(...)` to instance create/update methods
- Add `form.validate().map_err(...)` to server create/update methods
- Remove standalone validation calls now covered by Validate derive (name, slug, description, url)
- Keep manual validation for: auth config nested validation, OAuth endpoint URLs
- Update all `created_by` → `user_id` references throughout
- Remove `created_by` assignments in auth header/OAuth config creation
- **Note**: `store_oauth_token` (line ~1334) sets `tenant_id = ""` — investigate and fix if needed

**Repositories:**
- `mcp_instance_repository.rs`: `Column::CreatedBy` → `Column::UserId` (lines 69, 84, 147, 211)
- `mcp_auth_repository.rs`:
  - OAuth token queries: `Column::CreatedBy` → `Column::UserId` (lines 339, 385, 408)
  - Auth header inserts: remove `created_by` from ActiveModel
  - OAuth config inserts: remove `created_by` from ActiveModel
- `mcp_server_repository.rs`: no changes

**Route handlers:**
- `routes_mcps.rs`: `mcps_create` (line ~84), `mcps_update` (line ~149): `Json(form)` → `WithRejection(Json(form), _)`
- `routes_mcps_servers.rs`: `mcp_servers_create`, `mcp_servers_update`: same
- Add WithRejection imports

**Auth-scoped** (`auth_scoped_mcps.rs`): No structural changes (already uses `user_id` parameter name)

**Test files**: Update ALL in `crates/services/src/mcps/test_*.rs` and `crates/routes_app/src/mcps/test_*.rs`

**mod.rs re-exports**: Update `crates/services/src/mcps/mod.rs` — renamed type aliases

**Exploratory checks:**
- Check for `Utc::now()` usage
- Check for stale/unused types in `mcp_objs.rs`
- Verify `Mcp` output type doesn't expose `tenant_id` or `user_id`
- Verify all route handlers use scoped services (no `require_*` calls)
- Check `auth_scoped_mcps.rs` methods match standard CRUD signature pattern

### Sub-Agent 1C: Remove ValidatedJson

**Prescriptive:**
- Delete `crates/routes_app/src/shared/validated_json.rs`
- Remove module declaration from `crates/routes_app/src/shared/mod.rs`
- Remove re-export of `ValidatedJson` from shared module
- Remove `.validate()` calls from `routes_api_models.rs` utility handlers (lines 203-205, 280-282) — these are `TestPromptRequest` and `FetchModelsRequest` validation. Move validation to service layer or keep if these are non-CRUD utility endpoints that genuinely need route-level validation (investigate).
- Update `crates/routes_app/CLAUDE.md`: remove "ValidatedJson Extractor" section, document `WithRejection` as standard
- Check if `ObjValidationError` import is orphaned after removal

**Gate check**: `cargo test -p services -p routes_app -p server_app`

**Commit**: `refactor: CRUD uniformity — Toolset + MCP (WithRejection, Validate, entity aliases, column renames, remove ValidatedJson)`

---

## Phase 2: ApiModel + UserAlias

### Sub-Agent 2: ApiModel + UserAlias Domain

**Prescriptive fixes:**

**ApiModel entity alias:**
- `crates/services/src/models/api_model_alias_entity.rs`: Add `pub type ApiModelEntity = Model;` and migrate usages
- Note: `ApiAliasView` (DerivePartialModel) is fine — it's a query optimization

**UserAlias entity alias:**
- `crates/services/src/models/user_alias_entity.rs`: Add `pub type UserAliasEntity = Model;` and migrate usages

**Fix utility endpoint violations** (`routes_api_models.rs`):
- `api_models_test` (line 198): uses `require_tenant_id()` (line 209) and `require_user_id()` (line 210) to resolve credentials from existing API models via `TestCreds::ExistingId`. These are NOT CRUD — they're utility endpoints. Move credential resolution to a scoped service method if possible, or document as exception.
- `api_models_fetch_models` (line 275): same pattern (lines 286-287). Same approach.
- Also has `.validate()` calls (lines 203-205, 280-282) on `TestPromptRequest`/`FetchModelsRequest` — move validation to service or keep since these are non-CRUD endpoints.

**CRUD handlers** (`api_models_create`, `api_models_update`): Already use `auth_scope.api_models().create(form)` — already clean.

**Exploratory checks:**
- Verify `ApiModelOutput` is the proper output type (has `has_api_key: bool`)
- Verify `ApiModelForm` has `#[derive(Validate)]` and field-level validators
- Verify `api_model_service.rs` calls `form.validate()` in create/update
- Check `UserAlias` struct in `model_objs.rs` — is it used as output type? (It appears to be — line 526)
- Verify `UserAliasForm` has `#[derive(Validate)]`
- Check `auth_scoped_data.rs` handles UserAlias CRUD through scoped service
- Check `routes_models.rs` uses scoped services
- Check for stale structs in model domain

**Files to read**: `api_model_alias_entity.rs`, `user_alias_entity.rs`, `model_objs.rs` (ApiModelForm, ApiModelOutput, UserAlias, UserAliasForm sections), `api_model_service.rs`, `auth_scoped_api_models.rs`, `auth_scoped_data.rs`, `routes_api_models.rs`, `routes_models.rs`

**Gate check**: `cargo test -p services -p routes_app -p server_app`

**Commit**: `refactor: CRUD uniformity — ApiModel + UserAlias (entity aliases, scoped service fixes)`

---

## Phase 3: Token + Download

### Sub-Agent 3: Token + Download Domain

**Prescriptive fixes:**

**Token entity rename:**
- `crates/services/src/tokens/api_token_entity.rs` (line 29): `pub type ApiToken = Model;` → `pub type TokenEntity = Model;`
- Global find-replace: `ApiToken` (the type alias) → `TokenEntity`
- **CAUTION**: `ApiToken` may collide with other uses of the string "ApiToken". Only rename the entity type alias and its usages, not enum variants or other types named `ApiToken`.
- Add exception comments:
  - Near split forms: `// Token uses split forms because create requires immutable scope, while update only changes name/status.`
  - Near `TokenDetail.user_id`: `// user_id included because tokens are listed per-user and the UI shows ownership.`

**Download entity rename + output type:**
- `crates/services/src/models/download_request_entity.rs` (line 32): `pub type DownloadRequestModel = Model;` → `pub type DownloadEntity = Model;`
- Global find-replace
- Add exception comment: `// Downloads are tenant-wide shared resources — no user_id. Any authenticated tenant user can create/view downloads.`

**Create `Download` output type** (in `download_service.rs` or `model_objs.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Download {
  pub id: String,
  pub repo: String,
  pub filename: String,
  pub status: DownloadStatus,
  pub error: Option<String>,
  pub total_bytes: i64,
  pub downloaded_bytes: i64,
  pub started_at: Option<DateTime<Utc>>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}
```
- `impl From<DownloadEntity> for Download` conversion
- Update `PaginatedDownloadResponse` (line ~43): `Vec<DownloadRequestModel>` → `Vec<Download>`
- Update route handlers in `routes_models_pull.rs` to return `Download`
- Remove `#[serde(skip_serializing)]` on `tenant_id` from entity (no longer serialized to API)

**Fix routes_apps.rs + routes_users.rs** (require_user_id violations):
- `routes_apps.rs` `apps_approve_access_request` (line 288): Remove `let user_id = auth_scope.require_user_id()?;` and `info!()` log. Refactor `deny_request(id, user_id)` to get user_id from auth context inside scoped service.
- `routes_apps.rs` `apps_deny_access_request` (line 448): Same — remove require + logging.
- `routes_users.rs` `users_change_role` (line 73): Remove `let requester_id = auth_scope.require_user_id()?;` and `info!()` log. Already uses `auth_scope.users()`.
- `routes_users.rs` `users_destroy` (line 136): Same — remove require + logging.

**Exploratory checks:**
- Verify Token CRUD handlers already use WithRejection (they do)
- Verify Token service calls `form.validate()` — investigate (may be missing for Token)
- Verify Download service calls `form.validate()` — investigate
- Check for stale structs in token and download domains
- Verify `NewDownloadForm` has `#[derive(Validate)]`
- Check entity `#[serde(skip_serializing)]` on token entity `tenant_id` — already has proper output type so it's fine
- Verify all route handlers use scoped services

**Files to read**: `api_token_entity.rs`, `token_objs.rs`, `token_service.rs`, `auth_scoped_tokens.rs`, `routes_tokens.rs`, `download_request_entity.rs`, `download_service.rs`, `auth_scoped_downloads.rs`, `routes_models_pull.rs`, `routes_apps.rs`, `routes_users.rs`

**Gate check**: `cargo test -p services -p routes_app -p server_app`

**Commit**: `refactor: CRUD uniformity — Token + Download (entity renames, Download output type, remove require_user_id)`

---

## Phase 4: Access Requests + Remaining Aliases + Cleanup

### Sub-Agent 4: Access Requests + ModelMetadata + Documentation

**Prescriptive fixes:**

**User Access Request table rename:**
- Find migration that creates `access_requests` table — modify in place to `user_access_requests`
- `crates/services/src/users/access_request_entity.rs`: `#[sea_orm(table_name = "access_requests")]` → `user_access_requests`
- Entity alias: `pub type UserAccessRequest = Model;` → `pub type UserAccessRequestEntity = Model;`
- Global find-replace for the type alias

**App Access Request:**
- `crates/services/src/app_access_requests/app_access_request_entity.rs`: Add `pub type AppAccessRequestEntity = Model;`
- `crates/services/src/app_access_requests/access_request_objs.rs`: Rename `AppAccessRequestRow` → `AppAccessRequest`
- Global find-replace for `AppAccessRequestRow`

**ModelMetadata entity rename:**
- `crates/services/src/models/model_metadata_entity.rs` (line 43): `pub type ModelMetadataRow = Model;` → `pub type ModelMetadataEntity = Model;`
- Global find-replace

**Documentation:**
- Add to `crates/services/CLAUDE.md`:
  - `users/access_request_entity.rs` = non-role user requesting access to the app
  - `app_access_requests/` = 3rd party app requesting access to the app
- Update `crates/routes_app/CLAUDE.md`: remove `ValidatedJson Extractor` section, document `WithRejection` as standard

**Stale struct audit:**
- For each domain, check for extra structs not fitting CRUD flow
- If named wrong → rename
- If valid exception → document with comment
- If unused → delete

**Exploratory checks:**
- Verify access request route handlers don't have other CRUD violations
- Verify `UserAccessRequest` is used as API response directly or has output type
- Check `ModelMetadata` domain — internal type, not API-facing, so output type separation may not apply
- Verify no `Utc::now()` in any of these domains

**Files to read**: `access_request_entity.rs`, `app_access_request_entity.rs`, `access_request_objs.rs`, `model_metadata_entity.rs`, relevant service files, `routes_apps.rs`, `routes_users.rs`

**Gate check**: `cargo test -p services -p routes_app -p server_app`

**Commit**: `refactor: CRUD uniformity — Access Requests + remaining entity aliases + cleanup`

---

## Phase 5: Final Regeneration + Full Test

```bash
# Regenerate OpenAPI spec
cargo run --package xtask openapi

# Regenerate TypeScript client
cd ts-client && npm run generate && npm run build

# Run UI tests
cd crates/bodhi && npm run test

# Run full backend tests
make test.backend
```

**Commit**: `refactor: regenerate OpenAPI + TS client after CRUD uniformity`

---

## Sub-Agent Prompt Template

Each sub-agent receives:
1. **Cumulative summary** from prior phases (what changed, any surprises)
2. **Prescriptive section** (exact files, lines, changes as described above)
3. **Exploratory section** (CRUD Convention Reference + domain-specific checks)
4. **Gate check command**
5. **Commit message**

The sub-agent should:
- Read all files listed before making changes
- Apply prescriptive fixes first
- Then run exploratory checks, fixing any violations found
- Run gate check
- Report back: changes made, violations found/fixed, any issues needing main agent attention

---

## Verification

After all phases:
1. `cargo test -p services -p routes_app -p server_app` — all pass
2. `cargo run --package xtask openapi` — regenerate without errors
3. `cd ts-client && npm run generate && npm run build` — TS client compiles
4. `cd crates/bodhi && npm run test` — UI tests pass
5. `make test.backend` — full backend green
