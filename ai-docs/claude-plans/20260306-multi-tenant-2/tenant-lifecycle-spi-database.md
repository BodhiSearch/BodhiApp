# Tenant Lifecycle, SPI Integration & Database Evolution

> **Scope**: Cross-cutting analysis of multi-tenant stage 2 implementation.
> **Audience**: AI coding assistants working on BodhiApp's multi-tenant system.
> **Key files**: `crates/services/src/tenants/`, `crates/routes_app/src/tenants/`, `crates/services/src/db/`, `crates/services/src/auth/auth_service.rs`

---

## The Dual-System Architecture

BodhiApp's multi-tenant design is built on two complementary systems that each own distinct responsibilities:

**Keycloak** owns identity. It manages users, authentication, OAuth2 clients, group-based role assignment, and JWT issuance. The Bodhi SPI extension (`keycloak-bodhi-ext`) adds tenant-aware endpoints on top of Keycloak's core. Keycloak maintains its own tables (`bodhi_clients`, `bodhi_clients_users`) for fast tenant-membership queries at the SPI layer.

**BodhiApp** owns operational state. It stores tenant configuration (encrypted client secrets, app status), user-created resources (aliases, tokens, toolsets, MCP servers), and session management. BodhiApp's `tenants` table mirrors a subset of what Keycloak knows -- enough to operate without cross-system queries at runtime.

These systems are peers, not master/slave. Neither has a complete view of the other's data, and that is intentional. Keycloak never queries BodhiApp's database. BodhiApp queries Keycloak only during explicit SPI calls (tenant creation, tenant listing). At steady state -- serving API requests, running inference, managing resources -- BodhiApp operates from its own database using tenant context derived from JWT claims.

The key insight: **identity flows down from Keycloak via JWTs; operational data lives entirely in BodhiApp's DB.** A JWT's `azp` claim resolves to a tenant's `client_id`, which maps to a local tenant row. From there, all data access is local and RLS-scoped.

---

## Tenant Registration Flow

End-to-end, creating a new tenant involves four systems (UI, BodhiApp API, Keycloak SPI, BodhiApp DB) in a synchronous chain:

### Step 1: User initiates from UI

The user is authenticated against the dashboard client (`bodhi.client_type=multi-tenant`). The frontend sends `POST /bodhi/v1/tenants` with `{ name, description }`. No `redirect_uris` -- the user never sees OAuth plumbing.

### Step 2: Route handler prepares the SPI call

The `tenants_create` handler in `routes_tenants.rs` does three things before calling the SPI:
1. **Gate check**: Verifies `is_multi_tenant()` -- standalone deployments reject this endpoint.
2. **Extract dashboard token**: Calls `require_dashboard_token()` on the auth context. This token (stored in the user's session under `dashboard:access_token`) authenticates the user to the SPI.
3. **Build redirect URIs**: Constructs `{public_server_url}/ui/auth/callback` from server configuration. The user never supplies these.

### Step 3: SPI creates the Keycloak client

`AuthService::create_tenant()` sends `POST /realms/{realm}/bodhi/tenants` to the Keycloak SPI with the dashboard token as Bearer auth. The SPI validates the token's `azp` is a dashboard client, enforces the one-tenant-per-user constraint, and creates a full OAuth2 resource client with:
- A generated `client_id` (prefix `bodhi-tenant-`)
- A generated `client_secret`
- Role groups (`users`, `power-users`, `managers`, `admins`)
- The creator added to the `admins` group
- Rows in `bodhi_clients` and `bodhi_clients_users`

The SPI returns `{ client_id, client_secret }`.

### Step 4: BodhiApp creates the local tenant row

Back in the route handler, `TenantService::create_tenant()` is called with the SPI response. This:
1. Generates a ULID for the local tenant ID
2. Encrypts the client secret (AES-GCM via the app's encryption key)
3. Inserts the tenant row with `status = Ready` and `created_by = user_id`
4. Atomically upserts a `tenants_users` membership row in the same transaction

The handler returns `201 Created` with `{ client_id }`.

### Why status is Ready immediately

Decision D79: multi-tenant tenants skip the setup wizard. In standalone mode, a tenant starts in `Setup` status and transitions through `ResourceAdmin` to `Ready` via a gated flow. In multi-tenant mode, the SPI has already configured the Keycloak client with roles and groups -- there is nothing left to set up. The user can immediately log into the new tenant and start using it.

---

## The SPI Proxy Pattern

BodhiApp never talks to the Keycloak Admin API. All Keycloak interactions go through the Bodhi SPI -- a custom Keycloak extension deployed at `/realms/{realm}/bodhi/`.

### Why not the Admin API

The Keycloak Admin API requires service-account credentials with realm-level permissions. Granting BodhiApp those permissions would violate the trust boundary: BodhiApp could modify any client, any user, any role. The SPI provides a **scoped, auditable surface** where:
- Tenant creation enforces business rules (one-per-user, correct client type, mandatory groups)
- Role assignment validates authorization chains (only admins/managers can assign roles)
- Client type validation ensures only dashboard clients can create tenants

The SPI is a policy enforcement layer. BodhiApp treats it as a black box that accepts requests and returns results, without needing to understand Keycloak's internal data model.

### The dashboard token as bearer

SPI endpoints authenticate via the user's dashboard token -- the JWT obtained when the user logged into the dashboard client. This token's `azp` claim identifies the dashboard client, and its `sub` claim identifies the user. The SPI validates both.

This means SPI calls are user-scoped by design. A user can only create tenants under dashboard clients they are authenticated against. The one-tenant-per-user constraint is enforced at the SPI level using the `bodhi_clients` table.

### Forward request escape hatch

`AuthService::forward_request()` provides a generic HTTP proxy to any SPI endpoint. It is used by dev-only test endpoints (`/dev/clients/{client_id}/dag`, `/dev/tenants/cleanup`) to call SPI test helpers without defining typed methods for each. Production code uses typed methods (`create_tenant`, `list_tenants`).

---

## Trust Model & Source of Truth

The trust boundaries are intentionally asymmetric:

| Data | Source of Truth | Where Used |
|------|----------------|------------|
| User identity (sub, username) | Keycloak | JWT claims, session |
| User roles per tenant | Keycloak groups | JWT `resource_access` claims |
| Tenant membership (which users can access which tenants) | **Both** | SPI queries `bodhi_clients_users`; BodhiApp queries `tenants_users` |
| Tenant operational state (status, encrypted secret) | BodhiApp DB | Auth middleware, token exchange |
| User-created resources (aliases, tokens, MCP, toolsets) | BodhiApp DB | API endpoints |

The overlap on membership is deliberate. Keycloak's `bodhi_clients_users` table is the authoritative source for "can this user log into this tenant?" -- it controls visibility in the SPI's `GET /tenants` response. BodhiApp's `tenants_users` table is a local cache for "which tenants should I show this user?" -- it avoids an SPI round-trip on every page load.

When these diverge (e.g., a user is added to a tenant via `POST /resources/assign-role` in Keycloak), the BodhiApp `tenants_users` row is not automatically created. The expectation is that role assignment workflows will also trigger local membership creation. If they don't, the user can still log into the tenant (Keycloak controls that), but won't see it in the BodhiApp tenant list until a membership row is created.

BodhiApp never queries Keycloak for role information at runtime. After a user logs into a specific tenant, their role is embedded in the JWT's `resource_access` claims. The auth middleware extracts the role from the token -- no SPI call needed.

---

## tenants_users Membership Table

### Why it exists separately from Keycloak groups

The `tenants_users` table (`migration 000015`) solves a query performance problem. When a user visits the tenant list page, BodhiApp needs to answer "which tenants does this user belong to?" Answering that from Keycloak would require an SPI round-trip for every page load. The local table makes this a single indexed query: `SELECT tenant_id FROM tenants_users WHERE user_id = ?`.

The table is intentionally minimal -- a composite primary key of `(tenant_id, user_id)` plus timestamps. No role column. No status column. Presence means membership; absence means no membership. Role information comes from Keycloak groups, accessible via JWT claims after tenant login.

### The split RLS policy design

Most tenant-scoped tables use a single RLS policy: `tenant_id = current_tenant_id()` for all operations. The `tenants_users` table is different. It has **two separate policies**:

1. **`tenants_users_read`**: `FOR SELECT USING (true)` -- any authenticated session can read all rows. This is necessary because `list_user_tenants(user_id)` must read membership rows across all tenants, not just the "current" one. The user might belong to tenants A, B, and C; the query needs to see all three rows regardless of which tenant context is set.

2. **`tenants_users_mutation`**: `FOR ALL USING (tenant_id = current_tenant_id()) WITH CHECK (...)` -- writes are restricted to the current tenant context. You can only insert/update/delete membership rows for the tenant you are operating within.

This split is the key architectural decision (D84). Without it, either:
- The open read would expose all data to any tenant (bad for other tables), or
- The restricted read would break `list_user_tenants` (can't see cross-tenant memberships)

The `tenants_users` table is unique in the schema because membership is inherently a cross-tenant concept.

### Query pattern

`list_user_tenants` is a two-step query:
1. Get `tenant_ids` from `tenants_users` where `user_id` matches (cross-tenant read)
2. Fetch tenant details from `tenants` where `id` is in that set

This avoids a JOIN (keeping the query plan simple) and works identically on SQLite (no RLS) and PostgreSQL (read policy is permissive).

---

## Database Evolution

### How tenant_id was retrofitted

Every operational table in BodhiApp has a `tenant_id` column. This was not added retroactively -- the multi-tenant schema was designed from the start of the stage 1 refactor. All 13 tenant-scoped tables (download_requests, api_model_aliases, model_metadata, access_requests, api_tokens, toolsets, app_toolset_configs, user_aliases, app_access_requests, mcp_servers, mcp_auth_headers, mcp_oauth, tenants_users) include `tenant_id` in their initial migration.

The `tenants` table itself does not have a `tenant_id` column -- it is the tenant. Its `id` column is the value that appears in other tables' `tenant_id` fields.

### RLS policy design philosophy

The RLS architecture follows a consistent pattern across all tenant-scoped tables:

1. **PostgreSQL-only**: All RLS code is gated behind `if backend == Postgres` checks. SQLite has no RLS support.

2. **Session variable binding**: A PostgreSQL function `current_tenant_id()` reads the session variable `app.current_tenant_id`, returning NULL if unset. This function is created in migration `000000_extensions`.

3. **Transaction-scoped context**: `begin_tenant_txn(tenant_id)` calls `SET LOCAL app.current_tenant_id = ?` (via `set_config` with `true` for transaction-local). The `SET LOCAL` ensures the tenant context is automatically cleared when the transaction ends.

4. **Uniform policy**: Most tables use a single `tenant_isolation` policy: `FOR ALL USING (tenant_id = current_tenant_id()) WITH CHECK (tenant_id = current_tenant_id())`. Both reads and writes are restricted.

5. **FORCE ROW LEVEL SECURITY**: Applied to all tables so that even the table owner (the database role running migrations) is subject to RLS. Without FORCE, the superuser/owner bypasses policies.

Exceptions to the uniform pattern:
- `tenants_users`: Split read/write policies (discussed above)
- `app_access_requests`: Uses `tenant_id IS NULL OR tenant_id = current_tenant_id()` to handle legacy rows that predate tenant_id
- `tenants`: No RLS -- the tenants table is not tenant-scoped
- `settings`: Not tenant-scoped (global configuration)

### SQLite vs PostgreSQL handling

SQLite is used for development and desktop (Tauri) deployments. PostgreSQL is used for production Docker deployments. The difference in multi-tenant behavior:

- **SQLite**: `begin_tenant_txn` returns a plain transaction with no additional setup. There is no RLS enforcement. Tenant isolation is handled purely in application logic -- every query includes `WHERE tenant_id = ?`. This works because desktop deployments are single-tenant.

- **PostgreSQL**: `begin_tenant_txn` sets the session variable before returning the transaction. RLS policies automatically filter all queries. This is defense-in-depth: even if application code omits a `WHERE tenant_id = ?`, the database rejects cross-tenant access.

The application code is identical for both backends. `begin_tenant_txn` abstracts the difference. Queries always include `tenant_id` filters for correctness on SQLite, and those filters are redundant (but harmless) on PostgreSQL where RLS would catch violations anyway.

---

## The set_tenant_ready Pattern

### Atomic status transition

`set_tenant_ready(tenant_id, user_id)` is a single transactional operation that does three things atomically:
1. Updates the tenant's `app_status` to `Ready`
2. Sets `created_by` to the `user_id`
3. Upserts a `tenants_users` membership row

This replaced a previous multi-step pattern where the route handler made separate calls to update status, set created_by, and upsert membership. The old pattern had a failure window: if the status update succeeded but the membership upsert failed, the tenant would be in an inconsistent state.

### Why created_by is set at ready-time, not creation-time

In the standalone flow, a tenant is created during app initialization with `status = Setup` and `created_by = None`. At that point, no user has authenticated yet -- the app is bootstrapping. The first user to complete the setup wizard triggers `set_tenant_ready`, which atomically transitions the tenant and records who completed the setup.

In the multi-tenant flow, `created_by` is set at creation time because the user is already authenticated when they call `POST /bodhi/v1/tenants`. The `create_tenant` implementation validates this: if `status == Ready` and `created_by` is None, it returns a validation error.

The shared `create_tenant_impl` method handles both cases:
- **With `created_by`**: Opens a transaction, inserts tenant, upserts membership, commits
- **Without `created_by`**: Plain insert, no transaction needed (single operation)

This ensures that whenever a tenant has a known creator, the membership relationship is established atomically.

---

## Dev Cleanup & Testing Infrastructure

### How dev endpoints work

Two dev-only endpoints support integration testing of multi-tenant flows:

**`POST /dev/clients/{client_id}/dag`**: Enables Direct Access Grants on a Keycloak client. Integration tests need to obtain tokens without a browser-based OAuth flow. This endpoint proxies to the SPI's `test/clients/{client_id}/dag` endpoint (which sets the Keycloak client attribute), then looks up the local tenant to return `{ client_id, client_secret }`. Tests use these credentials for direct `grant_type=password` token requests.

**`DELETE /dev/tenants/cleanup`**: Tears down all test tenants. Proxies to the SPI's `test/tenants/cleanup` endpoint (which deletes Keycloak clients, groups, and `bodhi_clients`/`bodhi_clients_users` rows), then truncates the local `tenants` table. This gives tests a clean slate without manual cleanup.

### Security model

Both endpoints are registered only in the `!is_production()` block in route configuration (D106). Production builds never expose these routes -- they are compiled in but not mounted. Each endpoint additionally checks `is_multi_tenant()` and requires a valid dashboard token, so even in dev mode they cannot be called anonymously.

The endpoints use `AuthScope` (not raw `AppService`), meaning they go through the same auth middleware as production endpoints. The dashboard token provides user identity for SPI calls. The `DevError` enum handles multi-tenant gate checks, SPI failures, and local tenant lookups with proper error typing.

### The `reset_all_tables` carve-out

`reset_all_tables` (used by `POST /dev/db-reset`) intentionally does NOT truncate the `tenants` table. Tenants hold OAuth2 client credentials that must survive database resets during integration testing. A separate `reset_tenants` method exists for explicit tenant cleanup via the dedicated endpoint.

---

## The Orphan Tenant Problem

### D52: What happens when local DB insert fails after SPI creation

The tenant creation flow has a two-system write: first the SPI (Keycloak), then the local DB. If step 1 succeeds but step 2 fails, a Keycloak client exists with no corresponding BodhiApp tenant row. This is an **orphan**.

The decision was to accept orphans. The rationale:

1. **Orphan clients are harmless**: A Keycloak client with no BodhiApp counterpart cannot serve requests. Nobody can log into it because the auth middleware resolves tenant from `client_id` via the local DB. No DB row means the middleware returns "tenant not found."

2. **Compensating deletes are risky**: Calling the Keycloak Admin API to delete the orphaned client introduces a new failure mode. If the compensating delete also fails, you have an orphan anyway, plus more complex error handling.

3. **Frequency is extremely low**: The local DB insert is a simple row insert with encryption. It fails only under exceptional conditions (disk full, connection lost, encryption key issue).

4. **Manual cleanup is trivial**: Orphan clients can be identified and removed via the Keycloak admin console.

The handler logs the failure at error level with the orphaned `client_id` for operational visibility:

```
D52: local DB failed after SPI creation -- Keycloak orphan at client_id=bodhi-tenant-abc123
```

The error is propagated to the caller as a 5xx response. The user can retry, and the one-per-user constraint at the SPI level will either return a conflict (if using the same dashboard) or succeed with a new client_id.

---

## Extension Points

### Adding tenant metadata

The `tenants` table has a `description` field but no extensible metadata store. Future tenant-level settings (custom branding, feature flags, quota limits) could be added via:
- A `tenant_metadata` JSON column on the `tenants` table (simple)
- A separate `tenant_settings` key-value table (more flexible, follows the existing `settings` pattern)

The existing `settings` table is global (no `tenant_id`). Tenant-scoped settings would need their own table with RLS.

### Tenant-level settings

The current `SettingService` is not auth-scoped -- it is listed as one of the passthrough accessors on `AuthScopedAppService`. Tenant-scoped settings would require a new service with tenant isolation, similar to how `TokenService` became `AuthScopedTokenService`.

### Tenant deactivation

No deactivation flow exists. The `AppStatus` enum has `Setup`, `Ready`, `ResourceAdmin`, and `TenantSelection` -- no `Disabled` or `Suspended` state. Adding deactivation would require:
1. A new `AppStatus` variant
2. Middleware to reject requests for disabled tenants (currently middleware trusts any resolved tenant)
3. An admin endpoint to toggle status
4. Decision on whether deactivation propagates to Keycloak (disable the client) or is BodhiApp-only

### Known tech debt

The `get_standalone_app()` method on `TenantService` queries for a single tenant and errors with `DbError::MultipleTenant` if more than one exists. Four production code paths still use it (documented in TECHDEBT.md). These must be replaced with tenant-aware lookups before multi-tenant mode is fully production-ready.
