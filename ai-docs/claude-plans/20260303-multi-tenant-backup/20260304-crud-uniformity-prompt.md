# CRUD Uniformity — Analysis & Fix Prompt

## Objective

Analyze and fix the CRUD stack across all domains to follow a uniform architecture. Work domain-by-domain, applying prescriptive fixes for known issues and exploratory verification for conventions. Since there is no production release or backwards compatibility requirement, make changes in place.

## Standard CRUD Architecture

### Layer 1: Entity (`<Domain>Entity`)

Location: `crates/services/src/<module>/<domain>_entity.rs`

Every SeaORM entity module MUST have a public type alias:
```rust
pub type <Domain>Entity = Model;
```

Standard fields:
- `id: String` — ULID primary key
- `tenant_id: String` — multi-tenant isolation, always `#[serde(skip_serializing)]`
- `user_id: String` — owner for user-scoped resources
- `created_at: DateTime<Utc>`
- `updated_at: DateTime<Utc>`

For entities with secrets (encrypted columns):
- `encrypted_<col>: Option<String>` — AES-256-GCM encrypted value
- `<col>_salt: Option<String>` (or `salt`)
- `<col>_nonce: Option<String>` (or `nonce`)

### Layer 2: Domain Output Type (`<Domain>`)

Location: co-located in `*_objs.rs` or `*_service.rs` within the same module.

The API-safe output type. Contains all entity fields EXCEPT:
- `tenant_id` — never exposed
- `user_id` — never exposed (exception: Token exposes `user_id` because tokens are listed per-user and the UI needs to display ownership)
- Secret columns (`encrypted_*`, `*_salt`, `*_nonce`) — replaced with boolean indicators like `has_api_key: bool`

Service methods (`list`, `get`, `create`, `update`) return this type. Route handlers return this type as `Json<Domain>`.

### Layer 3: Input Form (`<Domain>Form`)

Location: co-located in `*_objs.rs` within the same module.

A single form type for both create and update operations. Contains all user-editable fields. MUST have:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
```

Fields it EXCLUDES:
- `id` — auto-generated (ULID) on create, received as `/:id` path param on update
- `tenant_id` — injected by auth-scoped service from `AuthContext`
- `user_id` — injected by auth-scoped service from `AuthContext`
- `created_at`, `updated_at` — managed by service via `TimeService`

For secrets: use `ApiKeyUpdate` tagged enum instead of raw secret values:
```rust
pub api_key: ApiKeyUpdate,  // Keep | Set(ApiKey(Option<String>))
```
- Create: `Set(Some("sk-..."))` required if api key is mandatory, `Set(None)` valid if optional
- Update: `Keep` to preserve existing, `Set(Some("sk-..."))` to change, `Set(None)` to remove (if optional)

Fields that are only relevant for create (e.g., `toolset_type` in ToolsetForm) use `Option<String>` with service-level validation: required on create, ignored on update.

Validation: ALL forms MUST use `#[derive(Validate)]` from the `validator` crate. Use field-level `#[validate(...)]` annotations and/or `#[validate(custom(function = "..."))]` for custom validation logic. Do NOT use standalone validation functions called manually in the service — wire them through the `Validate` derive.

### Layer 4: Auth-Scoped Service

Location: `crates/services/src/app_service/auth_scoped_<domain>.rs`

Wraps `Arc<dyn AppService>` + `AuthContext`. Injects `tenant_id` and `user_id` from auth context. Standard methods:
```rust
pub async fn create(&self, form: <Domain>Form) -> Result<Domain, DomainError>
pub async fn update(&self, id: &str, form: <Domain>Form) -> Result<Domain, DomainError>
pub async fn delete(&self, id: &str) -> Result<(), DomainError>
pub async fn get(&self, id: &str) -> Result<Option<Domain>, DomainError>
pub async fn list(&self, ...) -> Result<Vec<Domain>, DomainError>  // or paginated
```

### Layer 5: Route Handlers

Location: `crates/routes_app/src/<module>/routes_<domain>.rs`

All handlers use `AuthScope` extractor. ALL JSON body extractors MUST use `WithRejection<Json<Form>, JsonRejectionError>` for consistent error responses:
```rust
pub async fn domain_create(
  auth_scope: AuthScope,
  WithRejection(Json(form), _): WithRejection<Json<DomainForm>, JsonRejectionError>,
) -> Result<(StatusCode, Json<Domain>), ApiError> {
  let result = auth_scope.domain().create(form).await?;
  Ok((StatusCode::CREATED, Json(result)))
}

pub async fn domain_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  WithRejection(Json(form), _): WithRejection<Json<DomainForm>, JsonRejectionError>,
) -> Result<Json<Domain>, ApiError> {
  let result = auth_scope.domain().update(&id, form).await?;
  Ok(Json(result))
}
```

---

## Domain-by-Domain Analysis

### Batch 1: Toolset + MCP

#### Toolset (`crates/services/src/toolsets/`, `crates/routes_app/src/toolsets/`)

**Entity**: `crates/services/src/toolsets/toolset_entity.rs`
- ISSUE: Missing `pub type ToolsetEntity = Model;` type alias
- FIX: Add the type alias

**Domain output** (`Toolset` in `toolset_objs.rs`): Correct — has `has_api_key: bool`, skips `tenant_id`/`user_id`.

**Form** (`ToolsetForm` in `toolset_objs.rs`): Correct — uses `#[derive(Validate)]` with field-level validators.

**Route handlers** (`routes_toolsets.rs`):
- ISSUE: Uses plain `Json<ToolsetForm>` instead of `WithRejection<Json<ToolsetForm>, JsonRejectionError>`
- FIX: Change `toolsets_create` and `toolsets_update` handlers to use `WithRejection`

#### MCP (`crates/services/src/mcps/`, `crates/routes_app/src/mcps/`)

**Entities**: `mcp_entity.rs`, `mcp_server_entity.rs`, `mcp_auth_header_entity.rs`, `mcp_oauth_config_entity.rs`, `mcp_oauth_token_entity.rs`
- Current aliases: `McpRow`, `McpServerRow`, `McpAuthHeaderRow`, `McpOAuthConfigRow`, `McpOAuthTokenRow`
- ISSUE: Naming uses `*Row` suffix instead of `*Entity`
- FIX: Rename to `McpEntity`, `McpServerEntity`, `McpAuthHeaderEntity`, `McpOAuthConfigEntity`, `McpOAuthTokenEntity`

**EXCEPTION — `created_by`/`updated_by` columns**: MCP entities use `created_by` instead of `user_id`. This is intentional — MCP servers are tenant-level admin resources, not user-scoped. `created_by`/`updated_by` are audit columns tracking which admin performed the action, not ownership columns. When checking if an MCP server is enabled, only `tenant_id` filter applies — `created_by` is NOT used for access control. Add a code comment near the `created_by` field explaining this: `// Audit column — tracks admin who created. NOT used for access control (MCP servers are tenant-level resources).`

**Domain output** (`Mcp` in `mcp_objs.rs`): Correct — skips `tenant_id`/`created_by`. Secret masking uses domain-specific boolean fields (`has_header_value`, `has_client_secret`, etc.) on nested auth sub-types — this is fine, the `has_<secret>` pattern is flexible per domain.

**Forms** (`McpForm`, `McpServerForm` in `mcp_objs.rs`):
- ISSUE: Both forms do NOT derive `Validate`. Validation is done via standalone functions (`validate_mcp_instance_name()`, `validate_mcp_slug()`, `validate_mcp_description()`, `validate_mcp_server_name()`, `validate_mcp_server_url()`, `validate_mcp_server_description()`, `validate_mcp_auth_config_name()`, `validate_oauth_endpoint_url()`).
- FIX: Add `#[derive(Validate)]` to `McpForm` and `McpServerForm`. Convert standalone validation functions to field-level `#[validate(custom(function = "..."))]` annotations. The existing validation functions can stay as the backing implementations — just wire them through the derive. Validation constants already exist: `MAX_MCP_SLUG_LEN`, `MAX_MCP_INSTANCE_NAME_LEN`, `MAX_MCP_DESCRIPTION_LEN`, etc.

**Route handlers** (`routes_mcps.rs`, `routes_mcps_servers.rs`):
- ISSUE: Both use plain `Json<McpForm>` / `Json<McpServerForm>`
- FIX: Change to `WithRejection<Json<McpForm>, JsonRejectionError>` and `WithRejection<Json<McpServerForm>, JsonRejectionError>`

### Batch 2: ApiModel + UserAlias

#### ApiModel (`crates/services/src/models/`, `crates/routes_app/src/api_models/`)

**Entity**: `crates/services/src/models/api_model_alias_entity.rs`
- ISSUE: Missing `pub type ApiModelEntity = Model;` type alias
- There is an `ApiAliasView` (DerivePartialModel) that strips encryption fields — this is fine, it's a query optimization
- FIX: Add the type alias

**Domain output** (`ApiModelOutput` in `model_objs.rs`): Correct — has `has_api_key: bool`, skips `tenant_id`/`user_id`/encryption fields.

**Form** (`ApiModelForm` in `model_objs.rs`): Correct — has `#[derive(Validate)]` with `#[validate(url(...))]` on `base_url`.

**Route handlers** (`routes_api_models.rs`): Already uses `WithRejection` — correct.

#### UserAlias (`crates/services/src/models/`, `crates/routes_app/src/models/`)

**Entity**: `crates/services/src/models/user_alias_entity.rs`
- ISSUE: Missing `pub type UserAliasEntity = Model;` type alias
- FIX: Add the type alias

**Domain output** (`UserAlias` in `model_objs.rs`): Correct — skips `tenant_id`/`user_id`. No secrets.

**Form** (`UserAliasForm` in `model_objs.rs`): Correct — has `#[derive(Validate)]`. No field-level validators (none needed — all validation is semantic, done in service layer).

**Route handlers** (`routes_models.rs`): Already uses `WithRejection` — correct.

### Batch 3: Token + Download

#### Token (`crates/services/src/tokens/`, `crates/routes_app/src/tokens/`)

**Entity**: `crates/services/src/tokens/api_token_entity.rs`
- Current alias: `pub type ApiToken = Model;`
- ISSUE: Naming doesn't follow `<Domain>Entity` pattern — should be `TokenEntity`
- FIX: Rename `ApiToken` to `TokenEntity`

**EXCEPTION — Split forms**: Token uses separate `CreateTokenForm` and `UpdateTokenForm` instead of a single unified form. This is intentional because:
- Create needs `scope: TokenScope` (immutable after creation — a token's scope cannot be changed)
- Update needs `status: TokenStatus` (activate/deactivate) and `name: String` (rename)
- These are genuinely different field sets with no overlap except `name`
- Add a comment near the form definitions: `// Token uses split forms because create requires immutable scope, while update only changes name/status.`

**Domain output**: Token has TWO output types — this is intentional:
- `TokenCreated` — returned only on create, contains the raw unhashed token string (only time it's available)
- `TokenDetail` — returned on list/get/update, contains all fields except `token_hash` and `tenant_id`
- `TokenDetail` includes `user_id` — exception to the standard. Add comment: `// user_id included because tokens are listed per-user and the UI shows ownership.`

**Forms**: Both `CreateTokenForm` and `UpdateTokenForm` already derive `Validate` — correct.

**Route handlers** (`routes_tokens.rs`): Already uses `WithRejection` — correct.

#### Download (`crates/services/src/models/`, `crates/routes_app/src/models/`)

**Entity**: `crates/services/src/models/download_request_entity.rs`
- Current alias: `pub type DownloadRequestModel = Model;`
- ISSUE: Naming doesn't follow `<Domain>Entity` pattern — should be `DownloadEntity`
- FIX: Rename `DownloadRequestModel` to `DownloadEntity`

**EXCEPTION — No `user_id` column**: Downloads are tenant-wide shared resources, not user-scoped. Any user in the tenant can see and trigger downloads. `tenant_id` has `#[serde(skip_serializing)]` on the entity. Add comment near the entity definition: `// Downloads are tenant-wide shared resources — no user_id. Any authenticated tenant user can create/view downloads.`

**EXCEPTION — Standalone mode only**: Download CRUD is only meaningful in standalone (local desktop) mode. In multi-tenant/cloud mode, downloads are not supported. Add comment if not already present.

**Domain output**:
- ISSUE: Download has NO separate output type — the entity (`DownloadRequestModel`) IS used directly as the API response, with `tenant_id` hidden via `#[serde(skip_serializing)]`
- FIX: Create a `Download` output type without `tenant_id`. Using `skip_serializing` on the entity is fragile — adding new DB-only fields to the entity could accidentally expose them in the API. The output type should be explicit about what's returned.

**Form** (`NewDownloadForm` in `download_service.rs`): Has `#[derive(Validate)]` — correct. No field-level validators (semantic validation done in service).

**Route handlers** (`routes_models_pull.rs`): Already uses `WithRejection` — correct.

### Batch 4: Access Requests + Other Entity Aliases

#### Other Entity Aliases to Standardize

These entity type aliases also need renaming:
- `crates/services/src/users/access_request_entity.rs`: `UserAccessRequest` → `AccessRequestEntity`
- `crates/services/src/models/model_metadata_entity.rs`: `ModelMetadataRow` → `ModelMetadataEntity`

For Access Requests and other domains without full CRUD forms, only verify the entity type alias naming. Do not restructure domains that don't have form-based CRUD.

---

## Exploratory Checks (All Domains)

For each domain, verify these conventions even if no specific issue is known:

1. **Validate derive**: Every `*Form` struct has `#[derive(Validate)]`. If field-level validation exists, it uses `#[validate(...)]` annotations (not manual calls).
2. **WithRejection**: Every route handler accepting `Json<Form>` uses `WithRejection<Json<Form>, JsonRejectionError>`.
3. **Entity type alias**: Every entity module has `pub type <Domain>Entity = Model;`.
4. **TimeService**: No `Utc::now()` calls — all timestamps via `TimeService` / `self.db_service.now()`.
5. **ULID for IDs**: New records use `ulid::Ulid::new().to_string()` for `id` generation.
6. **Auth-scoped service**: Write operations call `require_tenant_id()` / `require_user_id()` from auth context. Read operations use `tenant_id_or_empty()` / `user_id_or_empty()` for graceful anonymous fallback.
7. **Output type separation**: Domain output types are distinct from entity `Model`. No `#[serde(skip_serializing)]` tricks on entities for API responses — use a proper output type.
8. **Secret field masking**: Entity secret fields (`encrypted_*`) replaced with `has_<secret>: bool` in output types.

---

## Execution Strategy

Work in sequential batches. Each batch:
1. Apply all fixes for the batch's domains
2. Run gate check: `cargo test -p services -p routes_app -p server_app`
3. If any test fails, fix before proceeding
4. After all batches: regenerate OpenAPI + TS client, run UI tests

### Batch Sequence

**Batch 1**: Toolset + MCP (most changes — form validation, WithRejection, entity aliases)
**Batch 2**: ApiModel + UserAlias (entity aliases only — already mostly correct)
**Batch 3**: Token + Download (entity alias renames, Download output type creation)
**Batch 4**: Access Requests + remaining entity aliases (minor renames)

### Gate Checks After Each Batch

```bash
# Rust compilation + tests
cargo test -p services -p routes_app -p server_app

# After final batch:
cargo run --package xtask openapi
cd ts-client && npm run generate && npm run build
cd crates/bodhi && npm run test
```

### Local Commit After Each Batch

After each batch passes gate checks, make a local commit:
```bash
git add -A
git commit -m "refactor: CRUD uniformity — <batch description>"
```

Commit message examples:
- `refactor: CRUD uniformity — Toolset + MCP (WithRejection, Validate, entity aliases)`
- `refactor: CRUD uniformity — ApiModel + UserAlias (entity aliases)`
- `refactor: CRUD uniformity — Token + Download (entity renames, Download output type)`
- `refactor: CRUD uniformity — Access Requests + remaining entity aliases`

### Important Rules

- Read CLAUDE.md files for affected crates before making changes: `crates/CLAUDE.md`, `crates/services/CLAUDE.md`, `crates/routes_app/CLAUDE.md`
- Follow layered development: change services first, then routes_app
- When renaming type aliases (e.g., `McpRow` → `McpEntity`), do a global find-and-replace across the entire codebase — these are referenced in DB queries, service impls, tests, and route handlers
- Use `replace_all` when renaming to catch all occurrences
- For `WithRejection` changes, import `axum_extra::extract::WithRejection` and `crate::JsonRejectionError` in the route handler file
- For `Validate` derive additions, add `use validator::Validate;` and `validator` field annotations. The standalone validation functions can remain as backing implementations — reference them via `#[validate(custom(function = "validate_fn_name"))]`
- When creating the Download output type, follow the pattern of `Toolset` or `ApiModelOutput` — explicit struct with only API-safe fields
- Add exception comments as specified in the domain analysis above
- After renaming entity type aliases, check that `#[cfg(test)]` modules and test files still compile
