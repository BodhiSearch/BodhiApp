# CLAUDE.md - lib_bodhiserver test_utils

See [PACKAGE.md](./PACKAGE.md) for implementation details.

## Purpose

The `test_utils` module provides testing infrastructure for `lib_bodhiserver`'s embeddable server library, enabling comprehensive testing of service composition, application bootstrap, and configuration management for embedded deployment scenarios.

## Architecture Position

**Upstream dependencies** (modules/crates this depends on):
- [`objs`](../../../objs/CLAUDE.md) -- `AppOptions`, `AppOptionsBuilder`, `EnvType`, `AppType`
- [`services`](../../../services/CLAUDE.md) -- `DefaultSettingService` for configuration testing

**Downstream consumers** (crates that use these test utilities):
- [`lib_bodhiserver_napi`](../../../lib_bodhiserver_napi/CLAUDE.md) -- NAPI binding tests use `AppOptionsBuilder::development()`
- [`bodhi/src-tauri`](../../../bodhi/src-tauri/CLAUDE.md) -- Tauri tests use configuration builders

## Key Components

- **`AppOptionsBuilder::development()`** -- Pre-configured builder with development defaults for testing
- **`AppOptionsBuilder::with_bodhi_home()`** -- Builder with custom BODHI_HOME for isolated test directories
- Available behind the `test-utils` feature flag
