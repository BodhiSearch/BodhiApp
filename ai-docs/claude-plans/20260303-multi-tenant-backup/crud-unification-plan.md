# CRUD Unification Plan — 2.5-Type Pattern

## Goal

Reduce type explosion and establish uniform CRUD patterns across all domains. Centralize business logic in domain services, make route handlers thin.

## Decisions

### Naming Convention

| Layer | Type Name | File Location | Purpose |
|-------|-----------|---------------|---------|
| DB Entity | `<entity>::Model` | `services/src/<domain>/<entity>.rs` | SeaORM columns, internal only |
| Input | `<Domain>Form` | `services/src/<domain>/<domain>_schemas.rs` | Serde + Validate + ToSchema. Used for create AND update (PUT). Registered directly as OpenAPI request body. |
| Output | `<Domain>` | `services/src/<domain>/<domain>_schemas.rs` | Serde + ToSchema. Returned by service, used directly as JSON response. Has derived fields (`has_api_key: bool`). |
| Service | `<Domain>Service` trait | `services/src/<domain>/<domain>_service.rs` | Business logic: validate, create/update entity, return composed output |
| Repository | `<Domain>Repository` trait | `services/src/<domain>/<domain>_repository.rs` | DB operations (existing traits, renamed/moved) |
| Error | `<Domain>ServiceError` | `services/src/<domain>/error.rs` | Domain-specific error enum |

### Architecture Flow

```
Handler receives Json<DomainForm>
  → auth_scope.<domain>().create(form) or .update(id, form)
    → AuthScoped<Domain>Service injects tenant_id, user_id
      → <Domain>Service validates form (Validate + business rules)
      → Converts form → entity::Model (generates ULID, timestamps via TimeService)
      → Calls repository for DB persistence
      → Converts entity::Model → <Domain> output (with derived fields, joins)
      → Returns Result<Domain, DomainServiceError>
  → Handler wraps in Json(), returns
```

### Key Design Rules

1. **Separate create/update methods**: `create(form) -> Domain` / `update(id, form) -> Domain`. NOT upsert.
2. **Auth-scoped wrappers stay**: `AuthScoped<Domain>Service` extracts tenant_id/user_id, forwards Form to service.
3. **Service validates internally**: Form derives `Validate`. Service calls `form.validate()?` + business rules.
4. **Service returns composed output**: Service does JOINs, computes derived fields (`has_api_key`), returns `<Domain>`.
5. **Forms live in services crate**: With `#[derive(Deserialize, Validate, ToSchema)]`. Handler uses `Json<DomainForm>` directly.
6. **No response wrappers in routes_app**: Form IS the OpenAPI request body. `<Domain>` IS the response body.
7. **Drop all `*Row` types**: Use `entity::Model` directly in service internals.
8. **Service does NOT own task spawning**: Background tasks (cache refresh) removed from service. See TECHDEBT.
9. **Secret masking**: Service output has `has_api_key: bool`. Presentation masking (`"***"`) is handler/frontend concern.

### API Key Handling

Unified `ApiKeyUpdate` enum (in services) with serde annotations, used in all Forms:

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(tag = "action", content = "value", rename_all = "lowercase")]
pub enum ApiKeyUpdate {
    Keep,           // On create: validation error. On update: no change.
    Set(Option<String>),  // None = clear key, Some(key) = set key
}
```

`ApiKey` wrapper type (validates length, custom deserializer) moves from routes_app to services.

### Module Structure (API Models example)

```
services/src/models/
├── mod.rs              # module declarations, re-exports
├── shared.rs           # Alias enum, AliasSource, ModelAlias, Repo, JsonVec, OAIRequestParams
├── api/
│   ├── mod.rs
│   ├── api_alias_entity.rs      # SeaORM entity (renamed from api_model_alias_entity.rs)
│   ├── api_alias_repository.rs  # Repository trait (moved from db/)
│   ├── api_model_schemas.rs     # ApiModelForm, ApiModel (output)
│   ├── api_model_service.rs     # ApiModelService trait + DefaultApiModelService
│   └── error.rs                 # ApiModelServiceError
├── user/
│   ├── mod.rs
│   ├── user_alias_entity.rs
│   ├── user_alias_repository.rs
│   ├── user_model_schemas.rs
│   ├── user_model_service.rs
│   └── error.rs
└── download/
    ├── mod.rs
    ├── download_request_entity.rs
    ├── download_repository.rs
    ├── download_schemas.rs       # NewDownloadForm, DownloadRequest (output = entity with skip_serializing)
    ├── download_service.rs
    └── error.rs
```

### Type Reduction Per Domain

| Domain | Before | After | Notes |
|--------|:---:|:---:|-------|
| **ApiModel** | ~7 (Entity, ApiAlias, CreateReq, UpdateReq, ApiModelResponse, ApiKey, ApiKeyUpdateAction) | 3 (Entity, ApiModelForm, ApiModel) | ApiKeyUpdate merged into services |
| **MCP Instance** | 7 (Entity, McpRow, McpWithServerRow, Mcp, CreateMcpReq, UpdateMcpReq, McpResponse) | 3 (Entity, McpForm, Mcp) | McpServerInfo composed by service |
| **Toolset** | 5 (Entity, ToolsetRow, Toolset, CreateReq, UpdateReq) | 3 (Entity, ToolsetForm, Toolset) | Drop ToolsetRow |
| **Download** | 4 (Entity, NewDownloadReq, DownloadRequestResponse, Paginated) | 2 (Entity, NewDownloadForm) | Entity IS the response (skip_serializing tenant_id) |
| **Token** | 7 (Entity, ApiTokenRow, CreateReq, UpdateReq, ApiTokenResponse, ApiTokenDetail, Paginated) | 5 (Entity, CreateTokenForm, UpdateTokenForm, TokenDetail, TokenCreated) | Cannot merge create/update (asymmetric). Drop ApiTokenRow |
| **AccessRequest** | existing types | TBD in Phase 1 | Fix bypasses first |

## Phases

### Phase 1: Fix Existing Bypasses

**Goal**: Validate pattern by fixing code that bypasses existing services.

1. `auth_scoped_data.rs:112` — `update_alias()` should use `data_service()` not `db_service()`
2. `auth_scoped_mcps.rs:205` — `delete_oauth_token()` should use `mcp_service()` not `db_service()`
3. `routes_users_access_request.rs` — all handlers should use `AccessRequestService` instead of direct DB

**Gate check**: `cargo test -p routes_app -p services -p server_app`

### Phase 2: ApiModel (Reference Implementation)

**Goal**: Full 2.5-type pattern on the most representative domain.

1. Create `services/src/models/api/` module structure
2. Move `api_model_alias_entity.rs` → `models/api/api_alias_entity.rs`
3. Move `api_alias_repository.rs` → `models/api/api_alias_repository.rs`
4. Create `api_model_schemas.rs` with `ApiModelForm` + `ApiModel` output
5. Move `ApiKeyUpdate` to services, merge with `ApiKeyUpdateAction`, add serde annotations
6. Move `ApiKey` wrapper to services
7. Create `ApiModelService` trait + `DefaultApiModelService`
8. Create `AuthScopedApiModelService` (add `.api_models()` to `AuthScopedAppService`)
9. Refactor `routes_api_models.rs` handlers to use `auth_scope.api_models().*`
10. Remove `CreateApiModelRequest`, `UpdateApiModelRequest`, `ApiModelResponse` from routes_app
11. Remove spawn_cache_refresh logic (add to TECHDEBT)
12. Move shared types (Repo, JsonVec, etc.) to `models/shared.rs`

**Gate check**: `cargo test -p routes_app -p services -p server_app`

### Phase 3: Download

1. Create `services/src/models/download/` module
2. Add `#[serde(skip_serializing)]` to entity's `tenant_id` — entity IS the response
3. Create `NewDownloadForm` in `download_schemas.rs`
4. Create `DownloadService` trait
5. Create `AuthScopedDownloadService`
6. Refactor `routes_models_pull.rs` handlers
7. Remove `DownloadRequestResponse` from routes_app

**Gate check**: `cargo test -p routes_app -p services -p server_app`

### Phase 4: Toolset

1. Create domain service structure under `services/src/toolsets/`
2. Drop `ToolsetRow` — use `toolset_entity::Model` directly
3. Create `ToolsetForm` + use existing `Toolset` as output
4. Create `ToolsetService` (if not already a full service)
5. Refactor toolset route handlers

**Gate check**: `cargo test -p routes_app -p services -p server_app`

### Phase 5: MCP

1. Drop `McpRow`, `McpServerRow` — use entity Models directly
2. Merge `CreateMcpRequest`/`UpdateMcpRequest` → `McpForm`
3. Drop `McpResponse` — use `Mcp` directly (change date fields from String to DateTime<Utc>)
4. Ensure `McpService` returns composed `Mcp` with `McpServerInfo`
5. Refactor MCP route handlers
6. Apply same pattern to MCP Server, Auth Header, OAuth Config sub-entities

**Gate check**: `cargo test -p routes_app -p services -p server_app`

### Phase 6: Token

1. Drop `ApiTokenRow` — use `ApiToken`/`entity::Model` directly
2. Keep separate `CreateTokenForm` and `UpdateTokenForm` (asymmetric CRUD)
3. Drop `ApiTokenResponse` → rename to `TokenCreated { token: String }`
4. Keep `TokenDetail` as output type (= entity minus tenant_id, token_hash)
5. Refactor token route handlers

**Gate check**: `cargo test -p routes_app -p services -p server_app`

### Phase 7: UserAlias

1. Create `services/src/models/user/` module
2. Move user alias types and repository
3. Create `UserModelService` (hybrid: file-based YAML + DB)
4. Refactor user alias route handlers

**Gate check**: `cargo test -p routes_app -p services -p server_app`

## TECHDEBT Items

- Remove `spawn_cache_refresh` from `routes_api_models.rs` — cache population for `forward_all_with_prefix` API models needs a proper async job/queue system
- `PaginatedXxxResponse` types are repetitive — consider a generic `Paginated<T>` wrapper
- `mask_api_key()` function in routes_app may become unused once `has_api_key: bool` replaces masked strings
