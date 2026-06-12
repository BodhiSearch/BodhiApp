# Test Utils Packaging Pattern

How BodhiApp shares test fixtures across crates while keeping test code out of production builds. Canonical infrastructure lives in `crates/services/src/test_utils/` — see `crates/services/src/test_utils/CLAUDE.md`.

## Dual Availability Mechanism

Test utilities are exposed two ways:

1. **Internal** (`#[cfg(test)]`): available during unit tests within the providing crate.
2. **Cross-crate** (`feature = "test-utils"`): available to downstream crates that enable the feature.

The conditional in each provider's `src/lib.rs` (see `crates/services/src/lib.rs:2`):

```rust
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;
```

This guarantees `test_utils` is always present under `cfg(test)`, opt-in for downstream consumers, and never compiled into production builds.

## Cargo.toml Wiring

**Provider crate** declares a `test-utils` feature listing the extra deps the fixtures need (see `crates/services/Cargo.toml`, `[features] test-utils = [...]`). Provider features can chain to upstream provider features (e.g. `mcp_client/test-utils`).

**Consumer crate** enables the feature as a dev-dependency:

```toml
[dev-dependencies]
services = { workspace = true, features = ["test-utils"] }
```

## Why

- Shared `TestDbService`, `AppServiceStub`, and `AuthContext` factories give every crate the same fixtures and FrozenTimeService behavior (see `crates/services/src/test_utils/CLAUDE.md`).
- Test deps never leak into production dependency graphs.
- No circular deps: providers expose fixtures; consumers opt in via dev-dependencies only.

## Adding New Test Utilities

1. Add the helper to the provider's `test_utils` module.
2. Re-export via `pub use` in `test_utils/mod.rs`.
3. Add any new external crate to the `test-utils` feature list in `Cargo.toml`.
4. Prefer rstest `#[fixture]` for shared setup and builder patterns for complex objects.

> Note: the `objs` crate was merged into `services`; all shared domain fixtures now live under `crates/services/src/test_utils/` (`fixtures.rs`, `model_fixtures.rs`), not a separate `objs` crate.
