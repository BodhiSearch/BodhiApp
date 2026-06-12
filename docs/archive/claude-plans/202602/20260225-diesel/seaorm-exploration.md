# SeaORM Exploration: Feasibility Analysis for BodhiApp Dual-Database Support

## Executive Summary

SeaORM is **recommended** as the ORM for BodhiApp's dual-database (SQLite + PostgreSQL) migration. It is async-native (built on sqlx), supports multiple backends via a single `DatabaseConnection` enum, has programmatic migration tooling that handles backend-specific DDL, and has strong community health (8.2k GitHub stars, 1.5M downloads/month). The migration preserves the existing repository trait architecture -- SeaORM becomes an implementation detail behind the current `DbService` API.

Diesel was evaluated and rejected due to lack of native async support (requires `diesel_async`, a separate crate with lower adoption).

## 1. Feasibility Assessment

### 1.1 Can SeaORM serve dual-DB + multi-tenant + RLS requirements?

**Yes**, with the following evidence:

| Requirement | SeaORM Support | Notes |
|---|---|---|
| SQLite + PostgreSQL | Native `DatabaseConnection` enum | URL-based backend detection at runtime |
| Async-first | Built on sqlx, tokio-native | No `spawn_blocking()` needed |
| ON CONFLICT / UPSERT | `.on_conflict()` API | Works across SQLite and PG |
| JSON columns | `serde_json::Value` mapping | Via `ColumnType::Text` with serde |
| Batch insert | `insert_many()` | Chunking for large datasets |
| Case-insensitive | `Expr::col().lower()` | Or CITEXT column trait override for PG |
| Raw SQL escape hatch | `Statement::from_sql_and_values()` | Backend-specific raw SQL when DSL insufficient |
| Programmatic migrations | `sea-orm-migration` crate | `SchemaManager` abstracts DDL per backend |
| Backend-conditional DDL | `manager.get_database_backend()` | Conditional logic in migration code |
| Multi-tenant (future org_id) | Standard column + filtering | SeaORM query conditions |
| PostgreSQL RLS (future) | Raw SQL in migrations | RLS policies via `execute_unprepared()` |
| Partial indexes | Raw SQL in migrations | SQLite WHERE indexes via raw DDL |

### 1.2 Version Recommendation

**SeaORM 2.0.0-rc.x** (latest: rc.34, Feb 21 2026). The team states: "Other than dependency upgrades (sqlx 0.9), there won't be more major breaking changes." Version 1.1.x is no longer actively maintained.

Note: SeaORM 2.0 uses sqlx 0.9.x. BodhiApp currently uses sqlx 0.8.6. The session service (staying on sqlx) would need to either:
- Upgrade to sqlx 0.9.x (if compatible with tower-sessions-sqlx-store)
- Or coexist via cargo feature resolution (both versions linked)

This dependency alignment needs validation during the prototype.

## 2. Codebase Analysis

### 2.1 Query Count by Repository

| Repository | File | Queries | Complexity |
|---|---|---|---|
| ModelRepository | service_model.rs | 14 | High (UPSERT, JSON, batch IN, encryption) |
| McpRepository | service_mcp.rs | 34 | High (JOINs, JSON, COLLATE NOCASE) |
| ToolsetRepository | service_toolset.rs | 11 | Medium (COLLATE NOCASE, multi-table, JSON) |
| AppInstanceRepository | repository_app_instance.rs | 5 | Medium (encryption, UPSERT) |
| TokenRepository | service_token.rs | 4 | Medium (encryption, UPSERT, expiry) |
| UserAliasRepository | service_user_alias.rs | 4 | Low (basic CRUD, JSON) |
| SettingsRepository | service_settings.rs | 4 | Low (UPSERT) |
| AccessRepository | service_access.rs | 2 | Low (basic CRUD) |
| AccessRequestRepository | service_access_request.rs | 1 | Low (pagination) |
| DbCore | service.rs | 2 | Low (seed, reset) |
| **Total** | | **81** | |

### 2.2 SQLite-Specific Patterns Found

| Pattern | Occurrences | SeaORM Handling |
|---|---|---|
| `?` bind parameters | All 81 queries | SeaORM handles per-backend automatically |
| `INSERT OR IGNORE` | 1 (seed) | `on_conflict().do_nothing()` |
| `strftime('%s','now')` | 1 (seed) | Replace with bind parameter via TimeService |
| `COLLATE NOCASE` in queries | 3 | `Expr::col().lower()` or custom expression |
| `COLLATE NOCASE` in DDL | 5 migrations | Backend-conditional in programmatic migration |
| `AUTOINCREMENT` | 3 migrations | `pk_auto()` in migration (backend-aware) |
| `INTEGER` for booleans | Widespread | SeaORM handles i64 columns |
| `TEXT` timestamps (model_metadata only) | 3 columns | Normalize to INTEGER (no production data) |
| Dynamic tuple IN clause | 1 (batch_get) | Raw SQL escape hatch |
| Multi-table DELETE | 1 (reset_all) | SeaORM entity-level deletes |

### 2.3 DSL Coverage Estimate

- **DSL-friendly**: ~73 queries (~90%) -- standard CRUD, COUNT, pagination, UPSERT, simple WHERE
- **Needs raw SQL or special handling**: ~8 queries (~10%):
  - Dynamic tuple IN clause (1 query in service_model.rs)
  - Complex multi-table JOINs (estimated 3-5 in service_mcp.rs)
  - COLLATE NOCASE WHERE clauses (3 queries -- can use `Expr::lower()` DSL)
  - Multi-table DELETE sequence (1 in reset_all_tables)

This is well within the acceptable threshold of <20% raw SQL.

## 3. Risk Analysis

### 3.1 Technical Risks

| Risk | Severity | Mitigation |
|---|---|---|
| sqlx version mismatch (0.8 vs 0.9) | Medium | Validate during prototype; session service may need sqlx upgrade |
| SeaORM 2.0 is RC, not stable | Low | RC is feature-complete; team says no more breaking changes |
| Complex JOINs in McpRepository | Medium | Raw SQL escape hatch available; validate during McpRepository phase |
| Compile time increase | Low | SeaORM derives add ~moderate overhead; measure during prototype |
| Tuple IN clause not in DSL | Low | Raw SQL `Statement::from_sql_and_values()` handles this |
| COLLATE NOCASE migration gap | Low | Use `Expr::lower()` for queries; backend-conditional DDL for indexes |

### 3.2 Unknowns

1. **sqlx 0.8 vs 0.9 coexistence**: Can the session service (sqlx 0.8.6 via tower-sessions-sqlx-store) coexist with SeaORM 2.0 (sqlx 0.9.x) in the same binary?
2. **SeaORM entity generation from existing schema**: Can `sea-orm-cli generate entity` produce entities from the existing SQLite schema, or must entities be written manually?
3. **Encryption at entity level**: Can SeaORM ActiveModel hooks or custom `FromQueryResult` implementations handle transparent encrypt/decrypt?

## 4. Architecture Design

### 4.1 Repository Trait Preservation

```
   DbService trait (public API, unchanged)
        |
   DefaultDbService (new, replaces SqliteDbService)
        |
   sea_orm::DatabaseConnection (internal)
        |
   SQLite or PostgreSQL (runtime selection)
```

- All 9 repository traits remain as the public API
- `MockDbService` (mockall) stays unchanged for downstream crate testing
- `DefaultDbService` holds `DatabaseConnection` + `Arc<dyn TimeService>` + `encryption_key`
- SeaORM entities live in `crates/services/src/db/entities/`

### 4.2 Migration Architecture

- `crates/services/migration/` -- SeaORM migration crate (separate directory from sqlx migrations)
- Programmatic Rust migrations with `SchemaManager`
- Backend-conditional DDL via `manager.get_database_backend()`
- Existing `crates/services/migrations/` (sqlx SQL files) kept temporarily for session service
- 14 SQL migrations converted to ~14 Rust migration structs

### 4.3 Error Handling Adaptation

```
sea_orm::DbErr -> DbError::SeaOrmError (new variant) -> AppError -> ApiError
```

Add `SeaOrmError` variant to `DbError` alongside existing `SqlxError`. During migration, both variants coexist. After full migration, `SqlxError` can be removed from the app DB path.

## 5. Recommendation

**Go: Proceed with SeaORM migration.**

Rationale:
- SeaORM is async-native, eliminating the `spawn_blocking()` concern that disqualified Diesel
- Built on sqlx (familiar ecosystem), so migration is incremental -- not a paradigm shift
- Programmatic migrations with backend-conditional DDL solve the dual-database migration problem cleanly
- ~90% of queries map to SeaORM DSL; the remaining ~10% use the raw SQL escape hatch
- Repository trait architecture is preserved, keeping MockDbService and downstream crates untouched
- Community health is strong (8.2k stars, 1.5M monthly downloads, active RC releases)
- No production deployments means zero data migration risk

## 6. Phased Migration Plan

### Phase order (following crate dependency chain):

1. **Prototype (ModelRepository)** -- Validate all patterns end-to-end
2. **Remaining services repositories** -- One at a time, simplest first
3. **DbCore adaptation** -- Migration runner, seed, reset
4. **lib_bodhiserver** -- AppServiceBuilder changes
5. **Downstream crates** -- Import renames
6. **CI + documentation** -- PG in CI, docs update

### Detailed repository migration order (services crate):

| Order | Repository | Queries | Key Patterns to Validate |
|---|---|---|---|
| 1 | ModelRepository (PROTOTYPE) | 14 | UPSERT, JSON, batch IN, encryption, pagination |
| 2 | SettingsRepository | 4 | UPSERT, basic CRUD |
| 3 | AccessRepository | 2 | Basic CRUD |
| 4 | AccessRequestRepository | 1 | Pagination |
| 5 | UserAliasRepository | 4 | JSON serialization |
| 6 | AppInstanceRepository | 5 | Encryption, UPSERT |
| 7 | TokenRepository | 4 | Encryption, expiry |
| 8 | ToolsetRepository | 11 | COLLATE NOCASE, multi-table |
| 9 | McpRepository | 34 | Complex JOINs, JSON, COLLATE NOCASE |
| 10 | DbCore | 2 | Migration runner, seed, reset |

## 7. Prototype Results

### What was validated

| Aspect | Result | Notes |
|---|---|---|
| SeaORM version | 1.1.19 (stable) | Compatible with sqlx 0.8.6, no version conflicts |
| SQLite tests | 8/8 passed | In-memory temp file per test |
| PostgreSQL tests | 6/6 passed | Docker container on port 64320 |
| Existing tests | 355/355 passed | Zero regressions |
| Downstream compile | All 11 crates | auth_middleware, routes_app, server_app, bodhi, etc. |
| MockDbService | Unchanged | Auto-generated from DbService trait, works as before |
| Raw SQL needed | 0% | Even `batch_get_metadata_by_files` uses `Condition::any()` DSL |
| ON CONFLICT | Works | Both SQLite and PG from same code |
| Encryption | Works | Repository-level encrypt/decrypt unchanged |
| JSON columns | Works | serde_json serialization in entity columns |
| Pagination | Works | SeaORM `offset()/limit()` |
| sea-orm-migration | Works | `SchemaManager` + raw SQL for backend-specific DDL |

### Architecture proven

```
DefaultDbService::new(sea_orm::DatabaseConnection, Arc<dyn TimeService>, encryption_key)
    |
    impl ModelRepository for DefaultDbService  (SeaORM DSL)
    |
    Works on both SQLite and PostgreSQL
```

Key patterns validated:
- `download_request::Entity::find_by_id()` -- single row lookup
- `Entity::find().filter().order_by_desc().offset().limit().all()` -- paginated list
- `Entity::insert(active_model).on_conflict(...).exec()` -- upsert
- `Entity::delete_by_id()` -- delete
- `Entity::delete_many().filter(...)` -- conditional delete
- `Entity::find().filter(Condition::any().add(...))` -- dynamic OR conditions
- `Entity::update(active_model)` -- partial update with `NotSet` fields

### Files created/modified (prototype)

| Action | File |
|---|---|
| New | `crates/services/src/db/entities/mod.rs` |
| New | `crates/services/src/db/entities/download_request.rs` |
| New | `crates/services/src/db/entities/api_model_alias.rs` |
| New | `crates/services/src/db/entities/model_metadata.rs` |
| New | `crates/services/src/db/sea_migrations/mod.rs` |
| New | `crates/services/src/db/sea_migrations/m20250101_000001_download_requests.rs` |
| New | `crates/services/src/db/sea_migrations/m20250101_000002_api_model_aliases.rs` |
| New | `crates/services/src/db/sea_migrations/m20250101_000003_model_metadata.rs` |
| New | `crates/services/src/db/default_service.rs` |
| New | `crates/services/src/db/service_model_sea.rs` |
| New | `crates/services/src/db/test_model_repository_sea.rs` |
| Modified | `crates/services/src/db/mod.rs` (new module registrations) |
| Modified | `crates/services/src/db/error.rs` (SeaOrmDbError variant) |
| Modified | `crates/services/Cargo.toml` (sea-orm, sea-orm-migration deps) |
| Modified | `Cargo.toml` (workspace sea-orm deps) |
| Modified | `docker-compose-test-deps.yml` (bodhi_app_db instance) |
| Modified | `crates/services/.env.test.example` (PG URL) |

## 8. Full Implementation Plan (Post-Prototype)

### Phase A: Merge DefaultDbService into SqliteDbService

1. Rename `SqliteDbService` to `DefaultDbService` across all files
2. Change `pool: SqlitePool` to `db: sea_orm::DatabaseConnection`
3. Add `sqlx_pool()` helper to extract `SqlitePool` for unconverted repos
4. Move `service_model_sea.rs` content into `service_model.rs` (replacing sqlx implementation)
5. Delete `service_model_sea.rs`, `default_service.rs`
6. Migrate `test_model_repository_sea.rs` patterns into `test_model_repository.rs`
7. Update TestDbService to wrap `DefaultDbService`
8. Gate: `cargo test -p services` -- all 355+ tests pass

### Phase B: Convert remaining repositories (one at a time)

For each repository, follow this pattern:

1. Create SeaORM entity in `entities/`
2. Add migration in `sea_migrations/`
3. Replace sqlx queries with SeaORM DSL
4. Update tests (add PG parameterization)
5. Gate: `cargo test -p services`

**Order** (simplest first):
1. SettingsRepository (4 queries) -- UPSERT pattern already proven
2. AccessRepository (2 queries) -- trivial CRUD
3. AccessRequestRepository (1 query) -- pagination already proven
4. UserAliasRepository (4 queries) -- JSON pattern already proven
5. AppInstanceRepository (5 queries) -- encryption pattern already proven
6. TokenRepository (4 queries) -- encryption + expiry
7. ToolsetRepository (11 queries) -- COLLATE NOCASE (use `Expr::col().to_lowercase()`)
8. McpRepository (34 queries) -- complex JOINs, most effort

### Phase C: Convert DbCore

1. Replace `sqlx::migrate!()` with `Migrator::up(&db, None).await`
2. Convert `seed_toolset_configs()` to SeaORM insert
3. Convert `reset_all_tables()` to SeaORM entity deletes
4. Replace `DbPool::connect()` with `sea_orm::Database::connect()`
5. Remove `sqlite_pool.rs`
6. Remove `sqlx_pool()` helper
7. Gate: `cargo test -p services`

### Phase D: Downstream crate updates

1. `lib_bodhiserver` -- `AppServiceBuilder` creates `DefaultDbService` from URL
2. `auth_middleware` -- import renames if any
3. `routes_app` -- import renames if any
4. `server_app` -- parameterized PG integration tests
5. `lib_bodhiserver_napi` -- export `BODHI_DATABASE_URL`
6. Gate: `cargo test` (full workspace)

### Phase E: Cleanup and documentation

1. Remove old sqlx migration files (they're now in `sea_migrations/`)
2. Remove sqlx `migrate!()` macro usage
3. Update `crates/services/CLAUDE.md` and `PACKAGE.md`
4. Update root `CLAUDE.md`
5. Update `ai-docs/01-architecture/backend-architecture.md`
6. CI: ensure PG app DB service container is configured

### Estimated effort per phase

| Phase | Repositories | Estimated queries | Effort |
|---|---|---|---|
| A | Merge + ModelRepository | 14 | Small (mostly renaming) |
| B1-B4 | Settings, Access, AccessRequest, UserAlias | 11 | Small (simple CRUD) |
| B5-B6 | AppInstance, Token | 9 | Medium (encryption) |
| B7 | Toolset | 11 | Medium (COLLATE NOCASE) |
| B8 | McpRepository | 34 | Large (complex JOINs) |
| C | DbCore | 2 | Small |
| D | Downstream | 0 | Small (import renames) |
| E | Cleanup | 0 | Small |
