# Fix: PostgreSQL E2E Test Startup Failure

## Context

The PostgreSQL E2E test project (`--project postgres`) fails to start because the NAPI binary is stale. Migration `m20250101_000015_fix_access_request_scope_index` was added to the Rust code and applied to the PostgreSQL test database (via `cargo test` or a direct server run), but the NAPI binary (`@bodhiapp/app-bindings`) hasn't been rebuilt to include it.

When Playwright starts the postgres server via NAPI, SeaORM's migrator sees migration 15 in the `seaql_migrations` table but doesn't find it in the `Migrator::migrations()` list (because the binary is old), producing:

```
Migration file of version 'm20250101_000015_fix_access_request_scope_index' is missing,
this migration has been applied but its file is missing.
```

## Root Cause

**Stale NAPI binary.** The migration file exists in source (`crates/services/src/db/sea_migrations/m20250101_000015_fix_access_request_scope_index.rs`) and is registered in `mod.rs`, but the compiled NAPI `.node` binary predates its addition.

## Fix

Rebuild the NAPI binary:

```bash
cd crates/lib_bodhiserver_napi && npm run build
```

This recompiles the Rust code (including the new migration) into the NAPI binary.

## Verification

After rebuild, run a single postgres test:

```bash
cd crates/lib_bodhiserver_napi && npx playwright test --project=postgres setup-flow
```

If there are further issues (e.g., the DB has stale data from prior runs), reset the postgres databases:

```bash
psql "postgres://bodhi_test:bodhi_test@localhost:64320/bodhi_app" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
psql "postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
```

Then the fresh server start will re-run all migrations cleanly.
