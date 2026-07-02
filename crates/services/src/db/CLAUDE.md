# services/src/db — CLAUDE.md

SeaORM data layer: `DbService` implementations, encryption, RLS, and the migration chain in `sea_migrations/`.

## Migration Governance

### Prod deployment boundary — `m20250101_000020_api_alias_extra_fields`

Production is currently deployed **up to and including `m20250101_000020_api_alias_extra_fields`**.

- **Migrations `000000`–`000020` are IMMUTABLE.** They have run against live databases. Never edit them in place — SeaORM tracks applied migrations by name only (no checksum), so an edited file never re-runs on a deployed DB; the file and the live schema silently diverge. Any schema change to a table these created requires a **new later ALTER migration**.
  - Pattern to copy: `m20250101_000026_drop_app_access_request_flow_columns.rs` (drop columns) or `m20250101_000024_download_archived_at.rs` (add column). One `Table::alter()` per column — SQLite does not allow chaining multiple column changes in a single `ALTER TABLE`.
- **Migrations after `000020` are NOT yet deployed** and may currently be edited in place while iterating.
- **TODO:** once anything past `000020` ships to prod, move this boundary forward. From that point every schema change — including to the just-shipped migrations — must go through a proper new migration.

Why this matters: commit `979b1cbf` edited the already-deployed `000009` in place to drop `flow_type`/`redirect_uri`. Deployed DBs kept the columns (with `flow_type NOT NULL`) while the new code stopped setting them → `NOT NULL constraint failed` on insert. Fix: `000009` was reverted to its shipped form and the drop moved into new migration `000026`.

## Registering a migration
Edit `sea_migrations/mod.rs` in two places: add the `mod m20250101_0000XX_…;` declaration (numeric order) and append `Box::new(m20250101_0000XX_…::Migration),` as the last entry of the `migrations()` vec. `DeriveMigrationName` derives the version from the module name, so the filename is the identifier.
