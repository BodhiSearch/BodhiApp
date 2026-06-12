# Services Crate Review

## Files Reviewed
- `crates/services/Cargo.toml`
- `crates/services/src/lib.rs`
- `crates/services/src/session_service/mod.rs`
- `crates/services/src/session_service/error.rs`
- `crates/services/src/session_service/session_store.rs`
- `crates/services/src/session_service/session_service.rs`
- `crates/services/src/session_service/postgres.rs`
- `crates/services/src/session_service/sqlite.rs`
- `crates/services/src/test_session_service.rs`
- `crates/services/src/setting_service/constants.rs`
- `crates/services/src/setting_service/default_service.rs`
- `crates/services/src/setting_service/service.rs`
- `crates/services/src/test_utils/app.rs`
- `crates/services/src/test_utils/envs.rs`
- `crates/services/src/test_utils/session.rs`

## Findings

### Finding 1: SessionStoreBackend dispatch uses enum-in-disguise Option pattern instead of proper enum
- **Priority**: Important
- **File**: crates/services/src/session_service/session_store.rs
- **Location**: `InnerStoreShared` struct and `SessionStore` impl
- **Issue**: `InnerStoreShared` uses two `Option` fields (`sqlite: Option<SqliteStore>`, `postgres: Option<PostgresStore>`) with `if let Some(...) / else if let Some(...)` dispatch. This is effectively an enum encoded as a struct with Options. The pattern has several problems: (a) it allows an invalid state where both are `None` (the `else` branches in `load()` and `delete()` silently return `Ok(None)` / `Ok(())`), (b) it allows an invalid state where both are `Some`, (c) every dispatch site must repeat the same boilerplate `if/else if/else` chain, and (d) new backends would require adding another Option field and extending every dispatch site.
- **Recommendation**: Replace `InnerStoreShared` with an enum:
  ```rust
  enum StoreInner {
    Sqlite(SqliteStore),
    Postgres(PostgresStore),
  }
  ```
  This makes invalid states unrepresentable. Each dispatch becomes a `match` with exhaustive pattern matching. Adding a new backend requires handling it at every callsite (compiler-enforced). The `else` branch that silently swallows "no store configured" would become a compile error.
- **Rationale**: Encoding sum types as product types with Options is a known Rust anti-pattern. It violates "make invalid states unrepresentable" and introduces silent failure paths.

### Finding 2: Dual pool creation in connect methods (typed pool + AnyPool) doubles connection overhead
- **Priority**: Important
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `connect_sqlite()` (lines 54-75) and `connect_postgres()` (lines 77-87)
- **Issue**: Both connect methods create two separate connection pools to the same database: one typed pool (`SqlitePoolOptions` / `PgPool`) for the tower-sessions store, and one `AnyPool` for custom user_id queries. This doubles the number of open connections. For SQLite, this may cause locking issues under concurrent access (SQLite uses file-level locking). For PostgreSQL, this doubles the connection count against the server's `max_connections` limit.
- **Recommendation**: Investigate whether `AnyPool` can be used for both purposes, or whether the typed pool can be extracted from the `SqliteStore`/`PostgresStore` to run custom queries. If dual pools are necessary (due to tower-sessions-sqlx-store API constraints), document the reason and consider reducing pool sizes to compensate.
- **Rationale**: Resource efficiency and potential for hard-to-debug connection exhaustion or SQLite locking issues in production.

### Finding 3: `$1` positional parameter style may not work with SQLite via AnyPool
- **Priority**: Critical
- **File**: crates/services/src/session_service/session_service.rs and session_store.rs
- **Location**: All `sqlx::query()` calls using `$1`, `$2` parameters
- **Issue**: The SQL queries use PostgreSQL-style positional parameters (`$1`, `$2`) exclusively. While sqlx's `AnyPool` does perform parameter translation in some cases, this behavior depends on the sqlx version and the `any` feature's maturity. The SQLite driver natively uses `?` parameters. If the automatic translation fails or is incomplete, all custom queries will break on SQLite. The code must be tested against both backends to confirm this works. The existing tests do pass (per the rstest parameterization), but this is a fragile assumption tied to sqlx internals.
- **Recommendation**: Either (a) verify explicitly in CI that SQLite tests pass with `$1` parameters via AnyPool and add a comment explaining the reliance on sqlx translation, or (b) use the backend-aware query pattern -- separate query strings per backend -- like `migrate_custom()` already does for the ALTER TABLE statement. Alternatively, since AnyPool already supports this translation in current sqlx, adding a targeted regression test that asserts the parameter binding works for SQLite would provide a safety net.
- **Rationale**: If a sqlx update changes AnyPool parameter translation behavior, all session operations on SQLite would silently fail at runtime. The existing parameterized tests do cover this, but the reliance should be explicit.

### Finding 4: `std::mem::forget(temp_dir)` leaks temporary directories in tests
- **Priority**: Important
- **File**: crates/services/src/test_session_service.rs
- **Location**: `create_session_service()` function, line 30
- **Issue**: `std::mem::forget(temp_dir)` is called to prevent the `TempDir` from being dropped (and deleted) before the test finishes using the SQLite file inside it. However, this permanently leaks the directory -- the cleanup destructor never runs, so temp directories accumulate on disk across test runs. For a CI environment running tests repeatedly, this can consume significant disk space over time.
- **Recommendation**: Store the `TempDir` alongside the `DefaultSessionService` (e.g., return a tuple `(DefaultSessionService, TempDir)` from `create_session_service()`) so the directory lives as long as the test but is cleaned up when the test completes. Alternatively, use an `Arc<TempDir>` pattern consistent with `AppServiceStubBuilder`.
- **Rationale**: Resource leak in tests. While each test creates a small SQLite file, accumulated runs (especially in CI) will leave orphaned directories in the OS temp directory.

### Finding 5: `AppSessionStoreExt` trait duplicates `SessionService` method signatures
- **Priority**: Important
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `AppSessionStoreExt` trait (lines 22-30) and `SessionService` trait (lines 13-20)
- **Issue**: `AppSessionStoreExt` and `SessionService` both declare `clear_sessions_for_user`, `clear_all_sessions`, and `count_sessions_for_user` with identical signatures. The `SessionService` impl for `DefaultSessionService` just delegates to `AppSessionStoreExt` methods. This creates a confusing two-trait architecture where `AppSessionStoreExt` also adds methods not in `SessionService` (`get_session_ids_for_user`, `dump_all_sessions`, `migrate_custom`). Consumers outside this module see `SessionService` (which is the public API), while `AppSessionStoreExt` is the internal implementation detail. The duplication makes it unclear which trait to call and adds maintenance burden for keeping signatures in sync.
- **Recommendation**: Consider one of: (a) Make `AppSessionStoreExt` a private implementation detail only and keep all public API on `SessionService`. The extra methods (`dump_all_sessions`, `get_session_ids_for_user`) that are only used in tests could be behind `#[cfg(test)]` or `test-utils` feature. (b) Alternatively, have `SessionService` extend `AppSessionStoreExt` as a supertrait. Either way, eliminate the signature duplication.
- **Rationale**: DRY violation and confusing public API surface. Test code calls `AppSessionStoreExt::clear_sessions_for_user(&service, ...)` with explicit trait disambiguation, which is awkward.

### Finding 6: `url` field in `DefaultSessionService` is stored but only used during construction
- **Priority**: Nice-to-have
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `DefaultSessionService` struct (line 34) and `run_custom_migration()` (line 97-99)
- **Issue**: The `url: String` field is stored permanently in `DefaultSessionService`, but it is only used in `run_custom_migration()` which is called once during `connect_*()`. After construction, the URL serves no purpose but remains in memory for the lifetime of the service. Additionally, the `url` may contain database credentials (for PostgreSQL connection strings), which means sensitive data is kept in memory longer than necessary.
- **Recommendation**: Remove the `url` field from the struct. Pass the URL directly to `migrate_custom()` during construction rather than storing it. This is already partially implemented (the `&self.url.clone()` call in `run_custom_migration` shows the URL is cloned then immediately used). Just pass the URL to the migration function without storing it.
- **Rationale**: Minimizing the lifetime of potentially sensitive data (DB credentials in URLs) and reducing unnecessary state in the struct.

### Finding 7: `install_default_drivers()` called multiple times without idempotency guarantee
- **Priority**: Nice-to-have
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `connect_sqlite()` (line 55) and `connect_postgres()` (line 78)
- **Issue**: `sqlx::any::install_default_drivers()` is called at the start of both `connect_sqlite()` and `connect_postgres()`. The `connect()` method dispatches to one of these, so each call invokes `install_default_drivers()` once. However, if multiple session services are created (e.g., in tests), this is called multiple times. The sqlx documentation states this function panics if called twice in some versions, though recent versions use `std::sync::Once` internally.
- **Recommendation**: Move the `install_default_drivers()` call to a `std::sync::Once` wrapper or call it once at application startup rather than in each connection factory method. This makes the idempotency guarantee explicit rather than relying on sqlx internals.
- **Rationale**: Defensive programming. If sqlx changes the behavior of `install_default_drivers()` to panic on double-call, tests creating multiple session services would break.

### Finding 8: `with_secure(false)` on session cookie should be deployment-dependent
- **Priority**: Important
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `session_layer()` method (line 185)
- **Issue**: The session layer is configured with `.with_secure(false)`, meaning session cookies will be sent over HTTP (not just HTTPS). While this is necessary for local development, production deployments (especially PostgreSQL-backed deployments likely running behind a reverse proxy with TLS) should use `Secure` cookies. The hardcoded `false` means production deployments are vulnerable to session hijacking via network sniffing.
- **Recommendation**: Make the `secure` flag configurable, ideally derived from the deployment context or the `BODHI_SCHEME` setting. When `scheme == "https"` or `deployment_mode != "standalone"`, set `with_secure(true)`. This could be passed as a parameter to `DefaultSessionService::new()` or derived from the setting service.
- **Rationale**: Security. Non-secure cookies in production enable session hijacking on any network path between client and server. This is especially relevant now that PostgreSQL support enables multi-tenant/production deployments.

### Finding 9: Missing `#[anyhow_trace]` and `-> anyhow::Result<()>` in test functions
- **Priority**: Nice-to-have
- **File**: crates/services/src/test_session_service.rs
- **Location**: All test functions
- **Issue**: The test functions do not follow the canonical async test pattern documented in `crates/services/CLAUDE.md`. The canonical pattern is `#[rstest] #[tokio::test] #[anyhow_trace]` returning `-> anyhow::Result<()>`. Instead, these tests use `.unwrap()` on every fallible operation, which produces unhelpful panic messages on failure. They also lack `use pretty_assertions::assert_eq;` which is required by convention (though it is imported at the top of the file).
- **Recommendation**: Refactor tests to return `anyhow::Result<()>`, use `?` instead of `.unwrap()`, and add `#[anyhow_trace]` for better error diagnostics on failure.
- **Rationale**: Convention compliance and debuggability. When a test fails, `.unwrap()` produces a generic panic with no context, while `?` with `#[anyhow_trace]` provides a full backtrace.

### Finding 10: `OffsetDateTime::now_utc()` used directly in test helper
- **Priority**: Nice-to-have
- **File**: crates/services/src/test_session_service.rs
- **Location**: `make_record()` function, line 62
- **Issue**: `OffsetDateTime::now_utc()` is called directly instead of going through `TimeService`. While this is in test code and the timestamp is for session expiry (not a domain timestamp), it deviates from the project's convention of always using `TimeService` for time operations.
- **Recommendation**: This is acceptable for test code since tower-sessions uses `time::OffsetDateTime` (not `chrono::DateTime`), and the `TimeService` abstraction works with chrono types. No change needed, but worth noting the deviation.
- **Rationale**: Minor convention deviation. The `TimeService` uses chrono types, while tower-sessions uses the `time` crate types, so direct use is unavoidable here.

### Finding 11: Missing `postgres` and `sqlite` modules not re-exported from `mod.rs`
- **Priority**: Nice-to-have
- **File**: crates/services/src/session_service/mod.rs
- **Location**: Lines 2, 6
- **Issue**: `postgres` and `sqlite` modules are declared but not re-exported via `pub use`. This is correct since they are `pub(crate)`, but the `mod.rs` declares them as just `mod` (private). If any code outside the `session_service` module within the `services` crate needs to access these, they cannot. Currently `test_utils/session.rs` calls `super::sqlite::create_sqlite_store` -- wait, it actually calls `DefaultSessionService::connect_sqlite` and `DefaultSessionService::connect_postgres`, so the internal modules are properly encapsulated.
- **Recommendation**: No change needed. The current visibility is correct -- `postgres` and `sqlite` are internal implementation details accessed only through `DefaultSessionService` methods.
- **Rationale**: N/A -- this is actually correct as-is.

### Finding 12: Postgres test cleanup relies on shared database state
- **Priority**: Important
- **File**: crates/services/src/test_session_service.rs
- **Location**: `create_session_service()`, postgres branch (lines 33-43)
- **Issue**: Postgres tests use a shared database (from `INTEG_TEST_SESSION_PG_URL`) and clean up by calling `clear_all_sessions()` before each test. While `#[serial(pg_session)]` ensures sequential execution, the cleanup approach has issues: (a) if a test panics mid-execution, subsequent tests may find stale data, (b) the `tower_sessions` table schema from a previous test run persists, and (c) there is no isolation between test runs if the database is shared across CI jobs. This is a fragile test setup.
- **Recommendation**: Consider using a per-test schema or wrapping each test in a transaction that is rolled back. Alternatively, `DROP TABLE tower_sessions` and re-run migrations at the start of each test for stronger isolation. The `#[serial]` attribute provides ordering but not cleanup guarantees on panic.
- **Rationale**: Test reliability. A panicking test will leave dirty state that can cause cascading failures in subsequent tests.

### Finding 13: `deployment_mode()` returns `String` rather than a typed enum
- **Priority**: Nice-to-have
- **File**: crates/services/src/setting_service/service.rs
- **Location**: `deployment_mode()` method (lines 238-243)
- **Issue**: The new `deployment_mode()` method returns a raw `String`. The default is `"standalone"`, but there is no validation or enumeration of allowed values. Consumers must compare against magic strings. If a typo is introduced (e.g., `"standlone"`), it will silently be treated as an unknown deployment mode. Other similar methods in `SettingService` return typed enums (e.g., `env_type() -> EnvType`, `app_type() -> AppType`).
- **Recommendation**: Define a `DeploymentMode` enum in `objs` (similar to `EnvType` and `AppType`) with variants like `Standalone`, `MultiTenant`, etc. Have `deployment_mode()` return `DeploymentMode` with proper parsing and a sensible default fallback.
- **Rationale**: Type safety and consistency with existing setting accessor patterns in the codebase.

### Finding 14: `BODHI_SESSION_DB_URL` and `BODHI_DEPLOYMENT` added to `SETTING_VARS` but no metadata validation
- **Priority**: Nice-to-have
- **File**: crates/services/src/setting_service/constants.rs and default_service.rs
- **Location**: `SETTING_VARS` array (lines 65-86) and `setting_metadata_static()` in default_service.rs
- **Issue**: `BODHI_SESSION_DB_URL` and `BODHI_DEPLOYMENT` are added to `SETTING_VARS` (making them visible in settings list and editable via API) but `setting_metadata_static()` does not have specific cases for them -- they fall through to the default `_ => SettingMetadata::String`. While `SettingMetadata::String` is technically correct, `BODHI_SESSION_DB_URL` should probably not be editable at runtime via the settings API (changing it would require recreating the session service), and `BODHI_DEPLOYMENT` should ideally have `SettingMetadata::Option` with known valid values.
- **Recommendation**: Either (a) exclude these from runtime-editable settings by removing them from `SETTING_VARS` (they can still have defaults), or (b) add explicit metadata entries that mark them as read-only or constrain their values.
- **Rationale**: Allowing runtime modification of the session database URL through the settings API could cause undefined behavior since the session service is initialized at startup and does not reinitialize on setting changes.

### Finding 15: Custom ALTER TABLE migration lacks down migration path
- **Priority**: Nice-to-have
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `migrate_custom()` method (lines 104-130)
- **Issue**: The custom migration adds a `user_id` column and index to `tower_sessions` via direct ALTER TABLE statements. This is a one-way migration with no rollback capability. While the index creation is idempotent (`IF NOT EXISTS`), the PostgreSQL column addition is also idempotent (`IF NOT EXISTS`), and the SQLite branch checks `pragma_table_info` before adding. However, there is no migration versioning or tracking -- if the migration logic changes in a future release, there is no way to know which migrations have already been applied.
- **Recommendation**: Consider tracking custom migration state (e.g., a simple version number in a separate table or a migration registry). For now, since the migrations are idempotent, this is acceptable, but it will become a problem as more custom columns or schema changes are needed.
- **Rationale**: Maintainability. As the session schema evolves, ad-hoc idempotent ALTER TABLE statements become increasingly fragile.

### Finding 16: `save()` in `SessionStoreBackend` silently ignores missing store
- **Priority**: Important
- **File**: crates/services/src/session_service/session_store.rs
- **Location**: `SessionStore::save()` impl (lines 55-77)
- **Issue**: The `save()` method checks `if let Some(store) = &self.inner.sqlite { ... } else if let Some(store) = &self.inner.postgres { ... }` -- but if neither is `Some` (which is an invalid state, see Finding 1), the save silently succeeds without actually persisting anything. The `user_id` UPDATE query would still execute (and potentially fail or be a no-op). The `load()` and `delete()` methods have the same problem, silently returning `Ok(None)` or `Ok(())` on a missing store.
- **Recommendation**: This is a direct consequence of Finding 1. If `InnerStoreShared` is converted to an enum, this problem disappears entirely. In the interim, add an `else { return Err(Error::Backend("No store configured".to_string())); }` branch to make the invalid state explicit.
- **Rationale**: Silent data loss. A misconfigured `SessionStoreBackend` would appear to work (all operations return `Ok`) while not persisting any session data. This would be extremely hard to debug in production.

### Finding 17: `MockSessionService` compatibility with new `count_sessions_for_user` method
- **Priority**: Nice-to-have
- **File**: crates/services/src/session_service/session_service.rs
- **Location**: `SessionService` trait (line 18) and mockall annotation (line 12)
- **Issue**: The `count_sessions_for_user` method was added to the `SessionService` trait. The `#[mockall::automock]` annotation on the trait will automatically generate `MockSessionService::expect_count_sessions_for_user()`. The `MockSessionService` is used in downstream crates (confirmed in `routes_app/src/routes_users/test_management_crud.rs`). Since `mockall` generates all methods, the mock will compile. However, any test using `MockSessionService` that calls `count_sessions_for_user` without setting up an expectation will get a panic. Existing tests that don't touch `count_sessions_for_user` are unaffected.
- **Recommendation**: No code change needed. The `mockall` integration handles this correctly. Just be aware when writing new tests that `count_sessions_for_user` must have an expectation set if it will be called.
- **Rationale**: Information only. The mock auto-generation handles the new method correctly.

### Finding 18: `assert_eq!` argument order inconsistency in tests
- **Priority**: Nice-to-have
- **File**: crates/services/src/test_session_service.rs
- **Location**: Multiple test functions
- **Issue**: The project convention is `assert_eq!(expected, actual)` (expected first, per CLAUDE.md). Most assertions follow this correctly (e.g., `assert_eq!(cleared, 3)` -- wait, this is `assert_eq!(actual, expected)` with actual first). Specifically, `assert_eq!(cleared, 3)` on line 156, `assert_eq!(remaining, 0)` on line 161, `assert_eq!(other_remaining, 1)` on line 166, `assert_eq!(cleared, 3)` on line 186, `assert_eq!(alice_count, 2)` on line 243, etc. all put the actual value first. Line 130 has `assert_eq!(loaded.unwrap().id, record.id)` (actual, expected). Line 220 has `assert_eq!(actual_ids, expected_ids)` (actual, expected).
- **Recommendation**: Swap all `assert_eq!` calls to use `assert_eq!(expected, actual)` order: e.g., `assert_eq!(3, cleared)`, `assert_eq!(0, remaining)`, `assert_eq!(record.id, loaded.unwrap().id)`.
- **Rationale**: Convention compliance. The project explicitly requires `assert_eq!(expected, actual)` in CLAUDE.md.

### Finding 19: No tracing/logging in session service operations
- **Priority**: Nice-to-have
- **File**: crates/services/src/session_service/session_service.rs and session_store.rs
- **Location**: All methods
- **Issue**: None of the session service methods emit any tracing/logging. There are no `tracing::info!`, `tracing::warn!`, or `tracing::error!` calls anywhere in the session service module. For a service that manages user sessions (a security-critical component), this makes debugging production issues very difficult. When sessions fail to clear, save, or migrate, there is no diagnostic output.
- **Recommendation**: Add `tracing::debug!` for normal operations (session created, cleared), `tracing::warn!` for unusual situations (migration already applied), and `tracing::error!` for failures. Be careful not to log the database URL if it contains credentials.
- **Rationale**: Observability. Session management issues are notoriously hard to debug without logging.

### Finding 20: `BODHI_SESSION_DB_URL` default uses `bodhi_home` path that may not exist yet
- **Priority**: Nice-to-have
- **File**: crates/services/src/setting_service/default_service.rs
- **Location**: `build_all_defaults()` function (lines 244-247)
- **Issue**: The default value for `BODHI_SESSION_DB_URL` is computed as `format!("sqlite:{}", bodhi_home.join(SESSION_DB).display())`. This constructs the URL using the `bodhi_home` path, but the directory or file may not exist at the time the default is computed. This is fine because SQLite `create_if_missing(true)` is used in `connect_sqlite()`, but it means the default URL points to a potentially non-existent path. If some code reads this default and tries to validate the path before connecting, it would fail.
- **Recommendation**: No change needed for the current code. The `connect_sqlite()` method handles file creation. Just be aware of this ordering dependency.
- **Rationale**: Information only. The current code handles this correctly.

## Summary
- Total findings: 20 (Critical: 1, Important: 6, Nice-to-have: 13)
- **Critical (1)**: Finding 3 -- `$1` parameter style reliance on sqlx AnyPool translation for SQLite needs explicit verification/documentation.
- **Important (6)**: Finding 1 (Option-based dispatch anti-pattern), Finding 2 (dual pool overhead), Finding 4 (temp dir leak in tests), Finding 5 (trait duplication), Finding 8 (`with_secure(false)` in production), Finding 12 (fragile postgres test cleanup), Finding 16 (silent no-op on missing store).
- **Nice-to-have (13)**: Findings 6, 7, 9, 10, 11, 13, 14, 15, 17, 18, 19, 20.

### Key Architectural Concerns

1. **The `InnerStoreShared` pattern (Findings 1, 16)** is the most structurally significant issue. It allows invalid states and silent failures. Converting to an enum is a straightforward refactor that would eliminate two findings and improve type safety.

2. **Security hardening for production (Finding 8)** is important as PostgreSQL support implies production/multi-tenant deployment. Hardcoded `with_secure(false)` undermines session security.

3. **The dual-pool design (Finding 2)** is worth investigating but may be constrained by the tower-sessions-sqlx-store API. If so, document the constraint.

4. **Test quality (Findings 4, 9, 12, 18)** has several deviations from project conventions. These are low-risk but should be cleaned up to maintain consistency.
