# Fix Multi-Tenant E2E Test Failures (27 failures)

## Context

After commit `30c6d9ff4` (multi-tenant stage 2), 27 E2E tests fail in `--project multi_tenant` while all standalone tests pass. Three root causes identified. Fix iteratively: Batch 1 first (access request — likely root cause of other failures), then re-run, then investigate remaining.

---

## Failure Classification

### Batch 1: Access Request Broken in Multi-Tenant (10+ tests, likely cascades to more)
**Specs**: mcps-auth-restrictions (3), mcps-header-auth (1), mcps-oauth-auth (3), mcps-oauth-dcr (1), oauth2-token-exchange (2), toolsets-auth-restrictions (2)
**Symptom**: `waitForAccessRequestRedirect` timeout, "[object Object]" error on test app
**Root cause**: `apps_create_access_request` calls `get_standalone_app()` → `DbError::MultipleTenant` when >1 tenant. Also: approval handler only matches `AuthContext::Session`, not `MultiTenantSession`.

### Batch 2: MCP Playground/Edit Don't Render (9 tests)
**Specs**: mcps-crud (3), mcps-header-auth (2), mcps-oauth-auth (1), mcps-oauth-dcr (2)
**Symptom**: `data-testid="mcp-playground-page"` not found, edit elements missing
**Root cause**: TBD — reproduce in Chrome after Batch 1 fix

### Batch 3: Toolsets Config Timeout (8 tests)
**Specs**: toolsets-config (4), toolsets-auth-restrictions (4)
**Symptom**: 120s timeout in `beforeEach` (autoResetDb), `page.goto` timeout
**Root cause**: Likely collateral from Batch 1 failures causing dangling PG transactions that block TRUNCATE. `tenants` and `tenants_users` should NOT be reset.

---

## Execution Order

1. **Batch 1**: Fix access request + approval for multi-tenant
2. **Re-run**: `make test.napi.multi_tenant` to see remaining failures
3. **Batch 2**: Reproduce MCP playground issue in Chrome, fix based on findings
4. **Batch 3**: If still failing, investigate separately

---

## Batch 1 Implementation: Fix Access Request for Multi-Tenant

### Design Decisions (user-confirmed)
- External app doesn't know tenant's `client_id` upfront — don't add it to request
- `tenant_id` on `app_access_requests` becomes **nullable** — drafts have NULL
- Tenant binding happens at **approval time** from admin's active tenant
- RLS policy updated: `tenant_id IS NULL OR tenant_id = current_tenant_id()`
- `create_draft()` loses its `tenant_id` parameter entirely
- Keep review URL as-is (id is unique, sufficient for security)
- Remove `clientId` from test-oauth-app ConfigForm

### Step 1: Migration — Make `tenant_id` nullable on `app_access_requests`

**New file**: `crates/services/src/db/sea_migrations/m20250101_000016_app_access_request_nullable_tenant.rs`
```sql
-- PostgreSQL:
ALTER TABLE app_access_requests ALTER COLUMN tenant_id DROP NOT NULL;

-- Update RLS policy:
DROP POLICY IF EXISTS tenant_isolation ON app_access_requests;
CREATE POLICY tenant_isolation ON app_access_requests
  USING (tenant_id IS NULL OR tenant_id = current_tenant_id());
```
- Register migration in `crates/services/src/db/sea_migrations/mod.rs`

**Existing files to reference**:
- Migration pattern: `crates/services/src/db/sea_migrations/m20250101_000014_rls.rs` (RLS policy creation)
- `crates/services/src/db/sea_migrations/m20250101_000015_tenants_users.rs` (latest migration)

### Step 2: Update entity and domain objects

**`crates/services/src/app_access_requests/app_access_request_entity.rs`**
- Change `tenant_id: String` → `tenant_id: Option<String>`

**`crates/services/src/app_access_requests/access_request_objs.rs`**
- Update `AppAccessRequest`: `tenant_id: Option<String>`

### Step 3: Update `create_draft()` service method

**`crates/services/src/app_access_requests/access_request_service.rs`**
- Remove `tenant_id: &str` parameter from `create_draft()`
- Set `tenant_id: sea_orm::ActiveValue::Set(None)` in the ActiveModel
- Use `self.db_service.begin_txn()` instead of `begin_tenant_txn(tenant_id)` (no RLS context for drafts)
- All callers updated to not pass `tenant_id`

### Step 4: Update route handler — draft creation

**`crates/routes_app/src/apps/routes_apps.rs`** (lines 90-108)
- **Remove entirely**: `get_standalone_app()` call and tenant lookup (lines 91-97)
- Call `access_request_service.create_draft(...)` without `tenant_id`
- `build_review_url()` unchanged

### Step 5: Update route handler — approval (CRITICAL BUG FIX)

**`crates/routes_app/src/apps/routes_apps.rs`** (lines 287-424)
- **Line 299-304**: Currently only matches `AuthContext::Session { role, .. }` — BREAKS in multi-tenant (returns `InsufficientPrivileges`)
- Fix: also match `AuthContext::MultiTenantSession { role, .. }`
- Add: `let tenant_id = auth_scope.require_tenant_id()?;`
- Pass `tenant_id` to `approve_request()` so it sets it on the access request row
- Update `approve_request()` in the service to accept and store `tenant_id`

### Step 6: Update `approve_request()` service method

**`crates/services/src/app_access_requests/access_request_service.rs`**
- Add `tenant_id: &str` parameter to `approve_request()` (or adjust existing params)
- When updating the access request row, set `tenant_id = Some(tenant_id.to_string())`
- Use `begin_tenant_txn(tenant_id)` for the approval transaction (RLS-scoped)

### Step 7: Fix test-oauth-app error handling

**`crates/lib_bodhiserver_napi/test-oauth-app/src/lib/api.ts`** (line 17-18)
- Current: `data.message || data.error` — `data.error` is an object → "[object Object]"
- Fix: `data?.error?.message || data?.message || \`Request failed: ${response.status}\``

### Step 8: Update test-oauth-app ConfigForm

**`crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx`**
- Remove `clientId` field from the form (not needed for access request flow)
- The `app_client_id` in the request body should come from the OAuth app's own client_id (already configured)

### Step 9: Regenerate OpenAPI + TypeScript types
```bash
cargo run --package xtask openapi
make build.ts-client
```

### Step 10: Update unit tests

**`crates/routes_app/src/apps/test_access_request.rs`**
- Update `create_draft()` calls: remove `tenant_id` parameter
- Add test: approval with `MultiTenantSession` auth context works
- Add test: approval sets `tenant_id` from admin's auth context
- Update assertions: draft `tenant_id` is `None`

**`crates/services/src/app_access_requests/` test files**
- Update `create_draft()` calls
- Update `approve_request()` calls with new `tenant_id` parameter
- Update assertions for nullable `tenant_id`

### Verification
```bash
cargo check -p services
cargo test -p services --lib
cargo test -p routes_app -- apps
make test.backend
make test.napi.multi_tenant  # Re-run ALL to see which failures remain
```

---

## Batch 2: Reproduce & Fix MCP Playground/Edit (after Batch 1)

### Approach
After Batch 1 fix and re-run, if MCP playground/edit tests still fail:
1. Start multi-tenant server in Chrome
2. Login via OAuth
3. Create MCP server + instance
4. Click playground button → observe behavior
5. Check `/bodhi/v1/info` response
6. Check server logs for auth/dashboard token errors
7. Identify root cause and fix

### Key hypothesis
In `optional_auth_middleware` (auth_middleware.rs:284-287), when resource token is valid but `try_resolve_dashboard_token()` returns None, session is destroyed. Dashboard token IS required in multi-tenant — the question is WHY it's failing/missing.

### Files to investigate
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — optional_auth_middleware
- `crates/routes_app/src/middleware/token_service/token_service.rs` — dashboard token handling
- `crates/routes_app/src/setup/routes_setup.rs` — /info endpoint

---

## Batch 3: Investigate Toolsets Timeout (if still failing after Batch 1)

If toolsets tests still timeout after Batch 1 fix:
- Check if the server is responsive during toolsets test execution
- Check PostgreSQL connection pool state
- Check session DB connectivity
- Check if earlier test failures leave state that prevents TRUNCATE

Note: `tenants` and `tenants_users` should NOT be in `reset_all_tables()` — they are system entities, not user-created data.

---

## Key Files Summary

| File | Change |
|------|--------|
| `services/src/db/sea_migrations/m20250101_000016_*.rs` | NEW: nullable tenant_id + RLS update |
| `services/src/db/sea_migrations/mod.rs` | Register new migration |
| `services/src/app_access_requests/app_access_request_entity.rs` | `tenant_id: Option<String>` |
| `services/src/app_access_requests/access_request_objs.rs` | `tenant_id: Option<String>` |
| `services/src/app_access_requests/access_request_service.rs` | Remove `tenant_id` from `create_draft()`, add to `approve_request()` |
| `routes_app/src/apps/routes_apps.rs` | Remove `get_standalone_app()`, fix approval for `MultiTenantSession` |
| `routes_app/src/apps/test_access_request.rs` | Update tests |
| `test-oauth-app/src/lib/api.ts` | Fix nested error extraction |
| `test-oauth-app/src/components/ConfigForm.tsx` | Remove clientId field |
