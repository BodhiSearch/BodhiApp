# Phase 2: SeaORM Schema Layer (`crates/services/src/db/entities/` and `sea_migrations/`)

Define the full SeaORM entity models and migration scripts that replace the old sqlx-based schema, covering all 17 entities and 14 migrations.

## Entity Files

- `entities/mod.rs` (new) -- Module declarations, re-exports of entity types
- `entities/access_request.rs` (new) -- UserAccessRequest entity (Pattern A, direct alias)
- `entities/api_model_alias.rs` (new) -- ApiModelAlias entity (Pattern B, encrypted api_key)
- `entities/api_token.rs` (new) -- ApiToken entity (Pattern A)
- `entities/app_access_request.rs` (new) -- AppAccessRequestRow entity (Pattern A)
- `entities/app_instance.rs` (new) -- AppInstanceRow entity (Pattern B, encrypted client_secret)
- `entities/app_toolset_config.rs` (new) -- AppToolsetConfigRow entity (Pattern A)
- `entities/download_request.rs` (new) -- DownloadRequest entity (Pattern A)
- `entities/mcp.rs` (new) -- McpRow entity (Pattern A, BelongsTo McpServer)
- `entities/mcp_auth_header.rs` (new) -- McpAuthHeaderRow entity (Pattern B, encrypted) + McpAuthHeaderView (DerivePartialModel, excludes encrypted fields, `From` impl → McpAuthHeader with `has_header_value: true`)
- `entities/mcp_oauth_config.rs` (new) -- McpOAuthConfigRow entity (Pattern B, encrypted) + McpOAuthConfigView (DerivePartialModel, includes `encrypted_client_secret`/`encrypted_registration_access_token` for `is_some()` checks only, excludes salt/nonce, `From` impl → McpOAuthConfig computing `has_client_secret`/`has_registration_access_token`)
- `entities/mcp_oauth_token.rs` (new) -- McpOAuthTokenRow entity (Pattern B, encrypted) + McpOAuthTokenView (DerivePartialModel, includes `encrypted_refresh_token` for `is_some()` check, excludes access token fields and salt/nonce, `From` impl → McpOAuthToken with `has_access_token: true`, computing `has_refresh_token`)
- `entities/mcp_server.rs` (new) -- McpServerRow entity (Pattern A, HasMany relations)
- `entities/model_metadata.rs` (new) -- ModelMetadataRow entity (Pattern A)
- `entities/setting.rs` (new) -- DbSetting entity (Pattern A, natural key)
- `entities/toolset.rs` (new) -- ToolsetRow entity (Pattern B, encrypted api_key)
- `entities/user_alias.rs` (new) -- UserAlias entity (Pattern A)

## Migration Files

- `sea_migrations/mod.rs` (new) -- Migrator struct registering all 14 migrations
- `sea_migrations/m20250101_000000_extensions.rs` (new) -- PostgreSQL CITEXT extension (no-op on SQLite)
- `sea_migrations/m20250101_000001_download_requests.rs` (new) -- download_requests table with GGUF model fields (no `if_not_exists()` — removed post-review)
- `sea_migrations/m20250101_000002_api_model_aliases.rs` (new) -- api_model_aliases with encrypted api_key, unique prefix constraint
- `sea_migrations/m20250101_000003_model_metadata.rs` (new) -- model_metadata with JSON architecture column
- `sea_migrations/m20250101_000004_access_requests.rs` (new) -- access_requests table
- `sea_migrations/m20250101_000005_api_tokens.rs` (new) -- api_tokens with token_hash
- `sea_migrations/m20250101_000006_toolsets.rs` (new) -- toolsets + app_toolset_configs with encrypted api_key
- `sea_migrations/m20250101_000007_user_aliases.rs` (new) -- user_aliases with JSON request/context params
- `sea_migrations/m20250101_000008_app_access_requests.rs` (new) -- app_access_requests with requested_role/approved_role
- `sea_migrations/m20250101_000009_mcp_servers.rs` (new) -- mcp_servers + mcps with CASCADE FK, CITEXT slugs
- `sea_migrations/m20250101_000010_mcp_auth_headers.rs` (new) -- mcp_auth_headers with encrypted header values
- `sea_migrations/m20250101_000011_mcp_oauth.rs` (new) -- mcp_oauth_configs + mcp_oauth_tokens with encrypted secrets
- `sea_migrations/m20250101_000012_settings.rs` (new) -- settings table (natural key)
- `sea_migrations/m20250101_000013_apps.rs` (new) -- apps table with encrypted client_secret

## Crate Config

- `Cargo.toml` (modified) -- Added sea-orm, sea-orm-migration, ulid deps
