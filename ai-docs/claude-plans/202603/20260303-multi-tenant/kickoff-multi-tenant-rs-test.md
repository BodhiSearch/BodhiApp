# Multi-Tenant Isolation Tests — Kickoff Prompt

## Goal

Add cross-tenant isolation tests that verify tenant A cannot see tenant B's data. Currently all tests use a single `TEST_TENANT_ID` — we need tests with two tenants proving data isolation actually works.

Additionally, audit and tighten the `AuthScopedAppService` surface: remove direct service accessors that let route handlers bypass auth-scoped wrappers.

This prompt covers the **services** crate only. See companion kickoff files for other layers:
- `kickoff-multi-tenant-routes-app-test.md` — routes_app HTTP-layer isolation
- `kickoff-multi-tenant-integration-test.md` — server_app end-to-end integration

---

## Context: What Exists Today

### Multi-Tenant Architecture
- All 14 data tables have `tenant_id TEXT NOT NULL`; alias tables also have `user_id TEXT NOT NULL`
- Repository methods accept `tenant_id: &str` (and `user_id: &str` for aliases) and filter via WHERE clauses
- PostgreSQL additionally has RLS policies as defense-in-depth (`begin_tenant_txn` does `SET LOCAL app.current_tenant_id`)
- SQLite relies solely on application-layer WHERE filtering (no RLS)
- Decision D4: unified schema — standalone is one tenant, multi-tenant is multiple tenants
- Decision D7: app-layer filtering primary, PostgreSQL RLS secondary

### Auth-Scoped Services (the isolation boundary)
8 auth-scoped wrappers inject `tenant_id`/`user_id` from `AuthContext` into underlying service calls:

| Wrapper | Service | Scope |
|---------|---------|-------|
| `AuthScopedTokenService` | `TokenService` | tenant_id + user_id |
| `AuthScopedMcpService` | `McpService` | tenant_id + user_id |
| `AuthScopedToolService` | `ToolsetService` | tenant_id + user_id |
| `AuthScopedUserService` | `UserService` | tenant_id (via token) |
| `AuthScopedDataService` | `DataService` | tenant_id + user_id |
| `AuthScopedApiModelService` | `ApiModelService` | tenant_id + user_id |
| `AuthScopedDownloadService` | `DownloadService` | tenant_id |
| `AuthScopedUserAccessRequestService` | `UserAccessRequestService` | tenant_id |

Read ops use `unwrap_or("")` for anonymous access; write ops use `require_tenant_id()`/`require_user_id()`.

### Existing Test Infrastructure
- `TEST_TENANT_ID = "01ARZ3NDEKTSV4RRFFQ69G5FAV"` and `TEST_USER_ID = "test-user"` in `services/src/test_utils/db.rs`
- `test_db_service` rstest fixture: in-memory SQLite with fresh migrations + `FrozenTimeService`
- `AuthContext::test_session(user_id, username, role)` — always uses `TEST_TENANT_ID`
- RLS tests exist in `services/src/db/test_rls.rs` using `TENANT_B_ID = "01ARZ3NDEKTSV4RRFFQ69G5FBB"`
- The RLS tests demonstrate the two-tenant pattern but only at the raw SQL level, not through services

### What's NOT Auth-Scoped (passthrough on AuthScopedAppService)
These accessors on `AuthScopedAppService` bypass auth scoping — route handlers can call them directly:
- `.db()` / `.db_service()` — raw database access
- `.settings()` / `.setting_service()` — global settings (intentionally global per D9)
- `.tenant()` / `.tenant_service()` — tenant management
- `.auth_flow()` / `.auth_service()` — OAuth flow
- `.network()` / `.network_service()` — network
- `.sessions()` / `.session_service()` — sessions
- `.hub()` / `.hub_service()` — hub queries
- `.ai_api()` / `.ai_api_service()` — AI API
- `.time()` / `.time_service()` — time
- `.inference()` / `.inference_service()` — inference routing
- Also legacy passthroughs: `data_service()`, `mcp_service()`, `tool_service()`, `token_service()`, etc.

**Concern**: The legacy passthrough methods (`data_service()`, `mcp_service()`, `tool_service()`, `token_service()`, `access_request_service()`) expose underlying services without auth scoping. Route handlers could accidentally call these instead of the auth-scoped versions. These should be removed or restricted.

### Relevant Plan Files (read these for full context)
- `ai-docs/claude-plans/20260303-multi-tenant/20260303-multi-tenant-rls.md` — RLS architecture
- `ai-docs/claude-plans/20260303-multi-tenant/20260303-crud-uniformity.md` — entity types, service structure
- `ai-docs/claude-plans/20260303-multi-tenant/decisions.md` — D4 (unified schema), D7 (app-layer + RLS), D9 (settings global)
- `ai-docs/claude-plans/20260303-multi-tenant/SUMMARY.md` — P1-2 (missing AuthScoped wrappers), P1-12 (tenant_id_or_empty inconsistency)
- `ai-docs/claude-plans/20260303-multi-tenant/TECHDEBT.md` — P2-8 (cross-tenant integration tests)

---

## Session 1: services crate

### Part A: Auth-Scoped Service Isolation Tests

For each auth-scoped service, write parameterized tests proving cross-tenant isolation:

**Test pattern** (exploratory — discover the right shape):
1. Create two tenants in the DB (use `TEST_TENANT_ID` and a second `TENANT_B_ID`)
2. Create resources under both tenants via the underlying service (raw, with explicit tenant_id)
3. Build an `AuthContext` for tenant A
4. Call the auth-scoped service methods
5. Assert: only tenant A's resources are returned; tenant B's resources are invisible
6. Repeat with `AuthContext` for tenant B to verify symmetry

**Parameterize** with `#[values("sqlite", "postgres")]` if the test infrastructure supports it, or at minimum run on SQLite (where WHERE-clause isolation is the only protection).

**Domains to cover** (each auth-scoped service):
- Tokens: list returns only tenant's tokens; create under tenant A, invisible to tenant B
- MCPs: list/get scoped to tenant; MCP servers, auth headers, OAuth configs all isolated
- Toolsets: list/get/execute scoped to tenant+user
- API Models: list/get scoped to tenant+user; CRUD isolated
- Downloads: list/get scoped to tenant; create isolated
- Data (aliases): list/find scoped to tenant+user; copy/delete isolated
- User Access Requests: list/get scoped to tenant

**Key question to explore**: Should the `unwrap_or("")` pattern for anonymous reads return ALL data across tenants (empty string = no filter) or NOTHING? What does the current WHERE clause do with `tenant_id = ""`? If it returns nothing, good. If it bypasses filtering, that's a security gap.

### Part B: Tighten AuthScopedAppService Surface

Explore removing legacy passthrough methods from `AuthScopedAppService`:
- `data_service()`, `mcp_service()`, `tool_service()`, `token_service()`, `access_request_service()` — these expose raw services without auth scoping
- Check if any production code (not test code) calls these. If only tests, migrate tests to use auth-scoped versions
- The infrastructure passthroughs (`.db()`, `.settings()`, `.time()`, etc.) are intentionally un-scoped and should remain

### Key Files to Read First
- `crates/services/src/app_service/auth_scoped.rs` — the central wrapper, see all accessors
- `crates/services/src/app_service/auth_scoped_*.rs` — each domain's scoping logic
- `crates/services/src/db/test_rls.rs` — existing two-tenant test pattern
- `crates/services/src/test_utils/db.rs` — TEST_TENANT_ID, test_db_service fixture
- `crates/services/src/test_utils/auth_context.rs` — AuthContext factories

### Testing Conventions
- Use rstest: `#[rstest]` with `#[case]` for parameterized, `#[fixture]` for setup
- `assert_eq!(expected, actual)` with `pretty_assertions`
- Error assertions via `.code()`, never message text
- Test file organization: `test_*.rs` sibling files via `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`
- No if-else logic in test code

---

## Open Questions (explore, don't prescribe)

1. **Two-tenant fixture**: What's the right shape? A fixture that returns `(TestDbService, tenant_a_id, tenant_b_id)` with both tenants seeded in the DB? Or separate tenant creation helpers?

2. **User isolation within tenant**: Should we also test that user A within tenant X can't see user B's aliases (for user-scoped resources like `api_model_aliases`)? This is a finer-grained isolation than tenant-level.

3. **Empty tenant_id behavior**: What happens when `tenant_id = ""` is passed to repository methods? Does it match rows with empty tenant_id (potentially dangerous) or return nothing? This determines whether the `unwrap_or("")` pattern in read ops is safe.

4. **Settings and metadata**: Per decisions D9 and P1-8, settings and model_metadata are intentionally global. Should isolation tests explicitly verify these are NOT filtered by tenant?

5. **PostgreSQL test feasibility**: The existing RLS tests use Docker PostgreSQL. Can we reuse that infrastructure for service-level isolation tests, or should we focus on SQLite (where app-layer filtering is the only defense)?
