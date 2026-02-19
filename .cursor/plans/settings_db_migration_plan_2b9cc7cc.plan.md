---
name: Settings DB Migration Plan
overview: Migrate settings persistence from settings.yaml file to an app_settings SQLite table. settings.yaml remains as a read-only config source (like env vars). Writes from UI/API go to DB instead of file.
todos:
  - id: objs-setting-source
    content: Add Database variant to SettingSource enum (between SettingsFile and Default), extend SettingInfo with group/editable fields
    status: pending
  - id: services-migration
    content: Create 0013_app_settings.up.sql migration with key-value table (no seeding)
    status: pending
  - id: services-repository
    content: Add SettingsRepository trait methods to DbService (get/set/delete/list)
    status: pending
  - id: services-registry
    content: Create static settings registry defining group, metadata, per-AppType editability for each setting
    status: pending
  - id: services-refactor-setting-service
    content: "Refactor DefaultSettingService: two-phase init, add DB cache layer, redirect writes from file to DB, keep settings.yaml as read-only source"
    status: pending
  - id: routes-app-settings
    content: "Update settings route handlers: remove hardcoded EDIT_SETTINGS_ALLOWED, use registry for editability, include group/editable in response"
    status: pending
  - id: lib-bodhiserver-init
    content: "Update app_service_builder: inject DB pool into SettingService after DB connect (Phase 2 init)"
    status: pending
  - id: blockers-document
    content: Create ai-docs/02-features/cluster-stateless-blockers.md documenting remaining stateful components for cluster deployment
    status: pending
  - id: openapi-ts-regen
    content: Regenerate OpenAPI spec and TypeScript client types, run full test suite
    status: pending
isProject: false
---

# Settings Persistence Migration: settings.yaml to SQLite DB

## What Changed (vs Previous Plan)

The previous plan treated settings.yaml as defunct. The corrected understanding:

- **settings.yaml had two uses**: (1) UI/API persistence target for admin changes, (2) direct file editing by power users for settings not exposed in UI
- **This migration**: moves only the **persistence** (use case 1) from file to DB
- **settings.yaml stays**: as a read-only config source -- power users can still create/edit it to override settings, just like env vars
- **No cluster/Container mode changes** in this scope -- purely a persistence layer swap

## Context and Decisions

**New precedence** (highest to lowest):

```
System > CommandLine > Environment > SettingsFile > Database > Default
```

- `SettingsFile` = settings.yaml (read-only, never written to by app anymore)
- `Database` = new `app_settings` SQLite table (persistence target for UI/API changes)
- settings.yaml **overrides** DB values, giving power users an escape hatch

**No auto-migration**: Table starts empty. Existing settings.yaml values continue to work via the SettingsFile layer. Admins can re-set values via API which persists to DB.

**devops/ impact**: None. Dockerfiles never reference settings.yaml -- it's a runtime artifact in `BODHI_HOME`. `defaults.yaml` (build-time, embedded at `/app/defaults.yaml`) is unaffected.

**Editability**: Curated allowlist per `AppType` from a settings registry, replacing the hardcoded `EDIT_SETTINGS_ALLOWED` in [route_settings.rs](crates/routes_app/src/routes_settings/route_settings.rs):13.

---

## Settings Classification

### DB-eligible settings (admin-editable via API)

- **server group**: `BODHI_PUBLIC_SCHEME`, `BODHI_PUBLIC_HOST`, `BODHI_PUBLIC_PORT`, `BODHI_CANONICAL_REDIRECT`
- **logging group**: `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`
- **inference group** (Native only): `BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`

### Bootstrap / env-only settings (never in DB)

`BODHI_HOME`, `BODHI_HOST`, `BODHI_PORT`, `BODHI_SCHEME`, `BODHI_LOGS`, `BODHI_ENCRYPTION_KEY`, `HF_HOME`, `BODHI_EXEC_LOOKUP_PATH`, `BODHI_EXEC_TARGET`, `BODHI_EXEC_NAME`, `BODHI_EXEC_VARIANTS`, `BODHI_LLAMACPP_ARGS`*, `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`, all system settings

---

## Implementation (Upstream to Downstream)

### 1. objs crate

**File**: [crates/objs/src/envs.rs](crates/objs/src/envs.rs)

- Add `Database` variant to `SettingSource` enum, positioned between `SettingsFile` and `Default`:

```rust
pub enum SettingSource {
  System,
  CommandLine,
  Environment,
  SettingsFile,
  Database,    // NEW
  Default,
}
```

- Extend `SettingInfo` with two new fields:

```rust
pub struct SettingInfo {
  pub key: String,
  pub current_value: serde_yaml::Value,
  pub default_value: serde_yaml::Value,
  pub source: SettingSource,
  pub metadata: SettingMetadata,
  pub group: Option<String>,  // NEW: "server", "logging", "inference", or None for system/env-only
  pub editable: bool,         // NEW: whether admin can edit via API in current app_type
}
```

- Update `ToSchema` derive and serialization

### 2. services crate

#### 2a. DB Migration

**File**: `crates/services/migrations/0013_app_settings.up.sql` (next after existing 0012)

```sql
CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  updated_by TEXT
);
```

Minimal schema -- type/group metadata lives in the Rust settings registry, not in DB. No seeding.

#### 2b. Settings Repository

**File**: `crates/services/src/db/service_settings.rs` (new, following [service_user_alias.rs](crates/services/src/db/service_user_alias.rs) pattern)

Add `SettingsRepository` trait to `DbService`:

- `list_app_settings() -> Vec<DbAppSetting>`
- `get_app_setting(key: &str) -> Option<DbAppSetting>`
- `upsert_app_setting(key: &str, value: &str, updated_at: &str, updated_by: Option<&str>)`
- `delete_app_setting(key: &str)`

```rust
pub struct DbAppSetting {
  pub key: String,
  pub value: String,
  pub updated_at: String,
  pub updated_by: Option<String>,
}
```

#### 2c. Settings Registry

**File**: `crates/services/src/setting_service/registry.rs` (new)

Static registry defining each setting's properties. Replaces the hardcoded `SETTING_VARS` array and `EDIT_SETTINGS_ALLOWED`:

```rust
pub struct SettingDefinition {
  pub key: &'static str,
  pub group: Option<&'static str>,      // None for system/bootstrap
  pub metadata_fn: fn(&dyn SettingService) -> SettingMetadata,
  pub editable_native: bool,
  pub editable_container: bool,
  pub db_eligible: bool,                 // false = never persisted to DB
}
```

Helper functions:

- `get_definition(key) -> Option<&SettingDefinition>`
- `all_definitions() -> &[SettingDefinition]`
- `is_editable(key, app_type) -> bool`
- `db_eligible_keys() -> Vec<&str>`

#### 2d. Refactored DefaultSettingService

**File**: [crates/services/src/setting_service/default_service.rs](crates/services/src/setting_service/default_service.rs)

**Key change**: Redirect writes from settings.yaml to DB. Keep settings.yaml as read-only source.

Add new fields:

```rust
pub struct DefaultSettingService {
  // ... existing fields (env_wrapper, settings_file, system_settings, defaults, listeners, cmd_lines) ...
  db_pool: RwLock<Option<DbPool>>,          // NEW: injected after DB connect
  db_cache: RwLock<HashMap<String, Value>>,  // NEW: in-memory cache of DB rows
}
```

**Two-phase initialization**:

- **Phase 1 (bootstrap, before DB)**: Resolution is System > CmdLine > Env > SettingsFile > Default. Same as current, settings.yaml is still read. No DB layer yet.
- **Phase 2 (after `set_db_pool()`)**: Resolution adds DB layer: System > CmdLine > Env > **SettingsFile** > **Database** > Default. All DB rows loaded into `db_cache`.

**Resolution chain** in `get_setting_value_with_source` (after Phase 2):

```
1. system_settings                        -> SettingSource::System
2. cmd_lines HashMap                      -> SettingSource::CommandLine
3. env_wrapper.var(key)                   -> SettingSource::Environment
4. with_settings_read_lock (settings.yaml)-> SettingSource::SettingsFile   [UNCHANGED]
5. db_cache HashMap                       -> SettingSource::Database       [NEW]
6. defaults HashMap                       -> SettingSource::Default
```

**Write path changes**:

- `set_setting_value()` default: changes from `SettingSource::SettingsFile` to `SettingSource::Database`
- `set_setting_with_source(_, _, SettingSource::Database)`: upserts to DB via db_pool + updates db_cache + notifies listeners
- `set_setting_with_source(_, _, SettingSource::SettingsFile)`: becomes a no-op with warning log (file is now read-only by the app)
- `delete_setting()`: deletes from DB + removes from db_cache + notifies listeners
- `with_settings_write_lock`: removed (no more file writes)
- `with_settings_read_lock`: **kept** (still reads settings.yaml)

`**set_db_pool()` method**:

```rust
pub fn set_db_pool(&self, pool: DbPool) {
  *self.db_pool.write().unwrap() = Some(pool);
  self.reload_db_cache();  // load all rows from app_settings into db_cache
}
```

The `reload_db_cache` uses `tokio::task::block_in_place` (or a sync SQLite query) since the SettingService trait is synchronous.

`**list()` method**: Updated to include `group` and `editable` from the settings registry.

**app_dirs_builder.rs line 92**: Change `SettingSource::SettingsFile` to `SettingSource::CommandLine` for CLI-provided `options.app_settings` (these are startup-time overrides, should not persist):

```rust
// Before:
setting_service.set_setting_with_source(key, &parsed_value, SettingSource::SettingsFile);
// After:
setting_service.set_setting_with_source(key, &parsed_value, SettingSource::CommandLine);
```

### 3. routes_app crate

**File**: [crates/routes_app/src/routes_settings/route_settings.rs](crates/routes_app/src/routes_settings/route_settings.rs)

- Remove `EDIT_SETTINGS_ALLOWED` constant (line 13)
- `update_setting_handler`: use `registry::is_editable(key, app_type)` instead of allowlist check
- `delete_setting_handler`: same editability check from registry
- `list_settings_handler`: response already returns `Vec<SettingInfo>` -- now includes `group` and `editable` fields
- Update OpenAPI annotations to reflect new `SettingInfo` shape

### 4. lib_bodhiserver crate

**File**: [crates/lib_bodhiserver/src/app_service_builder.rs](crates/lib_bodhiserver/src/app_service_builder.rs)

After `DbService` is created and migrated (the `app_settings` table exists via 0013 migration), inject the pool:

```rust
// After db_service.migrate() succeeds:
setting_service.set_db_pool(pool.clone());
```

No other changes to the initialization flow. `setup_app_dirs()` in [app_dirs_builder.rs](crates/lib_bodhiserver/src/app_dirs_builder.rs) continues to work as-is (Phase 1 bootstrap, settings.yaml path still constructed, file still read if it exists).

### 5. Blockers Document

**File**: `ai-docs/02-features/cluster-stateless-blockers.md` (new)

Document remaining stateful components that need addressing for full cluster deployment:

1. **SQLite to PostgreSQL** -- bodhi.sqlite and session.sqlite need shared DB for multi-instance
2. **secrets.yaml to organizations table** -- planned separately (ref: `ai-docs/claude-plans/planned/20260217-appreginfo-to-org-table.md`)
3. **HF_HOME / local inference** -- not applicable for Container mode
4. **InMemoryQueue/RefreshWorker** -- needs disable flag or DB-backed queue for Container
5. **LocalConcurrencyService** -- PostgreSQL advisory locks needed for distributed token refresh
6. **MokaCacheService** -- node-local acceptable (cache miss = extra auth round-trip)
7. **Log files** -- stdout via `BODHI_LOG_STDOUT=true` (already supported)
8. **Cross-instance settings notifications** -- `PgSettingsNotifier` deferred until PostgreSQL migration

---

## Verification

Following the layered development methodology:

1. `cargo test -p objs` -- SettingSource/SettingInfo changes
2. `cargo test -p services` -- migration, repository, refactored SettingService
3. `cargo test -p routes_app` -- updated handlers
4. `cargo test -p lib_bodhiserver` -- initialization flow
5. `cargo test` -- full backend
6. `cargo run --package xtask openapi` -- regenerate OpenAPI spec
7. `make build.ts-client` -- regenerate TypeScript types
8. `cargo fmt`

