# SeaORM Migration: Verified Patterns

All items below were reviewed and confirmed correct. No action needed.

## Foundation Layer (`crates/objs/`)

- **DeriveValueType**: All 10 DB enums have `DeriveValueType` with `value_type = "String"` (DownloadStatus, TokenStatus, AppStatus, AppAccessRequestStatus, FlowType, UserAccessRequestStatus, McpAuthType, RegistrationType, ApiFormat, AliasSource)
- **FromJsonQueryResult**: All JSON column types covered (JsonVec, ModelCapabilities, ToolCapabilities, ContextLimits, ModelArchitecture, OAIRequestParams)
- **Strum/Serde alignment**: All enum serialization patterns consistent. snake_case for DB storage, lowercase for ApiFormat (intentional)
- **JsonVec**: Deref, DerefMut, Default, From impls, FromIterator, serialization roundtrip test — all correct
- **sea_orm in objs**: Pragmatic trade-off accepted. Macros-only dependency, no runtime components

## Entities + Migrations (`crates/services/src/db/entities/`, `sea_migrations/`)

- **Pattern A/B assignment**: Correct per implementation-conventions.md
- **Migration FK ordering**: All dependent migrations run after parent tables
- **CASCADE constraints**: MCP table hierarchy cascades correctly (delete + update)
- **CITEXT/COLLATE NOCASE**: Case-insensitive columns handled for both SQLite and PostgreSQL
- **Timestamps**: All use `timestamp_with_time_zone()` / `timestamp_with_time_zone_null()`
- **String PKs (ULID)**: All tables except settings (natural key) and apps (client_id)
- **Relation enums**: Complete with both Relation variants and Related<> impls
- **json_binary()**: All JSON columns use cross-DB-safe json_binary()
- **down() order**: Multi-table migrations drop dependent tables first
- **Index coverage**: Comprehensive across all query patterns

## Repository Implementations (`crates/services/src/db/service_*.rs`)

- **TimeService**: No direct `Utc::now()` calls — all use `self.time_service.utc_now()`
- **ULID generation**: All inserts use `ulid::Ulid::new().to_string()`
- **Set()/NotSet**: Inserts set all fields, updates use `Default::default()` + selective Set
- **Error handling**: Consistent `.map_err(DbError::from)?` pattern
- **Encryption/decryption**: encrypt_api_key() before insert, decrypt_api_key() after select
- **DbError enum**: Complete — SeaOrmError, StrumParse, TokenValidation, EncryptionError, PrefixExists, ItemNotFound, MultipleAppInstance, Conversion. SqlxError still used by session service (not dead)
- **reset_all_tables()**: FK-aware — PostgreSQL uses TRUNCATE CASCADE, SQLite uses ordered DELETE
- **seed_toolset_configs()**: Idempotent — checks existence before insert
- **objs.rs conversions**: All Row types and ApiKeyUpdate enum correctly defined
- **delete_oauth_config_cascade**: Proper transaction usage (begin/commit)
- **create_toolset**: Returns `row.clone()` instead of `From<Model>` — minor inconsistency, functionally identical

## Test Infrastructure

- **Dual-DB parameterization**: All 9 test files use `#[values("sqlite", "postgres")]`
- **#[serial(pg_app)]**: Present on every dual-DB test
- **_setup_env fixture**: Present in all tests
- **#[anyhow_trace]**: On all async test functions
- **assert_eq!(expected, actual)**: Followed consistently with pretty_assertions
- **No use super::***: All test files use explicit imports
- **No inline timeouts**: Tests rely on defaults
- **SeaTestContext**: Correct — TempDir for SQLite, INTEG_TEST_APP_DB_PG_URL for PostgreSQL, FrozenTimeService, Migrator::fresh()
- **TestDbService**: Correct — wraps DefaultDbService with event broadcasting (1225 lines of trait delegations)
- **Encryption roundtrip tests**: Present for app_instance and toolset
- **Happy path + error coverage**: Each repository has CRUD + list + error scenario tests

## Upstream Consumers (`crates/auth_middleware/`, `crates/routes_app/`)

- **TimeService in auth_middleware**: DefaultTokenService stores and uses Arc<dyn TimeService>
- **AuthContext propagation**: ExternalApp variant carries access_request_id from DB
- **Route handler DB integration**: All access DB through service traits, no direct DB access
- **Draft-first access request flow**: Preserved — no auto-approve
- **OpenAPI schema registration**: Updated for new/changed enums
- **Test organization**: Sibling test_*.rs pattern, auth tests in dedicated files
- **Error chain integrity**: DB → service → domain → ApiError → OpenAI-compatible JSON
- **Service consumers**: ULID IDs, DateTime<Utc>, typed enums, TimeService injection all correct
- **No new MockDbService usage**: New tests use real DB
- **snake_case enum serialization**: Consistent in API responses

## E2E + Infrastructure

- **Playwright dual-DB**: SQLite (port 51135) and PostgreSQL (port 41135) configured
- **DB config utility**: Ports match docker-compose-test-deps.yml (64320 app DB, 54320 session DB)
- **Shared server startup**: PostgreSQL DB URLs injected via NAPI bindings constants
- **Fixtures**: Project-aware auto-reset with getServerUrl(testInfo.project.name)
- **Docker**: Two PostgreSQL 17 services with health checks, correct port mapping
- **NAPI exports**: BODHI_APP_DB_URL, BODHI_SESSION_DB_URL, BODHI_ENCRYPTION_KEY
- **lib_bodhiserver**: Uses DefaultDbService (no SqliteDbService references)
- **Makefile**: test.backend depends on test.deps.up for PostgreSQL containers
- **E2E specs**: All use project-aware fixtures from @/fixtures.mjs
- **Enum consistency**: snake_case verified across Rust → API → OpenAPI → ts-client → frontend → E2E
