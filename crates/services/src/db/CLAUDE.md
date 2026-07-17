# services/src/db — CLAUDE.md

SeaORM data layer: `DbService` implementations, encryption, RLS, and the migration chain in `sea_migrations/`.

## Migration Governance

**Every committed migration is IMMUTABLE. There is no in-place editing, ever.**

BodhiApp runs in production. Every migration in `sea_migrations/` has run against live databases, so all of them are frozen — including the one you added last week. Never edit a committed migration file. SeaORM tracks applied migrations by name only (no checksum), so an edited file never re-runs on a deployed DB; the file and the live schema silently diverge.

Any schema change — add column, drop column, alter type, add index, change a constraint — requires a **new later migration**.

- Pattern to copy: `m20250101_000026_drop_app_access_request_flow_columns.rs` (drop columns) or `m20250101_000024_download_archived_at.rs` (add column). One `Table::alter()` per column — SQLite does not allow chaining multiple column changes in a single `ALTER TABLE`.
- A new migration must be a no-op-safe forward step on a database that already holds production rows. Adding a `NOT NULL` column means supplying a default or backfilling first; dropping a column means the code that writes it is already gone.

Why this matters: commit `979b1cbf` edited the already-deployed `000009` in place to drop `flow_type`/`redirect_uri`. Deployed DBs kept the columns (with `flow_type NOT NULL`) while the new code stopped setting them → `NOT NULL constraint failed` on insert. Fix: `000009` was reverted to its shipped form and the drop moved into new migration `000026`.

## Data migration and backfill

Existing rows are real user data. A schema change that leaves them inconsistent is a production incident, not a dev-loop annoyance.

- **Plan the data alongside the schema.** If a new column must be populated for existing rows, the same migration (or an immediately following one) backfills it. Pattern to copy: `m20250101_000023_api_alias_name.rs` backfills `name = id` for rows that predate the column.
- **Backfill before you tighten.** Add the column nullable → backfill → add the `NOT NULL`/unique constraint in a later step. A single migration that adds a constrained column to a populated table will fail on deploy.
- **Semantic changes need a data migration too.** Re-tagging a serde enum, changing a JSON column's shape, or renaming an `ApiFormat` variant all strand rows written by the old code. Migrate the stored values; do not assume the table is empty.
- **Destructive steps are one-way.** `down()` exists for local dev; production rolls forward. Never rely on a `down()` to undo a deployed migration.

## Registering a migration
Edit `sea_migrations/mod.rs` in two places: add the `mod m20250101_0000XX_…;` declaration (numeric order) and append `Box::new(m20250101_0000XX_…::Migration),` as the last entry of the `migrations()` vec. `DeriveMigrationName` derives the version from the module name, so the filename is the identifier.
