# Code Review Index

## Review Scope
- **Ref**: HEAD~1 (commit 48f6673b)
- **Date**: 2026-02-25
- **Commit Message**: refactor: restructure session_service module and improve settings defaults
- **Files Changed**: 48 files across 7 crates (+2408/-558 lines)
- **Crates Affected**: services, auth_middleware, routes_app, server_app, lib_bodhiserver, lib_bodhiserver_napi, infra (CI/Makefile/Docker)

## Summary
- Total findings: 46
- Critical: 3 | Important: 11 | Nice-to-have: 24 | Positive/Info: 8

## Critical Issues (Blocks Merge)

| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| S3 | services | session_service/session_service.rs, session_store.rs | All `sqlx::query()` calls | `$1` param style relies on sqlx AnyPool translation for SQLite | Add explicit comment + regression test; OR use backend-aware queries | services-review.md |
| I1 | infra | .github/workflows/build.yml | services.postgres | CI PostgreSQL service provisioned but no tests run against it | Wire up `cargo test` in CI OR remove unused service | infra-review.md |
| I2 | infra | .github/workflows/build.yml | env section | `INTEG_TEST_SESSION_PG_URL` not set in CI | Add env var when test step is added | infra-review.md |

## Important Issues (Should Fix)

| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| S1 | services | session_store.rs | InnerStoreShared | Option-based dispatch anti-pattern allows invalid states | Convert to `enum StoreInner { Sqlite(SqliteStore), Postgres(PostgresStore) }` | services-review.md |
| S2 | services | session_service.rs | connect_sqlite, connect_postgres | Dual pool creation doubles connections | Investigate using single pool; if constrained by API, document | services-review.md |
| S4 | services | test_session_service.rs | create_session_service() | `std::mem::forget(temp_dir)` leaks temp dirs | Return `(DefaultSessionService, TempDir)` tuple | services-review.md |
| S5 | services | session_service.rs | AppSessionStoreExt + SessionService | Trait method signature duplication | Consolidate: make AppSessionStoreExt private, test-only methods behind feature | services-review.md |
| S8 | services | session_service.rs | session_layer() | `with_secure(false)` hardcoded; insecure for production PG deploys | Make configurable based on deployment mode or scheme | services-review.md |
| S12 | services | test_session_service.rs | postgres branch | Shared DB state + clear_all_sessions fragile on panic | Use DROP TABLE + remigrate per test; or per-test schema | services-review.md |
| D1 | routes_app | CLAUDE.md | OAuth flow test examples | 4 stale references to SqliteSessionService, old API | Update to DefaultSessionService, get_session_store() | downstream-review.md |
| I5 | infra | Makefile | test.backend | Now requires Docker - undocumented breaking change | Document requirement; OR make PG tests skipable | infra-review.md |
| I7 | infra | ci_optims/Cargo.toml | tower-sessions-sqlx-store | No backend features, defeats cache purpose | Add `features = ["sqlite", "postgres"]` | infra-review.md |
| I3 | infra | crates/services/ | .env.test | No .env.test.example for discoverability | Add template following codebase convention | infra-review.md |
| I13 | infra | test_session_service.rs | pg_url() | Postgres tests panic-fail if PG unavailable | Make tests skip gracefully when env var unset | infra-review.md |

## Nice-to-Have (Future)

| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| S6 | services | session_service.rs | DefaultSessionService.url | Stored URL only used during construction | Remove field, pass to migration directly | services-review.md |
| S7 | services | session_service.rs | connect_sqlite, connect_postgres | install_default_drivers() called multiple times | Move to Once wrapper | services-review.md |
| S9 | services | test_session_service.rs | All tests | Missing #[anyhow_trace], uses .unwrap() | Refactor to -> anyhow::Result<()> with ? | services-review.md |
| S13 | services | setting_service/service.rs | deployment_mode() | Returns raw String, not typed enum | Define DeploymentMode enum in objs | services-review.md |
| S14 | services | setting_service/constants.rs | SETTING_VARS | BODHI_SESSION_DB_URL editable at runtime | Exclude from runtime-editable or mark read-only | services-review.md |
| S15 | services | session_service.rs | migrate_custom() | Custom migration lacks versioning/tracking | Consider migration registry for future schema evolution | services-review.md |
| S18 | services | test_session_service.rs | All assert_eq! | assert_eq!(actual, expected) order reversed | Swap to assert_eq!(expected, actual) | services-review.md |
| S19 | services | session_service/ | All methods | No tracing/logging in session service | Add tracing::debug/warn/error | services-review.md |
| D2 | routes_app | test_utils/router.rs | Line 98 comment | Stale SqliteSessionService reference | Update comment | downstream-review.md |
| D3 | routes_app | test_utils/router.rs | Line 63 doc comment | Stale AppSessionStore reference | Update doc comment | downstream-review.md |
| D4 | services | test_utils/PACKAGE.md | Lines 372, 380-381 | Stale SqliteSessionService in examples | Update to DefaultSessionService | downstream-review.md |
| D5 | lib_bodhiserver | test_utils/PACKAGE.md | Lines 68, 276 | Stale session_db_path() references | Update to session_db_url() | downstream-review.md |
| I4 | infra | build.yml vs docker-compose | Health check | Inconsistent pg_isready params | Align CI to use `-U bodhi_test -d bodhi_sessions` | infra-review.md |
| I6 | infra | Makefile | test.backend | test.deps.down not called after tests | Document cleanup or add wrapper target | infra-review.md |
| I8 | infra | Cargo.toml | sqlx `any` feature | Workspace-level feature affects all crates | Move to services crate only | infra-review.md |

## Missing Test Coverage

| # | Crate | What's Missing | Priority | Report |
|---|-------|----------------|----------|--------|
| T1 | services | Error path tests: invalid URL, connection failure, migration failure | Important | services-review.md |
| T2 | services | Migration idempotency test (run migrate_custom twice) | Nice-to-have | services-review.md |
| T3 | services | Concurrent session operations test | Nice-to-have | services-review.md |
| T4 | services | `is_postgres_url()` edge cases (postgresql://, uppercase, etc.) | Nice-to-have | services-review.md |

## Fix Order (Layered)
When applying fixes, follow this order:
1. **services** architecture fixes (S1 enum, S5 trait cleanup, S6 url field) -> verify: `cargo test -p services`
2. **services** test fixes (S4 temp_dir, S9 anyhow_trace, S12 cleanup, S18 assert order) -> verify: `cargo test -p services`
3. **services** security (S8 secure cookie, S14 settings editability) -> verify: `cargo test -p services`
4. **infra** fixes (I7 ci_optims, I13 graceful skip, I3 env.test.example) -> verify: `make test.backend`
5. **infra** CI (I1+I2 wire up PG tests OR remove dead service) -> verify: push and check CI
6. **documentation** (D1-D5, crates/CLAUDE.md updates) -> no verification needed
7. Full backend: `make test.backend`

## Reports Generated
- `ai-docs/claude-plans/20260224-posgres/reviews/services-review.md` - 20 findings (1C, 6I, 13N)
- `ai-docs/claude-plans/20260224-posgres/reviews/downstream-review.md` - 13 findings (0C, 1I, 4N, 8 positive)
- `ai-docs/claude-plans/20260224-posgres/reviews/infra-review.md` - 13 findings (2C, 4I, 7N)
- `ai-docs/claude-plans/20260224-posgres/reviews/index.md` - this file
