# Fix: don't edit old migration in place — revert `000009`, drop columns in a new migration

## Context

Commit `979b1cbf` ("single-step 3rd-party OAuth access-request flow") removed the
`flow_type` and `redirect_uri` columns from `app_access_requests` by **editing the
already-shipped migration `m20250101_000009_app_access_requests.rs` in place** (the plan
explicitly assumed "no new migration; no prod data").

That assumption is wrong: there are **deployed environments** whose databases already ran
the original `000009` and therefore already have the `flow_type NOT NULL` / `redirect_uri`
columns. SeaORM tracks applied migrations **by name only (no checksum)**, so editing an
applied migration's file never re-runs it — the deployed schema stays as-is. Meanwhile the
new code no longer sets `flow_type`, so any insert fails with:

```
NOT NULL constraint failed: app_access_requests.flow_type
```

(This is exactly the error hit locally against `~/.bodhi-dev-makefile/bodhi.sqlite`.)

Commit `979b1cbf` is **not deployed yet**, so we can rewrite this history cleanly:
1. **Revert** `m20250101_000009` to its original, shipped form (re-add both columns) so it
   matches every deployed DB.
2. **Add a new later migration** (`m20250101_000026_drop_app_access_request_flow_columns`)
   that drops `flow_type` and `redirect_uri`.

After the full chain runs, both fresh DBs (create-with-columns → drop) and deployed DBs
(already-have-columns → drop) converge to the same schema. No runtime code references these
columns (the entity/repository/service in `crates/services/src/app_access_requests/` are
already clean — verified), so nothing touches them during the migration window.

## Changes

### 1. Revert migration `000009` to its pre-`979b1cbf` form
File: `crates/services/src/db/sea_migrations/m20250101_000009_app_access_requests.rs`

Restore the two removed items so the `create_table` matches the original shipped schema
(source of truth = `git show 979b1cbf^:crates/services/src/db/sea_migrations/m20250101_000009_app_access_requests.rs`):

- Re-add `FlowType` and `RedirectUri` to the `enum AppAccessRequests`.
- Re-add the two column defs in `up()`, in original order (between `AppDescription` and `Status`):
  ```rust
  .col(string(AppAccessRequests::FlowType))        // NOT NULL, matches deployed
  .col(string_null(AppAccessRequests::RedirectUri))
  ```

Everything else in the file (indexes, unique partial index, Postgres RLS policies, `down()`)
is already original — leave untouched.

### 2. New migration: drop the two columns
File (new): `crates/services/src/db/sea_migrations/m20250101_000026_drop_app_access_request_flow_columns.rs`

Follow the exact existing pattern in
`m20250101_000018_drop_mcp_tools_columns.rs` (backend-agnostic `Table::alter().drop_column()`,
no-op `down()` since removal is permanent):

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppAccessRequests {
  Table,
  FlowType,
  RedirectUri,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(Table::alter().table(AppAccessRequests::Table).drop_column(AppAccessRequests::FlowType).to_owned())
      .await?;
    manager
      .alter_table(Table::alter().table(AppAccessRequests::Table).drop_column(AppAccessRequests::RedirectUri).to_owned())
      .await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(()) // columns permanently removed
  }
}
```

Notes: neither column participates in any index or RLS policy (verified — indexes are on
`status`/`app_client_id`/`tenant_id`/`access_request_scope`; RLS uses `tenant_id`), so a plain
`drop_column` is safe on both SQLite (3.35+) and Postgres.

### 3. Register the new migration
File: `crates/services/src/db/sea_migrations/mod.rs`

- Add `mod m20250101_000026_drop_app_access_request_flow_columns;` after the `000025` mod line.
- Add `Box::new(m20250101_000026_drop_app_access_request_flow_columns::Migration),` as the last
  entry in the `migrations()` vec (after `000025_token_grants`).

### No code/test changes needed
- `crates/services/src/app_access_requests/{app_access_request_entity.rs, access_request_repository.rs, access_request_service.rs}` — already free of `flow_type`/`redirect_uri` (from `979b1cbf`); leave as-is.
- Repository/service/routes tests already dropped their `flow_type`/`redirect_uri` value sets in `979b1cbf`; the final post-migration schema has no such columns, so they stay consistent.

## Verification

1. **Fresh DB (create → drop chain):** delete a scratch dev DB and let the full migrator run.
   Strongest check that `000009` create + `000026` drop converge correctly:
   ```bash
   rm -f ~/.bodhi-dev/bodhi.sqlite   # scratch home, not the makefile one with local aliases
   cargo run --bin bodhi -- serve --port 1136   # BODHI_HOME=~/.bodhi-dev
   sqlite3 ~/.bodhi-dev/bodhi.sqlite '.schema app_access_requests' | grep -c flow_type   # expect 0
   ```
2. **Simulated deployed DB (already-has-columns → drop):** re-add the columns to a test DB,
   mark `000009` as applied, then run migrations and confirm `000026` drops them cleanly
   (mirrors what real deployments will do).
3. **Backend tests:** `cargo test -p services` (migrator runs in `test_db_service` fixture,
   exercising the full create→drop chain), then `make test.backend`. Capture long runs to a
   file per house convention.
4. **End-to-end (the original repro):** with the rebuilt binary serving, run the test-oauth-app
   on :5173, click **Request access → login**, and confirm `request-access` succeeds (no
   `NOT NULL constraint failed`).

### One local-DB caveat (not a code issue)
During diagnosis I manually `DROP COLUMN`-ed `flow_type`/`redirect_uri` from
`~/.bodhi-dev-makefile/bodhi.sqlite`. That DB has `000009` marked applied but the columns
already gone, so the new `000026` will fail there with "no such column". Before verifying
against that specific home, either reset it (`rm ~/.bodhi-dev-makefile/bodhi.sqlite`) or re-add
the two columns so `000026` can drop them. This affects only that one hand-edited local DB, not
clean deployments or fresh DBs.

## Commit
Single focused commit on `main` (trunk-based): revert `000009` + new `000026` + `mod.rs`
registration. Run gate checks (`make format`, `cargo test -p services` / `make test.backend`)
before committing.
