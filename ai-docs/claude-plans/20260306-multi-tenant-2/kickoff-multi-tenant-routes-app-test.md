# Multi-Tenant Isolation Tests — routes_app Kickoff — ✅ COMPLETED

## Goal

Add cross-tenant and intra-tenant user isolation tests at the HTTP handler layer in `crates/routes_app/`. These tests verify that when multiple tenants exist in the database, HTTP requests authenticated as tenant A only see tenant A's resources — even though tenant B's resources exist in the same database.

This is the routes_app companion to the services-layer repository isolation tests already in place. The tests should be parameterized with `#[values("sqlite", "postgres")]` — both backends use WHERE clauses for filtering, PostgreSQL adds defence-in-depth RLS, but the functional result is identical.

---

## Test Architecture Overview

### How Route Tests Work
- Single-turn HTTP tests via `tower::oneshot()` — no TCP listener, no real server
- State type: `Arc<dyn AppService>` (NOT `RouterState` — that was removed)
- `build_test_router()` in `crates/routes_app/src/test_utils/router.rs` returns `(Router, Arc<dyn AppService>, Arc<TempDir>)` with:
  - Real SQLite DB (in-memory via tempfile), real SessionService, DataService, HubService
  - Stubbed: MockInferenceService, StubNetworkService, StubQueue
  - Full route composition via `build_routes()` with session layer + auth middleware
  - Single tenant: `Tenant::test_default()` with `TEST_TENANT_ID`

### AuthScope Extractor (How Handlers Get Auth Context)
- `AuthScope` in `src/shared/auth_scope_extractor.rs` is a newtype around `AuthScopedAppService`
- Extracts `AuthContext` from request extensions (set by middleware) + `Arc<dyn AppService>` from state
- Falls back to `AuthContext::Anonymous { client_id: None, tenant_id: None }` when no middleware populated it
- Handlers access auth-scoped sub-services: `.tokens()`, `.mcps()`, `.tools()`, `.data()`, `.api_models()`, `.downloads()`, `.user_access_requests()`
- Auth-scoped services inject `tenant_id`/`user_id` from `AuthContext` automatically

### AuthContext Injection in Tests (Two Methods)

**Method 1: Direct extension injection** (bypasses middleware, tests handler logic):
```rust
Request::builder()
  .method("GET")
  .uri("/bodhi/v1/tokens")
  .body(Body::empty())?
  .with_auth_context(AuthContext::test_session("user1", "user@test.com", ResourceRole::Admin))
```

**Method 2: Session cookie** (goes through auth middleware):
```rust
let cookie = create_authenticated_session(
  app_service.session_service().as_ref(),
  &["resource_admin"],
).await?;
let response = router.oneshot(session_request("GET", "/bodhi/v1/tokens", &cookie)).await?;
```

### AuthContext Test Factories (services/test_utils/auth_context.rs)

All factories default to `TEST_TENANT_ID`. Chain with `.with_tenant_id()` / `.with_user_id()` for multi-tenant:
```rust
AuthContext::test_session("user1", "user@test.com", ResourceRole::Admin)
  .with_tenant_id(TEST_TENANT_B_ID)
  .with_user_id("user-in-tenant-b")
```

Available factories:
- `test_session(user_id, username, role)` — Session variant
- `test_session_no_role(user_id, username)` — Session without role
- `test_api_token(user_id, role: TokenScope)` — ApiToken variant
- `test_external_app(user_id, role, app_client_id, access_request_id)` — ExternalApp variant

### Test Constants
```rust
// From services::test_utils
pub const TEST_TENANT_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
pub const TEST_TENANT_B_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBB";
pub const TEST_USER_ID: &str = "test-user";
pub const TEST_TENANT_A_USER_B_ID: &str = "test-tenant-a-user-b";
pub const TEST_CLIENT_ID: &str = "test-client";
```

---

## Test Pattern: `test_<domain>_isolation.rs`

Each domain route module should get an isolation test file. Follow the same naming convention as the services layer (`test_<domain>_repository_isolation.rs` → `test_<domain>_isolation.rs` for routes).

### Expected Behavior Matrix

| Scenario | Expected Status |
|----------|----------------|
| Resource belongs to same tenant + same user | `200 OK` / `201 Created` |
| Resource belongs to different tenant (even same user) | `404 Not Found` |
| Resource belongs to same tenant, different user | `404 Not Found` |
| List endpoint, different tenant | `200 OK` with empty/filtered results |

The 404-not-403 pattern is critical — it prevents tenant B from even knowing tenant A's resource IDs exist.

### Test Structure Per Domain

Each isolation test file should have up to 3 test functions:

```rust
// 1. Cross-tenant isolation (same user, different tenants)
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_<domain>_cross_tenant_isolation() -> anyhow::Result<()> {
  // Setup: router + seed data in TENANT_A and TENANT_B for same user
  // Assert: GET list as TENANT_A → only TENANT_A's resources
  // Assert: GET list as TENANT_B → only TENANT_B's resources
  // Assert: GET by ID as wrong tenant → 404
  // Assert: PUT/DELETE as wrong tenant → 404
}

// 2. Intra-tenant user isolation (same tenant, different users)
// Only for user-scoped resources (tokens, mcps, toolsets, user aliases, api models)
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_<domain>_intra_tenant_user_isolation() -> anyhow::Result<()> {
  // Setup: seed data in TENANT_A for USER_A and USER_B
  // Assert: GET list as USER_A → only USER_A's resources
  // Assert: GET by ID as wrong user → 404
}

// 3. Write isolation (create/update/delete can't cross boundaries)
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_<domain>_write_isolation() -> anyhow::Result<()> {
  // Assert: POST as TENANT_A → resource created in TENANT_A
  // Assert: PUT with TENANT_B auth on TENANT_A's resource → 404
  // Assert: DELETE with TENANT_B auth on TENANT_A's resource → 404
}
```

---

## Domains to Cover

### Tenant + User Scoped (need both cross-tenant AND intra-tenant user tests)

| Domain | List Endpoint | Get Endpoint | Has CRUD? |
|--------|-------------|-------------|-----------|
| Tokens | `GET /bodhi/v1/tokens` | `GET /bodhi/v1/tokens/{id}` | Yes (C/R/U/D) |
| MCPs (instances) | `GET /bodhi/v1/mcps` | `GET /bodhi/v1/mcps/{id}` | Yes (C/R/U/D) |
| Toolsets | `GET /bodhi/v1/toolsets` | `GET /bodhi/v1/toolsets/{id}` | Yes (C/R/U/D) |
| API Models | `GET /bodhi/v1/api-models` | `GET /bodhi/v1/api-models/{id}` | Yes (C/R/U/D) |
| User Aliases | `GET /bodhi/v1/models` | — | Yes (C/R/D) |

### Tenant Scoped Only (cross-tenant tests only, no user isolation)

| Domain | List Endpoint | Get Endpoint | Has CRUD? |
|--------|-------------|-------------|-----------|
| MCP Servers | `GET /bodhi/v1/mcps/servers` | `GET /bodhi/v1/mcps/servers/{id}` | Yes |
| Downloads | `GET /bodhi/v1/downloads` | `GET /bodhi/v1/downloads/{id}` | Read-only + trigger |
| User Access Requests | `GET /bodhi/v1/user-access-requests` | — | Read-only |

### Global (negative test — verify NOT tenant-scoped)

| Domain | Endpoint | Reason |
|--------|----------|--------|
| Settings | `GET /bodhi/v1/settings` | Global, no tenant_id column |

---

## Test Infrastructure Needs

### Key Question: Router Setup for Multi-Tenant

`build_test_router()` calls `.with_tenant(Tenant::test_default())` which inserts ONE tenant (TEST_TENANT_ID) into the DB. For isolation tests, we need TWO tenants.

**Explore these options:**

1. **Extend `build_test_router()`** — add a `build_multi_tenant_test_router()` that inserts both `Tenant::test_default()` and a second tenant with `TEST_TENANT_B_ID`. Or provide a helper that takes the existing router's app_service and inserts a second tenant.

2. **Custom `AppServiceStubBuilder` setup** — build the router manually with two `.with_tenant()` calls. The builder supports calling `with_tenant()` multiple times since it just inserts into the DB.

3. **Direct DB seeding** — after `build_test_router()`, use `app_service.db_service()` to insert a second tenant directly via `TenantRepository::create_tenant()`.

**Recommendation**: Option 3 is simplest — create a helper like:
```rust
pub async fn seed_tenant_b(app_service: &dyn AppService) -> anyhow::Result<()> {
  app_service.tenant_service().create_tenant(
    "test-client-b",
    "test-client-secret-b",
  ).await?;
  // This creates a tenant with auto-generated ID, but we need TEST_TENANT_B_ID...
}
```

**Problem**: `create_tenant()` generates a new ULID for the tenant ID. We need the tenant to have `TEST_TENANT_B_ID` specifically so that `AuthContext::test_session(...).with_tenant_id(TEST_TENANT_B_ID)` matches. Explore whether:
- `AppServiceStubBuilder::with_tenant()` accepts a custom `Tenant` struct (it does — see the signature)
- We can create `Tenant::test_tenant_b()` factory alongside `Tenant::test_default()`
- We need a `Tenant` with `id = TEST_TENANT_B_ID` and a different `client_id`

### Data Seeding Pattern

Route isolation tests need data in the DB for both tenants. Two approaches:

**Approach A: Seed via DB service directly** (like services isolation tests)
```rust
let db = app_service.db_service();
db.create_api_token(TEST_TENANT_ID, &mut token_a).await?;
db.create_api_token(TEST_TENANT_B_ID, &mut token_b).await?;
```

**Approach B: Seed via HTTP POST** (more end-to-end, but complex)
```rust
// Create as tenant A
router.clone().oneshot(
  Request::builder().method("POST").uri("/bodhi/v1/tokens")
    .json(&create_req)?
    .with_auth_context(auth_a)
).await?;
// Create as tenant B
router.clone().oneshot(
  Request::builder().method("POST").uri("/bodhi/v1/tokens")
    .json(&create_req)?
    .with_auth_context(auth_b)
).await?;
```

**Recommendation**: Use Approach B (HTTP POST) where possible — it tests the full create path and ensures the handler correctly assigns tenant_id from AuthContext. Fall back to Approach A for domains where creation has complex prerequisites (e.g., MCP instances require servers, downloads require model repos).

### Response Parsing

Use `ResponseTestExt` from `server_core::test_utils`:
```rust
use server_core::test_utils::ResponseTestExt;
let body = response.json::<PaginatedResponse<TokenResponse>>().await?;
assert_eq!(1, body.data.len());
```

### Auth Context for Different Tenants

```rust
// Tenant A, User A (default)
let auth_tenant_a = AuthContext::test_session(TEST_USER_ID, "user-a@test.com", ResourceRole::Admin);

// Tenant B, same user
let auth_tenant_b = AuthContext::test_session(TEST_USER_ID, "user-b@test.com", ResourceRole::Admin)
  .with_tenant_id(TEST_TENANT_B_ID);

// Same tenant, different user
let auth_user_b = AuthContext::test_session(TEST_TENANT_A_USER_B_ID, "user-b@test.com", ResourceRole::Admin);
```

---

## Middleware Considerations

### Current Tenant Resolution Flow

The middleware resolves tenant_id differently for each auth type:

1. **Session path**: `get_standalone_app()` → gets the single registered tenant → uses its `tenant_id` and `client_id`. Currently assumes single-tenant standalone mode.

2. **API Token path**: Parses `bodhiapp_<random>.<client_id>` → looks up tenant by `client_id` → uses resolved `tenant_id`. Already supports multi-tenant.

3. **External App path**: Uses `get_standalone_app()` for the instance tenant. Currently single-tenant.

### For Isolation Tests: Use Direct Injection

Since route isolation tests focus on **handler logic** (not middleware tenant resolution), use `.with_auth_context()` to inject AuthContext directly. This:
- Bypasses middleware completely
- Lets us set exact `tenant_id` and `user_id`
- Tests that handlers correctly pass auth context to auth-scoped services
- Is the same pattern used by ALL existing route handler tests

Middleware-level multi-tenant resolution tests are a separate concern (see TECHDEBT P0-16).

### Test Utils to Add

Consider adding to `crates/routes_app/src/test_utils/`:

1. **`build_multi_tenant_test_router()`** — or a helper that adds tenant B to an existing router's DB
2. **`Tenant::test_tenant_b()`** — factory for the second test tenant with `id = TEST_TENANT_B_ID`
3. **Standard auth context helpers** for isolation tests:
   ```rust
   fn auth_tenant_a_user_a() -> AuthContext { ... }
   fn auth_tenant_b_user_a() -> AuthContext { ... }  // same user, different tenant
   fn auth_tenant_a_user_b() -> AuthContext { ... }  // same tenant, different user
   ```

---

## Key Files to Read

### Test Infrastructure
- `crates/routes_app/src/test_utils/router.rs` — `build_test_router()`, request builders, session/token helpers
- `crates/routes_app/src/test_utils/auth_context.rs` — `RequestAuthContextExt::with_auth_context()`
- `crates/routes_app/src/test_utils/mod.rs` — all test utility exports
- `crates/routes_app/src/test_utils/assertions.rs` — `assert_auth_rejected()`, `assert_forbidden()`
- `crates/routes_app/src/test_utils/mcp.rs` — MCP-specific test state builders
- `crates/routes_app/TESTING.md` — canonical test patterns
- `crates/routes_app/CLAUDE.md` — crate architecture overview

### Representative Test Files (study these patterns)
- `crates/routes_app/src/tokens/test_tokens_crud.rs` — CRUD with real DB, AppServiceStubBuilder, auth context injection
- `crates/routes_app/src/tokens/test_tokens_auth.rs` — role-based access with session cookies
- `crates/routes_app/src/toolsets/test_toolsets_crud.rs` — toolset CRUD tests
- `crates/routes_app/src/mcps/test_mcps.rs` — MCP tests with MockMcpService
- `crates/routes_app/src/api_models/` — API model CRUD tests

### Middleware (for context, not primary test target)
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — session + bearer token resolution
- `crates/routes_app/src/middleware/token_service/token_service.rs` — API token validation + tenant lookup
- `crates/routes_app/src/shared/auth_scope_extractor.rs` — AuthScope creates AuthScopedAppService

### Services Layer (reference for existing isolation tests)
- `crates/services/src/tokens/test_token_repository_isolation.rs` — token isolation pattern
- `crates/services/src/mcps/test_mcp_repository_isolation.rs` — MCP isolation pattern
- `crates/services/src/test_utils/auth_context.rs` — AuthContext factories with `.with_tenant_id()` / `.with_user_id()`
- `crates/services/src/test_utils/fixtures.rs` — `Tenant::test_default()`, seed helpers

### Domain Route Handler Files (the actual handlers being tested)
- `crates/routes_app/src/tokens/routes_tokens.rs`
- `crates/routes_app/src/mcps/routes_mcps.rs`, `routes_mcps_servers.rs`
- `crates/routes_app/src/toolsets/routes_toolsets.rs`
- `crates/routes_app/src/api_models/routes_api_models.rs`
- `crates/routes_app/src/models/routes_aliases.rs`
- `crates/routes_app/src/users/routes_users.rs`

---

## Open Questions (explore, don't prescribe)

1. **Two-tenant DB setup**: `AppServiceStubBuilder::with_tenant()` accepts a `Tenant` struct. Can we call it twice? If not, what's the best way to insert `TEST_TENANT_B_ID` into the DB? Does `TenantRepository::create_tenant()` allow specifying the ID, or does it auto-generate?

2. **Response types**: Each domain has its own API schema types in `<domain>_api_schemas.rs`. Which response types should isolation tests deserialize into? The paginated wrapper types? Raw `serde_json::Value` for simplicity?

3. **Router cloning**: `tower::oneshot()` consumes the router. Tests need multiple requests (seed + verify). Existing tests use `router.clone().oneshot()`. Confirm this pattern works across all domains.

4. **MCP test complexity**: MCP instances require a server to exist first (FK constraint). MCP auth configs require a server. How deep should the setup go? Use direct DB seeding for prerequisites, HTTP POST for the actual resource under test?

5. **Test file placement**: Should isolation tests live as sibling files in each domain module (e.g., `tokens/test_tokens_isolation.rs` declared from `tokens/mod.rs`), or in a central location? Follow the repository isolation pattern — per-domain files.

6. **Which endpoints need write isolation tests?** Not all domains support all CRUD operations. For read-only endpoints (downloads list, user access requests), cross-tenant GET isolation is sufficient.
