# Multi-Tenant Decisions

> **Created**: 2026-03-06
> **Updated**: 2026-03-08
> **Context**: Interviews during middleware refactor (D21-D28), multi-tenant login flow (D52-D91), SPI implementation reconciliation (D92-D97), and M4 integration test infrastructure (D103-D106)
> **Prior work**: `ai-docs/claude-plans/20260303-multi-tenant/decisions.md` (D1-D20)

---

## D21: JWT-only tenant resolution (defer cookie-based switching)

**Question**: D13 said "active tenant stored in cookie". Should middleware support tenant-switching via session cookie now?

**Decision**: JWT-only for now. Tenant resolved purely from JWT `azp` claim. No separate tenant cookie. Tenant switching deferred to future milestone when UI supports it.

**Rationale**: The JWT `azp` already identifies which tenant's OAuth client the user authenticated with. Adding a session-based override adds complexity without current need. Multi-tenant tenant-switching is a UI/UX feature that can be layered on later.

---

## D22: Ignore BODHI_MULTITENANT_CLIENT_ID in middleware

**Question**: What role does `BODHI_MULTITENANT_CLIENT_ID` play in the auth middleware?

**Decision**: Ignore it for now. The middleware resolves tenant from incoming tokens only:
- **Session**: JWT `azp` claim -> `get_tenant_by_client_id(azp)`
- **External app (3rd party JWT)**: JWT `aud` claim -> `get_tenant_by_client_id(aud)`
- **API token**: Split by `.`, get `client_id` from suffix -> `get_tenant_by_client_id(client_id)`

`BODHI_MULTITENANT_CLIENT_ID` is for bootstrap/provisioning, not runtime auth.

---

## D23: Unified code path (no deployment mode branching in middleware)

**Question**: Should middleware branch on `BODHI_DEPLOYMENT` (standalone vs multi)?

**Decision**: Unified code path. Always resolve tenant from incoming token claims. Works identically for standalone (1 tenant) and multi-tenant (N tenants).

**Standalone flow**:
- Setup/register flow is active when tenants table has 0 rows
- Once a single tenant row is created, setup flow is blocked
- All auth flows resolve tenant from token's client_id

**Multi-tenant flow**:
- Users login via a dashboard client_id, create a new tenant, receive client_id
- Connect to that tenant by logging against that client
- localStorage stores current tenant for future logins
- The "check db for any row" pattern is not applicable

**Exception**: When token-based tenant resolution fails (tenant not found in DB):
- Standalone: check if any tenant exists; if none -> Setup status
- Multi-tenant: return TenantNotFound error

---

## D24: Trust JWT `aud` for token exchange after issuer check

**Question**: Should `handle_external_client_token` trust `claims.aud` to determine which tenant's `client_secret` to use for RFC 8693 exchange?

**Decision**: Yes. If the JWT issuer matches our configured Keycloak (`auth_issuer()`), the `aud` claim is trustworthy. Standard OAuth2 behavior.

---

## D25: Use expired JWT claims for tenant resolution in refresh

**Question**: Is it safe to use an expired token's `azp` to look up tenant credentials for token refresh?

**Decision**: Yes. Expired JWTs are still cryptographically signed by Keycloak. Claims (`azp`, `sub`, etc.) are trustworthy. Only time-based validity has lapsed. Standard practice for token refresh flows. `extract_claims` already does raw decode without expiry check.

---

## D26: Anonymous context = None/None

**Question**: What should `AuthContext::Anonymous` contain when no auth token is present?

**Decision**: `Anonymous { client_id: None, tenant_id: None }`. The optional auth middleware:
- If session/JWT bearer/API token is present -> parse and inject as AuthContext (same resolution as mandatory)
- If none present -> inject `Anonymous { client_id: None, tenant_id: None }`
- Never fails/blocks the request (unlike mandatory middleware)

Routes using optional_auth_middleware must handle Anonymous with no tenant context.

---

## D27: Access request flow is straightforward

**Question**: Does the `AppAccessRequest` flow need auth-scoping changes for multi-tenant?

**Decision**: No. Once tenant is resolved from JWT `aud`, the access_request lookup by `(tenant_id, scope)` is correct. The `app_client_id` check ensures the requesting app matches. No additional changes needed. `AccessRequestService` remains a documented non-auth-scoped passthrough.

---

## D28: Middleware-only scope for this plan

**Question**: Should this plan cover all `get_standalone_app()` calls or just middleware?

**Decision**: Middleware + token_service only (4 calls in `auth_middleware.rs` and `token_service.rs`). Other `get_standalone_app()` usages in auth/setup/apps/dev routes are separate concerns for follow-up plans.

---

## D52: Accept orphans on tenant creation failure

**Question**: If `POST /bodhi/v1/tenants` succeeds at the SPI (Keycloak client created) but fails when creating the local BodhiApp tenant row, how should the orphaned Keycloak client be handled?

**Decision**: Accept orphans. Log the error at critical/warning level for manual cleanup via Keycloak admin console. No compensating delete or retry logic.

**Rationale**: Simplest implementation. Orphan Keycloak clients are harmless (no data, no access). Manual cleanup is rare and low-priority.

---

## D53: Transparent dashboard token refresh in SPI proxy

**Question**: Dashboard tokens expire. When `GET /bodhi/v1/tenants` (or any SPI proxy call) encounters an expired dashboard token, what should happen?

**Decision**: Transparent refresh. Before proxying to SPI, check dashboard token expiry. If expired, use `dashboard:refresh_token` to get a new dashboard token from Keycloak. Update session. If refresh fails, redirect to dashboard re-login.

**Rationale**: Same pattern as resource-client token refresh. Dashboard tokens have the same lifecycle. Poor UX if users are forced to re-login when a refresh token exists.

---

## D54: `/info` endpoint behind `optional_auth_middleware` — NOT IMPLEMENTED AS DECIDED

**Question**: `/info` currently runs without auth middleware. In multi-tenant mode, it needs session access to determine status (dashboard token, active_client_id). How should it access session state?

**Decision**: Move `/info` behind `optional_auth_middleware`. The middleware already handles the no-auth case gracefully (Anonymous). The handler checks AuthContext + reads session directly for dashboard token state.

**Actual implementation**: `/info` remains in `public_apis` (no auth middleware). `AuthScope` falls back to `Anonymous` without middleware. Session access works because the session layer is global. This means `client_id` in the response is always `None` in standalone mode (no AuthContext populated). Multi-tenant mode works correctly by reading session keys directly. See TECHDEBT.

---

## D55: Instant tenant switch endpoint

**Question**: What API endpoint should be used for instant tenant switching (updating `active_client_id` without OAuth re-login when the target tenant's token is still valid in session)?

**Decision**: `POST /bodhi/v1/tenants/{client_id}/activate`. Validates that `{client_id}:access_token` exists in session and is not expired. Sets `active_client_id`. Returns 200.

---

## D56: Breaking session key migration

**Question**: Standalone currently uses flat session keys (`access_token`, `refresh_token`). Migration to namespaced keys (`{client_id}:access_token`) — should existing sessions be gracefully migrated?

**Decision**: Breaking change. Switch to namespaced keys immediately. Existing sessions won't have `active_client_id`, so middleware treats them as unauthenticated. Users re-login once. Clean cut, no legacy code paths.

**Rationale**: No production multi-tenant deployments exist. Standalone users do a one-time re-login. Avoids temporary compatibility layer code.

---

## D57: Redirect URIs passed from BodhiApp backend

**Question**: Where should redirect_uris for newly created resource-clients come from?

**Decision**: BodhiApp backend constructs redirect_uris from its own `BODHI_APP_URL` config and passes them in the SPI `POST /tenants` request body. SPI trusts them.

---

## D58: Deployment mode injection — deferred

**Question**: How should `bodhi_deployment` be accessible in route handlers?

**Decision**: Deferred. Options under consideration: separate request extension, field on AuthContext, or method on AppService trait. Will decide when closer to implementation.

---

## D59: SPI uses JPA with custom entities

**Question**: How should the SPI access the new `bodhi_tenant_clients` and `bodhi_client_roles` tables?

**Decision**: JPA with custom `@Entity` classes. Use Keycloak's `EntityManager` (from `KeycloakSession`). Register entities via existing `BodhiJpaEntityProvider`. Tables created via Liquibase changelog.

---

## D60: Tenant registration API — name + description only

**Question**: Should `POST /bodhi/v1/tenants` accept redirect_uris from the user?

**Decision**: No. User sends `{ name, description }` only. BodhiApp backend adds `redirect_uris` internally (from `BODHI_APP_URL`). Users shouldn't need to know about OAuth redirect URIs.

---

## D61: Tenant list enrichment with BodhiApp data

**Question**: Should `GET /bodhi/v1/tenants` enrich the SPI response with BodhiApp tenant metadata?

**Decision**: Yes. For each `client_id` in the SPI response, look up BodhiApp's tenants table. Include `tenant_id`, `status`, `created_at`. If a `client_id` isn't in BodhiApp's DB, flag it as 'pending setup'.

---

## D62: Client type validation via Keycloak client attributes

**Question**: How should the SPI validate client types (resource, app, multi-tenant)?

**Decision**: Continue using the existing `bodhi.client_type` client attribute pattern. Set `bodhi_client_type=multi-tenant` on dashboard clients. SPI reads the attribute to validate `azp` on `/tenants` endpoints.

---

## D63: Logout scope semantics — deferred

**Question**: Should there be separate resource-client logout vs full logout?

**Decision**: Deferred to when the feature is implemented.

---

## D64: SPI proxy errors -> 500

**Question**: When the BodhiApp backend proxies to the SPI and the SPI is unreachable, what should happen?

**Decision**: Return 500 Internal Server Error with OpenAI-compatible error body. Frontend handles generically.

---

## D65: One-client-per-user — hard limit

**Question**: Should the one-client-per-user-per-dashboard restriction be configurable or expandable?

**Decision**: Hard limit of 1 for now. Sufficient for initial SaaS launch. Expansion deferred.

---

## D66: `created_by` is Keycloak user ID

**Question**: Should `created_by` on the tenants table be a Keycloak user ID (sub claim) or a BodhiApp-internal identifier?

**Decision**: Keycloak user ID (JWT `sub` claim). BodhiApp has no users table — Keycloak is the user identity source.

---

## D67: `/info` returns deployment mode (supersedes D34)

**Question**: Should the `/info` endpoint return the deployment mode?

**Decision**: Yes. Add `deployment: "standalone" | "multi_tenant"` to the `/info` response. Frontend uses it alongside status for conditional rendering. Status drives routing, deployment drives feature visibility.

**Supersedes D34**: D34 said "no deployment field in AppInfo." This is reversed — frontend needs deployment mode independently of status (e.g., hiding LLM features when status is Ready in multi-tenant mode).

---

## D68: `client_id` always required in `POST /auth/initiate`

**Question**: Should the existing auth_initiate endpoint accept an optional or required `client_id`?

**Decision**: Always required. Standalone frontend sends the single tenant's `client_id` (obtained from `/info` response, see D70). Multi-tenant frontend sends the selected tenant's `client_id`. Unified code path.

---

## D69: Tenant row — same schema, encrypted secret

**Question**: Should multi-tenant tenant rows have the same schema as standalone, including encrypted client_secret?

**Decision**: Yes. Same schema: `id` (ULID), `client_id`, `encrypted_client_secret`, `status=Ready`, `created_by`. Client secret encrypted same way as standalone. Tenant fully initialized before OAuth callback.

---

## D70: `/info` includes `client_id`

**Question**: Where does the standalone frontend get the `client_id` to pass to `POST /auth/initiate` (D68)?

**Decision**: `/info` response includes the active/standalone tenant's `client_id`. Frontend reads it from `/info`.

---

## D71: SPI table creation via Liquibase changelog

**Question**: How should the new SPI tables be created?

**Decision**: Liquibase changelog (added to existing `META-INF/bodhi-changelog.xml`). Keycloak automatically applies changesets on startup. Standard pattern for Keycloak extensions.

---

## D72: SPI deployed to dev env first

**Question**: How should the SPI be tested against the BodhiApp backend?

**Decision**: SPI changes deployed to `main-id.getbodhi.app` dev Keycloak environment first. BodhiApp backend tests run against the real dev Keycloak instance.

---

## D73: All tests use real Keycloak

**Question**: Should backend multi-tenant tests mock the SPI or use real Keycloak?

**Decision**: All tests use real Keycloak. No mocking of SPI HTTP calls. CI must have access to the dev Keycloak instance.

---

## D74: Auth callback uses tenant's `client_id` for session namespacing

**Question**: How does `POST /auth/callback` know which `client_id` to use for namespaced session keys?

**Original decision**: After code exchange, decode the returned access token's `azp` claim.

**Actual implementation**: Uses `instance.client_id` from the tenant lookup (via `auth_client_id` stored in session during `auth_initiate`). The tenant's `client_id` is used directly for namespacing session keys (`{client_id}:access_token`, `{client_id}:refresh_token`). JWT `azp` is not used for this purpose.

---

## D75: No trailing slash on SPI endpoints

**Question**: Should SPI endpoints use trailing slashes?

**Decision**: No trailing slash. Both `GET /realms/{realm}/bodhi/tenants` and `POST /realms/{realm}/bodhi/tenants` — consistent, no trailing slash. Matches BodhiApp convention.

---

## D76: Sequential development: SPI -> Backend -> Frontend

**Question**: Should the three codebases be developed in parallel or sequentially?

**Decision**: Sequential. SPI first (deploy to dev Keycloak), then BodhiApp backend (against real SPI), then frontend.

---

## D77: Separate frontend callback routes

**Question**: How does the frontend callback page distinguish between dashboard and resource-client OAuth flows?

**Decision**: Separate routes. Dashboard OAuth redirects to `/ui/auth/dashboard/callback`, resource-client to `/ui/auth/callback`. Two frontend pages, each calls the appropriate backend endpoint. Requires different `redirect_uris` registered in Keycloak for each client type.

---

## D78: SPI is source of truth for user's login-able clients

**Question**: What happens when a user is invited to a tenant (via `/resources/assign-role`) that they didn't create?

**Decision**: SPI `GET /tenants` is the source of truth for which clients a user can log into. BodhiApp's tenants table has rows for all resource-clients (single SaaS instance — all created via `POST /tenants` on the same BodhiApp). `created_by` is for audit and one-per-user constraint only, not for access control.

---

## D79: Tenant created Ready immediately, no setup wizard

**Question**: What triggers the multi-tenant setup wizard after tenant creation?

**Decision**: Tenant created with status `Ready` immediately (D36). No setup wizard. After registration + OAuth login, the return URL takes the user to the app. API key configuration is accessible from settings/nav, not a mandatory gated flow.

---

## D80: Shared parameterized code exchange utility

**Question**: Should dashboard and resource-client callback handlers share code exchange logic?

**Decision**: Yes. Single internal function parameterized by client credentials and session key prefix. Dashboard callback calls with `("dashboard:", dashboard_creds)`. Resource callback calls with `("{client_id}:", tenant_creds)`. DRY, single code path.

---

## D81: `/user/info` dashboard state — IMPLEMENTED

**Question**: What exactly should `/user/info` return when a dashboard token exists?

**Decision**: `UserInfoEnvelope` wraps `UserResponse` using `#[serde(flatten)]`. Adds `has_dashboard_session: bool` field, serialized only when `true` (via `skip_serializing_if`). Backend reads `dashboard:access_token` from session independently of `AuthContext`. Backward-compatible — existing clients see the same response when no dashboard session exists.

---

## D82: Client naming convention

**Question**: What naming convention should SPI-generated client IDs follow?

**Decision**: Multi-tenant resource-clients: `bodhi-tenant-<UUID>`. Existing standalone resource-clients renamed from `resource-<UUID>` to `bodhi-resource-<UUID>`.

---

## D83: Both client ID renames in this plan

**Question**: Should the `resource-<UUID>` to `bodhi-resource-<UUID>` rename be part of this plan?

**Decision**: Yes, both renames in this plan. New standalone registrations generate `bodhi-resource-<UUID>`. New multi-tenant registrations generate `bodhi-tenant-<UUID>`. Existing deployed clients keep their old IDs (no migration).

---

## D84: Role storage — groups + table (dual system)

**Question**: Should the SPI migrate away from Keycloak group-based role assignment?

**Decision**: Keep both. Groups remain for Keycloak-native token claims (`resource_access` in JWT). `bodhi_client_roles` table added for fast querying of user's client memberships (the `GET /tenants` listing use case). Centralized role assignment function writes to both.

**Rationale**: The table is specifically to enable fast `GET /tenants` queries. Querying Keycloak's group membership tables directly has unclear performance characteristics, and this endpoint will be called frequently.

---

## D85: SPI testing follows existing patterns

**Question**: How should the new SPI endpoints be tested?

**Decision**: Follow existing patterns. Unit tests with mocked `KeycloakSession`. Integration tests with Testcontainers (real Keycloak). Same structure as existing `ResourceClientRegistrationTest`.

---

## D86: Use existing reqwest in AuthService for SPI proxy

**Question**: How should the BodhiApp backend make HTTP calls to the SPI?

**Decision**: Use the existing `reqwest` client in `AuthService`. AuthService already makes HTTP calls to Keycloak (token endpoint, userinfo). Extend it with SPI proxy methods.

---

## D87: Keep test prefix unchanged

**Question**: Should the test prefix `test-resource-<UUID>` be renamed to match the new convention?

**Decision**: Keep `test-resource-<UUID>` for test/dev environment. Only rename the production prefix to `bodhi-resource-<UUID>`. Avoids churn in test fixtures.

---

## D88: Redirect URI reconstructed from config

**Question**: Should redirect_uris be stored on the tenant row?

**Decision**: No storage. Reconstruct from `BODHI_APP_URL` config each time it's needed. If the deployment URL changes, it automatically picks up the new value.

---

## D89: Four milestones (updated from original two)

**Question**: How should the plan be structured for deployment?

**Original decision**: Two milestones: M1 = SPI, M2 = Backend + Frontend.

**Updated**: Four milestones as implemented: M1 = SPI (done), M2 = Backend (done), M3 = Frontend + Backend prerequisites (done), M4 = Integration tests (done).

---

## D90: ~~`BODHI_APP_URL` env var~~ — SUPERSEDED

**Question**: How does the BodhiApp backend know its own external URL for constructing redirect_uris?

**Original decision**: New `BODHI_APP_URL` env var.

**Actual implementation**: No `BODHI_APP_URL` env var was created. Instead, `settings.public_server_url()` is used, computed from existing `BODHI_PUBLIC_SCHEME`, `BODHI_PUBLIC_HOST`, `BODHI_PUBLIC_PORT` settings. This is RunPod-aware and handles all deployment scenarios. Redirect URIs are `{public_server_url()}/ui/auth/callback`.

---

## D91: Single Keycloak realm — confirmed

**Question**: Do all tenants share one Keycloak realm, or could different tenants be in different realms?

**Decision**: Single realm confirmed. All tenants share one Keycloak realm. Single `BODHI_AUTH_ISSUER`. SSO works across all clients in the realm. Multi-realm is not in scope.

---

## D92: `has_valid_token` renamed to `logged_in`

**Question**: What should the session-based enrichment field be called in the GET /tenants response?

**Decision**: Rename `has_valid_token` to `logged_in`. Frontend indicates switching to a logged-out client may trigger a login flow. More user-friendly naming.

---

## D93: Redirect URIs for tenant resource-clients

**Question**: What redirect URIs should be registered for newly created tenant resource-clients?

**Decision**: Resource callback only: `{BODHI_APP_URL}/ui/auth/callback`. Tenant resource-clients don't need the dashboard callback route. Dashboard clients have their own redirect URIs configured separately by the Keycloak admin.

---

## D94: SPI table names differ from plan

**Question**: The SPI implementation used different table names and schema than originally planned.

**Decision**: Accept actual implementation:
- `bodhi_clients` (was `bodhi_tenant_clients`) — tracks ALL resource clients (standalone + multi-tenant). Standalone rows have null `multi_tenant_client_id`.
- `bodhi_clients_users` (was `bodhi_client_roles`) — membership proxy only, no role column. Presence = membership, absence = no membership.
- PK: VARCHAR(36) UUID (was BIGSERIAL). `realm_id` included in both tables.

---

## D95: D46 reversed — Keycloak groups are sole role source of truth

**Question**: D46 said `bodhi_client_roles` would be sole source of truth for roles. The SPI implementation has no role column.

**Decision**: D46 is reversed. `bodhi_clients_users` tracks membership only (no role column). Keycloak group membership is the sole source of truth for roles. Role data is available in JWT claims after user logs into a resource-client. GET /tenants does not return role.

---

## D96: No role in tenant dropdown for MVP

**Question**: Should the tenant selector dropdown display the user's role per tenant?

**Decision**: No role in dropdown for MVP. The SPI's GET /tenants doesn't return role. Role is visible after tenant login (from JWT `resource_access` claims). Adding role to the dropdown would require additional SPI calls per tenant.

---

## D97: POST /resources now dual-writes to bodhi_clients

**Question**: The SPI's POST /resources now writes to `bodhi_clients` (with null multi_tenant_client_id). Does this affect the standalone flow?

**Decision**: Accept. POST /resources creates a `bodhi_clients` row with null `multi_tenant_client_id` and null `created_by_user_id`. make-resource-admin updates `created_by_user_id` and inserts `bodhi_clients_users` membership. BodhiApp's tenants table keeps its own `created_by` independently (D37). No cross-system queries needed for audit data.

---

## Implementation Decisions (from M2 backend implementation)

### D98: `BODHI_MULTITENANT_CLIENT_SECRET` from env only

**Decision**: `multitenant_client_secret()` reads from environment variable only (via `get_env()`), never from database. Client ID is a regular setting (can be in DB or env), but secrets must only come from env for security.

### D99: Dashboard token expiry check via SystemTime

**Decision**: `ensure_valid_dashboard_token()` in `dashboard_helpers.rs` checks JWT `exp` claim against `SystemTime::now()` (not `TimeService`). This is a pragmatic choice — dashboard token refresh is a convenience helper, not a domain operation requiring deterministic time. TECHDEBT notes this should be unified with resource token lifecycle.

### D100: UserInfoEnvelope with serde flatten

**Decision**: `/user/info` wraps `UserResponse` in `UserInfoEnvelope` using `#[serde(flatten)]`. The `has_dashboard_session` field is only serialized when `true` (via `skip_serializing_if`). This preserves backward compatibility — existing clients see the same response when `has_dashboard_session` is false.

### D101: All multi-tenant endpoints return not_multi_tenant error in standalone

**Decision**: Dashboard auth and tenant management endpoints check `settings.is_multi_tenant()` at the start and return `DashboardAuthRouteError::NotMultiTenant` (500) when false. This is a hard gate — no fallback behavior.

### D102: D68 implemented — auth_initiate unified (updated)

**Original decision**: `POST /auth/initiate` was NOT unified to require `client_id` in the request body during M2.

**Update (M3)**: D68 is now fully implemented. `POST /auth/initiate` requires `AuthInitiateRequest { client_id: String }` in the request body. Uses `get_tenant_by_client_id(&request.client_id)` instead of `get_standalone_app()`. Stores `auth_client_id` in session for callback retrieval. `auth_callback` reads `auth_client_id` from session to look up tenant. Standalone frontend sends `client_id` from `/info` response.

---

## Integration Test Infrastructure Decisions (from M4)

### D103: `forward_spi_request` uses owned String params

**Decision**: `AuthService::forward_spi_request()` takes owned `String` parameters (`method: String, endpoint: String, authorization: Option<String>`) instead of `&str` references.

**Rationale**: mockall's `#[automock]` has lifetime incompatibilities with `&str` parameters in async trait methods. Owned params avoid the issue with no meaningful performance impact (called infrequently in test infrastructure).

### D104: DefaultDbService uses builder pattern for env_type

**Decision**: `DefaultDbService` adds `env_type: EnvType` field with a default of `EnvType::Development` and a `.with_env_type()` builder method, rather than changing the constructor signature.

**Rationale**: Avoids breaking all existing callers of `DefaultDbService::new()`. Only `AppServiceBuilder` (production path) needs to set it explicitly via the builder.

### D105: D99 superseded — `ensure_valid_dashboard_token` uses TimeService

**Decision**: `ensure_valid_dashboard_token()` now accepts `time_service: &dyn TimeService` as a 4th parameter and uses `time_service.utc_now().timestamp()` instead of `SystemTime::now()`.

**Supersedes D99**: D99 chose `SystemTime::now()` for pragmatic reasons. The M4 integration test infrastructure required deterministic time for testing, so this was updated. All callers (`routes_tenants.rs`, `routes_setup.rs`, dev handlers) pass `time_service`.

### D106: Dev-only test endpoints not exposed in production

**Decision**: Two dev-only endpoints added for integration test support:
- `POST /dev/clients/{client_id}/dag` — enables Direct Access Grants on a KC client via SPI, returns client credentials
- `DELETE /dev/tenants/cleanup` — cleans up KC tenants + truncates local tenants table

Both are registered in the `!is_production()` block in `routes.rs`. Production builds never expose these routes. Error variants: `DevError::NotMultiTenant`, `DevError::SpiRequestFailed`, `DevError::TenantNotFoundLocal`, etc.
