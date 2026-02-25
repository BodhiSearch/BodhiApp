# crates/CLAUDE.md — Shared Rust Conventions

## Module Organization

- `mod.rs` files must contain ONLY module declarations (`mod xxx;`) and re-exports (`pub use xxx::*;`).
  No trait definitions, error enums, structs, or implementation code in `mod.rs`.
- For service modules with multiple concerns, split into: `error.rs`, `service.rs` (or domain-named files), and `mod.rs` for wiring.

## Reference Implementation

`crates/services/src/session_service/` demonstrates the canonical multi-file module layout:

```
session_service/
  mod.rs           — module declarations + pub use re-exports only
  error.rs         — SessionServiceError enum, SessionResult type alias
  session_store.rs — SessionStoreBackend, InnerStoreShared, is_postgres_url
  session_service.rs — SessionService trait, AppSessionStoreExt trait, DefaultSessionService
  postgres.rs      — create_postgres_store (pub(crate))
  sqlite.rs        — create_sqlite_store (pub(crate))
```
