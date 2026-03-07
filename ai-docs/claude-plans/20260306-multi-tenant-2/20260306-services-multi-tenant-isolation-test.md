# Multi-Tenant Isolation Tests & Auth-Scoped Service Refactoring — ✅ COMPLETED

## Context

All 14 data tables have `tenant_id` columns with app-layer WHERE filtering (SQLite) and RLS policies (PostgreSQL). However, cross-tenant isolation is only tested at raw DB/RLS level (`test_rls.rs`), not through domain services. Auth-scoped wrappers live in `app_service/` instead of co-located with their domains. Legacy passthrough methods on `AuthScopedAppService` expose raw services bypassing auth scoping.

**Outcome**: Proven cross-tenant + intra-tenant user isolation at domain service layer, parameterized across SQLite+PG. Clean auth-scoped architecture with files in domain folders.

---

## Execution Model

Each phase is executed by a sub-agent. Gate checks before commit:
1. `cargo check -p services` (src-compile)
2. `cargo test -p services --lib --no-run` (test-compile)
3. `cargo test -p services --lib` (test-pass)
4. If routes_app affected: `cargo check -p routes_app` + `cargo test -p routes_app --lib`
5. Local commit with conventional commit message

Fix downstream crates immediately after each breaking services change (within same phase).

---

## Phase 0: Test Infrastructure

**Commit**: `test(services): add multi-tenant test constants and AuthContext builders`

### Step 0.1: Promote shared test constants

**File**: `crates/services/src/test_utils/db.rs` — Add:
```rust
pub const TEST_TENANT_B_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBB";
pub const TEST_TENANT_A_USER_B_ID: &str = "test-tenant-a-user-b";
```

**File**: `crates/services/src/db/test_rls.rs` — Remove local `TENANT_B_ID`, import `TEST_TENANT_B_ID` from `crate::test_utils`.

### Step 0.2: Add `with_tenant_id()` and `with_user_id()` builders

**File**: `crates/services/src/test_utils/auth_context.rs` — Add:
```rust
impl AuthContext {
  pub fn with_tenant_id(self, tenant_id: &str) -> Self { /* match all variants, replace tenant_id */ }
  pub fn with_user_id(self, user_id: &str) -> Self { /* match Session/ApiToken/ExternalApp, replace user_id */ }
}
```

### Gate Check
```bash
cargo check -p services 2>&1 | tail -5
cargo test -p services --lib -- test_rls 2>&1 | grep -E "test result|FAILED"
```

---

## Phase 1: Move Auth-Scoped Files to Domain Folders

**Commit**: `refactor(services): move auth_scoped services to domain folders`

For each file: `git mv`, add `mod` + `pub use` in target `mod.rs`, remove from `app_service/mod.rs`.

| Source | Target | Domain |
|--------|--------|--------|
| `app_service/auth_scoped_tokens.rs` | `tokens/auth_scoped.rs` | `tokens/mod.rs` |
| `app_service/auth_scoped_mcps.rs` | `mcps/auth_scoped.rs` | `mcps/mod.rs` |
| `app_service/auth_scoped_tools.rs` | `toolsets/auth_scoped.rs` | `toolsets/mod.rs` |
| `app_service/auth_scoped_users.rs` | `users/auth_scoped.rs` | `users/mod.rs` |
| `app_service/auth_scoped_user_access_requests.rs` | `users/auth_scoped_access_requests.rs` | `users/mod.rs` |
| `app_service/auth_scoped_data.rs` | `models/auth_scoped_data.rs` | `models/mod.rs` |
| `app_service/auth_scoped_api_models.rs` | `models/auth_scoped_api_models.rs` | `models/mod.rs` |
| `app_service/auth_scoped_downloads.rs` | `models/auth_scoped_downloads.rs` | `models/mod.rs` |

After all moves, `app_service/mod.rs` retains only:
```rust
mod app_service;
mod auth_scoped;
pub use app_service::*;
pub use auth_scoped::*;
```

### Gate Check
```bash
cargo check -p services -p routes_app 2>&1 | tail -5
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"
```

---

## Phase 2: Remove Legacy Passthroughs

**Commit**: `refactor: remove legacy service passthroughs from AuthScopedAppService`

### Step 2.1: Remove zero-call passthroughs from services

**File**: `crates/services/src/app_service/auth_scoped.rs` — Remove:
- `mcp_service()` (0 production calls)
- `token_service()` (0 production calls)
- `data_service()` (0 production calls)

Check `cargo check -p routes_app` — if any test code breaks, fix those tests in routes_app to use auth-scoped methods or `app_service()`.

### Step 2.2: Migrate `tool_service()` callers, then remove

**File**: `crates/routes_app/src/toolsets/routes_toolsets.rs` — 4 call sites all use `tool_service().get_type()` / `.list_types()` which are passthrough methods already on `AuthScopedToolService`. Change to `auth_scope.tools().get_type()` etc.

**File**: `crates/services/src/app_service/auth_scoped.rs` — Remove `tool_service()`.

### Gate Check
```bash
cargo check -p services -p routes_app 2>&1 | tail -5
cargo test -p services --lib -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
```

---

## Phase 3: Access Request Service Refactor

**Commit**: `refactor: access request service — remove AuthScoped passthrough, inline build_review_url`

### Design Decision: Non-Auth-Scoped Passthrough (documented)

`AccessRequestService` is intentionally NOT auth-scoped because:
- `create_draft`: Anonymous endpoint, tenant_id from DB lookup, no user_id on record
- `get_request`: Used by both anonymous (status polling) and session (review) — no user_id filtering
- `approve_request`/`deny_request`: user_id is the *reviewer*, added to record at approval time, not a scope filter

The method stays on `AuthScopedAppService` as a convenience passthrough (`self.app_service.access_request_service()`), but with a clear doc comment noting it is non-auth-scoped.

### Step 3.1: Add doc comment to AccessRequestService trait

**File**: `crates/services/src/app_access_requests/access_request_service.rs`

Add comment above trait:
```rust
/// App access request lifecycle service.
///
/// NOTE: This service is intentionally NOT auth-scoped. Unlike other domain services where
/// tenant_id/user_id scope which records are visible, app access requests have a different
/// lifecycle:
/// - create_draft: Anonymous (no authenticated user), tenant_id from DB lookup
/// - get_request: Used by both anonymous status polling and authenticated review
/// - approve/deny: reviewer's user_id is recorded as actor, not used as scope filter
///
/// Exposed on AuthScopedAppService as a non-auth-scoped passthrough for convenience.
```

### Step 3.2: Document the passthrough on AuthScopedAppService

**File**: `crates/services/src/app_service/auth_scoped.rs` — Keep `access_request_service()` but add doc comment:
```rust
/// Non-auth-scoped passthrough. See [`AccessRequestService`] doc comment for rationale.
/// All methods on this service manage their own tenant/user context — they are not
/// filtered by AuthContext's tenant_id/user_id.
pub fn access_request_service(&self) -> Arc<dyn AccessRequestService> {
  self.app_service.access_request_service()
}
```

No routes_app migration needed — existing `auth_scope.access_request_service()` calls remain valid.

### Step 3.4: Refactor build_review_url — store in entity

Currently `build_review_url(id)` is a separate method that computes a URL on the fly. Refactor:
1. Compute the review URL during `create_draft()` and store it in the `AppAccessRequest` entity (add `review_url` field if not present)
2. Return it as part of the serialized response
3. Remove the standalone `build_review_url()` method from the trait
4. Update routes_app callers that currently call `build_review_url()` separately

**Files**:
- `crates/services/src/app_access_requests/access_request_service.rs` — Compute + store URL in create_draft
- `crates/services/src/app_access_requests/access_request_objs.rs` — Add `review_url` to AppAccessRequest if needed
- Migration if DB column needed, OR compute at serialization time (explore which is simpler)
- `crates/routes_app/src/apps/routes_apps.rs` — Remove separate build_review_url call, use field from response

### Gate Check
```bash
cargo check -p services -p routes_app 2>&1 | tail -5
cargo test -p services --lib -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
```

---

## Phase 4: Cross-Tenant & Intra-Tenant Isolation Tests

**Commit**: `test(services): add cross-tenant and intra-tenant user isolation tests`

All tests parameterized with `#[values("sqlite", "postgres")]` + `#[serial(pg_app)]`. Test domain services directly with explicit `tenant_id`/`user_id` params.

### Test Pattern
```rust
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_cross_tenant_<domain>_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  // 1. Create resource in TEST_TENANT_ID for TEST_USER_ID
  // 2. Create resource in TEST_TENANT_B_ID for TEST_USER_ID (SAME user, different tenant)
  // 3. List(TEST_TENANT_ID, TEST_USER_ID) -> only tenant A's resource
  // 4. List(TEST_TENANT_B_ID, TEST_USER_ID) -> only tenant B's resource
  // 5. Get-by-ID across tenant boundary -> None
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_intra_tenant_user_<domain>_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  // 1. Create resource in TEST_TENANT_ID for TEST_USER_ID
  // 2. Create resource in TEST_TENANT_ID for TEST_TENANT_A_USER_B_ID (SAME tenant, different user)
  // 3. List(TEST_TENANT_ID, TEST_USER_ID) -> only user A's resource
  // 4. List(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID) -> only user B's resource
  Ok(())
}
```

### Isolation Test Files

| New File | Domain | Scope | Mod Decl In |
|----------|--------|-------|-------------|
| `tokens/test_token_service_isolation.rs` | Tokens | tenant + user | `tokens/mod.rs` |
| `mcps/test_mcp_service_isolation.rs` | MCPs + MCP servers | tenant + user (instances), tenant (servers) | `mcps/mod.rs` |
| `toolsets/test_toolset_isolation.rs` | Toolsets | tenant + user | `toolsets/mod.rs` |
| `models/test_data_service_isolation.rs` | User aliases + API models | tenant + user | `models/mod.rs` |
| `models/test_download_isolation.rs` | Downloads | tenant only | `models/mod.rs` |
| `users/test_access_request_isolation.rs` | User access requests | tenant only | `users/mod.rs` |
| `app_access_requests/test_isolation.rs` | App access requests | tenant only | `app_access_requests/mod.rs` |

### Global Resource Negative Tests

| New File | Domain | Assertion | Mod Decl In |
|----------|--------|-----------|-------------|
| `settings/test_settings_global.rs` | Settings | Settings readable regardless of tenant context (NOT isolated per D9) | `settings/mod.rs` |
| `models/test_model_metadata_global.rs` | Model metadata | Verify global vs tenant-scoped behavior, document | `models/mod.rs` |

### Gate Check
```bash
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"
# PG tests (if Docker available):
# INTEG_TEST_APP_DB_PG_URL="..." cargo test -p services --lib -- isolation 2>&1 | grep -E "test result|FAILED"
```

---

## Phase 5: Routes_App Audit

**Commit**: `chore(routes_app): audit and document auth-scoped service usage`

Grep-audit all route handlers for remaining raw domain service access:
```bash
grep -rn '\.token_service()\|\.mcp_service()\|\.tool_service()\|\.data_service()' crates/routes_app/src/
```

Expected: 0 matches for these on `AuthScopedAppService`. `access_request_service()` remains on `AuthScopedAppService` as a documented non-auth-scoped passthrough.

---

## Key Files Reference

| File | Role |
|------|------|
| `crates/services/src/app_service/auth_scoped.rs` | Central wrapper — remove passthroughs |
| `crates/services/src/app_service/mod.rs` | Module decls — remove auth_scoped_* after moves |
| `crates/services/src/test_utils/db.rs` | TEST_TENANT_B_ID, TEST_TENANT_A_USER_B_ID |
| `crates/services/src/test_utils/auth_context.rs` | with_tenant_id(), with_user_id() |
| `crates/services/src/test_utils/sea.rs` | sea_context("sqlite"/"postgres") — reuse |
| `crates/services/src/db/test_rls.rs` | Update to shared constant |
| `crates/services/src/app_access_requests/access_request_service.rs` | Doc comment, build_review_url refactor |
| `crates/routes_app/src/toolsets/routes_toolsets.rs` | Migrate tool_service() calls to tools() |

## Existing Utilities to Reuse

- `sea_context(db_type)` — `crates/services/src/test_utils/sea.rs` — dual-backend test context
- `begin_tenant_txn(tenant_id)` — `DbCore` trait — RLS-aware transactions
- `#[serial(pg_app)]` — serialize PG tests
- `AuthContext::test_session(user_id, username, role)` — base factory, chain with `.with_tenant_id()`
- `FrozenTimeService` — auto-provided by test fixtures
