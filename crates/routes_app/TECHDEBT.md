# routes_app Technical Debt

## Move EDIT_SETTINGS_ALLOWED to setting_service

- **Currently**: `EDIT_SETTINGS_ALLOWED` is defined in `crates/routes_app/src/routes_settings/route_settings.rs`
- **Should be**: Moved to `crates/services/src/setting_service/` so the editability allowlist lives alongside `SETTING_VARS`
- **Reason**: Setting visibility (`SETTING_VARS`) and editability (`EDIT_SETTINGS_ALLOWED`) are related concerns that should be co-located in the settings service layer rather than in the HTTP routing layer
