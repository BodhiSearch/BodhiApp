# Phase 3: Persist Settings to SQLite Database

## Goal

Replace YAML-file writes with SQLite persistence. Settings are now read from and written to the database via `SettingsRepository`. The YAML settings file becomes read-only (legacy import only). The priority chain adds `Database` between `Environment` and `SettingsFile`.

## Priority Chain After This Phase

```
System > CommandLine > Environment > Database > SettingsFile > Default
```

- `System`: Injected at startup (BODHI_HOME, ENV_TYPE, APP_TYPE, AUTH_URL, etc.) — immutable
- `CommandLine`: CLI flags (host, port) — overrides everything else writable
- `Environment`: Env vars — runtime overrides
- `Database`: `settings` table in SQLite — user-configured persistent settings
- `SettingsFile`: `settings.yaml` — read-only import (legacy)
- `Default`: Hardcoded fallbacks

---

## Step 3.1: Add `SettingSource::Database` variant — `objs` crate

**File**: `crates/objs/src/envs.rs`

```rust
pub enum SettingSource {
  System,
  CommandLine,
  Environment,
  Database,    // NEW — insert between Environment and SettingsFile
  SettingsFile,
  Default,
}
```

Order matters for display only; actual priority is enforced in `get_setting_value_with_source()`.

**Gate**: `cargo test -p objs`

---

## Step 3.2: Add settings table migration — `services` crate

**New file**: `crates/services/migrations/0013_settings.up.sql`

```sql
CREATE TABLE IF NOT EXISTS settings (
  key         TEXT    NOT NULL PRIMARY KEY,
  value       TEXT    NOT NULL,
  value_type  TEXT    NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
```

**New file**: `crates/services/migrations/0013_settings.down.sql`

```sql
DROP TABLE IF EXISTS settings;
```

---

## Step 3.3: Add `SettingsRepository` trait and `DbSetting` type — `services` crate

**New file**: `crates/services/src/db/settings_repository.rs`

```rust
#[derive(Debug, Clone)]
pub struct DbSetting {
  pub key: String,
  pub value: String,
  pub value_type: String,
  pub created_at: i64,
  pub updated_at: i64,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SettingsRepository: Send + Sync {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError>;
  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError>;
  async fn delete_setting(&self, key: &str) -> Result<(), DbError>;
  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError>;
}
```

Export from `services/src/db/mod.rs`.

---

## Step 3.4: Implement `SettingsRepository` on `SqliteDbService`

**New file**: `crates/services/src/db/service_settings.rs`

```rust
#[async_trait::async_trait]
impl SettingsRepository for SqliteDbService {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError> {
    // SELECT key, value, value_type, created_at, updated_at FROM settings WHERE key = ?
  }

  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError> {
    let now = self.time_service.utc_now().timestamp();
    // INSERT ... ON CONFLICT(key) DO UPDATE SET value=excluded.value, ...
  }

  async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
    // DELETE FROM settings WHERE key = ?
  }

  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError> {
    // SELECT ... FROM settings ORDER BY key
  }
}
```

Add `mod service_settings;` to `db/mod.rs`.

---

## Step 3.5: Wire `SettingsRepository` into `DefaultSettingService`

**File**: `crates/services/src/setting_service/default_service.rs`

Add field:
```rust
pub struct DefaultSettingService {
  // ... existing fields ...
  db_service: Arc<dyn SettingsRepository>,
}
```

Update `from_parts()` to accept `db_service: Arc<dyn SettingsRepository>` (already done in Phase 2 stub).

### `get_setting_value_with_source()` — Read priority chain

```rust
async fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource) {
  // 1. System settings
  if let Some(s) = self.system_settings.iter().find(|s| s.key == key) {
    return (Some(s.value.clone()), s.source.clone());
  }
  // 2. CommandLine (cmd_lines RwLock)
  if let Some(v) = self.cmd_lines.read().unwrap().get(key) {
    return (Some(v.clone()), SettingSource::CommandLine);
  }
  // 3. Environment
  if let Some(raw) = self.env_wrapper.var(key).ok() {
    let metadata = Self::setting_metadata_static(key);
    return (Some(metadata.parse(Value::String(raw))), SettingSource::Environment);
  }
  // 4. Database (NEW)
  if let Ok(Some(db_setting)) = self.db_service.get_setting(key).await {
    let metadata = Self::setting_metadata_static(key);
    let value = metadata.parse(Value::String(db_setting.value));
    return (Some(value), SettingSource::Database);
  }
  // 5. SettingsFile (settings.yaml values loaded in from_parts)
  if let Some(v) = self.settings_file_values.read().unwrap().get(key) {
    return (Some(v.clone()), SettingSource::SettingsFile);
  }
  // 6. Default
  if let Some(v) = self.defaults.read().unwrap().get(key) {
    return (Some(v.clone()), SettingSource::Default);
  }
  (None, SettingSource::Default)
}
```

### `set_setting_with_source()` — Write to DB

```rust
async fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource) -> Result<()> {
  match source {
    SettingSource::System | SettingSource::CommandLine | SettingSource::Environment => {
      return Err(SettingServiceError::ReadOnlySource(source));
    }
    SettingSource::Database => {
      self.validate_db_key(key)?;  // only allow SETTING_VARS + BODHI_LLAMACPP_ARGS_*
      let db_setting = DbSetting {
        key: key.to_string(),
        value: value_to_string(value),
        value_type: value_type_string(value),
        created_at: 0,  // filled by upsert
        updated_at: 0,
      };
      self.db_service.upsert_setting(&db_setting).await?;
    }
    SettingSource::SettingsFile => {
      // Read-only — YAML is no longer written
      return Err(SettingServiceError::ReadOnlySource(source));
    }
    SettingSource::Default => {
      // In-memory only
      self.defaults.write().unwrap().insert(key.to_string(), value.clone());
    }
  }
  // Notify listeners
  ...
  Ok(())
}
```

### `set_setting_value()` default — writes to Database

```rust
async fn set_setting_value(&self, key: &str, value: &Value) -> Result<()> {
  self.set_setting_with_source(key, value, SettingSource::Database).await
}
```

### `delete_setting()` — removes from DB only

```rust
async fn delete_setting(&self, key: &str) -> Result<()> {
  self.db_service.delete_setting(key).await?;
  ...
  Ok(())
}
```

**Key validation**: `is_valid_db_key(key)` returns true for `SETTING_VARS` constants or keys matching `"BODHI_LLAMACPP_ARGS_*"`.

**Gate**: `cargo test -p services`

---

## Step 3.6: Remove YAML write path

The `settings.yaml` write path was previously used by `set_setting_with_source(..., SettingsFile)`. Remove all code that writes to the YAML file:

- Remove `write_settings_yaml()` helper
- Remove any `fs::write` calls for settings.yaml
- `SettingsFile` source is now **read-only**: values from the YAML file are imported into `settings_file_values` at startup but never written back

---

## Step 3.7: Update `AppServiceBuilder` to pass `db_service` to `from_parts()`

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

In `build()`, `DbService` is now created before `DefaultSettingService`:

```rust
let db_service = Self::build_db_service(
  parts.bodhi_home.join(PROD_DB),
  time_service.clone(),
  encryption_key.clone(),
).await?;

let setting_service = Arc::new(
  DefaultSettingService::from_parts(parts, db_service.clone())
);
```

`PROD_DB` constant is `"bodhi.sqlite"`.

---

## Step 3.8: Update test utilities

**File**: `crates/services/src/test_utils/db.rs`

- Add `InMemorySettingsRepository` implementing `SettingsRepository` with `HashMap` storage
- Expose as `test-utils` feature-gated type for test stubs

**File**: `crates/services/src/test_utils/envs.rs`

- `SettingServiceStub` now uses `SettingSource::Database` as the default write source

**Gate**: `cargo test -p services && cargo test -p lib_bodhiserver`

---

## Step 3.9: Regenerate OpenAPI spec + TypeScript types

The `SettingSource` enum gained a new variant (`database`). Regenerate:

```bash
cargo run --package xtask openapi
cd ts-client && npm run generate
```

Verify `openapi.json` and `ts-client/src/types/types.gen.ts` reflect `"database"` as a valid `SettingSource` value.

---

## Full Validation

```
make test.backend
```

---

## Key Files Changed

| File | Change |
|------|--------|
| `objs/src/envs.rs` | Add `SettingSource::Database` variant |
| `services/migrations/0013_settings.up.sql` | **New** — create settings table |
| `services/migrations/0013_settings.down.sql` | **New** — drop settings table |
| `services/src/db/settings_repository.rs` | **New** — `SettingsRepository` trait + `DbSetting` |
| `services/src/db/service_settings.rs` | **New** — `SqliteDbService` impl |
| `services/src/db/mod.rs` | Export new types |
| `services/src/setting_service/default_service.rs` | Add `db_service` field; DB read/write in priority chain; remove YAML write |
| `services/src/test_utils/db.rs` | Add `InMemorySettingsRepository` |
| `services/src/test_utils/envs.rs` | Use `SettingSource::Database` in stubs |
| `lib_bodhiserver/src/app_service_builder.rs` | Pass `db_service` to `from_parts()` |
| `openapi.json` | Add `"database"` to `SettingSource` enum |
| `ts-client/src/types/types.gen.ts` | Regenerated TypeScript types |

## Behaviour Change Summary

| Operation | Before | After |
|-----------|--------|-------|
| `set_setting_value(key, val)` | Writes to `settings.yaml` | Writes to `settings` table |
| `delete_setting(key)` | Removes from `settings.yaml` | `DELETE FROM settings WHERE key=?` |
| `get_setting_value(key)` priority | Sys > Cmd > Env > File > Default | Sys > Cmd > Env > **DB** > File > Default |
| `settings.yaml` | Read + Write | Read-only (legacy import) |
