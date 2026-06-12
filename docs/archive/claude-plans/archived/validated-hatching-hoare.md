# Migration Simplification & Entity Cleanup Plan

## Context

The centralized RLS migration (`m20250101_000014_rls.rs`) applies row-level security policies for 14 tables in a single file, far from the table definitions. This makes it hard to understand each table's security model at a glance. Since there's no production deployment, we can restructure migrations freely without backwards compatibility concerns.

Additionally, the codebase has scattered `ulid::Ulid::new().to_string()` calls (~26 files), deprecated `*Row` type aliases, and missing SeaORM relations on tenant entities.

## Changes Overview

### A. Migration Restructuring
1. Distribute RLS into per-table migrations
2. Move `current_tenant_id()` to extensions migration (000)
3. Split `app_toolset_configs` from migration 006 into its own file
4. Rename `000013_apps` → `000014_tenants`
5. Delete `000014_rls.rs`
6. Renumber affected migrations (007→008 through 013→014)
7. Remove redundant `client_id` index on tenants table

### B. Code Cleanup
1. Create `services/src/utils/ulid.rs` with `new_ulid()` (lowercase)
2. Replace all `ulid::Ulid::new().to_string()` with `new_ulid()`
3. Remove deprecated `*Row` type aliases (6 types, ~30 files)
4. Fix missing SeaORM relations on tenant entities

---

## Phase 1: ULID Utility

### Step 1.1: Create utility module

**New file**: `crates/services/src/utils/mod.rs`
```rust
mod ulid;
pub use ulid::*;
```

**New file**: `crates/services/src/utils/ulid.rs`
```rust
pub fn new_ulid() -> String {
  ulid::Ulid::new().to_string().to_lowercase()
}
```

**Modify**: `crates/services/src/lib.rs` — add `mod utils; pub use utils::*;`

### Step 1.2: Replace all ULID calls

Replace `ulid::Ulid::new().to_string()` and `Ulid::new().to_string()` with `new_ulid()` in all files. Remove `use ulid::Ulid;` imports where no longer needed.

**Services crate** (~24 files):
- `src/app_access_requests/access_request_service.rs`
- `src/app_access_requests/test_access_request_repository.rs`
- `src/app_access_requests/test_access_request_repository_isolation.rs`
- `src/db/test_rls.rs`
- `src/mcps/mcp_service.rs`
- `src/models/api_model_service.rs`
- `src/models/data_service.rs`
- `src/models/download_request_entity.rs`
- `src/models/download_service.rs`
- `src/models/model_metadata_repository.rs`
- `src/models/model_objs.rs`
- `src/models/test_api_alias_repository.rs`
- `src/models/test_user_alias_repository.rs`
- `src/models/test_user_alias_repository_isolation.rs`
- `src/tenants/tenant_repository.rs`
- `src/tokens/token_service.rs`
- `src/tokens/test_token_repository.rs`
- `src/toolsets/tool_service.rs`
- `src/toolsets/toolset_repository.rs`
- `src/toolsets/test_toolset_repository.rs`
- `src/toolsets/test_toolset_repository_isolation.rs`

**routes_app crate** (1 file):
- `src/mcps/routes_mcps_auth.rs:157` — import `services::new_ulid`

**lib_bodhiserver_napi crate** (1 file):
- `src/config.rs:132` — add `new_ulid` re-export through `lib_bodhiserver`, or import from `services` directly

**Verify**: `cargo check -p services -p routes_app`

---

## Phase 2: Remove Deprecated Row Aliases

Remove these 6 type aliases/structs and replace all usages with the canonical Entity type or `entity::Model`:

| Alias | Definition File | Replace With |
|-------|----------------|--------------|
| `McpServerRow` | `mcps/mcp_server_entity.rs:54` | `McpServerEntity` |
| `McpRow` | `mcps/mcp_entity.rs:47` | `McpEntity` |
| `McpAuthHeaderRow` | `mcps/mcp_auth_header_entity.rs:69` | `McpAuthHeaderEntity` |
| `McpOAuthConfigRow` | `mcps/mcp_oauth_config_entity.rs:107` | `McpOAuthConfigEntity` |
| `McpOAuthTokenRow` | `mcps/mcp_oauth_token_entity.rs:78` | `McpOAuthTokenEntity` |
| `AppToolsetConfigRow` | `toolsets/app_toolset_config_entity.rs:27` | `app_toolset_config::Model` (struct literal) |

### Key files affected (heaviest usage):
- `src/mcps/mcp_server_repository.rs` (~17 usages)
- `src/mcps/mcp_instance_repository.rs` (~11 usages)
- `src/mcps/mcp_auth_repository.rs` (~20 usages)
- `src/mcps/mcp_service.rs` (~15 usages)
- `src/test_utils/db.rs` (~30 usages across all Row types)
- `src/toolsets/toolset_repository.rs` (~11 usages)
- Various `test_*.rs` files in mcps/ and toolsets/

### Special case: `AppToolsetConfigRow`
This is a **struct** (not just a type alias) used in struct literal construction at `toolset_repository.rs:327,354`. Replace with `app_toolset_config::Model { ... }` — the import alias `app_toolset_config` already exists in that file.

### Note on Playwright tests
`getMcpRowByName()` in `lib_bodhiserver_napi/tests-js/` is a UI concept (table row), NOT the Rust type alias. Do NOT touch these.

**Verify**: `cargo check -p services -p routes_app`

---

## Phase 3: Fix SeaORM Relations

### Step 3.1: `crates/services/src/tenants/tenant_entity.rs`

Add has_many relation to tenants_users:
```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::tenant_user_entity::Entity")]
  TenantsUsers,
}

impl Related<super::tenant_user_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TenantsUsers.def()
  }
}
```

### Step 3.2: `crates/services/src/tenants/tenant_user_entity.rs`

Add belongs_to relation to tenants:
```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::tenant_entity::Entity",
    from = "Column::TenantId",
    to = "super::tenant_entity::Column::Id"
  )]
  Tenant,
}

impl Related<super::tenant_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tenant.def()
  }
}
```

**Verify**: `cargo check -p services`

---

## Phase 4: Migration Restructuring

### New migration numbering

| # | File | Content | RLS |
|---|------|---------|-----|
| 000 | `_extensions.rs` | citext + `current_tenant_id()` | N/A |
| 001 | `_download_requests.rs` | download_requests table | Standard |
| 002 | `_api_model_aliases.rs` | api_model_aliases table | Standard |
| 003 | `_model_metadata.rs` | model_metadata table | Standard |
| 004 | `_access_requests.rs` | user_access_requests table | Standard |
| 005 | `_api_tokens.rs` | api_tokens table | Standard |
| 006 | `_toolsets.rs` | toolsets table ONLY | Standard |
| **007** | **`_app_toolset_configs.rs`** | **app_toolset_configs (NEW)** | **Standard** |
| 008 | `_user_aliases.rs` | user_aliases (was 007) | Standard |
| 009 | `_app_access_requests.rs` | app_access_requests (was 008) | Custom 3-policy |
| 010 | `_mcp_servers.rs` | mcp_servers + mcps (was 009) | Standard ×2 |
| 011 | `_mcp_auth_headers.rs` | mcp_auth_headers (was 010) | Standard |
| 012 | `_mcp_oauth.rs` | oauth_configs + tokens (was 011) | Standard ×2 |
| 013 | `_settings.rs` | settings (was 012) | None |
| 014 | `_tenants.rs` | tenants (was 013_apps) | None |
| 015 | `_tenants_users.rs` | tenants_users (was 015) | Already inline |
| ~~014~~ | ~~`_rls.rs`~~ | ~~DELETED~~ | — |

### Step 4.1: Add `current_tenant_id()` to extensions (000)

**Modify**: `m20250101_000000_extensions.rs`
- `up()`: After citext, create `current_tenant_id()` function (PG only)
- `down()`: Before citext drop, drop the function

### Step 4.2: Add standard RLS to migrations 001-005

For each: append RLS block after table/index creation in `up()`, prepend RLS teardown in `down()`.

**Standard RLS pattern** (PostgreSQL only):
```sql
ALTER TABLE <t> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <t> FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON <t>
  FOR ALL
  USING (tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id = (SELECT current_tenant_id()));
```

### Step 4.3: Split migration 006

**Modify** `m20250101_000006_toolsets.rs`:
- Remove `AppToolsetConfigs` iden enum
- Remove `app_toolset_configs` table creation and indexes
- Add standard RLS for `toolsets` only

**New file** `m20250101_000007_app_toolset_configs.rs`:
- Move `AppToolsetConfigs` iden enum from 006
- Move table creation and indexes from 006
- Add standard RLS

### Step 4.4: Rename/renumber migrations 007→008 through 012→013

| Old file | New file |
|----------|----------|
| `m20250101_000007_user_aliases.rs` | `m20250101_000008_user_aliases.rs` |
| `m20250101_000008_app_access_requests.rs` | `m20250101_000009_app_access_requests.rs` |
| `m20250101_000009_mcp_servers.rs` | `m20250101_000010_mcp_servers.rs` |
| `m20250101_000010_mcp_auth_headers.rs` | `m20250101_000011_mcp_auth_headers.rs` |
| `m20250101_000011_mcp_oauth.rs` | `m20250101_000012_mcp_oauth.rs` |
| `m20250101_000012_settings.rs` | `m20250101_000013_settings.rs` |

Add standard RLS to each (except settings — no tenant_id).

For `app_access_requests` (now 009), add **custom** 3-policy RLS:
- `app_access_requests_read`: `FOR SELECT USING (true)`
- `app_access_requests_insert`: `FOR INSERT WITH CHECK (tenant_id IS NULL OR tenant_id = current_tenant_id())`
- `app_access_requests_update`: `FOR UPDATE USING/WITH CHECK (tenant_id IS NULL OR tenant_id = current_tenant_id())`

For `mcp_servers` (now 010), add RLS for **both** `mcp_servers` AND `mcps`.
For `mcp_oauth` (now 012), add RLS for **both** `mcp_oauth_configs` AND `mcp_oauth_tokens`.

### Step 4.5: Rename 013_apps → 014_tenants

**Rename**: `m20250101_000013_apps.rs` → `m20250101_000014_tenants.rs`

Also remove the redundant separate index on `client_id` (the `unique_key()` on the column definition already creates a unique constraint).

No RLS (tenants table is global).

### Step 4.6: Delete centralized RLS

**Delete**: `m20250101_000014_rls.rs`

### Step 4.7: Update mod.rs

Rewrite `crates/services/src/db/sea_migrations/mod.rs` with new module declarations and migration list reflecting the new numbering.

### Step 4.8: tenants_users (015) — no changes needed

Already has inline RLS. Stays at 015. FK to tenants(id) CASCADE is kept.

**Verify**: `cargo check -p services && cargo test -p services --lib`

---

## Phase 5: Verification

1. `cargo check -p services -p routes_app -p server_app` — compile check
2. `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"` — services tests
3. `cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED|failures:"` — routes tests
4. `cargo test -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"` — server tests
5. `make test.backend` — full backend suite (includes PostgreSQL via Docker)

---

## Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Tenant FK on domain tables | No FK | Test cleanup handles it; no production deletes |
| Polymorphic auth_uuid | Keep as-is | App logic handles integrity |
| Migration grouping | Parent+child grouped | Fewer files, natural coupling |
| Tenants RLS | None | Global/admin table, middleware needs cross-tenant reads |
| tenants_users FK | Keep CASCADE | Membership junction table, semantic fit |
| MCP domain FKs | Keep CASCADE | Domain-internal, atomic server deletion |
| Case-insensitive indexes | Leave dual-DB | Works correctly, inherent to SQLite vs PG |
| TenantRow struct | Deferred | Will revisit separately |
| ULID format | Lowercase | More conventional, centralized via `new_ulid()` |
