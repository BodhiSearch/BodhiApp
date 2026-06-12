# Database Migrations & Entity Architecture

## Overview

Multi-tenant stage 2 restructured the migration system from a centralized RLS file into per-table inline RLS, added the `tenants_users` membership table, extended the `tenants` table with `name`/`description`/`created_by` columns, and cleaned up entity conventions. Migrations were renumbered (inserting `app_toolset_configs` at 007, shifting 007-012 to 008-013, renaming `000013_apps` to `000014_tenants`, deleting `000014_rls`), and a `new_ulid()` utility replaced scattered `ulid::Ulid::new().to_string()` calls across ~26 files.

For how these tables are used in the application, see [03-tenant-management-and-spi.md](03-tenant-management-and-spi.md) (tenant CRUD) and [05-data-isolation-rls.md](05-data-isolation-rls.md) (RLS policies and isolation patterns).

## Functional Behavior

### Tenants table

Stores OAuth2 client registrations. In standalone mode there is one tenant; in multi-tenant mode there are many. The table is global (no `tenant_id` column, no RLS). Key operations:

- **Create tenant** (`create_tenant`): Generates a ULID id, encrypts client_secret, inserts row. When `created_by` is provided, enforces a one-tenant-per-user constraint by calling `list_tenants_by_creator` first and returning `TenantError::UserAlreadyHasTenant` if the user already owns a tenant. In standalone mode (`created_by=None`), this check is skipped. When `created_by` is provided and status is `Ready`, atomically inserts a `tenants_users` membership row in the same transaction.
- **Set tenant ready** (`set_tenant_ready`): Atomically updates `app_status=Ready` + `created_by=user_id` + upserts membership, all within a single `begin_tenant_txn`. Replaces the former separate `update_tenant_status` + `update_tenant_created_by` calls.
- **Lookup**: By id (`get_tenant_by_id`), by client_id (`get_tenant_by_client_id`), by creator (`list_tenants_by_creator`), or get the single standalone tenant (`get_tenant`).

### Tenants_users table (membership)

Junction table tracking which users belong to which tenants. Composite PK `(tenant_id, user_id)`. FK to `tenants(id)` with CASCADE delete. Key operations:

- **Upsert membership** (`upsert_tenant_user`): INSERT ON CONFLICT updates `updated_at`. Uses `with_tenant_txn` for RLS.
- **Delete membership** (`delete_tenant_user`): Idempotent delete. Uses `with_tenant_txn`.
- **List user tenants** (`list_user_tenants`): Cross-tenant read (reads all tenants for a user). Uses `&self.db` directly, not `with_tenant_txn`, because the read RLS policy allows all reads.
- **Has memberships** (`has_tenant_memberships`): EXISTS check. Same cross-tenant read pattern.

### API contracts (downstream effects)

- `GET /bodhi/v1/tenants` reads tenant list from local DB via `list_user_tenants` (no Keycloak SPI call).
- `POST /bodhi/v1/tenants` creates both Keycloak OAuth client (SPI) and local tenant row + membership atomically.
- `/bodhi/v1/info` uses `has_tenant_memberships` to distinguish `TenantSelection` vs `Setup` status for dashboard-only users.

## Architecture & Data Model

### Tenants table schema

```sql
CREATE TABLE tenants (
  id              VARCHAR PRIMARY KEY,       -- ULID (lowercase)
  client_id       VARCHAR UNIQUE NOT NULL,   -- OAuth2 client_id
  encrypted_client_secret VARCHAR NULL,      -- AES-encrypted
  salt_client_secret      VARCHAR NULL,
  nonce_client_secret     VARCHAR NULL,
  name            VARCHAR NOT NULL DEFAULT '',
  description     TEXT NULL,
  app_status      VARCHAR NOT NULL DEFAULT 'setup',  -- setup|ready|resource_admin|tenant_selection
  created_by      VARCHAR NULL,              -- Keycloak user_id (JWT sub claim)
  created_at      TIMESTAMPTZ NOT NULL,
  updated_at      TIMESTAMPTZ NOT NULL
);
-- No RLS (global table, middleware needs cross-tenant reads)
```

### Tenants_users table schema

```sql
CREATE TABLE tenants_users (
  tenant_id   VARCHAR NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  user_id     VARCHAR NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL,
  updated_at  TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, user_id)
);
CREATE INDEX idx_tenants_users_user_id ON tenants_users(user_id);

-- RLS (PostgreSQL only):
-- SELECT: USING (true)  -- cross-tenant reads allowed
-- ALL mutations: USING/WITH CHECK (tenant_id = current_tenant_id())
```

For detailed RLS policy definitions, see [05-data-isolation-rls.md](05-data-isolation-rls.md#three-rls-policy-patterns).

### Entity types

**`tenant_entity.rs`** (`tenant_entity::Model`):
- SeaORM `DeriveEntityModel` with `table_name = "tenants"`
- Has `has_many` relation to `tenant_user_entity::Entity`
- `TenantRow` struct (decrypted form): mirrors Model but with plain `client_secret: String` instead of encrypted fields

**`tenant_user_entity.rs`** (`tenant_user_entity::Model`):
- SeaORM `DeriveEntityModel` with `table_name = "tenants_users"`
- Composite PK: `tenant_id` + `user_id` (both `primary_key, auto_increment = false`)
- Has `belongs_to` relation to `tenant_entity::Entity`

**`tenant_objs.rs`** domain types:
- `DeploymentMode` enum: `Standalone` (default) | `MultiTenant`, serde snake_case
- `AppStatus` enum: `Setup` (default) | `Ready` | `ResourceAdmin` | `TenantSelection`, derives `DeriveValueType` for SeaORM
- `Tenant` struct: Domain object with `From<TenantRow>` conversion. Contains decrypted `client_secret`, `name`, `description`, `created_by`

### ID generation

All entity IDs use `new_ulid()` from `crates/services/src/utils/ulid.rs`:
```rust
pub fn new_ulid() -> String {
  ulid::Ulid::new().to_string().to_lowercase()
}
```
Replaces scattered `ulid::Ulid::new().to_string()` calls. Lowercase format is the convention.

### Migration numbering (final state)

| # | File | Content | RLS |
|---|------|---------|-----|
| 000 | `_extensions.rs` | citext + `current_tenant_id()` function | N/A |
| 001 | `_download_requests.rs` | download_requests table | Standard tenant_isolation |
| 002 | `_api_model_aliases.rs` | api_model_aliases table | Standard |
| 003 | `_model_metadata.rs` | model_metadata table | Standard |
| 004 | `_access_requests.rs` | user_access_requests table | Standard |
| 005 | `_api_tokens.rs` | api_tokens table | Standard |
| 006 | `_toolsets.rs` | toolsets table only | Standard |
| 007 | `_app_toolset_configs.rs` | app_toolset_configs (split from 006) | Standard |
| 008 | `_user_aliases.rs` | user_aliases (was 007) | Standard |
| 009 | `_app_access_requests.rs` | app_access_requests (was 008) | Custom 3-policy |
| 010 | `_mcp_servers.rs` | mcp_servers + mcps (was 009) | Standard x2 |
| 011 | `_mcp_auth_headers.rs` | mcp_auth_headers (was 010) | Standard |
| 012 | `_mcp_oauth.rs` | oauth_configs + tokens (was 011) | Standard x2 |
| 013 | `_settings.rs` | settings (was 012) | None (global) |
| 014 | `_tenants.rs` | tenants (was 013_apps) | None (global) |
| 015 | `_tenants_users.rs` | tenants_users | Custom: read-all + mutation-restricted |

### Transaction patterns in tenant operations

| Operation | Uses Transaction? | Rationale |
|-----------|-------------------|-----------|
| `create_tenant` (with `created_by`) | `begin_tenant_txn` | Atomic tenant insert + membership upsert |
| `create_tenant` (without `created_by`) | No | Single insert, no atomicity needed |
| `set_tenant_ready` | `begin_tenant_txn` | Atomic status update + membership upsert |
| `upsert_tenant_user` | `with_tenant_txn` | RLS enforcement on PostgreSQL |
| `delete_tenant_user` | `with_tenant_txn` | RLS enforcement on PostgreSQL |
| `list_user_tenants` | No (`&self.db`) | Cross-tenant read, RLS allows all SELECTs |
| `has_tenant_memberships` | No (`&self.db`) | Cross-tenant read |
| `get_tenant*`, `delete_tenant` | No (`&self.db`) | Global table, no RLS |

For the full transaction patterns across all domains (not just tenants), see [05-data-isolation-rls.md](05-data-isolation-rls.md#transaction-patterns-for-tenant-operations).

## Technical Implementation

### Migration files

- `crates/services/src/db/sea_migrations/mod.rs` -- Migration registry, lists all 16 migrations in order
- `crates/services/src/db/sea_migrations/m20250101_000000_extensions.rs` -- `citext` extension + `current_tenant_id()` function (PostgreSQL only)
- `crates/services/src/db/sea_migrations/m20250101_000014_tenants.rs` -- Tenants table DDL
- `crates/services/src/db/sea_migrations/m20250101_000015_tenants_users.rs` -- Tenants_users table DDL + RLS policies + FK + index

### Entity and domain files

- `crates/services/src/tenants/tenant_entity.rs` -- SeaORM entity for tenants, `TenantRow` decrypted struct, has_many relation
- `crates/services/src/tenants/tenant_user_entity.rs` -- SeaORM entity for tenants_users, belongs_to relation
- `crates/services/src/tenants/tenant_objs.rs` -- `DeploymentMode`, `AppStatus`, `Tenant` domain struct, `From<TenantRow>` impl
- `crates/services/src/tenants/tenant_repository.rs` -- `TenantRepository` trait + `DefaultDbService` impl, all tenant + membership DB operations
- `crates/services/src/tenants/tenant_service.rs` -- `TenantService` trait, thin delegation to repository
- `crates/services/src/tenants/mod.rs` -- Module declarations, re-exports, test module registration

### Supporting files

- `crates/services/src/utils/ulid.rs` -- `new_ulid()` centralized ULID generation
- `crates/services/src/db/encryption.rs` -- `encrypt_api_key` / `decrypt_api_key` used for client_secret
- `crates/services/src/db/db_core.rs` -- `DbCore` trait with `begin_tenant_txn` and `with_tenant_txn`

### Test files

- `crates/services/src/tenants/test_tenant_repository.rs` -- Repository method tests (dual SQLite/PostgreSQL)
- `crates/services/src/tenants/test_tenant_repository_isolation.rs` -- Cross-tenant isolation tests for tenants_users

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Decision | Status |
|---|----------|--------|
| D14 | Modify existing migration files (fresh DBs only) | Applied |
| D52 | Accept orphans on tenant creation failure | Applied |
| D66 | `created_by` is Keycloak user ID (JWT `sub` claim) | Applied |
| D69 | Same tenant schema for standalone and multi-tenant | Applied |
| D79 | Multi-tenant tenants created `Ready` immediately | Applied |
| D94 | SPI table names differ from plan | Applied |
| D95 | Keycloak groups sole role source; `tenants_users` membership only | Applied |

**Unnumbered decisions (migration-specific)**:

| Decision | Status |
|----------|--------|
| Distribute RLS from centralized `000014_rls.rs` into per-table migrations | Applied |
| Rename `000013_apps` to `000014_tenants` | Applied |
| Split `app_toolset_configs` out of `000006_toolsets` into `000007` | Applied |
| `current_tenant_id()` moved from RLS migration to `000000_extensions` | Applied |
| `new_ulid()` utility for centralized lowercase ULID generation | Applied |
| `TenantRow` struct retained (not removed/renamed to Entity alias) | Deferred |
| tenants table has no RLS (global table) | Applied |
| tenants_users RLS: read-all + mutation-restricted | Applied |
| `tenants_users` FK to `tenants(id)` with CASCADE | Applied |
| No FK from domain tables to tenants table | Applied |
| SeaORM relations added between tenant entities | Applied |
| Removed deprecated `*Row` type aliases (6 types) | Applied |
| `set_tenant_ready` replaces `set_client_ready` | Applied |
| `create_tenant_impl` shared by `create_tenant` and `create_tenant_test` | Applied |

## Known Gaps & TECHDEBT

1. **`list_user_tenants` does not filter by `app_status=Ready`**: The arch-refactor doc specified filtering Ready tenants only, but the current implementation returns all tenants for a user regardless of status. This means the `/tenants` list may include tenants still in setup.

2. **`app_access_requests.tenant_id` changed to nullable**: Migration 009 changed `tenant_id` from `NOT NULL` to `NULL` to support cross-tenant approval flow. The custom 3-policy RLS allows `tenant_id IS NULL` rows. This is intentional but means app_access_requests can exist outside any tenant scope.

3. **No isolation test for `app_toolset_configs`**: The table has RLS but no dedicated cross-tenant isolation test. Covered implicitly through toolset tests. See [07-testing-infrastructure.md](07-testing-infrastructure.md#known-gaps--techdebt) (Gap 13).

4. **Untested `TenantRepository` methods**: `upsert_tenant_user`, `delete_tenant_user`, `list_user_tenants`, `has_tenant_memberships` need dedicated dual-db method-level tests. See [07-testing-infrastructure.md](07-testing-infrastructure.md#known-gaps--techdebt) (Gap 10).

5. **`TenantRow` struct retained**: Unlike other domains that migrated from `*Row` aliases to `*Entity` type aliases, `TenantRow` is a distinct struct (decrypted form of the entity) rather than a type alias. Deferred.

6. **ULID lowercase convention not enforced on existing data**: `new_ulid()` generates lowercase ULIDs, but existing data may have mixed-case ULIDs from the pre-centralization era. No migration to normalize existing IDs.
