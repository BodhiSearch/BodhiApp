# PACKAGE.md - lib_bodhiserver test_utils

## Module Structure

| File | Purpose |
|------|---------|
| `mod.rs` | Module declaration for `app_options_builder` |
| `app_options_builder.rs` | `AppOptionsBuilder::development()` and `::with_bodhi_home()` |

## AppOptionsBuilder Test Extensions

`development()` returns a builder with:
- `EnvType::Development`
- `AppType::Container`
- `app_version` from `CARGO_PKG_VERSION`
- `auth_url`: `"https://test-id.getbodhi.app"`
- `auth_realm`: `"bodhi"`

`with_bodhi_home(path)` calls `development()` then `set_env(BODHI_HOME, path)`.

## Commands

```bash
cargo test -p lib_bodhiserver --features test-utils
```
