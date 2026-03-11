# Data Isolation & Row-Level Security (RLS)

## Overview

BodhiApp enforces multi-tenant data isolation through a dual-layer strategy: application-level WHERE clause filtering on every tenant-scoped query (both SQLite and PostgreSQL), plus PostgreSQL Row-Level Security (RLS) policies as defense-in-depth. All 14 tenant-scoped tables have `tenant_id` columns with app-layer filtering; 14 of those also have RLS policies on PostgreSQL. Two global tables (`settings`, `tenants`) have no `tenant_id` column and no RLS. The `current_tenant_id()` PostgreSQL function, set via `begin_tenant_txn()`, provides the session variable that RLS policies evaluate against.

For table schemas and migration details, see [04-database-migrations-entities.md](04-database-migrations-entities.md). For testing of isolation guarantees, see [07-testing-infrastructure.md](07-testing-infrastructure.md#repository-isolation-test-pattern).

## Functional Behavior

### Isolation guarantees

Every domain query that touches tenant-scoped data includes a `tenant_id` filter in the WHERE clause. For user-scoped resources (tokens, MCPs, toolsets, API models, user aliases), an additional `user_id` filter provides intra-tenant user isolation. This means:

- **Cross-tenant**: User A in Tenant A cannot see, list, get, update, or delete resources belonging to Tenant B -- even if User A exists in both tenants. Cross-boundary operations return empty lists or `None`/404 (never 403, to prevent ID enumeration).
- **Intra-tenant user**: Within the same tenant, User A cannot see User B's resources for user-scoped domains.
- **Global resources**: Settings are readable regardless of tenant context. The `tenants` table is also global -- middleware needs cross-tenant reads to resolve tenant from JWT claims.

### SQLite vs PostgreSQL

| Aspect | SQLite | PostgreSQL |
|--------|--------|------------|
| Data isolation mechanism | Application-layer WHERE clauses only | Application-layer WHERE + RLS policies |
| `begin_tenant_txn()` behavior | Returns plain transaction (no-op for RLS) | Runs `SET LOCAL app.current_tenant_id = $1` |
| Functional result | Identical isolation guarantees | Identical, with defense-in-depth |
| Use case | Dev/desktop (single-tenant typical) | Production/Docker (multi-tenant) |

### Endpoint behavior under isolation

| Scenario | Expected HTTP Status |
|----------|---------------------|
| Resource belongs to same tenant + same user | 200 OK / 201 Created |
| Resource belongs to different tenant (even same user) | 404 Not Found |
| Resource belongs to same tenant, different user (user-scoped domains) | 404 Not Found |
| List endpoint, different tenant | 200 OK with empty/filtered results |

### Tables by isolation category

**Tenant + user scoped** (cross-tenant AND intra-tenant user isolation):
`api_tokens`, `mcps`, `toolsets`, `api_model_aliases`, `user_aliases`, `mcp_auth_headers`, `mcp_oauth_configs`, `mcp_oauth_tokens`

**Tenant scoped only** (cross-tenant isolation, no user scoping):
`download_requests`, `model_metadata`, `user_access_requests`, `mcp_servers`, `app_toolset_configs`

**Custom scoping**:
- `app_access_requests` -- `tenant_id` is nullable; cross-tenant admin approval flow requires reads across tenants and inserts with `tenant_id IS NULL`
- `tenants_users` -- cross-tenant reads allowed (membership lookups need to span tenants); mutations restricted to current tenant context

**Global (no tenant_id, no RLS)**:
`settings`, `tenants`

## Architecture & Data Model

### `current_tenant_id()` PostgreSQL function

Defined in migration `000000_extensions.rs`. Returns the session variable `app.current_tenant_id` as TEXT, or NULL if unset:

```sql
CREATE OR REPLACE FUNCTION current_tenant_id() RETURNS TEXT AS $$
  SELECT NULLIF(current_setting('app.current_tenant_id', true), '')
$$ LANGUAGE SQL SECURITY DEFINER STABLE;
```

All RLS policies reference `(SELECT current_tenant_id())` rather than `current_setting()` directly. The function must exist before any per-table RLS policy is created.

### `begin_tenant_txn(tenant_id)` -- DbCore trait

The entry point for tenant-scoped transactions. On PostgreSQL, begins a transaction and sets the session variable via parameterized query:

```
SELECT set_config('app.current_tenant_id', $1, true)
```

The third argument `true` makes the setting transaction-local (`SET LOCAL` equivalent). On SQLite, returns a plain transaction with no RLS configuration.

### `with_tenant_txn(tenant_id, closure)` -- DefaultDbService convenience

Wraps `begin_tenant_txn` + closure execution + commit into a single call. All mutating repository methods on tenant-scoped tables use this pattern:

```
self.with_tenant_txn(tenant_id, |txn| { Box::pin(async move { ... }) }).await
```

### Three RLS policy patterns

**Pattern 1 -- Standard `tenant_isolation`** (13 tables):
Single policy covering all operations. Used by `download_requests`, `api_model_aliases`, `model_metadata`, `user_access_requests`, `api_tokens`, `toolsets`, `app_toolset_configs`, `user_aliases`, `mcp_servers`, `mcps`, `mcp_auth_headers`, `mcp_oauth_configs`, `mcp_oauth_tokens`.

```sql
ALTER TABLE <table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <table> FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON <table>
  FOR ALL
  USING (tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id = (SELECT current_tenant_id()));
```

`FORCE ROW LEVEL SECURITY` ensures RLS applies even to table owners (the superuser/connection role), preventing accidental bypasses.

**Pattern 2 -- Custom 3-policy** (`app_access_requests`):
Supports cross-tenant admin approval flow where `tenant_id` can be NULL.

```sql
CREATE POLICY app_access_requests_read ON app_access_requests
  FOR SELECT USING (true);
CREATE POLICY app_access_requests_insert ON app_access_requests
  FOR INSERT
  WITH CHECK (tenant_id IS NULL OR tenant_id = (SELECT current_tenant_id()));
CREATE POLICY app_access_requests_update ON app_access_requests
  FOR UPDATE
  USING (tenant_id IS NULL OR tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id IS NULL OR tenant_id = (SELECT current_tenant_id()));
```

**Pattern 3 -- Custom read-all + mutation-restricted** (`tenants_users`):
Cross-tenant reads are required for `list_user_tenants` and `has_tenant_memberships`. Mutations restricted to current tenant.

```sql
CREATE POLICY tenants_users_read ON tenants_users
  FOR SELECT USING (true);
CREATE POLICY tenants_users_mutation ON tenants_users
  FOR ALL
  USING (tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id = (SELECT current_tenant_id()));
```

### Transaction patterns for tenant operations

| Operation | Uses `with_tenant_txn`? | Rationale |
|-----------|------------------------|-----------|
| All standard domain CRUD methods | Yes | RLS enforcement + atomicity |
| `create_tenant` (with `created_by`) | `begin_tenant_txn` | Atomic tenant insert + membership upsert |
| `set_tenant_ready` | `begin_tenant_txn` | Atomic status + created_by + membership |
| `upsert_tenant_user` | `with_tenant_txn` | RLS enforcement on `tenants_users` |
| `delete_tenant_user` | `with_tenant_txn` | RLS enforcement on `tenants_users` |
| `list_user_tenants` | No (`&self.db`) | Cross-tenant read; RLS `USING (true)` on SELECT |
| `has_tenant_memberships` | No (`&self.db`) | Cross-tenant read |
| `get_tenant*`, `delete_tenant` | No (`&self.db`) | Global table, no RLS |
| Settings operations | No (`&self.db`) | Global table, no RLS, no `tenant_id` |
| `get_api_token_by_prefix` | No (`&self.db`) | Cross-tenant prefix lookup by design; tenant resolved from `client_id` suffix after hash verification |
| `access_request_repository` approval/denial | Hybrid | Global lookup via `&self.db`, then mutation via `with_tenant_txn` |

### How RLS interacts with the middleware/auth layer

1. Middleware resolves tenant from JWT `azp`/`aud` claim or API token `client_id` suffix (see [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#tenant-resolution-strategy-summary))
2. Middleware populates `AuthContext` with `tenant_id` and `user_id`
3. `AuthScope` extractor creates `AuthScopedAppService` wrapping `AuthContext` + `AppService`
4. Auth-scoped sub-services (`.tokens()`, `.mcps()`, etc.) inject `tenant_id`/`user_id` from `AuthContext` into repository calls
5. Repository methods call `with_tenant_txn(tenant_id, ...)` which sets the PostgreSQL session variable
6. RLS policies filter rows to the current tenant within the transaction
7. Application-layer WHERE clauses provide the same filtering on SQLite

The `tenants` table has no RLS because the middleware itself needs cross-tenant reads to resolve tenant from incoming JWT claims (step 1 above). It is a chicken-and-egg situation: tenant must be resolved before RLS context can be set.

## Technical Implementation

### RLS infrastructure

- `crates/services/src/db/sea_migrations/m20250101_000000_extensions.rs` -- `current_tenant_id()` function definition
- `crates/services/src/db/db_core.rs` -- `DbCore` trait with `begin_tenant_txn()` signature
- `crates/services/src/db/default_service.rs` -- `begin_tenant_txn()` implementation (parameterized SET LOCAL), `with_tenant_txn()` convenience wrapper

### Per-table RLS in migrations

Each migration file owns its table's RLS policies (no centralized RLS file). The pattern is: CREATE TABLE, then conditional `if Postgres { ENABLE RLS, FORCE RLS, CREATE POLICY }`. RLS DDL lives in the `up()` method; `down()` drops policies before dropping the table.

- `crates/services/src/db/sea_migrations/m20250101_000001_download_requests.rs` through `m20250101_000012_mcp_oauth.rs` -- Standard `tenant_isolation` policy
- `crates/services/src/db/sea_migrations/m20250101_000009_app_access_requests.rs` -- Custom 3-policy (read/insert/update)
- `crates/services/src/db/sea_migrations/m20250101_000015_tenants_users.rs` -- Custom read-all + mutation-restricted
- `crates/services/src/db/sea_migrations/m20250101_000013_settings.rs` -- No RLS (global)
- `crates/services/src/db/sea_migrations/m20250101_000014_tenants.rs` -- No RLS (global)

### RLS verification test

- `crates/services/src/db/test_rls.rs` -- Three tests:
  1. `test_sqlite_tenant_isolation_app_layer` -- Verifies app-layer WHERE clause isolation on SQLite using tokens
  2. `test_postgres_rls_policies_and_function_installed` -- Verifies `current_tenant_id()` exists, returns NULL when unset, checks `tenant_isolation` policy exists on all 13 standard tables, checks 3 custom policies on `app_access_requests`, verifies `ENABLE` + `FORCE` RLS on all 14 tables
  3. `test_begin_tenant_txn_special_chars` -- Verifies parameterized query prevents SQL injection in tenant_id

For the full isolation test matrix (repository and handler levels), see [07-testing-infrastructure.md](07-testing-infrastructure.md#repository-isolation-test-pattern).

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Decision | Status |
|---|----------|--------|
| D7 | App-layer filtering + PG RLS defense-in-depth | Applied |
| D21 | JWT-only tenant resolution | Applied |
| D22 | Ignore `BODHI_MULTITENANT_CLIENT_ID` in middleware | Applied |
| D23 | Unified code path (no deployment mode branching) | Applied |
| D27 | `AppAccessRequest` flow needs no auth-scoping changes | Applied |

**Unnumbered decisions (RLS-specific)**:

| Decision | Status |
|----------|--------|
| Distribute RLS from centralized file into per-table migrations | Applied |
| `current_tenant_id()` function in `000000_extensions.rs` | Applied |
| `FORCE ROW LEVEL SECURITY` on all RLS tables | Applied |
| Parameterized query for `SET LOCAL app.current_tenant_id` | Applied |
| `tenants` table has no RLS (global) | Applied |
| `settings` table has no RLS (no `tenant_id` column) | Applied |
| `tenants_users` RLS: read-all + mutation-restricted | Applied |
| `app_access_requests` RLS: 3-policy (read-all + nullable tenant_id) | Applied |
| All `tenants_users` mutations use `with_tenant_txn` | Applied |
| `list_user_tenants`/`has_tenant_memberships` use `&self.db` directly | Applied |
| `get_api_token_by_prefix` is cross-tenant by design | Applied |

## Known Gaps & TECHDEBT

1. **`tenants_users` not verified in `test_rls.rs`**: The `test_postgres_rls_policies_and_function_installed` test checks RLS policies on 14 tables but does not check `tenants_users`. The `tenants_users_read` and `tenants_users_mutation` policies are created by the migration but are not verified by the RLS audit test. The table is exercised by `test_tenant_repository_isolation.rs` at the repository level.

2. **No isolation test for `app_toolset_configs`**: The table has standard `tenant_isolation` RLS and a `tenant_id` column, but no dedicated cross-tenant isolation test exists at the repository level. Covered implicitly through toolset tests. See [07-testing-infrastructure.md](07-testing-infrastructure.md#known-gaps--techdebt) (Gap 13).

3. **No routes_app isolation tests for user aliases or user access requests**: Routes-layer isolation tests exist for tokens, MCPs, MCP servers, toolsets, API models, and downloads, but not for user aliases (`GET /bodhi/v1/models`) or user access requests. These domains have repository-level isolation tests only. See [07-testing-infrastructure.md](07-testing-infrastructure.md#known-gaps--techdebt) (Gap 14).

4. **`list_user_tenants` does not filter by `app_status=Ready`**: The architecture doc specified returning only Ready tenants, but the current implementation returns all tenants for a user regardless of status. See [04-database-migrations-entities.md](04-database-migrations-entities.md#known-gaps--techdebt) (Gap 1).

5. **`app_access_requests.tenant_id` nullable**: Migration 009 changed `tenant_id` from `NOT NULL` to `NULL` to support cross-tenant approval flow. The custom 3-policy RLS allows `tenant_id IS NULL` rows. This is intentional but means access requests can exist outside any tenant scope.

6. **No end-to-end RLS enforcement test**: The RLS audit test verifies policies exist and are enabled. Repository isolation tests verify functional isolation on both SQLite and PostgreSQL. But there is no test that directly demonstrates a PostgreSQL RLS policy blocking a query (e.g., beginning a transaction for Tenant A and trying to read Tenant B's data via raw SQL). The defense-in-depth claim relies on policy presence verification plus functional isolation tests, not direct RLS blocking evidence.
