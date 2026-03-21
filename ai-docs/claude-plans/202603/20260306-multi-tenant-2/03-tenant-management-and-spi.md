# Tenant Management, Membership, and SPI Integration

## Overview

BodhiApp's tenant management covers the full lifecycle of OAuth2 client registrations ("tenants"): creation via the Keycloak Bodhi SPI, local persistence with encrypted secrets, user-tenant membership via a `tenants_users` join table, and tenant selection/switching through session-based activation. In multi-tenant mode, a two-phase login flow requires dashboard authentication before tenant operations (see [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#multi-tenant-auth-flow-two-phase)); in standalone mode, a single tenant is created through a setup wizard. The `AuthScopedTenantService` and `AuthScopedUserService` maintain membership consistency by dual-writing to both Keycloak groups (source of truth for roles) and the local `tenants_users` table (source of truth for fast membership queries). BodhiApp's app-level DB is the sole owner of tenant and membership data; the Keycloak SPI creates OAuth2 clients, groups, and service accounts in Keycloak natively but maintains no separate tracking tables.

## Functional Behavior

### Tenant CRUD Endpoints

| Endpoint | Method | Middleware | Mode | Purpose |
|----------|--------|------------|------|---------|
| `/bodhi/v1/tenants` | GET | `optional_auth` | Multi-tenant only | List user's tenants from local DB, enriched with session metadata |
| `/bodhi/v1/tenants` | POST | `optional_auth` | Multi-tenant only | Create tenant via SPI + local DB row + membership (atomic) |
| `/bodhi/v1/tenants/{client_id}/activate` | POST | `optional_auth` | Multi-tenant only | Switch active tenant (session update, no re-auth if token cached) |
| `/bodhi/v1/setup` | POST | `public` | Standalone only | Register client via SPI `POST /resources`, create tenant with status `ResourceAdmin` |

All multi-tenant endpoints check `auth_context.is_multi_tenant()` and return `DashboardAuthRouteError::NotMultiTenant` (error code: `dashboard_auth_route_error-not_multi_tenant`) in standalone mode (D101).

### GET /bodhi/v1/tenants -- List Tenants

**Requires**: Authenticated `MultiTenantSession` (dashboard token present, `user_id` extractable).

**Behavior**: Queries local `tenants_users` table for the user's memberships, joins with `tenants` table. No SPI call. Each tenant is enriched with session metadata:
- `is_active: bool` -- `true` if `client_id` matches session's `active_client_id`
- `logged_in: bool` -- `true` if `{client_id}:access_token` exists in session

**Response** (200):
```json
{
  "tenants": [
    {
      "client_id": "bodhi-tenant-abc123",
      "name": "My Workspace",
      "description": "A team workspace",
      "status": "ready",
      "is_active": true,
      "logged_in": true
    }
  ]
}
```

### POST /bodhi/v1/tenants -- Create Tenant

**Requires**: Authenticated `MultiTenantSession` with dashboard token.

**Request**:
```json
{
  "name": "string (required, 1-255 chars)",
  "description": "string (optional, max 1000 chars)"
}
```

**Behavior** (two-phase: SPI then local):
1. Extract `dashboard_token` from `AuthContext::MultiTenantSession`
2. Construct `redirect_uris` from `settings.public_server_url()` + `/ui/auth/callback`
3. Proxy to Keycloak SPI: `POST /realms/{realm}/bodhi/tenants` with dashboard bearer token
4. SPI creates resource client (`bodhi-tenant-{uuid}`), assigns creator as `resource_admin` via Keycloak groups, and creates a service account
5. App enforces one-tenant-per-user: `tenant_repository.create_tenant()` checks `list_tenants_by_creator(user_id)` and returns `TenantError::UserAlreadyHasTenant` (400 Bad Request) if the user already owns a tenant
6. Create local tenant row with `status: Ready`, `created_by: user_id`, encrypted `client_secret`
7. Atomically insert `tenants_users` membership in the same transaction as the tenant row
8. If local DB fails after SPI success, log error (D52: accept Keycloak orphan) and return 5xx

**Response** (201):
```json
{ "client_id": "bodhi-tenant-abc123" }
```

**One-per-user constraint** (D65): Enforced at the app repository level. When `created_by` is `Some(user_id)`, `tenant_repository.create_tenant()` calls `list_tenants_by_creator(user_id)` and returns `TenantError::UserAlreadyHasTenant` (400 Bad Request) if the user already owns a tenant. Standalone mode (`created_by: None`) always succeeds.

### POST /bodhi/v1/tenants/{client_id}/activate -- Activate Tenant

**Requires**: Multi-tenant mode. The target tenant's `{client_id}:access_token` must exist in session.

**Behavior**: Sets `active_client_id` in session to the given `client_id`. Returns 200. No SPI call, no DB write. If the token does not exist in session, returns `DashboardAuthRouteError::TenantNotLoggedIn` (400).

### Standalone Tenant Creation (POST /bodhi/v1/setup)

**Behavior**: Calls SPI `POST /resources` (anonymous, no auth header). SPI creates the resource client and service account in Keycloak. Creates local tenant with `status: ResourceAdmin`, `created_by: None`. The `created_by` and membership are set later during the first admin login via `set_tenant_ready()`.

### Tenant Ready Transition (Standalone)

During `POST /auth/callback` when tenant status is `ResourceAdmin`:
1. `auth_service.make_resource_admin()` -- assigns admin role in Keycloak groups
2. `tenant_service.set_tenant_ready(tenant_id, user_id)` -- atomically sets `status=Ready`, `created_by=user_id`, and upserts `tenants_users` membership in one transaction

### Tenant Selection and Switching Flow

**Initial selection** (after dashboard login):
- 0 tenants: frontend redirects to `/ui/setup/tenants/` (registration form)
- 1 tenant: auto-initiates resource-client OAuth2 (seamless with Keycloak SSO)
- N tenants: shows tenant selector dropdown with `is_active` and `logged_in` status

**Switching**:
- Target has `logged_in: true`: call `POST /tenants/{client_id}/activate` (instant, no re-auth)
- Target has `logged_in: false`: initiate OAuth2 login for target resource-client (Keycloak SSO reuses session, no password prompt)

See [06-frontend-ui.md](06-frontend-ui.md#login-page-state-machine-multi-tenant) for the frontend rendering of these states.

### User Invitation via Shareable Link

BodhiApp uses shareable invite links for user invitation. Email-based user addition is not supported due to the email enumeration risk inherent in the shared Keycloak realm — the SPI's `POST /resources/assign-role` accepts only `user_id` (UUID), not email, by design.

**Flow**:
1. Admin copies invite URL from the users management page: `{public_url}/ui/login/?invite={client_id}`
2. Invited user clicks link → dashboard OAuth → tenant OAuth for the specific `client_id`
3. If the user has no role on the tenant, `AppInitializer` redirects to `/ui/request-access`
4. User submits access request → admin approves via existing access-request management
5. On approval, the session is cleared, forcing re-authentication with the newly assigned role

No Keycloak SPI changes are needed — the invite flow uses existing endpoints:
- Standard OAuth2 for authentication
- `POST /resources/assign-role` (by `user_id`) for admin approval
- `GET /resources/users` for user listing

The `AppInfo` response from `GET /bodhi/v1/info` includes a `url: String` field containing `settings.public_server_url()`, which the frontend uses to construct invite URLs.

See [06-frontend-ui.md](06-frontend-ui.md#invite-link-flow) for the frontend implementation and [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md) for the auth flow details.

## Architecture & Data Model

### Tenant Struct

```rust
// crates/services/src/tenants/tenant_objs.rs
pub struct Tenant {
  pub id: String,                       // ULID, auto-generated
  pub client_id: String,                // OAuth2 client_id (e.g. "bodhi-tenant-{uuid}")
  pub client_secret: String,            // Decrypted at read time
  pub name: String,                     // User-provided
  pub description: Option<String>,      // User-provided
  pub status: AppStatus,                // Setup | ResourceAdmin | Ready | TenantSelection
  pub created_by: Option<String>,       // Keycloak user ID (JWT sub claim)
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
```

For database entity details, see [04-database-migrations-entities.md](04-database-migrations-entities.md#tenants-table-schema).

### tenants_users Table (Membership)

Composite primary key: `(tenant_id, user_id)`. FK to `tenants(id)` with CASCADE delete. Index on `user_id` for user-centric lookups.

- Presence = membership; absence = no membership (no role column)
- Roles are stored in Keycloak groups, not in this table (D95)
- No RLS on SELECT -- queried by `user_id` cross-tenant; mutations restricted to current tenant context

For full schema and RLS policies, see [04-database-migrations-entities.md](04-database-migrations-entities.md#tenants_users-table-schema) and [05-data-isolation-rls.md](05-data-isolation-rls.md#three-rls-policy-patterns).

### SPI Request/Response Types

```rust
// crates/services/src/tenants/kc_types.rs
pub struct CreateTenantRequest {
  pub name: String,
  pub description: String,
  pub redirect_uris: Vec<String>,
}

pub struct KcCreateTenantResponse {
  pub client_id: String,         // "bodhi-tenant-{uuid}"
  pub client_secret: String,     // Generated by Keycloak
}
```

### Client ID Naming Convention

| Context | Prefix |
|---------|--------|
| Standalone resource (production) | `bodhi-resource-{UUID}` |
| Multi-tenant resource (production) | `bodhi-tenant-{UUID}` |
| Standalone resource (test) | `test-resource-{UUID}` |
| Multi-tenant resource (test) | `test-tenant-{UUID}` |
| App client (production) | `bodhi-app-{UUID}` |

Both standalone and multi-tenant resource clients share `bodhi.client_type=resource` in Keycloak. The prefix distinguishes origin. Existing deployed clients keep old IDs (no migration).

### Service Layer

**TenantRepository** (`crates/services/src/tenants/tenant_repository.rs`):

| Method | Signature | Notes |
|--------|-----------|-------|
| `create_tenant` | `(client_id, secret, name, desc, status, created_by) -> TenantRow` | Auto-generates ULID. When `created_by` is `Some(user_id)`, checks `list_tenants_by_creator(user_id)` first and returns `TenantError::UserAlreadyHasTenant` if the user already owns a tenant, then atomically inserts tenant + membership in one transaction. Validates: `status == Ready` requires `created_by`. |
| `list_tenants_by_creator` | `(created_by: &str) -> Vec<TenantRow>` | `SELECT * FROM tenants WHERE created_by = ?`. Used for one-per-user enforcement. |
| `set_tenant_ready` | `(tenant_id, user_id) -> ()` | Transactional: UPDATE status + created_by + upsert tenants_users membership. Single call replaces former `update_tenant_status` + `update_tenant_created_by` + `upsert_tenant_user`. |
| `upsert_tenant_user` | `(tenant_id, user_id) -> ()` | INSERT ON CONFLICT updates `updated_at`. Idempotent. |
| `delete_tenant_user` | `(tenant_id, user_id) -> ()` | Idempotent delete. |
| `list_user_tenants` | `(user_id) -> Vec<TenantRow>` | Two-step: get tenant_ids from membership, then fetch tenants. |
| `has_tenant_memberships` | `(user_id) -> bool` | EXISTS query on tenants_users. |
| `create_tenant_test` | `(Tenant) -> TenantRow` | Test-only. Upsert guard (returns existing if client_id exists). Shares `create_tenant_impl`. |

Internal helper `create_tenant_impl` is shared between `create_tenant` and `create_tenant_test` to ensure consistent validation, encryption, and atomic membership creation.

**TenantService** (`crates/services/src/tenants/tenant_service.rs`): Thin delegation over `TenantRepository`. Wraps `Arc<dyn DbService>`. All methods (including `list_tenants_by_creator`) delegate directly to the repository.

**AuthScopedTenantService** (`crates/services/src/tenants/auth_scoped.rs`):

| Method | Behavior |
|--------|----------|
| `list_my_tenants()` | Injects `user_id` from `AuthContext.require_user_id()` |
| `has_memberships()` | Injects `user_id` from `AuthContext.require_user_id()` |
| `create_tenant(...)` | Passthrough (takes explicit params, no user_id injection) |
| `list_tenants_by_creator(...)` | Passthrough |
| `set_tenant_ready(...)` | Passthrough |
| `upsert_tenant_user(...)` | Passthrough |
| `get_standalone_app()` | Passthrough |
| `get_tenant_by_client_id(...)` | Passthrough |

**AuthScopedUserService** (`crates/services/src/users/auth_scoped.rs`): Dual-writes membership on role changes:
- `assign_user_role(target_user_id, role)`: Calls `auth_service.assign_user_role()` (Keycloak groups), then `tenant_service.upsert_tenant_user(tenant_id, target_user_id)` (local membership). The SPI's `POST /resources/assign-role` accepts only `user_id` (UUID), not email, to prevent email enumeration in the shared realm.
- `remove_user(target_user_id)`: Calls `auth_service.remove_user()` (Keycloak groups), then `tenant_service.delete_tenant_user(tenant_id, target_user_id)` (local membership)

### AuthService SPI Proxy

`AuthService.create_tenant()` (`crates/services/src/auth/auth_service.rs`): HTTP POST to Keycloak SPI `{auth_url}/realms/{realm}/bodhi/tenants` with dashboard bearer token. Uses the existing `reqwest` client. Returns `KcCreateTenantResponse { client_id, client_secret }`.

`AuthService.forward_spi_request()`: Generic SPI proxy method (owned `String` params for mockall compatibility, D103). Used by dev-only endpoints.

The `list_tenants` method was removed from `AuthService` during the architecture refactor -- tenant listing is now entirely local DB-based.

### Dev Tenant Cleanup

`dev_tenants_cleanup_handler` (dev-only endpoint):
1. Gets `user_id` from auth context
2. Calls `list_tenants_by_creator(user_id)` to discover the user's tenants locally
3. Filters out tenants whose names start with `[do-not-delete]`
4. Sends explicit `{ "client_ids": [...] }` body to SPI `DELETE /test/tenants/cleanup`
5. Optimistically deletes all sent `client_ids` locally

This avoids any dependency on SPI-side tables for discovering which tenants belong to a user -- the app's local `tenants` table is the sole source.

### Error Types

**TenantError** (`crates/services/src/tenants/error.rs`):
- `NotFound` -- ErrorType::InternalServer (tenant should always exist in authenticated context)
- `UserAlreadyHasTenant` -- ErrorType::BadRequest (one-per-user enforcement, D65)
- `Db(DbError)` -- transparent
- `AuthContext(AuthContextError)` -- transparent

**DashboardAuthRouteError** (`crates/routes_app/src/tenants/error.rs`):
- `NotMultiTenant` -- ErrorType::InvalidAppState
- `TenantNotLoggedIn` -- ErrorType::BadRequest (activate without session token)
- `SessionError`, `SessionInfoNotFound`, `OAuthError`, `StateDigestMismatch`, `MissingState`, `MissingCode`, `ParseError`
- `SettingServiceError`, `AuthServiceError`, `TokenError` -- transparent wrappers

### Data Flow: Tenant Creation (Multi-Tenant)

```
POST /bodhi/v1/tenants { name, description }
  |
  v
tenants_create handler (routes_app/src/tenants/routes_tenants.rs)
  |-- is_multi_tenant() guard
  |-- require_dashboard_token() from AuthContext
  |-- Build redirect_uris from public_server_url() + LOGIN_CALLBACK_PATH
  |
  v
auth_service.create_tenant(dashboard_token, name, desc, redirect_uris)
  |-- HTTP POST to Keycloak SPI: /realms/{realm}/bodhi/tenants
  |-- SPI creates resource client + groups + service account in Keycloak
  |-- Returns { client_id, client_secret }
  |
  v
tenant_service.create_tenant(client_id, secret, name, desc, Ready, Some(user_id))
  |-- list_tenants_by_creator(user_id) -> TenantError::UserAlreadyHasTenant if non-empty
  |-- create_tenant_impl: validate, encrypt secret
  |-- begin_tenant_txn(id) (RLS-aware)
  |-- INSERT tenants row
  |-- INSERT ON CONFLICT tenants_users membership
  |-- COMMIT
  |
  v
Return 201 { client_id }
```

### Data Flow: Standalone First-Login Transition

```
POST /bodhi/v1/auth/callback { code, state }
  |-- tenant status == ResourceAdmin
  |
  v
auth_service.make_resource_admin(client_id, secret, user_id)
  |-- SPI: POST /resources/make-resource-admin
  |-- SPI assigns admin role in Keycloak groups
  |
  v
tenant_service.set_tenant_ready(tenant_id, user_id)
  |-- begin_tenant_txn(tenant_id)
  |-- UPDATE tenants SET status=Ready, created_by=user_id
  |-- INSERT ON CONFLICT tenants_users membership
  |-- COMMIT
  |
  v
Refresh token, store in session, redirect to /ui/chat
```

### Dual-Write Consistency Model

All role mutations maintain consistency between Keycloak groups and local `tenants_users`:

| Operation | Keycloak (via SPI) | Local DB (tenants_users) |
|-----------|-------------------|--------------------------|
| Create tenant (MT) | SPI creates client + admin group membership | `create_tenant` atomically inserts tenant + membership |
| Make resource admin (standalone) | SPI assigns admin group + sets created_by | `set_tenant_ready` atomically updates + upserts membership |
| Assign role | SPI assigns user to group | `upsert_tenant_user` (idempotent) |
| Remove user | SPI removes user from all groups | `delete_tenant_user` (idempotent) |

Keycloak groups are the source of truth for **roles** (embedded in JWT `resource_access` claims). The local `tenants_users` table is the source of truth for **fast membership queries** (used by `GET /tenants`, `has_memberships()`, `/info` status resolution). The dual-write is between these two stores only -- the SPI has no separate tracking tables. If they diverge, membership errors are benign -- the user can still authenticate via Keycloak and membership will be re-synced on next role assignment.

### Organization Features (Deferred)

Keycloak Organizations (GA since 26.0) are intentionally NOT used. The current Keycloak groups + local `tenants_users` table model covers all launch requirements. Organizations provide enterprise features (external IdP linking, email domain auto-enrollment, managed membership) that are deferred to a future upgrade path.

**Upgrade path** (future): A `POST /tenants/{client_id}/upgrade-enterprise` SPI endpoint would retroactively create a Keycloak Organization, link existing members via Keycloak APIs, and configure external IdPs. No schema migration needed -- all membership data already exists in the local `tenants_users` table and Keycloak groups.

## Technical Implementation

### Key Files

| File | Purpose |
|------|---------|
| `crates/services/src/tenants/tenant_objs.rs` | `Tenant` struct, `AppStatus` enum, `DeploymentMode` enum |
| `crates/services/src/tenants/tenant_entity.rs` | `tenants` table SeaORM entity, `TenantRow` struct |
| `crates/services/src/tenants/tenant_user_entity.rs` | `tenants_users` table SeaORM entity (composite PK) |
| `crates/services/src/tenants/tenant_repository.rs` | `TenantRepository` trait + `DefaultDbService` impl |
| `crates/services/src/tenants/tenant_service.rs` | `TenantService` trait + `DefaultTenantService` impl (thin delegation) |
| `crates/services/src/tenants/auth_scoped.rs` | `AuthScopedTenantService` (user_id injection) |
| `crates/services/src/tenants/kc_types.rs` | `CreateTenantRequest`, `KcCreateTenantResponse` |
| `crates/services/src/tenants/error.rs` | `TenantError` enum |
| `crates/services/src/users/auth_scoped.rs` | `AuthScopedUserService` (dual-writes: role assignment + membership) |
| `crates/services/src/auth/auth_service.rs` | `AuthService.create_tenant()` (SPI proxy via reqwest) |
| `crates/routes_app/src/tenants/routes_tenants.rs` | `tenants_index`, `tenants_create`, `tenants_activate` handlers |
| `crates/routes_app/src/tenants/routes_dashboard_auth.rs` | `dashboard_auth_initiate`, `dashboard_auth_callback` handlers |
| `crates/routes_app/src/tenants/tenant_api_schemas.rs` | `TenantListItem`, `TenantListResponse`, `CreateTenantRequest`, `CreateTenantResponse` |
| `crates/routes_app/src/tenants/error.rs` | `DashboardAuthRouteError` enum |
| `crates/routes_app/src/setup/routes_setup.rs` | `setup_create` (standalone tenant creation), `setup_show` (/info) |
| `crates/routes_app/src/auth/routes_auth.rs` | `auth_callback` (calls `set_tenant_ready` during ResourceAdmin transition) |
| `crates/services/src/session_keys.rs` | `SESSION_KEY_ACTIVE_CLIENT_ID`, `access_token_key()`, `DASHBOARD_ACCESS_TOKEN_KEY` |

### Refactoring History

1. **Pre-MT (Stage 1)**: Single `tenants` table, no `tenants_users`. `get_standalone_app()` everywhere. Flat session keys. `update_tenant_status` + `update_tenant_created_by` as separate non-transactional calls.

2. **Stage 2 M2 (Backend)**: Added `tenants_users` table. SPI proxy methods on `AuthService` (`list_tenants`, `create_tenant`). Dashboard auth routes. `ensure_valid_dashboard_token()` helper. `/info` called SPI `list_tenants()` remotely on every request.

3. **Architecture refactor (Phase A)**: Removed `list_tenants` from `AuthService`. Deleted `ensure_valid_dashboard_token()`. `/info` and `/tenants` became fully local DB-based (no SPI calls at runtime). `MultiTenantSession` AuthContext variant encodes deployment mode. Middleware validates/refreshes dashboard tokens. `AuthScopedTenantService` with `list_my_tenants()`, `has_memberships()`.

4. **Atomic tenant creation refactor**: Extracted `create_tenant_impl` shared between `create_tenant` and `create_tenant_test`. `set_tenant_ready` replaced `update_tenant_status` + `update_tenant_created_by` + separate `upsert_tenant_user` with a single transactional method. Removed dead methods: `update_tenant_status`, `update_tenant_created_by`, `update_status_by_id`, `set_client_ready`.

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Title | Status |
|----|-------|--------|
| D29 | SPI is source of truth for login-able clients | Implemented |
| D30 | Auto-redirect for single client | Implemented |
| D36 | Tenant created Ready in MT mode | Implemented |
| D37 | `created_by` column on tenants | Implemented |
| D41 | Same path for GET/POST /tenants | Implemented |
| D50 | GET /tenants enriched with session metadata | Implemented |
| D51 | Tenant row pre-exists before resource OAuth | Implemented |
| D52 | Accept orphans on SPI/local mismatch | Implemented |
| D55 | Instant tenant switch via activate | Implemented |
| D57 | Redirect URIs from backend config | Implemented |
| D60 | Tenant API: name + description only | Implemented |
| D64 | SPI proxy errors -> OpenAI-compatible format | Implemented |
| D65 | One-tenant-per-user hard limit | Implemented |
| D66 | `created_by` is Keycloak user ID | Implemented |
| D69 | Same tenant schema for both modes | Implemented |
| D79 | No setup wizard for multi-tenant | Implemented |
| D82 | Client naming: bodhi-tenant-{UUID} | Implemented |
| D84 | Keycloak groups = role source of truth | Implemented |
| D86 | Reqwest in AuthService for SPI proxy | Implemented |
| D88 | Redirect URI reconstructed from config | Implemented |
| D91 | Single Keycloak realm | Implemented |
| D93 | Resource callback URI only for tenants | Implemented |
| D95 | No role column in membership table | Implemented |
| D96 | No role in tenant dropdown for MVP | Implemented |
| D101 | MT endpoints error in standalone | Implemented |
| D103 | `forward_spi_request` uses owned String params | Implemented |
| D104 | Email-based user addition rejected (enumeration risk) | Accepted |

## Known Gaps & TECHDEBT

1. **No reconciliation mechanism**: If `tenants_users` and Keycloak groups diverge (e.g., role assignment happens outside BodhiApp via Keycloak admin console), there is no sync/reconciliation process. The user can still authenticate via Keycloak, but `GET /tenants` may not show the membership until a local write occurs.

2. **D52 orphan cleanup is manual**: When SPI succeeds but local tenant creation fails, the Keycloak client becomes an orphan. No automated cleanup or retry mechanism exists.

3. **D65 one-tenant-per-user limit is rigid**: The one-resource-client-per-user constraint is enforced at the app repository level in `tenant_repository.create_tenant()`. No configuration to expand this.

4. **list_user_tenants uses two-step query**: The repository fetches tenant_ids from `tenants_users`, then fetches tenants by those IDs. A JOIN query would be more efficient but requires SeaORM relation-based querying or raw SQL.

5. **No tenant deletion endpoint**: No `DELETE /tenants/{client_id}` API endpoint. The `delete_tenant` repository method exists but is not exposed.

6. **Tenant name/description not updatable**: No `PUT /tenants/{client_id}` endpoint for updating tenant metadata.

7. **`tenants_create` handler orchestration**: The handler directly orchestrates SPI call + local creation rather than centralizing this in `AuthScopedTenantService.create_tenant()`. This means handler-level orchestration logic that could be centralized.

8. **Service construction not deployment-aware**: `AppServiceBuilder` initializes all services (including `InferenceService`, `HubService`) in multi-tenant mode where local LLM features are unused.
