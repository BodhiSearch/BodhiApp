# SeaORM Migration — Post-Implementation Context

Brief intro: BodhiApp's persistence layer migrated from raw sqlx to SeaORM (commit d49357535). This document captures the current state for review and future reference.

## 1. Migration Outcome

What was replaced:
- SqliteDbService (raw sqlx with hand-written SQL, 10 repo files ~3.5k lines)
- 28 sqlx migration files in `crates/services/migrations/`
- `DbError::SqlxMigrateError` variant from DbError enum
- `uuid` dependency (replaced by `ulid`)
- Direct `Utc::now()` calls in production code (replaced by TimeService)

What remains:
- Session service still uses sqlx directly (separate from main migration)
- `SqlxError` struct kept in `crates/services/src/db/error.rs` for session service use
- `sqlx` crate dependency retained for session service

Key stats:
- 9 repository trait implementations in `crates/services/src/db/service_*.rs`
- 17 SeaORM entity definitions in `crates/services/src/db/entities/`
- 14 SeaORM migrations in `crates/services/src/db/sea_migrations/` (including PostgreSQL CITEXT extension)
- DefaultDbService is the sole DbService implementation
- Dual-database support: SQLite (dev/desktop) + PostgreSQL (production/Docker)

## 2. Persistence Architecture

### DefaultDbService (`crates/services/src/db/default_service.rs`)
- Sole implementation of `DbService` trait
- Wraps `sea_orm::DatabaseConnection`, `Arc<dyn TimeService>`, and `encryption_key: String`
- Implements `DbCore` trait: `migrate()`, `seed_toolset_configs()`, `reset_all_tables()`
- All 9 repository traits implemented on `DefaultDbService`

### Repository Traits (`crates/services/src/db/`)
9 repository traits define the data access API:
- `AccessRepository` — user access management
- `AccessRequestRepository` — access request workflow (draft → approved/denied)
- `AppInstanceRepository` — OAuth client credential CRUD with encryption
- `McpRepository` — MCP server/instance/auth CRUD (35 methods, CASCADE-aware). List/get methods return domain types (`McpAuthHeader`, `McpOAuthConfig`, `McpOAuthToken`) with `has_*` boolean flags instead of Row types with encrypted fields. Decrypt methods (`get_decrypted_auth_header`, `get_decrypted_client_secret`, `get_decrypted_oauth_bearer`, `get_decrypted_refresh_token`) query entities directly for decryption.
- `ModelRepository` — download_requests, api_model_aliases, model_metadata
- `SettingsRepository` — key-value settings with upsert
- `TokenRepository` — API token lifecycle
- `ToolsetRepository` — toolset config with encryption. `create_toolset` uses fetch-after-insert pattern (not input clone)
- `UserAliasRepository` — user alias CRUD

### Entity Definitions (`crates/services/src/db/entities/`)
17 entity files following two patterns:
- **Pattern A (direct alias)**: Entity fields match domain struct 1:1 (e.g., `access_request.rs`, `api_token.rs`, `mcp_server.rs`)
- **Pattern B (View + DerivePartialModel)**: Entity has encrypted columns hidden from domain API via View structs. Views use `DerivePartialModel` + `FromQueryResult` to select only non-sensitive columns, with `From<View> for DomainType` impls that compute `has_*` boolean flags. Examples: `ApiAliasView` in `api_model_alias.rs`, `McpAuthHeaderView` in `mcp_auth_header.rs`, `McpOAuthConfigView` in `mcp_oauth_config.rs`, `McpOAuthTokenView` in `mcp_oauth_token.rs`. Also used by `toolset.rs` and `app_instance.rs`
- All entities have populated `Relation` enums for FK relationships
- MCP tables use CASCADE delete constraints

### Migrations (`crates/services/src/db/sea_migrations/`)
14 migrations supporting both SQLite and PostgreSQL:
- Migration 0000: PostgreSQL CITEXT extension (no-op on SQLite)
- Migrations 0001-0013: Table creation (no `if_not_exists()` — removed to avoid masking migration ordering bugs), all PKs are String (ULID), all timestamps are `timestamp_with_time_zone`
- CITEXT/COLLATE NOCASE for case-insensitive columns (slugs, URLs)
- CASCADE FK constraints on MCP table hierarchy

## 3. Dual Database Support

### SQLite (development/desktop)
- Default for local development and Tauri desktop app
- In-memory SQLite for unit tests
- File-based SQLite for session storage

### PostgreSQL (production/Docker)
- Configured via `BODHI_DB_URL` environment variable
- Session DB via `BODHI_SESSION_DB_URL`
- CI tests run against both via Docker Compose (`docker-compose-test-deps.yml`)

### Configuration
- `BODHI_DB_URL` — Main database URL (defaults to SQLite)
- `BODHI_SESSION_DB_URL` — Session database URL (defaults to SQLite)
- `INTEG_TEST_APP_DB_PG_URL` — PostgreSQL URL for integration tests

## 4. Key Patterns

### DeriveValueType for enums
Domain enums in `crates/objs/` derive `sea_orm::DeriveValueType` with `#[sea_orm(value_type = "String")]`. Combined with `strum::Display`/`EnumString`, enables transparent enum↔string column mapping.

Enums: `DownloadStatus`, `TokenStatus`, `AppStatus`, `UserAccessRequestStatus`, `AppAccessRequestStatus`, `FlowType`, `McpAuthType`, `RegistrationType`, `ApiFormat`

Enum serialization uses **snake_case** via strum `serialize_all = "snake_case"` attributes.

### FromJsonQueryResult for JSON columns
Types stored as JSON columns derive `sea_orm::FromJsonQueryResult`:
- `JsonVec<T>` — Generic list stored as JSON array (private inner field, access via Deref)
- `ModelArchitecture` — Model architecture metadata

### DerivePartialModel for encrypted field protection
Entities with encrypted columns use View structs to prevent encrypted data from leaking through list/get repository methods:
- View struct derives `DerivePartialModel` + `FromQueryResult` with `#[sea_orm(entity = "Entity")]`
- View includes non-sensitive fields plus optionally encrypted fields needed for `is_some()` checks (e.g., `encrypted_client_secret: Option<String>` to compute `has_client_secret: bool`), but excludes salt/nonce fields
- `From<View> for DomainType` impl computes `has_*` boolean flags and discards any included encrypted field values
- Repository implementations use `.into_partial_model::<View>()` in list/get queries, then `.map(Into::into)` to convert to domain types
- Decrypt methods query entities directly (bypassing Views) when raw encrypted values are needed
- Reference: `ApiAliasView` in `api_model_alias.rs` (original pattern), MCP View structs in `mcp_auth_header.rs`, `mcp_oauth_config.rs`, `mcp_oauth_token.rs`

### TimeService injection
All timestamp operations flow through `TimeService` trait (`crates/services/src/db/service.rs`):
- `DefaultTimeService` — calls `Utc::now()` in production
- `FrozenTimeService` — returns fixed 2025-01-01T00:00:00Z in tests
- Injected into `DefaultDbService`, `DefaultTokenService`, and route handlers

### ULID for ID generation
All new record IDs use `ulid::Ulid::new().to_string()`:
- 26-char Crockford Base32, lexicographically sortable, time-ordered
- Stored as TEXT in both SQLite and PostgreSQL
- Generated in service layer (not DB default)

### Error handling
- `SeaOrmDbError` wraps `sea_orm::DbErr` (in `crates/services/src/db/error.rs`)
- `DbError` enum: `SeaOrmError`, `StrumParse`, `TokenValidation`, `EncryptionError`, `PrefixExists`, `ItemNotFound`, `MultipleAppInstance`, `Conversion`
- `SqlxError` struct retained for session service usage

## 5. Test Infrastructure

### SeaTestContext (`crates/services/src/test_utils/sea.rs`)
Dual-database test fixture:
- `sea_context("sqlite")` — in-memory SQLite with temp directory
- `sea_context("postgres")` — connects to `INTEG_TEST_APP_DB_PG_URL`, runs fresh migrations
- Returns `SeaTestContext { service: DefaultDbService, now: DateTime<Utc> }`

### TestDbService (`crates/services/src/test_utils/db.rs`)
Wraps `DefaultDbService` with event broadcasting:
- In-memory SQLite with fresh SeaORM migrations
- `FrozenTimeService` for deterministic timestamps
- `subscribe()` returns broadcast `Receiver<String>` for operation assertions

### Test parameterization
Dual-DB tests use `#[values("sqlite", "postgres")]` with `#[serial(pg_app)]`:
- PostgreSQL is mandatory (tests panic if unavailable)
- Both variants serialized for safety
- Requires `make test.deps.up` for PostgreSQL Docker container

### Test file organization
9 repository test files in `crates/services/src/db/test_*_repository.rs`:
- One test file per repository trait
- All use `sea_context` fixture for dual-DB testing

## 6. E2E Infrastructure

### Dual-DB Playwright
- `playwright.config.mjs` defines two projects: SQLite (port 51135) and PostgreSQL (port 41135)
- `tests-js/utils/db-config.mjs` provides DB configuration utility
- `tests-js/fixtures.mjs` injects server URL based on DB project
- `tests-js/scripts/start-shared-server.mjs` starts server with appropriate DB config

### Docker test dependencies
- `docker-compose-test-deps.yml` provides PostgreSQL service for integration tests
- `make test.deps.up` / `make test.deps.down` manage test dependencies

## 7. API & Frontend Changes

### OpenAPI schema
- `openapi.json` updated for snake_case enum values
- TypeScript client regenerated (`ts-client/src/`)

### UI components
- Updated for snake_case enum values (e.g., `"pre-registered"` → `"pre_registered"`)
- MCP form components updated for new auth config patterns
- Setup pages updated for AppStatus enum changes

## 8. Session Service

The session service (`crates/services/src/session_service/`) remains on raw sqlx:
- Uses `sqlx::SqlitePool` and `sqlx::PgPool` directly
- `SqlxError` struct in `crates/services/src/db/error.rs` supports this
- Separate from the main SeaORM migration (out of scope)
- Supports both SQLite and PostgreSQL backends via `SessionStoreBackend`
