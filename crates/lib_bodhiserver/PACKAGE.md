# PACKAGE.md - lib_bodhiserver

See [CLAUDE.md](CLAUDE.md) for architectural guidance.

## Module Structure

| File | Purpose |
|------|---------|
| `src/lib.rs` | Re-exports from `services`, `routes_app`, `server_app`; `BUILD_COMMIT_SHA` const |
| `src/app_service_builder.rs` | `AppServiceBuilder`, `build_app_service()`, `update_with_option()` |
| `src/app_dirs_builder.rs` | `setup_app_dirs()`, `setup_bootstrap_service()`, `create_bodhi_home()` |
| `src/app_options.rs` | `AppOptions`, `AppOptionsBuilder` |
| `src/bootstrap_service.rs` | `BootstrapService` (pre-computed bootstrap values, `into_parts()`) |
| `src/error.rs` | `BootstrapError` (unified enum) |
| `src/ui_assets.rs` | `EMBEDDED_UI_ASSETS` (compile-time Next.js embed via `include_dir!`) |
| `src/test_utils/mod.rs` | Feature-gated module declaration |
| `src/test_utils/app_options_builder.rs` | `AppOptionsBuilder::development()`, `::with_bodhi_home()` |
| `src/test_app_dirs_builder.rs` | Tests for `setup_app_dirs` |
| `src/test_app_service_builder.rs` | Tests for `AppServiceBuilder` |

## AppServiceBuilder -- Two-Phase Build

`AppServiceBuilder::new(bootstrap_parts: BootstrapParts)` is the sole constructor. Injectable services (`time_service`, `cache_service`) are `Option<Arc<dyn Trait>>` fields with chainable setters returning `Err(ServiceAlreadySet)` on double-set.

`build()` runs in two phases (see `src/app_service_builder.rs`):

- **Phase 1** -- Destructure `BootstrapParts`. Check `is_production` (by matching `BODHI_ENV_TYPE == "production"` in `system_settings`). Build `encryption_key` (keyring or hash via `build_encryption_key()`). Resolve `app_db_url` from env var, file defaults, or convention (`sqlite:$BODHI_HOME/bodhi.sqlite`). Build `DbService`, then `DefaultSettingService::from_parts(parts, db_service)`.

- **Phase 2** -- Build remaining services using `setting_service` for configuration. Multi-tenant detection via `setting_service.is_multi_tenant()` controls `DataService` and `InferenceService` implementations. Spawns `RefreshWorker` in background for queue processing.

### setup_app_dirs -- Directory Bootstrap

`setup_app_dirs(options: &AppOptions)` returns `(PathBuf, SettingSource, HashMap<String, Value>)`:

1. Load `defaults.yaml` from the executable directory (optional, silently empty if missing).
2. Resolve BODHI_HOME: env var > `defaults.yaml` > `~/.cache/bodhi[-dev]` > error `BodhiHomeNotResolved`.
3. Create the BODHI_HOME directory if absent.

`setup_bootstrap_service(options, bodhi_home, source, file_defaults, command)` returns `BootstrapService`:
- Assembles system settings (BODHI_HOME, BODHI_ENV_TYPE, BODHI_APP_TYPE, BODHI_VERSION, BODHI_COMMIT_SHA, BODHI_AUTH_URL, BODHI_AUTH_REALM)
- Constructs `BootstrapService::new(...)` which reads settings YAML and resolves log config

## Error Types

All error enums derive `thiserror::Error` + `errmeta_derive::ErrorMeta`. See `src/error.rs`.

| Enum | Variants | ErrorType |
|------|----------|-----------|
| `BootstrapError` | `BodhiHomeNotResolved`, `DirCreate { source, path }`, `BodhiHomeNotSet`, `ValidationError(String)`, `Parse(strum::ParseError)`, `UnknownSystemSetting(String)`, `ServiceAlreadySet(String)`, `PlaceholderValue(String)`, `MissingBootstrapParts`, `SettingNotFound(String)`, `Db(DbError)`, `Tenant(TenantError)`, `SessionService(SessionServiceError)`, `Keyring(KeyringError)`, `Io(IoError)` | InternalServer / BadRequest |

## Commands

```bash
cargo test -p lib_bodhiserver
cargo test -p lib_bodhiserver app_service_builder
cargo test -p lib_bodhiserver --features test-utils
```
