# Phase 1: Foundation Layer Changes (`crates/objs/`)

Add SeaORM type derivations and wrapper types to the `objs` crate so domain enums and structs can be used directly as SeaORM column types.

## New Files

- `src/db_enums.rs` (new) -- Domain enums (DownloadStatus, TokenStatus, AppStatus) with DeriveValueType for SeaORM column mapping
- `src/json_vec.rs` (new) -- JsonVec wrapper with private inner field, FromJsonQueryResult for SeaORM JSON columns

## Modified Files

- `src/access_request.rs` (modified) -- Added DeriveValueType to FlowType, UserAccessRequestStatus, AppAccessRequestStatus
- `src/mcp.rs` (modified) -- Added DeriveValueType to McpAuthType, RegistrationType (snake_case serialization)
- `src/model_metadata.rs` (modified) -- FromJsonQueryResult for ModelArchitecture
- `src/api_model_alias.rs` (modified) -- DeriveValueType for ApiFormat, added Repo/HubFile derives
- `src/user_alias.rs` (modified) -- DeriveValueType for Repo type
- `src/lib.rs` (modified) -- New module declarations and re-exports for db_enums, json_vec
- `Cargo.toml` (modified) -- Added sea-orm, ulid deps; removed uuid

## Documentation

- `CLAUDE.md` (modified) -- Updated for SeaORM patterns
- `PACKAGE.md` (modified) -- Added SeaORM type documentation
