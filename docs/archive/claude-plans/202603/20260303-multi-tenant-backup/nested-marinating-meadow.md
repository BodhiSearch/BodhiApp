# Multi-Tenant Backend Implementation Plan

## Context

BodhiApp currently operates as a single-tenant application. All data tables are either global or scoped by `user_id`. The `apps` table has a single row with the OAuth client registration. To support SaaS deployment (`BODHI_DEPLOYMENT=multi`), the app needs tenant isolation: every resource belongs to a tenant identified by `tenant_id`, and services scope all queries by `(tenant_id, user_id)`.

This plan makes the backend multi-tenant capable while ensuring standalone mode continues to work identically (unified schema, one tenant row). Frontend changes are out of scope - tracked in context docs for future work. Multi-tenant auth flow (platform login, tenant selection) is also deferred - this plan focuses on schema, services, and isolation.

**Key research docs:**
- `ai-docs/claude-plans/20260303-multi-tenant/claude-research-multi-tenant.md`
- `ai-docs/claude-plans/20260303-multi-tenant/claude-research-multi-tenancy-research-corpus.md`

---

## Decision Record

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Rename `apps` table → `tenants`, `AppInstance` → `Tenant`, `AppInstanceService` → `TenantService` | Clear domain language: "app" = global deployment, "tenant" = scoped resource |
| D2 | Add `id` (ULID) as new PK to tenants table. `client_id` becomes unique index | Decouples DB identity from Keycloak client string. ULID matches codebase convention |
| D3 | `tenant_id` FK on EVERY data table (industry standard) | RLS without JOINs, defense-in-depth, simpler queries. Nile/Supabase/Citus recommend this |
| D4 | Unified schema for both modes | Standalone has one tenant row. Same SeaORM entities, same query patterns. Migration standalone→multi is trivial |
| D5 | `BODHI_DEPLOYMENT` setting: `standalone` (default) or `multi` | Runtime deployment mode differentiation |
| D6 | `BODHI_MULTITENANT_CLIENT_ID` env var for platform client | Error if standalone. Error if not set when multi. Configurable per deployment |
| D7 | App-layer filtering via auth-scoped services + PG RLS defense-in-depth | SeaORM doesn't have RLS support. App-layer works with SQLite too. RLS as final phase |
| D8 | `tenant_id` added to all AuthContext variants | Middleware resolves client_id → tenant_id. Single lookup, cached |
| D9 | Settings table stays global permanently | If per-tenant settings needed, create separate `tenant_settings` table |
| D10 | Conditional route registration + feature flag for LLM features in multi mode | Both belt and suspenders: routes not registered + service-level guards |
| D11 | Keep `AppStatus` enum name as-is | Widely used across codebase. Rename would have large blast radius for little benefit |
| D12 | Always store `encrypted_client_secret` in tenants table | Needed for token exchange in both modes |
| D13 | Defer slug/tier columns, session-based tenant routing (not URL path) | Active tenant stored in cookie. Slug not needed for routing |
| D14 | Modify existing CREATE TABLE migrations in place | No production deployments exist |
| D15 | External provisioning for multi-tenant tenants initially | No admin API. Tenants created via scripts/migration. Focus on making app multi-tenant ready |
| D16 | Backend-only in this plan | Frontend tasks collected in context docs |
| D17 | Defer cache externalization (Redis) | In-memory cache sufficient for now. Sticky sessions via Cloudflare if needed |
| D18 | Generate tenant_id during setup flow for standalone | Setup creates tenants row with ULID id |
| D19 | Foundation-up phasing | Schema first, then AuthContext, then services, then features, then RLS |
| D20 | Remove seed logic (toolset_configs seeding) entirely | Seeding was temporary. With multi-tenancy, seeds need tenant_id. Remove instead of adapting. Mark failing tests as `#[ignore]` |

---

## Context Documentation (DONE)

Context docs created in `ai-docs/claude-plans/20260303-multi-tenant/`:

| File | Content |
|------|---------|
| `index.md` | Summary index with progressive disclosure |
| `decisions.md` | All decisions D1-D20 with full rationale and alternatives considered |
| `settings-analysis.md` | Every BODHI_* setting categorized (global/LLM/editable/dead-in-multi) |
| `table-analysis.md` | All 15 tables: current schema, tenant_id changes, unique index adjustments, repository changes |
| `frontend-tasks.md` | Deferred frontend work items with priorities and dependencies |
| `auth-flow-analysis.md` | Current auth flow, proposed changes, deferred two-phase flow |

## Implementation: Each Phase by Sub-Agent

Each phase is designed to be executed by a specialized sub-agent. The sub-agent receives this plan + relevant context docs and implements the phase end-to-end, verifying with tests before completing.

---

## Phase 1: Tenants Table + Deployment Mode

**Sub-agent**: general-purpose agent with full tool access
**Prompt context**: This plan file + `decisions.md` + `table-analysis.md` (tenants table section)
**Goal**: Rename `apps` → `tenants`, add ULID PK, introduce deployment mode settings. Ensure standalone still works.
**Exit criteria**: `make test.backend` passes. All references to `AppInstance`/`AppInstanceService` renamed.

**Crate order**: `services` → `auth_middleware` → `routes_app` → `server_app` → `lib_bodhiserver` → `bodhi/src-tauri`

### 1.1 Migration: Rename apps → tenants

**File**: `crates/services/src/db/sea_migrations/m20250101_000013_apps.rs`
- Rename table from `apps` to `tenants`
- Add `id` (TEXT, PK) column — ULID generated by app
- Demote `client_id` from PK to unique index
- Keep: `encrypted_client_secret`, `salt_client_secret`, `nonce_client_secret`, `app_status`, `created_at`, `updated_at`

### 1.2 Rename entity, repository, service, error, domain object

**Files in `crates/services/src/apps/`**:
- `app_instance_entity.rs` → `tenant_entity.rs` — table_name="tenants", add `id` field as PK
- `app_instance_repository.rs` → `tenant_repository.rs` — `AppInstanceRepository` → `TenantRepository`
  - Add `get_tenant_by_client_id(client_id) → Option<TenantRow>` for middleware lookup
- `app_instance_service.rs` → `tenant_service.rs` — `AppInstanceService` → `TenantService`, `DefaultAppInstanceService` → `DefaultTenantService`
  - `create_instance()` → `create_tenant()` — generates ULID `id` internally
  - Add `get_tenant_by_client_id()` method
- `app_objs.rs` — `AppInstance` → `Tenant` struct, add `id: String` field. Keep `AppStatus` as-is
- `error.rs` — `AppInstanceError` → `TenantError`
- `mod.rs` — Update module declarations and re-exports

### 1.3 Update DbService super-trait

**File**: `crates/services/src/db/service.rs`
- `AppInstanceRepository` → `TenantRepository` in trait bounds

**File**: `crates/services/src/db/default_service.rs`
- Update `reset_all_tables` to reference `tenants` not `apps`
- Update `DefaultDbService` impl

### 1.4 Update AppService trait

**File**: `crates/services/src/app_service/app_service.rs`
- `app_instance_service()` → `tenant_service()` returning `Arc<dyn TenantService>`

**File**: `crates/services/src/app_service/auth_scoped.rs`
- `app_instance()` → `tenant()` accessor

### 1.5 Add deployment mode settings

**File**: `crates/services/src/settings/constants.rs`
- Add `BODHI_MULTITENANT_CLIENT_ID` constant

**File**: `crates/services/src/settings/setting_service.rs` (trait + impl)
- Add `is_multi_tenant(&self) -> bool` method
- Add `multitenant_client_id(&self) -> Result<String, SettingError>` — errors if standalone or not set

### 1.6 Update all downstream consumers

Key files across crates (grep for `app_instance`, `AppInstance`, `AppInstanceService`):
- `crates/auth_middleware/src/auth_middleware/middleware.rs` — `app_instance_service` → `tenant_service`
- `crates/auth_middleware/src/token_service/service.rs` — field and method renames
- `crates/auth_middleware/src/utils.rs` — `app_status_or_default`
- `crates/routes_app/src/setup/routes_setup.rs` — setup handler
- `crates/routes_app/src/auth/` — OAuth flows
- `crates/server_app/`, `crates/lib_bodhiserver/`, `crates/bodhi/src-tauri/` — service construction

### 1.7 Update test infrastructure

- `crates/services/src/test_utils/app.rs` — `AppServiceStubBuilder` uses TenantService
- All test files referencing AppInstance/AppInstanceService
- Test factories create `Tenant` not `AppInstance`

### Verification
```
cargo test -p services
cargo test -p auth_middleware
cargo test -p routes_app
cargo test -p server_app
make test.backend
```

---

## Phase 2: AuthContext + Tenant ID Resolution

**Sub-agent**: general-purpose agent with full tool access
**Prompt context**: This plan file + `auth-flow-analysis.md`
**Goal**: Add `tenant_id` to AuthContext. Middleware resolves `client_id → tenant_id`.
**Exit criteria**: `make test.backend` passes. AuthContext carries tenant_id in all variants. Middleware resolves it.

### 2.1 Add tenant_id to AuthContext

**File**: `crates/services/src/auth/auth_context.rs`
- Add `tenant_id: Option<String>` to all 4 variants
- Add `tenant_id() -> Option<&str>` accessor
- Add `require_tenant_id() -> Result<&str, AuthContextError>` accessor
- Add `AuthContextError::MissingTenantId` variant

### 2.2 Update AuthScopedAppService

**File**: `crates/services/src/app_service/auth_scoped.rs`
- Add `require_tenant_id()` and `tenant_id()` delegation methods

### 2.3 Update auth middleware - resolve tenant_id

**File**: `crates/auth_middleware/src/auth_middleware/middleware.rs`
- After getting tenant instance, set `tenant_id: instance.id.clone()` in AuthContext

**File**: `crates/auth_middleware/src/token_service/service.rs`
- For ApiToken: resolve tenant → set `tenant_id: tenant.id`
- For ExternalApp: resolve tenant → set `tenant_id: tenant.id`
- Update `CachedExchangeResult` to include `tenant_id`

### 2.4 Update all AuthContext construction sites

- Every `AuthContext::Session { ... }`, `AuthContext::ApiToken { ... }`, etc. needs `tenant_id` field
- Test factories (`test_session`, `test_api_token`, `test_external_app`) need default tenant_id
- `RequestAuthContextExt` in test utilities

### Verification
```
cargo test -p services
cargo test -p auth_middleware
cargo test -p routes_app
make test.backend
```

---

## Phase 3: Schema Migration (All Tables Get tenant_id)

**Sub-agent**: general-purpose agent with full tool access
**Prompt context**: This plan file + `table-analysis.md` (full table-by-table reference)
**Goal**: Add `tenant_id` FK to all 13 data tables (excluding `tenants` itself and `settings`).
**Exit criteria**: `make test.backend` passes. All tables have tenant_id. All repositories accept and filter by tenant_id. Seed logic removed.

### 3.1 Modify CREATE TABLE migrations in place

Add `tenant_id TEXT NOT NULL` + FK to `tenants(id)` + index to each migration file:

| Migration File | Tables |
|---|---|
| `m20250101_000001_download_requests.rs` | download_requests |
| `m20250101_000002_api_model_aliases.rs` | api_model_aliases |
| `m20250101_000003_model_metadata.rs` | model_metadata |
| `m20250101_000004_access_requests.rs` | access_requests |
| `m20250101_000005_api_tokens.rs` | api_tokens |
| `m20250101_000006_toolsets.rs` | toolsets, app_toolset_configs |
| `m20250101_000007_user_aliases.rs` | user_aliases |
| `m20250101_000008_app_access_requests.rs` | app_access_requests |
| `m20250101_000009_mcp_servers.rs` | mcp_servers, mcps |
| `m20250101_000010_mcp_auth_headers.rs` | mcp_auth_headers |
| `m20250101_000011_mcp_oauth.rs` | mcp_oauth_configs, mcp_oauth_tokens |

For each: add column, FK constraint, index. Update unique indexes to be composite with `tenant_id` where appropriate.

### 3.2 Update SeaORM entity files

Add `tenant_id: String` field to every entity model. Add `Relation` to Tenants entity.

Entity files across domain modules: `tokens/`, `toolsets/`, `models/`, `users/`, `app_access_requests/`, `mcps/`

### 3.3 Update repository trait methods

Add `tenant_id: &str` parameter to all CRUD methods in all repository traits:
- `TokenRepository` — `crates/services/src/tokens/token_repository.rs`
- `ToolsetRepository` — `crates/services/src/toolsets/`
- `McpServerRepository`, `McpInstanceRepository`, `McpAuthRepository` — `crates/services/src/mcps/`
- `DownloadRepository`, `ApiAliasRepository`, `ModelMetadataRepository`, `UserAliasRepository` — `crates/services/src/models/`
- `AccessRepository` — `crates/services/src/users/`
- `AccessRequestRepository` — `crates/services/src/app_access_requests/`

Each `DefaultDbService` impl adds `.filter(Column::TenantId.eq(tenant_id))` to queries and sets `tenant_id` on inserts.

### 3.4 Remove seed logic

**REMOVE** `seed_toolset_configs` and any similar seed methods entirely. Seeding was temporary (Decision D20).
- Delete the seed function from `DefaultDbService` or wherever it lives
- Remove calls to seed functions from service initialization / migration
- Mark any tests that fail due to seed removal as `#[ignore]` with comment: `// TODO: adapt for tenant-scoped toolset config creation`

### 3.5 Update test infrastructure

- `TestDbService` / `SeaTestContext` — create a default tenant row during migrate, expose tenant_id
- `AppServiceStubBuilder` — create tenant during build
- All repository test files — pass tenant_id to all operations

### Verification
```
cargo test -p services  # Largest test surface
make test.backend
```

---

## Phase 4: Service Scoping by Tenant ID

**Sub-agent**: general-purpose agent with full tool access
**Prompt context**: This plan file + `auth-flow-analysis.md`
**Goal**: Auth-scoped services extract `tenant_id` from AuthContext and pass to all underlying service/repository calls.
**Exit criteria**: `make test.backend` passes. All auth-scoped services pass tenant_id. Standalone mode works identically.

### 4.1 Update auth-scoped services

Each method extracts `tenant_id` via `self.auth_context.require_tenant_id()?` and passes through:

- `crates/services/src/app_service/auth_scoped_tokens.rs`
- `crates/services/src/app_service/auth_scoped_mcps.rs`
- `crates/services/src/app_service/auth_scoped_tools.rs`
- `crates/services/src/app_service/auth_scoped_users.rs`

### 4.2 Update service trait signatures

Add `tenant_id: &str` to all methods in:
- `TokenService` — `crates/services/src/tokens/token_service.rs`
- `McpService` — `crates/services/src/mcps/mcp_service.rs`
- `ToolService` — `crates/services/src/toolsets/tool_service.rs`
- Other service traits with data operations

### 4.3 Update route handlers with direct service calls

Search `crates/routes_app/src/` for handlers that call services directly (not via auth-scoped wrappers). These need explicit `tenant_id` passing.

### 4.4 Standalone verification

Verify standalone mode works identically: one tenant, all data has same tenant_id, behavior unchanged.

### Verification
```
cargo test -p services
cargo test -p routes_app
make test.backend
```

---

## Phase 5: Deployment Mode Feature Gating

**Sub-agent**: general-purpose agent with full tool access
**Prompt context**: This plan file + `settings-analysis.md`
**Goal**: Disable LLM features when `BODHI_DEPLOYMENT=multi`.
**Exit criteria**: `make test.backend` passes. LLM routes not registered in multi mode. Service guards return 501. Settings edit guarded.

### 5.1 Conditional route registration

**File**: `crates/routes_app/src/routes.rs`
- Split LLM routes into separate group
- Only register LLM routes when `is_multi_tenant() == false`
- LLM routes: OAI chat/completions, Ollama endpoints, model pull/delete, downloads, metadata refresh

### 5.2 Service-level guards

Add deployment mode checks in service implementations:
- `DataService` — model management methods
- `HubService` — HuggingFace download methods
- `QueueProducer` — metadata extraction

Return `ErrorType::NotImplemented` (HTTP 501) with descriptive message.

### 5.3 Settings restrictions

In multi-tenant mode, LLM-related settings (BODHI_EXEC_VARIANT, BODHI_LLAMACPP_ARGS) are not editable. Guard the settings edit endpoint.

### Verification
```
cargo test -p routes_app
make test.backend
```

---

## Phase 6: PostgreSQL RLS

**Sub-agent**: general-purpose agent with full tool access
**Prompt context**: This plan file + `table-analysis.md` + research corpus (§5 PostgreSQL RLS)
**Goal**: Add PG RLS policies as defense-in-depth for tenant isolation.
**Exit criteria**: `make test.backend` passes. RLS policies on all tenant-scoped tables. Integration tests verify cross-tenant isolation.

### 6.1 New migration for RLS

New migration file (PG-only, skip on SQLite):
- Create `bodhi_app_user` role (NOLOGIN, limited permissions)
- Create `bodhi_admin` role (BYPASSRLS, for migrations)
- Enable RLS on all 13 tenant-scoped tables
- Create `current_tenant_id()` function reading `app.current_tenant_id` session var
- Create isolation policy per table using `(SELECT current_tenant_id())` for initPlan optimization

### 6.2 SET LOCAL wrapper

**File**: `crates/services/src/db/default_service.rs`
- Add `with_tenant_context(tenant_id, closure)` utility
- Wraps queries in transaction with `SET LOCAL app.current_tenant_id = ?`
- No-op on SQLite

### 6.3 Integration tests

PG-specific tests verifying:
- Tenant A cannot see Tenant B's data
- Cross-tenant insert rejected by WITH CHECK
- No tenant set → zero rows (fail-closed)
- FORCE ROW LEVEL SECURITY applies to table owner

### Verification
```
make test.backend  # Includes PG tests via Docker
```

---

## Frontend Tasks (Deferred - tracked in context docs)

1. Hide LLM-related UI sections (models, chat, downloads) when deployment=multi
2. Tenant selector dropdown for multi-tenant mode
3. Two-phase auth flow UI (platform login → tenant selection)
4. Setup flow updates for Tenant terminology
5. Error handling for 501 (feature disabled) responses
6. TypeScript client regeneration after API changes
7. App info endpoint changes (include deployment mode, tenant_id)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Phase 1 rename touches dozens of files across 6+ crates | High churn | Follow layered dev methodology strictly. `cargo check -p <crate>` after each |
| Phase 3 adds tenant_id param to every repository method | Breaks all callers and tests | Mechanical change, do crate-by-crate |
| AuthContext variant changes break all pattern matches | Compilation failures everywhere | Use `..` in patterns where tenant_id not needed. Update factories first |
| Unique index changes (composite with tenant_id) | Behavioral change | Audit all unique indexes, make composite |
| RLS performance overhead | Query slowdown | Proper indexes on tenant_id. RLS is final phase, can defer |
