# SeaORM Migration — Design Decisions & Context

This document captures the design decisions made during the SeaORM migration. It serves as reference for understanding why certain patterns were chosen.

## Background

BodhiApp is migrating its data layer from raw sqlx to SeaORM to support dual-database (SQLite standalone + PostgreSQL cluster). A prototype migrated 3 entities (download_requests, api_model_aliases, model_metadata). This plan extends to all remaining entities.

No production release exists -- all changes are in-place, no backwards compatibility or data migration needed.

## Primary Key Strategy

**Decision: ULID for ALL tables.**

- Replace UUID v4 (`uuid::Uuid::new_v4().to_string()`) with ULID (`ulid::Ulid::new().to_string()`)
- Replace INTEGER AUTOINCREMENT PKs with String ULID
- ULID is 26-char Crockford Base32, lexicographically sortable, time-ordered
- Use the `ulid` crate
- Stored as TEXT in both SQLite and PostgreSQL
- Tables affected by AUTOINCREMENT -> ULID: `access_requests`, `app_toolset_configs`, `model_metadata`
- Tables affected by UUID -> ULID: all other tables (download_requests, api_model_aliases, api_tokens, toolsets, mcps, mcp_servers, mcp_auth_headers, mcp_oauth_configs, mcp_oauth_tokens, app_access_requests, user_aliases, apps)
- Pre-iteration normalizes DB schema (PK types). Actual ID generation code switches to ULID per-iteration as each repo is migrated.

## Entity Struct Approach

**Decision: Apply SeaORM macros to existing domain structs. Create separate entity structs ONLY when encrypted columns need to be hidden.**

### When to apply macros directly:
- Domain struct fields match DB columns 1:1
- No encrypted columns that should be hidden from the domain API
- Examples: `UserAccessRequest`, `ApiToken`, `McpServerRow`, `McpRow`, `AppToolsetConfigRow`, `DbSetting`, `AppAccessRequestRow`

### When to create separate entity:
- Domain struct has decrypted fields but DB stores encrypted columns
- Need to hide encrypted_*, salt_*, nonce_* columns from domain API
- Examples: `api_model_alias` (ApiAlias hides encrypted_api_key), `apps` (AppInstanceRow has decrypted client_secret)

### When to refactor + create separate entity:
- Domain struct currently exposes encrypted columns but SHOULD hide them
- Refactor domain struct to remove encrypted fields, create entity with full columns
- Examples: `ToolsetRow`, `McpAuthHeaderRow`, `McpOAuthConfigRow`, `McpOAuthTokenRow`

### For enums:
- Add `sea_orm::DeriveValueType` to existing enums (e.g., `DownloadStatus`, `TokenStatus`, `UserAccessRequestStatus`, `ApiFormat`)
- Enums already have strum `Display`/`EnumString` derives that handle String conversion

### For custom types:
- `Repo`: Add `DeriveValueType` (has `FromStr` + `Display`)
- `OAIRequestParams`: Add `FromJsonQueryResult` for JSON column mapping
- `JsonVec`: Already has `FromJsonQueryResult`

## Timestamp Normalization

**Decision: ALL timestamps become `DateTime<Utc>` using `timestamp_with_time_zone` in SeaORM.**

- Tables with i64 Unix timestamps: toolsets, app_toolset_configs, user_aliases, app_access_requests, mcp_servers, mcps, mcp_auth_headers, mcp_oauth_configs, mcp_oauth_tokens, settings, apps
- Tables already with DateTime<Utc>: download_requests, api_model_aliases, model_metadata, access_requests, api_tokens
- Domain structs updated accordingly (e.g., `ToolsetRow.created_at`: `i64` -> `DateTime<Utc>`)

## Case-Insensitive Columns (CITEXT)

**Decision: CITEXT extension on PostgreSQL + COLLATE NOCASE on SQLite.**

Implementation:
- First SeaORM migration creates `CREATE EXTENSION IF NOT EXISTS citext` (PG only, conditional on backend)
- For PG: columns needing case-insensitivity use CITEXT type
- For SQLite: columns use TEXT with COLLATE NOCASE in index/constraint definitions
- Queries: no special handling needed (CITEXT handles PG comparison, COLLATE NOCASE handles SQLite)
- Backend-conditional code in migration using `manager.get_database_backend()`

Tables with case-insensitive columns:
- `toolsets`: UNIQUE(user_id, slug) -- slug is CITEXT/COLLATE NOCASE
- `mcps`: UNIQUE(created_by, slug) -- slug is CITEXT/COLLATE NOCASE
- `mcp_servers`: UNIQUE INDEX ON url -- url is CITEXT/COLLATE NOCASE
- `mcp_auth_headers`: UNIQUE INDEX ON (mcp_server_id, name) -- name is CITEXT/COLLATE NOCASE
- `mcp_oauth_configs`: UNIQUE INDEX ON (mcp_server_id, name) -- name is CITEXT/COLLATE NOCASE

## Migration Strategy

**Decision: IF NOT EXISTS during transition, remove after all migrated.**

- During transition: both sqlx migrations (`sqlx::migrate!()`) and SeaORM migrations (`Migrator::up()`) run
- SeaORM migrations use `Table::create().if_not_exists()` for idempotent overlap
- At finalization: sqlx migrations folder removed, `sqlx::migrate!()` call removed, only SeaORM `Migrator::up()` remains
- Prefer code-based migrations using sea-orm-migration API
- Raw SQL only for DB-specific features (CITEXT extension, COLLATE NOCASE constraints, partial indexes)
- Session DB stays on sqlx (out of scope for this migration)

## Test Strategy

### Parameterization Pattern
Use `#[values("sqlite", "postgres")]` (NOT `#[case]`) for DB type parameter, so `#[case]` remains free for test variations:

```rust
#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_something(
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> { ... }
```

### PostgreSQL is Mandatory
- Tests FAIL (panic) if PostgreSQL is not available
- Docker compose must be running (`make test.deps.up`)
- No skip behavior -- enforce mandatory dual-DB testing

### Serialization
- Both SQLite and PostgreSQL tests serialized with `#[serial(pg_app)]`
- Acceptable tradeoff: simplicity > marginal parallelism

### MockDbService
- Keep existing MockDbService (auto-generated by mockall from traits)
- For NEW or EDITED or MIGRATED tests: use real DB (TestDbService / DefaultDbService)
- Don't introduce new MockDbService usage
- Existing downstream MockDbService usage stays until a separate initiative replaces it

### Test File Naming
- All tests consolidated into `test_<repo>.rs` (no `_sea` suffix)

## Field Value Strategy

**Decision: Always set all fields explicitly from Rust code (prototype pattern).**

- DB column defaults (DEFAULT 'pending', DEFAULT CURRENT_TIMESTAMP) are safety nets only
- Repository impls always set timestamps via `self.time_service.utc_now()`
- Repository impls always set status/defaults explicitly
- ActiveModel uses `Set(value)` for all fields, not `NotSet` for defaultable fields

## Iteration-Specific Decisions

### Iteration 1: access_requests
- Apply `DeriveEntityModel` to `UserAccessRequest`
- Add `DeriveValueType` to `UserAccessRequestStatus`
- PK: change from i64 AUTOINCREMENT to String ULID

### Iteration 2: api_tokens
- Apply `DeriveEntityModel` to `ApiToken`
- Add `DeriveValueType` to `TokenStatus`
- token_hash is NOT encrypted (one-way hash), stays as-is

### Iteration 3: toolsets + app_toolset_configs
- **ToolsetRow**: REFACTOR to hide encrypted columns. Create separate entity with encrypted_api_key/salt/nonce. Domain ToolsetRow loses those fields.
- **AppToolsetConfigRow**: Add `id: String` (ULID) field, apply `DeriveEntityModel`
- Sub-agent must check downstream usage of ToolsetRow.encrypted_api_key/salt/nonce before removing

### Iteration 4: user_aliases
- **Rename DB columns**: request_params_json -> request_params, context_params_json -> context_params
- Apply `DeriveEntityModel` to `UserAlias`
- Add `DeriveValueType` to `Repo`
- Add `FromJsonQueryResult` to `OAIRequestParams`
- JSON columns use `json_binary()` type

### Iteration 5: app_access_requests
- Apply `DeriveEntityModel` to `AppAccessRequestRow`
- Remove `sqlx::FromRow` derive
- **Create proper enums**: `AppAccessRequestStatus` (draft/approved/denied/failed), `FlowType` (redirect/popup) with `DeriveValueType`
- Change timestamps from i64 to DateTime<Utc>

### Iteration 6: MCP group
- **McpServerRow**: Apply macros directly, timestamps to DateTime<Utc>. url column uses CITEXT/COLLATE NOCASE.
- **McpRow**: Apply macros directly, add SeaORM Relation (BelongsTo McpServer). slug uses CITEXT/COLLATE NOCASE.
- **McpWithServerRow**: Replace with `(McpRow, McpServerRow)` tuple return from repository. Use `find_also_related()`. Changes McpRepository trait API.
- **McpAuthHeaderRow**: REFACTOR -- separate entity with encrypted columns, clean domain struct
- **McpOAuthConfigRow**: REFACTOR -- separate entity with encrypted columns, clean domain struct
- **McpOAuthTokenRow**: REFACTOR -- separate entity with encrypted columns, clean domain struct
- Sub-agent checks downstream usage of encrypted fields before removing from domain structs

### Iteration 7: settings
- Apply `DeriveEntityModel` to `DbSetting`
- Natural key (key column as PK), no ULID
- Timestamps to DateTime<Utc>

### Iteration 8: apps
- **Separate entity** (AppInstanceRow has decrypted client_secret, DB stores encrypted)
- Refactor AppInstanceRow to remove salt/nonce fields
- Timestamps to DateTime<Utc>

## Pre-iteration Scope

1. Normalize all AUTOINCREMENT PKs to String ULID (model_metadata, access_requests, app_toolset_configs)
2. Refactor prototype entities to new pattern:
   - Remove `entities/download_request.rs` -- apply DeriveEntityModel to DownloadRequest struct
   - Remove `entities/model_metadata.rs` -- apply DeriveEntityModel to ModelMetadataRow struct
   - Keep `entities/api_model_alias.rs` (encrypted columns)
3. Update sea migrations for PK type changes
4. Schema changes only in pre-iteration; ID generation code switches per-iteration

## Finalization Outcome

All finalization steps completed:
1. DbCore implemented on DefaultDbService (migrate, seed_toolset_configs, reset_all_tables)
2. Bootstrap (lib_bodhiserver) switched to DefaultDbService
3. TestDbService wraps DefaultDbService
4. Test files consolidated (no `_sea` suffix files)
5. SqliteDbService and all sqlx-based repo impls removed
6. sqlx migrations folder removed (28 files)
7. sqlx dependency retained for session_service (out of scope)
8. SqlxError kept in `crates/services/src/db/error.rs` (used by session_service); SqlxMigrateError removed from DbError
