# CLAUDE.md - lib_bodhiserver test_utils

See parent [CLAUDE.md](../../CLAUDE.md) for crate overview.

## Purpose

Feature-gated (`test-utils`) testing helpers for `lib_bodhiserver`. Provides `AppOptionsBuilder` convenience constructors for downstream test code.

## Components

- `AppOptionsBuilder::development()` -- pre-configured with `EnvType::Development`, `AppType::Container`, test auth URL
- `AppOptionsBuilder::with_bodhi_home(path)` -- development defaults + custom BODHI_HOME

Used by `lib_bodhiserver_napi` and `bodhi/src-tauri` test code.

Source: `src/test_utils/app_options_builder.rs`
