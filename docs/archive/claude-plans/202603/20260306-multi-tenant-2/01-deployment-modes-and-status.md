# Deployment Modes and AppStatus State Machine

## Overview

BodhiApp supports two deployment modes -- **Standalone** (single-tenant, desktop/self-hosted) and **MultiTenant** (SaaS, multiple tenants sharing one instance) -- controlled by the `BODHI_DEPLOYMENT` environment variable. The deployment mode determines the AppStatus state machine transitions, the `/info` endpoint behavior, authentication flow shape, and which features are available. The `AuthContext` enum encodes deployment mode at the type level via distinct variants (`Session` for standalone, `MultiTenantSession` for multi-tenant), enabling zero-cost deployment-mode checks in route handlers. See [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#authcontext-enum) for full `AuthContext` details.

## Functional Behavior

### Deployment Modes

| Aspect | Standalone (`standalone`) | Multi-Tenant (`multi_tenant`) |
|--------|---------------------------|-------------------------------|
| **Default?** | Yes (fallback for any unrecognized value) | Requires explicit `BODHI_DEPLOYMENT=multi_tenant` |
| **Tenant count** | Exactly 1 | 0..N per user |
| **Auth phases** | Single: resource-client OAuth2 | Two-phase: dashboard OAuth2, then resource-client OAuth2 |
| **Setup flow** | Full wizard: Login -> Models -> API Keys -> Toolsets -> Extension -> Complete | No setup wizard -- tenant is `Ready` immediately after creation |
| **LLM features** | Active (local inference, model downloads) | Disabled (no local models, API models only) |
| **Dashboard endpoints** | Return error `dashboard_auth_route_error-not_multi_tenant` | Fully functional |
| **Setup endpoint** | Functional (`POST /bodhi/v1/setup`) | Returns error `setup_route_error-not_standalone` |

### AppStatus State Machine

**Standalone mode transitions:**
```
[No tenants in DB]
  -> Setup (default)
      | POST /bodhi/v1/setup -> registers client via SPI /resources, creates tenant
  -> ResourceAdmin
      | First login callback -> calls SPI /make-resource-admin, sets created_by
  -> Ready
```

**Multi-tenant mode transitions (session-driven, not DB-driven):**
```
[No dashboard token in session]
  -> TenantSelection (frontend shows "Login to Bodhi Platform" button)
      | Dashboard OAuth2 login completes
[Dashboard token present, no active_client_id]
  -> TenantSelection (user has existing tenants) or Setup (user has zero tenants)
      | User selects/creates tenant + resource-client OAuth2
[active_client_id set, valid resource-client token]
  -> Ready
```

### `GET /bodhi/v1/info` Endpoint

Returns `AppInfo` with fields: `version`, `commit_sha`, `status` (AppStatus), `deployment` (DeploymentMode), `client_id` (optional, active tenant's OAuth client_id).

**Status resolution is purely local** -- no remote auth server calls. The handler matches on `AuthContext` variants:

| AuthContext Variant | Resulting Status | client_id |
|---------------------|-----------------|-----------|
| `Session { .. }` (standalone authenticated) | `Ready` | `Some(client_id)` |
| `MultiTenantSession { client_id: Some(..) }` (MT fully authenticated) | `Ready` | `Some(client_id)` |
| `MultiTenantSession { client_id: None }` (MT dashboard-only) | `TenantSelection` if user has memberships, `Setup` if none | `None` |
| `Anonymous { deployment: MultiTenant }` | `TenantSelection` | `None` |
| `Anonymous { deployment: Standalone }` | DB-based lookup via `get_standalone_app()` | From tenant if exists |
| `ApiToken` / `ExternalApp` | `Ready` | `Some(client_id)` |

### Configuration Differences

| Setting | Standalone | Multi-Tenant |
|---------|-----------|--------------|
| `BODHI_DEPLOYMENT` | `standalone` (or unset) | `multi_tenant` |
| `BODHI_MULTITENANT_CLIENT_ID` | Not required | Required (dashboard OAuth2 client) |
| `BODHI_MULTITENANT_CLIENT_SECRET` | Not required | Required (env var only, never in DB) |
| `BODHI_AUTH_URL` / `BODHI_AUTH_REALM` | Required | Required |
| `BODHI_PUBLIC_SCHEME/HOST/PORT` | Used for redirect URIs | Used for redirect URIs |

Accessing `multitenant_client_id()` or `multitenant_client_secret()` in standalone mode returns `SettingServiceError::InvalidDeploymentMode`. Accessing them in multi-tenant mode without the values set returns `SettingServiceError::MissingConfig`.

### How Deployment Mode Affects Auth and Data Isolation

- **Auth middleware**: Unified code path (D23). Always resolves tenant from incoming token claims (`azp` for session JWT, `aud` for external app JWT, `client_id` suffix for API tokens). Works identically for one tenant (standalone) or N tenants (multi-tenant). See [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#request-authentication-flow) for full auth flow.
- **Multi-tenant middleware addition**: When `deployment_mode == MultiTenant`, middleware also reads/validates/refreshes the dashboard token from session, constructing a `MultiTenantSession` variant.
- **Data isolation**: All mutating DB operations use `begin_tenant_txn(tenant_id)` with PostgreSQL RLS. Same mechanism in both modes. See [05-data-isolation-rls.md](05-data-isolation-rls.md) for full RLS details.
- **Session key namespacing**: Both modes use namespaced session keys: `{client_id}:access_token`, `{client_id}:refresh_token`, `active_client_id`. Multi-tenant adds `dashboard:access_token`, `dashboard:refresh_token`. See [02-auth-sessions-middleware.md](02-auth-sessions-middleware.md#session-key-schema) for the full schema.

## Architecture & Data Model

### DeploymentMode Enum

```rust
// crates/services/src/tenants/tenant_objs.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
  #[default]
  Standalone,
  MultiTenant,
}
```

Serializes as `"standalone"` / `"multi_tenant"`. Default is `Standalone` -- any unrecognized `BODHI_DEPLOYMENT` value falls back to standalone.

### AppStatus Enum

```rust
// crates/services/src/tenants/tenant_objs.rs
pub enum AppStatus {
  #[default] Setup,
  Ready,
  ResourceAdmin,
  TenantSelection,  // Added for multi-tenant
}
```

- `Setup` -- initial state, no tenants exist (standalone) or user has zero tenant memberships (multi-tenant dashboard-only)
- `ResourceAdmin` -- standalone only, tenant created but first admin login not yet completed
- `TenantSelection` -- multi-tenant only, user needs to select or create a tenant
- `Ready` -- fully operational

### AppInfo Response Type

```rust
// crates/routes_app/src/setup/setup_api_schemas.rs
pub struct AppInfo {
  pub version: String,
  pub commit_sha: String,
  pub status: AppStatus,
  pub deployment: DeploymentMode,         // typed enum, not String
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_id: Option<String>,
}
```

### SettingService Methods

- `deployment_mode() -> DeploymentMode` -- parses `BODHI_DEPLOYMENT` string to enum, falls back to `Standalone`
- `is_multi_tenant() -> bool` -- delegates to `deployment_mode()`
- `multitenant_client_id() -> Result<String>` -- guards against standalone mode, errors if not configured
- `multitenant_client_secret() -> Result<String>` -- env-only read, guards against standalone mode

### Data Flow: /info Request

```
Request -> optional_auth_middleware (populates AuthContext)
  -> AuthScope extractor (wraps AuthContext + AppService)
  -> setup_show handler:
      1. settings.deployment_mode() -> DeploymentMode
      2. match auth_scope.auth_context() {
           Session -> (Ready, Some(client_id))
           MultiTenantSession(Some(cid)) -> (Ready, Some(cid))
           MultiTenantSession(None) -> check tenants().has_memberships()
           Anonymous(MultiTenant) -> (TenantSelection, None)
           Anonymous(Standalone) -> get_standalone_app() from DB
           ApiToken/ExternalApp -> (Ready, Some(client_id))
         }
      3. Return AppInfo { version, commit_sha, status, deployment, client_id }
```

## Technical Implementation

### Key Files

| File | Purpose |
|------|---------|
| `crates/services/src/tenants/tenant_objs.rs` | `DeploymentMode` enum, `AppStatus` enum, `Tenant` struct |
| `crates/services/src/auth/auth_context.rs` | `AuthContext` enum with `MultiTenantSession` variant, `AuthContextError` |
| `crates/services/src/settings/setting_service.rs` | `deployment_mode()`, `is_multi_tenant()`, `multitenant_client_id()`, `multitenant_client_secret()` |
| `crates/services/src/settings/constants.rs` | `BODHI_DEPLOYMENT`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET` |
| `crates/services/src/settings/error.rs` | `SettingServiceError::InvalidDeploymentMode`, `MissingConfig` |
| `crates/routes_app/src/setup/routes_setup.rs` | `setup_show` handler (/info), `setup_create` handler (/setup with standalone guard) |
| `crates/routes_app/src/setup/setup_api_schemas.rs` | `AppInfo` struct with `deployment: DeploymentMode` |
| `crates/routes_app/src/setup/error.rs` | `SetupRouteError::NotStandalone` |
| `crates/routes_app/src/middleware/auth/auth_middleware.rs` | Middleware constructing `MultiTenantSession` vs `Session` based on deployment mode |
| `crates/services/src/tenants/auth_scoped.rs` | `AuthScopedTenantService` with `has_memberships()`, `list_my_tenants()` |

### Refactoring History

1. **Original (pre-MT)**: `/info` called `app_status_or_default()` -- simple DB lookup for the single standalone tenant.
2. **Stage 2 initial**: `resolve_multi_tenant_status()` added -- made remote SPI calls (`list_tenants()`, `ensure_valid_dashboard_token()`) on every `/info` request. Too heavy for a bootstrap endpoint.
3. **Architecture refactor**: `MultiTenantSession` AuthContext variant introduced. `tenants_users` local table added. `/info` became fully local -- pattern match on `AuthContext` + `has_memberships()` DB query. Remote calls removed. `list_tenants` removed from `AuthService`. `ensure_valid_dashboard_token()` deleted.

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Title | Status |
|----|-------|--------|
| D23 | Unified middleware code path | Implemented |
| D34 | No deployment field in AppInfo | Superseded by D67 |
| D54 | `/info` behind optional_auth_middleware | Implemented |
| D56 | Breaking session key migration | Implemented |
| D58 | Deployment mode injection into handlers | Implemented (via `AuthContext` variant type encoding + `is_multi_tenant()`) |
| D67 | `/info` returns deployment mode | Implemented (supersedes D34) |
| D68 | `client_id` required in POST /auth/initiate | Implemented |
| D70 | `/info` includes client_id | Implemented |
| D101 | MT endpoints error in standalone | Implemented |

## Known Gaps & TECHDEBT

1. **Service construction not deployment-aware**: `AppServiceBuilder` does not yet conditionally skip `InferenceService`, `HubService`, or LLM routes based on deployment mode. All services are initialized in both modes. This wastes resources in multi-tenant mode where local LLM is not used.

2. **Navigation item visibility**: Frontend does not yet hide LLM-specific navigation items (Models, Downloads, LLM settings) in multi-tenant mode, even though `deployment` is available in the `/info` response. See [06-frontend-ui.md](06-frontend-ui.md#known-gaps--techdebt) (Gap 7).

3. **Deployment mode is async**: `settings.deployment_mode()` is async (DB/config lookup). In the architecture refactor, 9 call sites in `routes_app` were migrated to use `auth_context.is_multi_tenant()` (sync) instead, but some non-handler code paths still use the async version.

4. **D80 shared code exchange**: Code exchange logic is duplicated between `routes_auth.rs` (resource callback) and `routes_dashboard_auth.rs` (dashboard callback). Low priority -- no functional impact.
