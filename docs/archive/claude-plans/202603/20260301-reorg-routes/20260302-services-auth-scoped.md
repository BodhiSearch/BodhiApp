# Auth-Scoped Service Layer Holistic Redesign

## Context

The service layer currently has ad-hoc auth handling: handlers extract `AuthContext` and `AppService` separately, manually pull `user_id`/`token`/`client_credentials`, and pass them to individual service methods. `AuthScopedAppService` was recently introduced but isn't used by any handler yet. This redesign makes auth-scoped services the **standard** entry point for all route handlers in `routes_app`, preparing for multi-tenant support and eventual framework decoupling.

**Goals:**
1. Add `client_id` to all `AuthContext` variants (multi-tenant readiness)
2. Make `AuthScopedAppService` the standard service entry point for all `routes_app` handlers (including setup/auth routes)
3. Create auth-scoped sub-services that auto-inject `user_id`, `token`, `client_credentials`
4. Store client service token (Keycloak client_credentials grant) encrypted in `apps` table
5. Extract shared token validation utilities to `services` crate

## Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| `client_id` placement | On all AuthContext variants | Multi-tenant future: different tenants have different client_ids |
| `client_id` naming | `client_id` on all variants; ExternalApp keeps `app_client_id` for external app | `client_id` = "my tenant", `app_client_id` = "the external caller" |
| `client_secret` | NOT on AuthContext; lazy-loaded from `AppInstanceService` using `client_id` | AuthContext is Serialize/Debug/Clone — secret would leak |
| Visibility enforcement | **Manual verification** — no `pub(crate)` on `app_service()` | Verify via grep that routes_app uses auth-scoped services |
| Setup + auth routes | Use `AuthScopedAppService` with `Anonymous { client_id: None }` | Anonymous with `client_id: None` represents pre-registration state; methods needing client_id return errors |
| Session handling | **Out of auth scope** — session is tower-specific stateful state | Auth scope is stateless. Session data passed explicitly from handlers. Session represents mutable state in otherwise stateless services — document this distinction |
| Sub-service pattern | Thin wrappers: inject auth-derived params (`user_id`, `token`, `client_credentials`) | Multi-service coordination stays in handlers (flexibility). Auth scope handles only props derivable from AuthContext |
| Auth-agnostic services | Passthrough accessors on `AuthScopedAppService` | All access goes through auth scope for consistency |
| Client service token | Encrypted columns on existing `apps` table (no new table, no production release) | Add to existing migration. In-memory cache for hot path |
| Token refresh | Shared utility methods in `services` crate | Both middleware and auth scope use same logic for JWT expiry check |

### Who Uses AppService Directly vs AuthScoped?

| Layer | Uses | Rationale |
|---|---|---|
| `routes_app` handlers | `AuthScopedAppService` | All API request flows. Auth-derived params auto-injected. |
| `auth_middleware` | `AppService` directly | Infrastructure layer that **creates** AuthContext. Needs raw service access for token validation, DB lookups, session management. Runs before auth context exists. |
| `server_app` | `AppService` directly | Server bootstrap and composition. Sets up the HTTP server. Integration tests exercise the full stack via HTTP requests (middleware creates AuthContext, handlers use AuthScoped). |
| `lib_bodhiserver` | `AppService` directly | Embeddable server bootstrap. Composes services and creates AppService. Does not handle requests directly. |

**Rule: All API request handling flows use AuthScopedAppService. Infrastructure (bootstrap, middleware, composition) uses AppService directly.** Document in project-level CLAUDE.md.

### Session as State

Session (`tower-sessions`) represents **mutable per-request state** in an otherwise stateless service architecture:
- Stores PKCE codes, CSRF state, access/refresh tokens
- Tied to tower/axum framework
- Auth scope is stateless — derives everything from AuthContext + AppService
- When auth-scoped methods need session-stored data (e.g., PKCE code for OAuth callback), handlers extract it from session and pass explicitly

---

## Phase 1: Infrastructure — `services` + `auth_middleware` (No Auth-Scoped Services Yet)

Foundation changes that touch AuthContext, shared utilities, and middleware. No auth-scoped sub-services created — just the core AuthScopedAppService structure. Compile, test-compile, fix tests, pass, local commit.

### 1a. AuthContext Variant Redesign

**File:** `crates/services/src/auth/auth_context.rs`

```rust
pub enum AuthContext {
  Anonymous { client_id: Option<String> },
  Session { client_id: String, user_id: String, username: String, role: Option<ResourceRole>, token: String },
  ApiToken { client_id: String, user_id: String, role: TokenScope, token: String },
  ExternalApp { client_id: String, user_id: String, role: Option<UserScope>, token: String, external_app_token: String, app_client_id: String, access_request_id: Option<String> },
}
```

Source of `client_id` per variant:
- **Session**: `AppInstance.client_id` (middleware already fetches AppInstance for token refresh)
- **ApiToken**: `AppInstance.client_id` (add AppInstance fetch to ApiToken validation branch)
- **ExternalApp**: JWT `aud` claim (already validated against `instance.client_id` — just store it)
- **Anonymous**: `AppInstance.client_id` if available (`None` during pre-registration/setup)

Update convenience methods:
- Add `pub fn client_id(&self) -> Option<&str>` — returns `Some` for all authenticated variants, `None` for Anonymous with `None`
- Update `user_id()`, `require_user_id()`, `token()`, etc. for new field positions
- Add `pub fn require_client_id(&self) -> Result<&str, ApiError>` — 403/error when client_id is None

**File:** `crates/services/src/test_utils/auth_context.rs`
- Update all 6 test factory methods to accept `client_id` parameter
- Default `"test-client-id"` for convenience

### 1b. AuthScopedAppService Core Structure

**File:** `crates/services/src/app_service/auth_scoped.rs`

Redesign with passthrough accessors only — no domain sub-services yet (those come in later phases):

```rust
pub struct AuthScopedAppService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedAppService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self;

  // Auth context accessors
  pub fn auth_context(&self) -> &AuthContext;
  pub fn require_user_id(&self) -> Result<&str, ApiError>;
  pub fn client_id(&self) -> Option<&str>;
  pub fn require_client_id(&self) -> Result<&str, ApiError>;

  // Raw AppService access (manually verified: only used in services crate internals + tests)
  pub fn app_service(&self) -> &Arc<dyn AppService>;

  // Pass-through accessors for all services (initial state before sub-services are created)
  pub fn data_service(&self) -> Arc<dyn DataService>;
  pub fn hub_service(&self) -> Arc<dyn HubService>;
  pub fn setting_service(&self) -> Arc<dyn SettingService>;
  pub fn time_service(&self) -> Arc<dyn TimeService>;
  pub fn db_service(&self) -> Arc<dyn DbService>;
  pub fn session_service(&self) -> Arc<dyn SessionService>;
  pub fn network_service(&self) -> Arc<dyn NetworkService>;
  pub fn ai_api_service(&self) -> Arc<dyn AiApiService>;
  pub fn queue_producer(&self) -> Arc<dyn QueueProducer>;
  pub fn app_instance_service(&self) -> Arc<dyn AppInstanceService>;
  pub fn access_request_service(&self) -> Arc<dyn AccessRequestService>;
  pub fn cache_service(&self) -> Arc<dyn CacheService>;
  pub fn mcp_service(&self) -> Arc<dyn McpService>;
  pub fn tool_service(&self) -> Arc<dyn ToolService>;
  pub fn token_service(&self) -> Arc<dyn TokenService>;
  pub fn auth_service(&self) -> Arc<dyn AuthService>;
  pub fn concurrency_service(&self) -> Arc<dyn ConcurrencyService>;
}
```

### 1c. Client Service Token — Add Columns to `apps` Table

**File:** `crates/services/src/db/sea_migrations/m20250101_000013_apps.rs` (modify existing migration)

Add columns to existing `apps` table:
```sql
encrypted_service_token TEXT,
salt_service_token TEXT,
nonce_service_token TEXT,
service_token_expires_at TIMESTAMP WITH TIME ZONE
```

All nullable (empty when no cached token yet).

**File:** `crates/services/src/apps/app_instance_repository.rs` — add methods:
- `get_client_service_token(client_id) -> Option<(String, DateTime<Utc>)>` (decrypted token + expiry)
- `upsert_client_service_token(client_id, token, expires_at)` (encrypts before storing)

**File:** `crates/services/src/auth/auth_service.rs` — make `get_client_access_token(client_id, client_secret)` a public trait method on `AuthService` (currently private on `KeycloakAuthService`).

### 1d. Token Validation Shared Utilities

**New file:** `crates/services/src/auth/token_utils.rs`

Extract from `auth_middleware`'s `DefaultTokenService`:
- `parse_jwt_claims<T>(token) -> Result<T>` — base64 decode, no signature check
- `is_token_expired(claims, now) -> bool` — check `exp` vs current time
- `needs_eager_refresh(claims, now, buffer_secs) -> bool` — refresh before expiry
- `extract_resource_role(claims, client_id) -> Option<ResourceRole>` — parse resource_access

Pure functions (no I/O) usable by both middleware and auth-scoped layer.

### 1e. Auth Middleware — Populate client_id

**File:** `crates/auth_middleware/src/auth_middleware/middleware.rs`

- **Session branch**: already fetches AppInstance for token refresh — use `instance.client_id`
- **ApiToken branch**: add `app_instance_service.get_instance()` call (consider caching)
- **ExternalApp branch**: JWT `aud` claim already validated — store as `client_id`
- **Anonymous branch**: fetch AppInstance if exists → `Some(id)`, else `None`

**File:** `crates/auth_middleware/src/token_service/service.rs`
- Use shared `services::token_utils` for JWT parsing and expiry checks

### 1f. Tests & Gate Check

- Update all AuthContext constructors across: services tests, auth_middleware tests, routes_app tests (compile fixes only — don't change handler logic yet)
- Run: `cargo test -p services`
- Run: `cargo test -p auth_middleware`
- Run: `cargo check -p routes_app` (compile check — handler tests may need AuthContext field updates)
- Run: `cargo test -p routes_app` (fix any test failures from AuthContext field changes)
- Run: `cargo test -p server_app`
- **Gate:** `make test.backend` passes

**Local commit after passing.**

---

## Phase 2: AuthScopedAppService Extractor + Reference Vertical (tokens)

Create the axum extractor and migrate the simplest domain as reference implementation.

### 2a. Axum Extractor

**New file:** `crates/routes_app/src/shared/auth_scope_extractor.rs`

```rust
#[async_trait]
impl<S> FromRequestParts<S> for AuthScopedAppService
where S: Send + Sync {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(auth_context) = Extension::<AuthContext>::from_request_parts(parts, state).await
            .map_err(|_| AnonymousNotAllowedError)?;
        let State(router_state) = State::<Arc<dyn RouterState>>::from_request_parts(parts, state).await
            .map_err(|_| /* internal error */)?;
        Ok(AuthScopedAppService::new(router_state.app_service(), auth_context))
    }
}
```

### 2b. AuthScopedTokenService — Complete Implementation

**File:** `crates/services/src/app_service/auth_scoped_tokens.rs` (exists, expand)

Wraps `TokenService` + `TimeService`. Injects `user_id`.
- `list_tokens(page, per_page)` — injects user_id *(exists)*
- `get_token(id)` — injects user_id
- `create_token(name, scope)` — injects user_id, generates token hash, uses time_service
- `update_token(id, status/name)` — injects user_id, uses time_service

### 2c. Migrate `tokens/` Handlers

**File:** `crates/routes_app/src/tokens/routes_tokens.rs`

```
Before: Extension(auth_context) + State(state) → extract_claims(token) → db_service.xxx(user_id)
After:  auth_scope: AuthScopedAppService → auth_scope.tokens().xxx()
```

- `tokens_index` → `auth_scope.tokens().list_tokens(page, per_page)`
- `tokens_create` → `auth_scope.tokens().create_token(name, scope)`
- `tokens_update` → `auth_scope.tokens().update_token(id, ...)`
- Remove `extract_claims` calls — sub-service uses `auth_context.user_id()` directly

### 2d. Tests & Gate Check

- Update tokens test files
- Run: `cargo test -p services`
- Run: `cargo test -p routes_app`
- **Gate:** all token tests pass

**Local commit.**

---

## Phase 3: Batch 2 — mcps + toolsets (user_id injection pattern)

Both follow the same pattern: wrap underlying service, inject `user_id` for all user-scoped methods.

### 3a. AuthScopedMcpService

**New file:** `crates/services/src/app_service/auth_scoped_mcps.rs`

Wraps `McpService`. Injects `user_id`.
- `list()`, `get(id)`, `create(...)`, `update(...)`, `delete(id)`, `fetch_tools(id)`, `execute(id, tool, request)`
- `create_auth_header(...)`, `create_auth_config(...)`, `store_oauth_token(...)`, `get_oauth_token(token_id)`, `exchange_oauth_token(...)`

### 3b. AuthScopedToolService

**New file:** `crates/services/src/app_service/auth_scoped_tools.rs`

Wraps `ToolService`. Injects `user_id` for user-scoped ops, `updated_by` for admin ops.
- `list()`, `get(id)`, `create(...)`, `update(...)`, `delete(id)`, `execute(...)`, `list_tools_for_user()`
- `set_app_toolset_enabled(type, enabled)` — injects `updated_by` from user_id

### 3c. Migrate mcps/ Handlers

All files in `crates/routes_app/src/mcps/`:
- `routes_mcps.rs`: Replace `auth_context.user_id().expect(...)` + `state.app_service().mcp_service()` with `auth_scope.mcps()`
- `routes_mcps_servers.rs`: Use `auth_scope.require_user_id()` for `created_by`/`updated_by`, passthrough `auth_scope.mcp_service()` for server ops
- `routes_mcps_auth.rs`: Use `auth_scope.mcps()` for user_id-injected methods
- `routes_mcps_oauth.rs`: Use `auth_scope.mcps()` for user_id-injected methods
- Special: `mcp_oauth_login` needs Session — pass session data explicitly

### 3d. Migrate toolsets/ Handlers

**File:** `crates/routes_app/src/toolsets/routes_toolsets.rs`
- Replace all `auth_context.user_id().expect(...)` + `state.app_service().tool_service()` with `auth_scope.tools()`
- All 8+ handlers

### 3e. Tests & Gate Check

- Update all mcps/ and toolsets/ test files
- Run: `cargo test -p services`
- Run: `cargo test -p routes_app`
- **Gate:** all mcps + toolsets tests pass

**Local commit.**

---

## Phase 4: Batch 3 — users + apps (token injection pattern)

These sub-services inject the access `token` for Keycloak calls in addition to `user_id`.

### 4a. AuthScopedUserService

**New file:** `crates/services/src/app_service/auth_scoped_users.rs`

Wraps `AuthService` (Keycloak) + `DbService` (AccessRepository). Injects `token` + `user_id`.
- `list_users(page, page_size)` — injects reviewer `token`
- `assign_user_role(target_user_id, role)` — injects reviewer `token`
- `remove_user(target_user_id)` — injects reviewer `token`
- `insert_pending_request(username)` — injects `user_id`
- `get_pending_request()` — injects `user_id`
- `update_request_status(id, status)` — injects reviewer username from auth_context

### 4b. AuthScopedAppAccessService

**New file:** `crates/services/src/app_service/auth_scoped_apps.rs`

Wraps `AccessRequestService`. Injects `user_id` + `token`.
- `approve_request(id, tool_approvals, mcp_approvals, approved_role)` — injects `user_id` + `token`
- `deny_request(id)` — injects `user_id`

### 4c. Migrate users/ Handlers

**File:** `crates/routes_app/src/users/routes_users.rs`
- `users_index` → `auth_scope.users().list_users(page, page_size)`
- `users_change_role` → `auth_scope.users().assign_user_role(target_id, role)` + `auth_scope.session_service().clear_sessions_for_user(target_id)` (session coordination stays in handler)
- `users_destroy` → `auth_scope.users().remove_user(target_id)`

**File:** `crates/routes_app/src/users/routes_users_access_request.rs`
- `users_request_access` → `auth_scope.users().insert_pending_request(username)`
- `users_request_status` → `auth_scope.users().get_pending_request()`
- `users_access_request_approve` → `auth_scope.users().assign_user_role()` + session coordination in handler
- `users_access_request_reject` → `auth_scope.users().update_request_status()`

### 4d. Migrate apps/ Handlers

**File:** `crates/routes_app/src/apps/routes_apps.rs`
- `apps_get_access_request_review` → `auth_scope.tools().list()` + `auth_scope.mcps().list()` for user-scoped listings
- `apps_approve_access_request` → validation with `auth_scope.tools().get(id)` + `auth_scope.mcps().get(id)` + `auth_scope.apps().approve_request(...)`
- `apps_deny_access_request` → `auth_scope.apps().deny_request(id)`
- `apps_create_access_request` → passthrough `auth_scope.access_request_service()` (no auth params)

### 4e. Tests & Gate Check

- Update all users/ and apps/ test files
- Run: `cargo test -p services`
- Run: `cargo test -p routes_app`
- **Gate:** all users + apps tests pass

**Local commit.**

---

## Phase 5: Batch 4 — Passthrough domains (models, settings, api_models, oai, ollama, setup, auth)

These handlers are auth-agnostic or use only passthrough accessors. Change handler signature from `Extension<AuthContext> + State(state)` to `auth_scope: AuthScopedAppService`.

### 5a. Migrate models/ Handlers

- Use passthrough: `auth_scope.data_service()`, `auth_scope.hub_service()`, `auth_scope.db_service()`, `auth_scope.time_service()`, `auth_scope.queue_producer()`
- Multi-service coordination stays in handlers

### 5b. Migrate settings/, api_models/, oai/, ollama/

- Same passthrough pattern
- `auth_scope.setting_service()`, `auth_scope.ai_api_service()`, `auth_scope.db_service()`

### 5c. Migrate setup/ + auth/ Handlers

- Change from `Extension<AuthContext> + State(state)` to `auth_scope: AuthScopedAppService`
- `setup_create`: uses `auth_scope.auth_service()`, `auth_scope.app_instance_service()`, `auth_scope.setting_service()`, `auth_scope.network_service()`
- Methods needing `client_id` call `auth_scope.require_client_id()` — returns error if Anonymous with `None`
- `auth_callback`: uses `auth_scope.auth_service()`, `auth_scope.app_instance_service()`, `auth_scope.setting_service()` — session data passed explicitly
- `auth_initiate`, `auth_logout`: similar passthrough pattern

### 5d. Tests & Gate Check

- Update all test files for migrated domains
- Run: `cargo test -p routes_app`
- Run: `cargo test -p server_app`
- **Gate:** `make test.backend` passes

**Local commit.**

---

## Phase 6: Documentation

- Update `crates/services/CLAUDE.md` and `PACKAGE.md` — AuthContext with client_id, AuthScopedAppService, sub-services, token_utils, client service token persistence
- Update `crates/routes_app/CLAUDE.md` and `PACKAGE.md` — extractor pattern, handler conventions, all-handlers-use-AuthScoped rule
- Update `crates/auth_middleware/CLAUDE.md` — client_id population in middleware
- Update root `CLAUDE.md`:
  - Architecture overview: "All API request flows use AuthScopedAppService. Infrastructure (bootstrap, middleware, composition) uses AppService directly."
  - Session note: "Session represents mutable per-request state in otherwise stateless services. Auth scope is stateless — session data passed explicitly to auth-scoped methods when needed."
- Manual verification: grep for `state.app_service()` in `routes_app` — should only appear in the extractor

---

## Verification Summary

| Gate Check | Command | When |
|---|---|---|
| Services compile | `cargo check -p services` | After Phase 1a-1d |
| Services tests | `cargo test -p services` | After Phase 1f |
| Auth middleware tests | `cargo test -p auth_middleware` | After Phase 1e |
| Routes compile | `cargo check -p routes_app` | After Phase 1f (AuthContext field updates) |
| Routes tests | `cargo test -p routes_app` | After each Phase 2-5 |
| Integration tests | `cargo test -p server_app` | After Phase 5 |
| Full regression | `make test.backend` | After Phase 5 |
| Manual grep | `grep -r "state.app_service()" routes_app` | After Phase 6 |

---

## Sub-Service Reference

| Sub-Service | File | Wraps | Injects | Phase |
|---|---|---|---|---|
| `AuthScopedTokenService` | `auth_scoped_tokens.rs` | TokenService + TimeService | `user_id` | 2 |
| `AuthScopedMcpService` | `auth_scoped_mcps.rs` | McpService | `user_id` | 3 |
| `AuthScopedToolService` | `auth_scoped_tools.rs` | ToolService | `user_id`, `updated_by` | 3 |
| `AuthScopedUserService` | `auth_scoped_users.rs` | AuthService + DbService | `token`, `user_id`, `username` | 4 |
| `AuthScopedAppAccessService` | `auth_scoped_apps.rs` | AccessRequestService | `user_id`, `token` | 4 |

All sub-service files in `crates/services/src/app_service/`.
