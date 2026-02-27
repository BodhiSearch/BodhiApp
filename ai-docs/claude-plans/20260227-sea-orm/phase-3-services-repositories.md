# Phase 3: Repository Implementations (`crates/services/src/db/`)

Replace all sqlx-based repository implementations with SeaORM, defining repository traits and their DefaultDbService implementations, plus core infrastructure.

## Repository Trait Definitions

- `access_repository.rs` (modified) -- AccessRepository trait (method signatures updated for SeaORM types)
- `access_request_repository.rs` (modified) -- AccessRequestRepository trait
- `app_instance_repository.rs` (modified) -- AppInstanceRepository trait (updated for ULID, DateTime)
- `mcp_repository.rs` (modified) -- McpRepository trait (35 methods for full MCP CRUD). Post-review: list/get methods return domain types (`McpAuthHeader`, `McpOAuthConfig`, `McpOAuthToken`) with `has_*` flags instead of Row types. Added `get_decrypted_refresh_token` method.
- `model_repository.rs` (modified) -- ModelRepository trait
- `settings_repository.rs` (modified) -- SettingsRepository trait (updated return types)
- `token_repository.rs` (modified) -- TokenRepository trait
- `toolset_repository.rs` (modified) -- ToolsetRepository trait
- `user_alias_repository.rs` (modified) -- UserAliasRepository trait

## SeaORM Implementations

- `service_access.rs` (new) -- AccessRepository impl for DefaultDbService
- `service_access_request.rs` (new) -- AccessRequestRepository impl
- `service_app_instance.rs` (new) -- AppInstanceRepository impl with encryption
- `service_mcp.rs` (new) -- McpRepository impl (CASCADE-aware). Post-review: list/get methods use `into_partial_model::<*View>()` to return domain types. `get_decrypted_client_secret` queries entity directly (not via `get_mcp_oauth_config`). Added `get_decrypted_refresh_token` implementation.
- `service_model.rs` (new) -- ModelRepository impl (download_requests, api_model_aliases, model_metadata)
- `service_settings.rs` (new) -- SettingsRepository impl (upsert pattern)
- `service_token.rs` (new) -- TokenRepository impl
- `service_toolset.rs` (new) -- ToolsetRepository impl with encryption. Post-review: `create_toolset` uses fetch-after-insert pattern (not input clone).
- `service_user_alias.rs` (new) -- UserAliasRepository impl

## Core Infrastructure

- `default_service.rs` (new) -- DefaultDbService struct, DbCore impl (migrate, seed_toolset_configs, reset)
- `db_core.rs` (modified) -- DbCore trait definition
- `service.rs` (modified) -- DbService super-trait, blanket impl, TimeService integration
- `error.rs` (modified) -- Added SeaOrmDbError, removed SqlxMigrateError from DbError, added Conversion/EncryptionError variants
- `objs.rs` (modified) -- Updated domain objects for SeaORM types (DateTime, ULID, new builder patterns)
- `mod.rs` (modified) -- Updated module declarations, entity re-exports
- `time_service.rs` (modified) -- TimeService trait, DefaultTimeService, FrozenTimeService (referenced in service.rs)

## Deleted Old sqlx Files

- `service_access.rs` (deleted) -- Old sqlx AccessRepository impl, replaced by SeaORM version
- `service_access_request.rs` (deleted) -- Old sqlx AccessRequestRepository impl, replaced
- `service_mcp.rs` (deleted) -- Old sqlx McpRepository impl (977 lines), replaced
- `service_model.rs` (deleted) -- Old sqlx ModelRepository impl (754 lines), replaced
- `service_settings.rs` (deleted) -- Old sqlx SettingsRepository impl, replaced
- `service_token.rs` (deleted) -- Old sqlx TokenRepository impl (286 lines), replaced
- `service_toolset.rs` (deleted) -- Old sqlx ToolsetRepository impl (367 lines), replaced
- `service_user_alias.rs` (deleted) -- Old sqlx UserAliasRepository impl (209 lines), replaced
- `repository_app_instance.rs` (deleted) -- Old sqlx AppInstanceRepository impl (106 lines), merged into service_app_instance.rs
- `sqlite_pool.rs` (deleted) -- SQLite pool utility no longer needed
