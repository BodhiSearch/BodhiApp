# Multi-Tenant RLS — Functional Plan

**Milestone**: Multi-tenant data isolation for BodhiApp
**HEAD commit**: `3d10659e7` — "RLS first cut"
**Date**: 2026-03-03

---

## A. Data Isolation (RLS + tenant_id filtering)

### Schema Changes
- `tenant_id TEXT NOT NULL` added to all 14 data tables via migration `m20250101_000014_rls.rs`
- Composite unique indexes scoped to tenant + user where applicable:
  - `UNIQUE(tenant_id, user_id, prefix)` on `api_model_aliases`
  - `UNIQUE(tenant_id, token_prefix)` on `api_tokens`
  - Similar tenant-scoped uniqueness on other tables

### Transaction-Level Isolation
- `begin_tenant_txn(tenant_id)` on `DbCore`:
  - **PostgreSQL**: executes `SET LOCAL app.current_tenant_id = '<tenant_id>'` within transaction, enabling RLS policies
  - **SQLite**: plain transaction begin (no RLS support, relies on application-level filtering)
- RLS policies on PostgreSQL for SELECT/UPDATE/DELETE — rows filtered by `app.current_tenant_id` setting

### Repository Layer
- All repository methods accept `tenant_id: &str` as first parameter
- Queries filter by `tenant_id` column in WHERE clauses
- Write operations set `tenant_id` on new rows

### Key Files
- `crates/services/src/db/sea_migrations/m20250101_000014_rls.rs`
- `crates/services/src/db/db_core.rs` — `begin_tenant_txn()`
- `crates/services/src/db/test_rls.rs`

---

## B. User-Scoped Aliases

- `user_id TEXT NOT NULL` added to `api_model_aliases` and `user_aliases`
- Unique indexes include `user_id`: `UNIQUE(tenant_id, user_id, prefix)`
- Repository methods accept `user_id: &str` parameter for alias operations
- API models route access lowered from PowerUser to User role — users manage their own model aliases

---

## C. Token Identity & Auth

### Token Format
- Format: `bodhiapp_<random>.<client_id>` — client_id embedded as token suffix
- Bearer validation flow:
  1. Parse `client_id` from token suffix (after last `.`)
  2. `get_tenant_by_client_id(client_id)` — look up tenant
  3. Build `AuthContext::ApiToken` with resolved `tenant_id`

### TenantService
- `TenantService` trait replaces `AppInstanceService`
- Methods: `get_tenant()`, `get_tenant_by_client_id()`, `get_standalone_app()`
- Used by auth middleware to resolve tenant context from bearer tokens

### AuthContext
- All `AuthContext` variants carry `tenant_id`
- Methods: `require_tenant_id()`, `tenant_id()`, `require_user_id()`, `user_id()`

### Key Files
- `crates/routes_app/src/middleware/token_service/token_service.rs`
- `crates/routes_app/src/middleware/auth/auth_middleware.rs`

---

## D. Auth-Scoped Services (tenant/user injection)

Auth-scoped service wrappers inject `tenant_id` and `user_id` from `AuthContext` into underlying service calls:

| Auth-Scoped Wrapper | Underlying Service | Scope |
|---|---|---|
| `AuthScopedDataService` | `DataService` | tenant_id + user_id |
| `AuthScopedApiModelService` | `ApiModelService` | tenant_id + user_id |
| `AuthScopedDownloadService` | `DownloadService` | tenant_id |
| `AuthScopedUserAccessRequestService` | `UserAccessRequestService` | tenant_id |
| `AuthScopedTokenService` | `TokenService` | tenant_id |
| `AuthScopedMcpService` | `McpService` | tenant_id + user_id |
| `AuthScopedToolService` | `ToolsetService` | tenant_id + user_id |
| `AuthScopedUserService` | `UserService` | tenant_id |

**Scope rules:**
- Read ops: `unwrap_or("")` for anonymous/unauthenticated access
- Write ops: `require_tenant_id()` / `require_user_id()` — returns error if missing

### Key Files
- `crates/services/src/app_service/auth_scoped_*.rs`

---

## E. Inference Routing (standalone vs multi-tenant)

### InferenceService Trait
- Defined in `crates/services/src/inference/`
- Methods: `forward_local()`, `forward_remote()`, `stop()`, `set_variant()`, `set_keep_alive()`, `is_loaded()`

### Implementations
- **`StandaloneInferenceService`** (server_core): wraps `SharedContext` for local LLM process management, includes keep-alive timer
- **`MultitenantInferenceService`** (server_core): stateless, `forward_remote()` only, `forward_local()` returns `Unsupported` error

### Architectural Changes
- `RouterState` removed entirely; `Arc<dyn AppService>` used as Axum state directly
- `SharedContext` wrapped by `StandaloneInferenceService` (still public in `server_core`, constructed by `app_service_builder`)
- `server_core` moved from regular dependency to dev-dependency in `server_app`

### Bootstrap
- Decision in `AppServiceBuilder::build()`:
  - Multi-tenant mode → `MultitenantInferenceService`
  - Standalone mode → `StandaloneInferenceService`

### Key Files
- `crates/services/src/inference/` — trait + error types
- `crates/server_core/src/standalone_inference.rs`
- `crates/server_core/src/multitenant_inference.rs`
- `crates/lib_bodhiserver/src/app_service_builder.rs`

---

## F. New Domain Services

### ApiModelService
- Trait + `DefaultApiModelService` — CRUD for API model configurations
- Manages remote API provider configs (endpoint, API key, model mappings)

### DownloadService
- Trait + `DefaultDownloadService` — download request lifecycle
- Manages HuggingFace model download requests and status tracking

### DataService Enhancements
- New methods: `create_alias_from_form()`, `update_alias_from_form()`, `copy_alias()`
- Unified interface for user alias + API alias management

### MultiTenantDataService
- Restricted variant of DataService for multi-tenant deployments
- API aliases only — no local model file management
- Write ops for unsupported features return `Unsupported` error

---

## G. Frontend & API Schema

- `tenant_id` added to DB entities (`TokenEntity`, `DownloadRequestEntity`, `UserAccessRequestEntity`) with `#[serde(skip_serializing)]` — not exposed in OpenAPI schemas
- Regenerate OpenAPI spec and TypeScript client types via `make build.ts-client`
- Update frontend test fixtures and MSW handlers for updated schemas
