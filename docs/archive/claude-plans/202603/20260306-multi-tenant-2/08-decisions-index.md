# Decisions Index — Multi-Tenant Stage 2

## Overview

This is the canonical decision reference for the BodhiApp multi-tenant implementation. Decisions are numbered D1-D106 across two stages:

- **D1-D20**: Stage 1 — foundational schema, deployment modes, tenant isolation strategy (source: `20260303-multi-tenant/decisions.md`)
- **D21-D28**: Middleware refactor interview decisions (source: `decisions.md`)
- **D29-D51**: Multi-tenant login flow design decisions (source: `multi-tenant-flow-ctx.md` Decision Index)
- **D52-D91**: Multi-tenant login flow deep-dive decisions (source: `decisions.md`)
- **D92-D97**: SPI implementation reconciliation decisions (source: `decisions.md`)
- **D98-D102**: M2/M3 backend implementation decisions (source: `decisions.md`)
- **D103-D106**: M4 integration test infrastructure decisions (source: `decisions.md`)

Gaps in numbering (D34, D40, D46) represent decisions that were superseded or renumbered during the planning process.

**Status legend**: Accepted = active and implemented or planned. Superseded = replaced by a later decision. Deferred = explicitly postponed.

**Related docs**: Each domain doc contains a focused decision table referencing IDs from this index:
- [01-deployment-modes-and-status.md](01-deployment-modes-and-status.md#decisions) -- Deployment & configuration decisions
- [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#decisions) -- Auth & session decisions
- [03-tenant-management-and-spi.md](03-tenant-management-and-spi.md#decisions) -- Tenant management & SPI decisions
- [04-database-migrations-entities.md](04-database-migrations-entities.md#decisions) -- Database & migration decisions
- [05-data-isolation-rls.md](05-data-isolation-rls.md#decisions) -- RLS decisions
- [06-frontend-ui.md](06-frontend-ui.md#decisions) -- Frontend decisions
- [07-testing-infrastructure.md](07-testing-infrastructure.md#decisions) -- Testing decisions

---

## Decision Summary by Domain

### Deployment & Configuration

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D5 | `BODHI_DEPLOYMENT` setting values | Accepted | Values: `standalone` (default) and `multi_tenant`. Multi-tenant disables local LLM, enables multi-tenant features. |
| D6 | `BODHI_MULTITENANT_CLIENT_ID` env var | Accepted | Configurable env var for the dashboard/multi-tenant client. Required when `BODHI_DEPLOYMENT=multi_tenant`, error if set when standalone. |
| D10 | Conditional route registration + feature flag for LLM | Accepted | Skip LLM route groups when `is_multi_tenant()`. Service-level guards as backup. |
| D22 | Ignore `BODHI_MULTITENANT_CLIENT_ID` in middleware | Accepted | Used for bootstrap/provisioning only, not runtime auth. Middleware resolves tenant from token claims. |
| D23 | Unified code path (no deployment mode branching in middleware) | Accepted | Always resolve tenant from incoming token claims. Works identically for standalone (1 tenant) and multi-tenant (N tenants). |
| D58 | Deployment mode injection into handlers | Accepted (Implemented) | Resolved via `AuthContext` variant type encoding (`Session` vs `MultiTenantSession`) + `deployment` field on `Anonymous`. Handlers use `auth_context.is_multi_tenant()` (sync) instead of `settings.is_multi_tenant().await` (async). |
| D67 | `/info` returns deployment mode | Accepted (supersedes D34) | Adds `deployment: "standalone" | "multi_tenant"` to `/info` response. Frontend uses it for feature visibility. |
| D76 | Sequential development: SPI -> Backend -> Frontend | Accepted | SPI first (deploy to dev Keycloak), then BodhiApp backend, then frontend. |
| D89 | Four milestones | Accepted | M1=SPI (done), M2=Backend (done), M3=Frontend+Backend prerequisites (done), M4=Integration tests (done). Updated from original two milestones. |
| D90 | ~~`BODHI_APP_URL` env var~~ | Superseded | Originally planned new env var. Actually uses `settings.public_server_url()` computed from existing `BODHI_PUBLIC_SCHEME/HOST/PORT`. |
| D91 | Single Keycloak realm | Accepted | All tenants share one Keycloak realm. Single `BODHI_AUTH_ISSUER`. SSO across all clients. Multi-realm not in scope. |
| D98 | `BODHI_MULTITENANT_CLIENT_SECRET` from env only | Accepted | Client secret reads from env var only (via `get_env()`), never from database. Client ID can be in DB or env. |
| D101 | Multi-tenant endpoints return error in standalone | Accepted | Dashboard auth and tenant management endpoints return `DashboardAuthRouteError::NotMultiTenant` (`ErrorType::InvalidAppState`) when `is_multi_tenant()` is false. |

### Auth & Sessions

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D8 | `tenant_id` in AuthContext | Accepted | `tenant_id: Option<String>` on all AuthContext variants. Middleware resolves during auth via JWT claims. |
| D13 | Session-based tenant routing, no slug column | Accepted | Active tenant stored in session cookie, not URL-path routing. No slug column on tenants. |
| D21 | JWT-only tenant resolution | Accepted | Tenant resolved from JWT `azp` claim. No separate tenant cookie. Cookie-based tenant switching deferred. |
| D24 | Trust JWT `aud` for token exchange | Accepted | After issuer check, JWT `aud` is trustworthy for determining tenant's `client_secret` for RFC 8693 exchange. |
| D25 | Use expired JWT claims for refresh | Accepted | Expired JWTs still cryptographically signed. Claims (`azp`, `sub`) trustworthy for tenant resolution during token refresh. |
| D26 | Anonymous context = None/None | Accepted | `Anonymous { client_id: None, tenant_id: None }`. Optional auth middleware never fails/blocks requests. |
| D27 | Access request flow unchanged | Accepted | Once tenant is resolved from JWT `aud`, access_request lookup by `(tenant_id, scope)` is correct. No changes needed. |
| D31 | Standard OAuth2 auth code flow for resource-client login | Accepted | Not token exchange. Standard authorization code + PKCE flow. |
| D32 | Keep all tokens in session, namespaced by client_id | Accepted | Dashboard + resource-client tokens coexist. Pattern: `{client_id}:access_token`, `{client_id}:refresh_token`. |
| D33 | Two-step middleware token lookup | Accepted | Read `active_client_id` -> read `{active_client_id}:access_token`. Works for both deployment modes. |
| D35 | Separate dashboard auth endpoints | Accepted | New `/auth/dashboard/initiate` and `/auth/dashboard/callback`. Existing `/auth/initiate` reused with client_id in body. |
| D47 | Dashboard callback redirects to `/ui/login` | Accepted | After dashboard OAuth, redirect to login page for tenant selection. |
| D53 | Transparent dashboard token refresh | Accepted | Before proxying to SPI, check dashboard token expiry. Auto-refresh if expired. Redirect to re-login if refresh fails. |
| D54 | `/info` behind `optional_auth_middleware` | Accepted (Implemented) | `/info` is in the `optional_auth` route group behind `optional_auth_middleware`. `AuthScope` populates `AuthContext` for authenticated users, falls back to `Anonymous` for unauthenticated. |
| D55 | Instant tenant switch endpoint | Accepted | `POST /bodhi/v1/tenants/{client_id}/activate`. Validates cached token exists and is not expired. Sets `active_client_id`. |
| D56 | Breaking session key migration | Accepted | Switch to namespaced keys immediately. Existing sessions treated as unauthenticated. Users re-login once. |
| D63 | Logout scope semantics | Deferred | Whether to support resource-client-only logout vs full logout — deferred to implementation. |
| D68 | `client_id` always required in `POST /auth/initiate` | Accepted | Unified code path. Standalone frontend sends `client_id` from `/info`. Multi-tenant sends selected tenant's `client_id`. |
| D70 | `/info` includes `client_id` | Accepted | Response includes the active/standalone tenant's `client_id` so frontend can pass it to `POST /auth/initiate`. |
| D74 | Auth callback uses tenant's `client_id` for session namespacing | Accepted | Uses `instance.client_id` from tenant lookup (via `auth_client_id` in session), NOT JWT `azp`. |
| D80 | Shared parameterized code exchange utility | Accepted (not yet implemented) | Single function parameterized by client credentials and session key prefix. Currently duplicated between auth and dashboard auth handlers. |
| D81 | `/user/info` dashboard state | Accepted | `UserInfoEnvelope` wraps `UserResponse` with `#[serde(flatten)]`. Adds `has_dashboard_session: bool` (skip_serializing_if when false). |
| D99 | Dashboard token expiry check via SystemTime | Superseded (by D105) | Originally used `SystemTime::now()`. Superseded by D105 which uses `TimeService` for deterministic testing. |
| D100 | UserInfoEnvelope with serde flatten | Accepted | `/user/info` wraps `UserResponse` in `UserInfoEnvelope`. `has_dashboard_session` only serialized when true. Backward-compatible. |
| D102 | D68 implemented — auth_initiate unified | Accepted | `POST /auth/initiate` requires `AuthInitiateRequest { client_id: String }`. Uses `get_tenant_by_client_id()` instead of `get_standalone_app()`. |
| D105 | `ensure_valid_dashboard_token` uses TimeService | Accepted (supersedes D99) | Accepts `time_service: &dyn TimeService` parameter. Uses `time_service.utc_now().timestamp()` for deterministic testing. |
| — | Email-based user addition rejected | Accepted | In a shared Keycloak realm, email lookup exposes whether a user exists (enumeration risk). SPI `POST /resources/assign-role` accepts only `user_id` (UUID). |
| — | Shareable invite link pattern | Accepted | Admin shares `{url}/ui/login/?invite={client_id}`. User authenticates, gets redirected to access-request if no role. Replaces "add user by email". |
| — | `sessionStorage` for invite state | Accepted | `login_to_tenant` stored in sessionStorage to survive OAuth redirects. Cleared after consumption. |

### Tenant Management

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D1 | Rename `apps` table to `tenants` | Accepted | Domain terminology clarity. `client_id` becomes unique index. ULID as PK. |
| D2 | ULID for tenant ID format | Accepted | Codebase convention. Every other table uses ULID. |
| D11 | Keep `AppStatus` enum name | Accepted | Widely used across codebase. Values still make semantic sense for tenants. |
| D15 | External provisioning for multi-tenant tenants | Superseded (by D41/D60) | Originally deferred to scripts. Now handled by `POST /bodhi/v1/tenants` API. |
| D18 | Generate tenant_id during setup for standalone | Accepted | `setup_create()` generates tenant row with ULID id during setup flow. |
| D29 | SPI is source of truth for user's client list | Accepted | Keycloak SPI, not BodhiApp tenants table, is authoritative for which clients a user can access. |
| D30 | Auto-redirect for single client, dropdown for multiple | Accepted | 0 clients -> registration, 1 client -> auto-redirect, N clients -> dropdown selector. |
| D36 | Tenant created Ready in multi-tenant mode | Accepted | Skip ResourceAdmin dance. Tenant status set to Ready immediately after creation. |
| D37 | `created_by` column on tenants | Accepted | Nullable VARCHAR(255). Keycloak user ID (sub claim). Standalone: set during auth_callback ResourceAdmin->Ready. Multi-tenant: set during POST /tenants. |
| D38 | No mandatory setup wizard for multi-tenant | Accepted | Tenant Ready immediately. API key configuration accessible from settings, not gated. |
| D41 | `POST /bodhi/v1/tenants` — standard REST | Accepted | Same path for GET (list) and POST (create). |
| D51 | Tenant row pre-exists before resource-client callback | Accepted | Tenant row created during `POST /bodhi/v1/tenants`, before OAuth callback. |
| D52 | Accept orphans on tenant creation failure | Accepted | If SPI succeeds but local tenant creation fails, log for manual cleanup. No compensating delete. |
| D60 | Tenant registration API — name + description only | Accepted | User sends `{ name, description }`. Backend adds `redirect_uris` internally from config. |
| D61 | Tenant list enrichment with BodhiApp data | Accepted | For each SPI `client_id`, look up local tenants table for `tenant_id`, `status`, `created_at`. |
| D65 | One-client-per-user hard limit | Accepted | Hard limit of 1 resource-client per user. Enforced at app level in `tenant_repository.create_tenant()` — checks `list_tenants_by_creator(user_id)` before creation. Returns `TenantError::UserAlreadyHasTenant` (400 Bad Request). Enforcement only applies when `created_by` is `Some` (multi-tenant mode); standalone (created_by=None) always succeeds. |
| D66 | `created_by` is Keycloak user ID | Accepted | JWT `sub` claim. BodhiApp has no users table — Keycloak is the identity source. |
| D69 | Tenant row — same schema, encrypted secret | Accepted | Same schema for standalone and multi-tenant. `encrypted_client_secret` on all rows. |
| D78 | SPI is source of truth for login-able clients | Accepted | `GET /tenants` from SPI is authoritative. `created_by` is for audit and one-per-user constraint only. |
| D79 | Tenant Ready immediately, no setup wizard | Accepted | After registration + OAuth login, user goes directly to app. Same as D36/D38. |
| D82 | Client naming convention | Accepted | Multi-tenant: `bodhi-tenant-<UUID>`. Standalone: `bodhi-resource-<UUID>`. |
| D83 | Both client ID renames in this plan | Accepted | New standalone: `bodhi-resource-<UUID>`. New multi-tenant: `bodhi-tenant-<UUID>`. Existing clients keep old IDs. |
| D87 | Keep test prefix unchanged | Accepted | Test env uses `test-resource-<UUID>`. Only production prefix renamed to `bodhi-resource-<UUID>`. |
| D92 | `has_valid_token` renamed to `logged_in` | Accepted | Frontend-friendly naming for session-based enrichment in GET /tenants response. |
| — | One-per-user enforcement at app level | Accepted | App-level check in `create_tenant()` via `list_tenants_by_creator`. SPI no longer enforces (no tracking tables). |
| — | Cleanup discovery is local | Accepted | `dev_tenants_cleanup_handler` discovers tenants via local DB (`list_tenants_by_creator`), sends explicit `client_ids` to SPI. |
| — | Cleanup `[do-not-delete]` filter | Accepted | Tenants with names starting with `[do-not-delete]` are excluded from cleanup. |
| — | Optimistic local deletion on cleanup | Accepted | After sending client_ids to SPI, delete all sent client_ids locally regardless of SPI response. |

### Database & Migrations

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D3 | `tenant_id` on every data table | Accepted | Industry standard. RLS policies per table without JOINs. Defense-in-depth. |
| D4 | Unified schema for both deployment modes | Accepted | Standalone and multi-tenant share same schema. Standalone has one tenant row. |
| D9 | Settings table stays global permanently | Accepted | Create separate `tenant_settings` table if per-tenant settings needed later. |
| D12 | Always store encrypted_client_secret | Accepted | Every tenant row stores `encrypted_client_secret` for server-side token operations. |
| D14 | Modify existing CREATE TABLE migrations | Accepted | No production deployments exist. Clean schema from day one. |
| D19 | Foundation-up phasing strategy | Accepted | Phase 1 (tenants) -> Phase 2 (AuthContext) -> Phase 3 (all tables) -> Phase 4 (service scoping) -> Phase 5 (feature gating) -> Phase 6 (RLS). |
| D20 | Remove seed logic | Accepted | Seeding was temporary. Toolset configs created on-demand by tenant admins. |
| D94 | SPI operates on Keycloak native entities | Accepted | SPI operates directly on Keycloak's native entities (clients, groups, service accounts). No SPI-specific tracking tables. |
| D104 | DefaultDbService uses builder pattern for env_type | Accepted | `.with_env_type()` builder method instead of changing constructor signature. Avoids breaking existing callers. |

### Data Isolation (RLS)

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D7 | App-layer filtering + PG RLS defense-in-depth | Accepted | Auth-scoped services add `tenant_id` filter (primary). PG RLS as defense-in-depth layer. |

### Keycloak SPI

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D42 | SPI endpoints: `GET/POST /realms/{realm}/bodhi/tenants` | Accepted | Separate from `/resources`. No trailing slash. |
| D43 | Reuse `/resources/*` for role management | Accepted | Centralized role management via Keycloak groups for assign-role, remove-user, make-resource-admin. |
| D44 | 4-level role hierarchy | Accepted | `resource_admin > resource_manager > resource_power_user > resource_user` in Keycloak groups. |
| D45 | `/make-resource-admin` manages Keycloak entities | Accepted | Assigns resource-admin group membership in Keycloak. No SPI-side tracking tables. |
| D57 | Redirect URIs passed from BodhiApp backend | Accepted | Backend constructs from config (`public_server_url()`), passes in SPI request. SPI trusts them. |
| D59 | SPI uses Keycloak's native entity model | Accepted | SPI operates on Keycloak's native entities (clients, groups, service accounts) via Keycloak's admin APIs and session model. |
| D62 | Client type validation via Keycloak attributes | Accepted | `bodhi.client_type=multi-tenant` on dashboard clients. SPI reads attribute to validate `azp`. |
| D64 | SPI proxy errors -> 500 | Accepted | Return 500 Internal Server Error with OpenAI-compatible error body when SPI is unreachable. |
| D71 | SPI configuration via Liquibase changelog | Accepted | SPI configuration managed via `META-INF/bodhi-changelog.xml`. Keycloak auto-applies on startup. |
| D72 | SPI deployed to dev env first | Accepted | Changes deployed to `main-id.getbodhi.app` dev Keycloak for real testing. |
| D75 | No trailing slash on SPI endpoints | Accepted | Consistent with BodhiApp convention. |
| D84 | Role storage — groups + local table (dual system) | Accepted | Keycloak groups for JWT claims (`resource_access`). Local `tenants_users` table for fast membership queries. No SPI-specific tracking tables. |
| D86 | Use existing reqwest in AuthService for SPI proxy | Accepted | Extend AuthService with SPI proxy methods. No new HTTP client. |
| D88 | Redirect URI reconstructed from config | Accepted | Not stored on tenant row. Reconstructed from `public_server_url()` each time. |
| D93 | Redirect URIs for tenant resource-clients | Accepted | Resource callback only: `{public_server_url()}/ui/auth/callback`. Dashboard clients configured separately by admin. |
| D95 | D46 reversed — Keycloak groups are sole role source of truth | Accepted (supersedes D46) | Keycloak groups are sole role source of truth. Local `tenants_users` table tracks membership for fast queries. No SPI-specific tracking tables. |
| D96 | No role in tenant dropdown for MVP | Accepted | SPI `GET /tenants` doesn't return role. Role visible after login from JWT `resource_access` claims. |
| D97 | POST /resources creates resource client in Keycloak | Accepted | Standalone `POST /resources` creates resource client in Keycloak (groups, service account). No SPI-side tracking tables. |

### Frontend

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D16 | Backend-only scope (Stage 1) | Accepted | Stage 1 covered schema, services, middleware, routes only. Frontend tracked separately. |
| D39 | `/ui/login` page reused for tenant selection and switching | Accepted | Login page serves dual purpose: standalone login and multi-tenant tenant selection/switching. |
| D48 | Registration UI at `/ui/setup/tenants/` | Accepted | Calls `POST /bodhi/v1/tenants`. Form: name + description. |
| D49 | `/user/info` extended for dashboard session detection | Accepted | Shape finalized in D81/D100: `has_dashboard_session: bool`. |
| D50 | `GET /tenants` enriched with `is_active`, `logged_in` | Accepted | Per-client session metadata for frontend rendering. |
| D77 | Separate frontend callback routes | Accepted | Dashboard: `/ui/auth/dashboard/callback`. Resource-client: `/ui/auth/callback`. Separate redirect_uris in Keycloak. |
| — | Invite link UI on users page (multi-tenant only) | Accepted | Read-only URL + copy button. Standalone mode has no invite link. |
| — | `url` field in AppInfo response | Accepted | `settings.public_server_url()` exposed via `/info` for frontend invite URL construction. |
| — | `role:None` → request-access redirect | Accepted | Login page redirects to `/ui/request-access` when user has active session but no role (invited user flow). |

### Testing

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D73 | All tests use real Keycloak | Accepted | No mocking of SPI HTTP calls. CI must have access to dev Keycloak. |
| D85 | SPI testing follows existing patterns | Accepted | Unit tests with mocked `KeycloakSession`. Integration tests with Testcontainers. |
| D103 | `forward_spi_request` uses owned String params | Accepted | mockall's `#[automock]` has lifetime issues with `&str` in async trait methods. Owned params avoid it. |
| D106 | Dev-only test endpoints | Accepted | `POST /dev/clients/{client_id}/dag` and `DELETE /dev/tenants/cleanup`. Not exposed in production builds. |

### Deferred / Future Work

| ID | Title | Status | Description |
|----|-------|--------|-------------|
| D17 | Defer cache externalization | Deferred | Keep in-memory MokaCacheService. Redis deferred until true horizontal scaling needed. |
| D21 | Cookie-based tenant switching | Deferred | JWT-only for now. UI-driven tenant switching via cookie is a future milestone. |
| D28 | Middleware-only scope for auth refactor | Accepted (scope limiter) | Only middleware + token_service refactored. Other `get_standalone_app()` usages deferred to follow-up. |
| D58 | Deployment mode injection into handlers | Accepted (Implemented) | Resolved via `AuthContext` variant type encoding. See Deployment & Configuration section. |
| D63 | Logout scope semantics | Deferred | Separate resource-client logout vs full logout deferred to when feature is implemented. |
| D65 | One-client-per-user expansion | Deferred | Hard limit of 1 for now. Expansion deferred. |
| D80 | Shared code exchange utility | Deferred (TECHDEBT) | Code duplicated between `routes_auth.rs` and `routes_dashboard_auth.rs`. Low priority. |
| — | Organization creation | Deferred | Keycloak Organizations deferred to enterprise upgrade path. Email-based user lookup may be safe within an organization (bounded context, not shared realm). See `decision-organization-feature-deferred.md`. |
| — | E2E/Playwright tests for multi-tenant | Deferred | Not covered in M1-M4. |
| — | Service construction by deployment mode | Deferred | Conditional route registration for deployment modes not yet implemented. |
| — | Navigation item visibility | Deferred | Hide LLM-specific items in multi-tenant mode. |
| — | CI pipeline for integration tests | Deferred | Needs real Keycloak + `.env.test` credentials in CI. |
| — | Multi-tenant-aware logout | Deferred | Currently `session.delete()` clears all tokens, not selective per tenant. |

---

## Decision Details

### D7: App-layer filtering + PG RLS defense-in-depth

The tenant isolation strategy uses two complementary layers. The primary layer is application-level filtering: auth-scoped services (e.g., `AuthScopedMcpService`, `AuthScopedTokenService`) add `tenant_id` to all queries, extending the existing `user_id` scoping to `(tenant_id, user_id)`. This works with both SQLite (standalone) and PostgreSQL (multi-tenant). The secondary layer is PostgreSQL Row-Level Security policies as defense-in-depth: even if the application layer has a bug, RLS policies at the database level prevent cross-tenant data access. SeaORM executes raw SQL (`SET LOCAL`) within transactions for RLS context. Mutating DB operations use `begin_tenant_txn(tenant_id)`.

### D23: Unified code path (no deployment mode branching in middleware)

A pivotal architectural decision: middleware always resolves tenant from incoming token claims, regardless of deployment mode. Standalone is simply multi-tenant with one tenant. This eliminates deployment-mode branching in the hot path. The only exception: when token-based tenant resolution fails (tenant not found in DB), standalone checks if any tenant exists (Setup status) while multi-tenant returns TenantNotFound. This pattern propagates through the entire codebase — route handlers, services, and data access all operate identically regardless of whether one or many tenants exist.

### D56: Breaking session key migration

Session keys migrated from flat (`access_token`, `refresh_token`) to namespaced (`{client_id}:access_token`, `{client_id}:refresh_token`) with no backwards compatibility layer. Existing sessions lack `active_client_id`, so middleware treats them as unauthenticated. Users perform a one-time re-login. This was safe because no production multi-tenant deployments exist, and the complexity of a temporary compatibility layer was not justified for a single re-login per standalone user.

### D67: `/info` returns deployment mode (supersedes D34)

D34 originally said "no deployment field in AppInfo." This was reversed because the frontend needs the deployment mode independently of status. For example, when status is `Ready` in multi-tenant mode, LLM-related features should be hidden. The `deployment` field drives feature visibility while `status` drives routing. This separation is critical for the frontend's conditional rendering strategy.

### D84: Role storage — groups + local table (dual system)

Keycloak groups remain for native token claims (`resource_access` in JWT). The local `tenants_users` table provides fast `GET /tenants` queries — determining which clients a user belongs to. Querying Keycloak's internal group membership tables directly has unclear performance characteristics and couples the query pattern to Keycloak's schema. The SPI manages Keycloak groups (the source of truth for roles), while the local `tenants_users` table provides a fast membership index for application queries.

### D90: BODHI_APP_URL superseded by public_server_url()

The original plan called for a new `BODHI_APP_URL` environment variable for constructing redirect URIs. During implementation, `settings.public_server_url()` was used instead, computed from the existing `BODHI_PUBLIC_SCHEME`, `BODHI_PUBLIC_HOST`, and `BODHI_PUBLIC_PORT` settings. This approach is RunPod-aware and handles all deployment scenarios without introducing a new env var. Redirect URIs are constructed as `{public_server_url()}/ui/auth/callback`.

### D94-D95: SPI implementation reconciliation

The SPI operates directly on Keycloak's native entities (clients, groups, service accounts) without maintaining its own tracking tables. This reverses D46 which planned for an SPI-managed table to be the sole source of truth for roles. Instead, Keycloak group membership is the sole role source of truth, with role data available in JWT claims after login. The local `tenants_users` table in the BodhiApp database provides a fast membership index for `GET /tenants` listings.

### D105: TimeService supersedes SystemTime for dashboard tokens (supersedes D99)

D99 chose `SystemTime::now()` for dashboard token expiry checks as a pragmatic shortcut. During M4 integration test implementation, deterministic time was required for testing. `ensure_valid_dashboard_token()` was updated to accept `time_service: &dyn TimeService` as a parameter, aligning with the codebase convention that all time operations go through `TimeService` for testability.

### Organization feature deferred

A detailed analysis concluded that Keycloak Organizations should NOT be created alongside tenants. The current design (SPI operating on Keycloak native entities + local `tenants_users` table + Keycloak groups) fully covers all launch requirements. Organizations add value only for enterprise features (external IdP linking, email domain auto-enrollment, managed membership). Keycloak's APIs make retroactive Organization creation trivial — there is no migration penalty from deferring. When enterprise features are requested, an additive `POST /tenants/{client_id}/upgrade-enterprise` endpoint can create the Organization and migrate existing users with no schema changes or downtime. Email-based user lookup, rejected for the shared-realm model due to enumeration risk, may be safe within an Organization (bounded context with known membership) — this is a natural addition to the enterprise upgrade path.

---

## Supersession Chain

| Superseded Decision | Superseded By | Summary |
|---------------------|---------------|---------|
| D15 (External provisioning) | D41/D60 | Tenant creation API implemented via `POST /bodhi/v1/tenants` |
| D34 (No deployment field in AppInfo) | D67 | `/info` now returns `deployment` field |
| D46 (Table is sole role source of truth) | D95 | Keycloak groups are sole role source; local `tenants_users` table tracks membership for fast queries |
| D99 (SystemTime for dashboard token expiry) | D105 | Uses `TimeService` for deterministic testing |
| D90 (`BODHI_APP_URL` env var) | Implementation | Uses existing `public_server_url()` from `BODHI_PUBLIC_SCHEME/HOST/PORT` |
