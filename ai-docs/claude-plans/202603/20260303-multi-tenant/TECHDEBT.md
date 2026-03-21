# TECHDEBT — Multi-Tenant / CRUD Unification

---

## Multi-Tenant Deferred Items

Items deliberately deferred from the standalone-first multi-tenant implementation.
Required for production multi-tenant mode but out of scope for the current standalone-first implementation.

### P0-12: Anonymous tenant resolution for multi-tenant mode

**Status**: Deferred

In standalone mode, `AuthContext::Anonymous { tenant_id }` carries the single tenant's ID
(populated by the `anon()` closure in `optional_auth_middleware`). In multi-tenant mode,
pre-auth requests (e.g., `/api/v1/setup`, OAuth initiation) arrive before any JWT is available.
The correct approach depends on URL-slug or subdomain routing — deferred until multi-tenant
routing strategy is finalized.

**Current state**: `anon()` closure uses `instance_tenant_id.clone()` from the cached standalone
tenant lookup (`crates/routes_app/src/middleware/auth/auth_middleware.rs:188-191`).

### P0-15: Bearer token cross-tenant JWT validation

**Status**: Deferred

External JWT bearer tokens (for `AuthContext::ExternalApp`) have their `azp` (authorized party)
claim validated against the token exchange result, but cross-tenant isolation (ensuring the JWT's
issuer matches the expected tenant's Keycloak realm) is not enforced. In multi-tenant mode,
each tenant would have its own Keycloak client and realm — the issuer check must be per-tenant.

**Current state**: Single `BODHI_AUTH_ISSUER` checked against all JWTs
(`crates/routes_app/src/middleware/token_service/token_service.rs:198-199`).

### P0-16: `get_standalone_app()` multi-tenant branching in middleware

**Status**: Deferred

`auth_middleware` and `optional_auth_middleware` still call `tenant_service.get_standalone_app()`
as their first operation, assuming one tenant. In multi-tenant mode, tenant lookup must be
derived from:
- JWT `azp` (client_id) claim for ExternalApp/Session flows
- Token suffix `.<client_id>` for ApiToken flows (already implemented)
- URL slug or subdomain for Anonymous/pre-auth flows

**Current state**: Both middleware functions call `get_standalone_app()` at lines 92 and 170 of
`crates/routes_app/src/middleware/auth/auth_middleware.rs`. ApiToken flow correctly derives
tenant from token suffix (Layer 4).

### P2-8: Cross-tenant integration tests

**Status**: Deferred

The current test suite validates single-tenant isolation via `TEST_TENANT_ID`. True cross-tenant
isolation tests (verifying that tenant A cannot see tenant B's data) require:
- A `two_tenants_fixture()` producing two separate tenants
- Tests that insert data under tenant A and verify tenant B cannot see it
- RLS policy activation in PostgreSQL test context
- A test router variant that initializes multiple tenants and routes requests by `tenant_id`
  header or JWT claim

**Prerequisite**: Multi-tenant middleware branching (P0-16) must be implemented first.

### Queue service tenant context

**Status**: Deferred

`queue_service.rs` creates `ModelMetadataEntity` with `tenant_id: String::new()` (empty string)
at line 97. The metadata queue worker runs asynchronously without an auth context. In
multi-tenant mode, the queue message must carry `tenant_id` so extracted metadata is attributed
to the correct tenant.

**File**: `crates/services/src/utils/queue_service.rs`

### D12: `LocalDataService` and `MultiTenantDataService` naming

**Status**: Deferred (naming change only)

Consider renaming `LocalDataService` → `StandaloneModelService` and
`MultiTenantDataService` → `MultitenantModelService` to better reflect that they
manage model aliases (not general data). The current names are technically accurate
but slightly misleading given the broader `DataService` trait.

**Files**: `crates/services/src/models/data_service.rs`,
`crates/services/src/models/multi_tenant_data_service.rs`

---

## CRUD Unification Tech Debt

### Generic Paginated Response

`PaginatedApiModelOutput`, `PaginatedTokenResponse`, `PaginatedDownloadResponse` etc. are
all identical wrappers (`data: Vec<T>, total, page, page_size`). Consider a single `Paginated<T>`
generic type.

---

## Architecture Decisions Reference

| Decision | Summary |
|---|---|
| D4 | Unified schema: standalone = one tenant, multi-tenant = multiple tenants |
| D7 | App-layer WHERE clauses primary; PostgreSQL RLS secondary (defense-in-depth) |
| D9 | Settings table is global (no tenant_id); per-tenant settings → future `tenant_settings` table |
| D10 | LLM routes disabled in multi-tenant mode at route registration time (intentional) |
| D13 | Tenant resolved from JWT `azp` (client_id) → `tenants` table lookup |
| D18 | Tenant row created during `setup_create()` |
| D20 | `app_toolset_configs` seeding removed; per-tenant toolset configs are admin-managed |
