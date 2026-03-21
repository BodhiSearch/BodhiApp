# Plan: Consolidate Migrations 0006-0013

## Context

Migrations 0001-0005 are released. Migrations 0006-0013 have never been released, so they can be freely reorganized. Several migrations create tables then modify them in later migrations, and one table (`app_client_toolset_configs`) was created in 0008 then dropped in 0010. The goal is to produce a clean, minimal migration set where each table is created in its final form.

## Current State (0006-0013): 8 migrations, 16 files

| # | Name | Action |
|---|------|--------|
| 0006 | model_metadata | CREATE `model_metadata` |
| 0007 | toolsets_config | CREATE `toolsets` (with scope_uuid) + CREATE `app_toolset_configs` (with scope/scope_uuid) |
| 0008 | app_client_toolset_configs | CREATE `app_client_toolset_configs` |
| 0009 | user_aliases | CREATE `user_aliases` |
| 0010 | app_client_toolset_configs_drop | DROP `app_client_toolset_configs` |
| 0011 | app_access_requests | CREATE `app_access_requests` |
| 0012 | toolsets_scope_to_toolset_type | ALTER `toolsets`: add toolset_type, drop scope_uuid |
| 0013 | app_toolset_configs_tool_type | REBUILD `app_toolset_configs`: replace scope/scope_uuid with toolset_type |

## Target State: 4 migrations, 8 files

| New # | Name | Source | Action |
|-------|------|--------|--------|
| 0006 | model_metadata | Old 0006 unchanged | CREATE `model_metadata` |
| 0007 | toolsets_config | **Merged** old 0007+0012+0013 | CREATE `toolsets` (final schema with toolset_type) + CREATE `app_toolset_configs` (final schema with toolset_type) |
| 0008 | user_aliases | Old 0009 renumbered | CREATE `user_aliases` |
| 0009 | app_access_requests | Old 0011 renumbered | CREATE `app_access_requests` |

**Eliminated**: Old 0008+0010 (create+drop cancel out), old 0012+0013 (folded into new 0007).

## Step-by-step Implementation

### Step 1: Delete old files (12 files)

Delete these migration files:
- `0008_app_client_toolset_configs.up.sql`
- `0008_app_client_toolset_configs.down.sql`
- `0009_user_aliases.up.sql`
- `0009_user_aliases.down.sql`
- `0010_app_client_toolset_configs_drop.up.sql`
- `0010_app_client_toolset_configs_drop.down.sql`
- `0011_app_access_requests.up.sql`
- `0011_app_access_requests.down.sql`
- `0012_toolsets_scope_to_toolset_type.up.sql`
- `0012_toolsets_scope_to_toolset_type.down.sql`
- `0013_app_toolset_configs_tool_type.up.sql`
- `0013_app_toolset_configs_tool_type.down.sql`

### Step 2: Rewrite `0007_toolsets_config.up.sql` (merge old 0007+0012+0013)

Write final-schema `toolsets` table (with `toolset_type TEXT` instead of `scope_uuid TEXT NOT NULL`) and final-schema `app_toolset_configs` table (with `toolset_type TEXT NOT NULL` instead of `scope`/`scope_uuid`).

**toolsets** final columns: id, user_id, **toolset_type**, name, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at. Indexes on user_id, toolset_type, (user_id, toolset_type).

**app_toolset_configs** final columns: id, **toolset_type**, enabled, updated_by, created_at, updated_at. Unique index on toolset_type.

### Step 3: Rewrite `0007_toolsets_config.down.sql`

Drop both tables with their indexes (toolset_type-based indexes, not old scope_uuid indexes).

### Step 4: Create new `0008_user_aliases.{up,down}.sql`

Content identical to old `0009_user_aliases.{up,down}.sql`.

### Step 5: Create new `0009_app_access_requests.{up,down}.sql`

Content identical to old `0011_app_access_requests.{up,down}.sql`.

### Step 6: Remove dead `AppClientToolsetConfigRow` code

Since the `app_client_toolset_configs` table no longer exists, remove dead code:

1. **`crates/services/src/db/objs.rs`** ~line 203-215: Delete `AppClientToolsetConfigRow` struct
2. **`crates/services/src/db/toolset_repository.rs`** ~line 2: Remove `AppClientToolsetConfigRow` from import; ~lines 55-60: Remove `get_app_client_toolset_config` and `upsert_app_client_toolset_config` trait methods
3. **`crates/services/src/db/service.rs`** ~line 4: Remove `AppClientToolsetConfigRow` from import; ~lines 1733-1801: Remove both impl methods
4. **`crates/services/src/test_utils/db.rs`** ~line 3: Remove `AppClientToolsetConfigRow` from import; ~lines 520-539: Remove from `TestDbService` impl; ~lines 737-738: Remove from `MockDbService` mock definition

Note: `AppClientToolset` in `auth_service.rs` is a **different** struct (Keycloak API model) - do NOT touch it.

## Final Directory State

```
crates/services/migrations/
  0001_download-requests.{up,down}.sql      (unchanged)
  0002_pending-access-requests.{up,down}.sql (unchanged)
  0003_create_api_tokens.{up,down}.sql       (unchanged)
  0004_api_models.{up,down}.sql              (unchanged)
  0005_api_model_forward_all.{up,down}.sql   (unchanged)
  0006_model_metadata.{up,down}.sql          (unchanged)
  0007_toolsets_config.{up,down}.sql         (REWRITTEN - merged final schema)
  0008_user_aliases.{up,down}.sql            (NEW - content from old 0009)
  0009_app_access_requests.{up,down}.sql     (NEW - content from old 0011)
```

## Verification

1. `cargo check -p services` - verify compilation with merged migrations and removed dead code
2. `make test.backend` - run full backend test suite
3. Visually confirm the migrations directory has exactly 9 numbered migrations (0001-0009)
