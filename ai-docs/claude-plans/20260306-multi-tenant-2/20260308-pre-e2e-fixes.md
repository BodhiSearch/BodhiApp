# Pre-E2E Fixes — Implementation Plan

## Context

Implementation gaps were discovered during a doc-vs-code audit of the multi-tenant feature (M2/M3). These must be resolved before E2E tests can exercise the full multi-tenant flow. Additionally, the session token namespace convention is being corrected to group tokens by tenant (`{client_id}:token_type`) rather than by token type (`token_type:{client_id}`).

Five fix items plus one infrastructure refactor, executed in three batches by sub-agents.

---

## Batch 0: Session Token Namespace Refactoring (Sub-agent)

**Run first — all subsequent batches depend on this.**

### What changes

1. **Flip session key format**: `access_token:{client_id}` → `{client_id}:access_token`, same for refresh_token
2. **Flip lock key**: `refresh_token:{session_id}:{client_id}` → `{client_id}:{session_id}:refresh_token` (template: `{client_id}:{session_id}:<lock_type>` — reusable for future locks)
3. **Dashboard keys unchanged**: `dashboard:access_token`, `dashboard:refresh_token` stay as-is
4. **Global keys unchanged**: `user_id`, `active_client_id` stay as-is
5. **Remove dead code**:
   - `SESSION_KEY_ACCESS_TOKEN`, `SESSION_KEY_REFRESH_TOKEN` (legacy non-namespaced, unused)
   - `id_token_key()` in auth_middleware.rs
   - `DASHBOARD_ID_TOKEN_KEY` in tenants/mod.rs
6. **Move session key definitions to services crate** (new module `session_keys.rs`)
7. **Create `TestSessionBuilder`** in `services::test_utils` + `inject_session()` helper
8. **Migrate test session setup** in `server_app/tests/` to use TestSessionBuilder

### Step-by-step

**Step 0.1 — New module `crates/services/src/session_keys.rs`**

```rust
pub const SESSION_KEY_USER_ID: &str = "user_id";
pub const SESSION_KEY_ACTIVE_CLIENT_ID: &str = "active_client_id";
pub const DASHBOARD_ACCESS_TOKEN_KEY: &str = "dashboard:access_token";
pub const DASHBOARD_REFRESH_TOKEN_KEY: &str = "dashboard:refresh_token";

pub fn access_token_key(client_id: &str) -> String {
    format!("{client_id}:access_token")
}
pub fn refresh_token_key(client_id: &str) -> String {
    format!("{client_id}:refresh_token")
}
```

Wire in `services/src/lib.rs`: `mod session_keys; pub use session_keys::*;`

**Step 0.2 — TestSessionBuilder in `crates/services/src/test_utils/session.rs`** (extend existing file)

Builder API:
```rust
TestSessionBuilder::default()
    .user_id("user-123")
    .dashboard("dash_access", "dash_refresh")
    .add_tenant("client1", "access1", "refresh1")
    .add_tenant("client2", "access2", "refresh2")
    .activate("client1")
    .add("oauth_state", Value::String("abc".into()))  // generic key-value
    .build()  // -> HashMap<String, serde_json::Value>
```

`inject_session(session_service: &dyn SessionService, data: HashMap<String, Value>) -> Result<String>` — creates Record, saves to store, returns session_id.

**Step 0.3 — Remove definitions from `crates/routes_app/src/middleware/auth/auth_middleware.rs`**
- Remove lines 18-36 (all constants + 3 functions)
- File already imports from `services::` — update imports to include session key symbols
- In `middleware/auth/mod.rs`: re-export from services for backward compat within routes_app

**Step 0.4 — Remove from `crates/routes_app/src/tenants/mod.rs`**
- Remove 3 `DASHBOARD_*` constants (lines 13-15)
- Re-export `DASHBOARD_ACCESS_TOKEN_KEY` and `DASHBOARD_REFRESH_TOKEN_KEY` from services

**Step 0.5 — Update lock key in `crates/routes_app/src/middleware/token_service/token_service.rs`**
- Change `format!("refresh_token:{}:{}", session_id, client_id)` → `format!("{}:{}:refresh_token", client_id, session_id)`

**Step 0.6 — Update all import paths** across routes_app files that use session key functions. The `access_token_key()` and `refresh_token_key()` call sites don't change (same function names), just import sources.

**Step 0.7 — Migrate test session setup in `crates/server_app/tests/utils/live_server_utils.rs`**
- `create_authenticated_session()` and `create_test_session_for_live_server()` → use TestSessionBuilder + inject_session
- Update imports from `routes_app::middleware::` → `services::`

### Files changed

| File | Change |
|------|--------|
| `crates/services/src/session_keys.rs` | **NEW** — key constants + format functions |
| `crates/services/src/lib.rs` | Add `mod session_keys; pub use session_keys::*;` |
| `crates/services/src/test_utils/session.rs` | Add TestSessionBuilder + inject_session |
| `crates/services/src/test_utils/mod.rs` | Already exports `session::*` |
| `crates/routes_app/src/middleware/auth/auth_middleware.rs` | Remove key defs, update imports |
| `crates/routes_app/src/middleware/auth/mod.rs` | Re-export from services |
| `crates/routes_app/src/middleware/token_service/token_service.rs` | Flip lock key format |
| `crates/routes_app/src/tenants/mod.rs` | Remove DASHBOARD constants, re-export from services |
| `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` | Update hardcoded key strings |
| `crates/server_app/tests/utils/live_server_utils.rs` | Migrate to TestSessionBuilder |

### Gate check
```bash
cargo test -p services -p routes_app 2>&1 | grep -E "test result|FAILED"
```

---

## Batch 1: Items 1–2 (Sub-agent)

### Item 1: BODHI_DEPLOYMENT value rename

`"multi-tenant"` → `"multi_tenant"` (snake_case consistency).

**Backend** — grep all `.rs` for `"multi-tenant"` literal string, change to `"multi_tenant"`:
- `crates/services/src/settings/setting_service.rs` ~line 255 — `is_multi_tenant()` comparison
- `crates/server_app/tests/utils/live_server_utils.rs` — setup values
- `crates/server_app/tests/test_live_multi_tenant.rs` — assertions
- `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` — test params
- `crates/routes_app/src/tenants/test_dashboard_auth.rs` — mock returns
- `crates/routes_app/tests/test_live_multi_tenant.rs` — env var values

**Frontend** — grep `.tsx`/`.ts` for `'multi-tenant'`:
- `crates/bodhi/src/app/ui/login/page.tsx` ~line 348
- `crates/bodhi/src/components/AppInitializer.tsx` ~line 58

**Note**: Only change programmatic string literals. English prose in comments ("multi-tenant mode") stays.

**Regenerate**: `cargo run --package xtask openapi && cd ts-client && npm run generate`

### Item 2: Move `/info` to `optional_auth` router

- `crates/routes_app/src/routes.rs`: remove `ENDPOINT_APP_INFO` from `public_apis`, add to `optional_auth` Router
- `/info` still works without auth (optional_auth falls back to Anonymous), but now populates `AuthContext` when auth is present → standalone authenticated users get `client_id` in response
- Keep `ENDPOINT_APP_INFO` in `public_endpoints` list in `openapi.rs` GlobalErrorResponses (still accessible without auth)
- Test expectations in `test_setup.rs` should still pass (anonymous request → `client_id: None`)

### Gate check
```bash
cargo test -p services -p routes_app 2>&1 | grep -E "test result|FAILED"
```

---

## Batch 2: Items 3, 4, 6 (Sub-agent)

### Item 3: Standalone `created_by` fix

Replace `update_status()` + `update_created_by()` with a single `set_client_ready(client_id, user_id)` method.

**Verified**: `tenant_svc.update_status(&AppStatus::Ready)` is only called at `routes_auth.rs:273` (standalone auth_callback). Safe to replace.

**Service layer** — `crates/services/src/tenants/tenant_service.rs`:
- Remove `update_status(&self, status: &AppStatus)` from trait + impl (lines 24, 75-85)
- Remove `update_created_by(&self, client_id, created_by)` from trait + impl (lines 27-28, 99-105)
- Add `set_client_ready(&self, client_id: &str, user_id: &str)` — atomically sets status=Ready + created_by=user_id via two DB calls (standalone-only, acceptable without single-query atomicity)
- Keep `update_status_by_id()` — used elsewhere

**Route handler** — `crates/routes_app/src/auth/routes_auth.rs`:
- Line 273: `tenant_svc.update_status(&AppStatus::Ready).await?;` → `tenant_svc.set_client_ready(&instance.client_id, &user_id).await?;`

**Tests**:
- `crates/services/src/tenants/test_tenant_service.rs`: rewrite `test_update_status_*` tests for `set_client_ready`
- `crates/routes_app/src/auth/test_login_resource_admin.rs`: update mock expectations from `update_status` → `set_client_ready`

### Item 4: Dashboard endpoints in OpenAPI + frontend hooks

**OpenAPI** — `crates/routes_app/src/shared/openapi.rs`:
- Import `__path_dashboard_auth_initiate` and `__path_dashboard_auth_callback`
- Add to `paths(...)` section
- Verify `routes_dashboard_auth.rs` utoipa tags match convention (should use `API_TAG_TENANTS`)

**Regenerate**: `cargo run --package xtask openapi && cd ts-client && npm run generate`

**Frontend** — `crates/bodhi/src/hooks/useAuth.ts`:
- Update `useDashboardOAuthInitiate` and `useDashboardOAuthCallback` to use generated client types AND functions from `@bodhiapp/ts-client`
- Check existing hooks for consistent pattern (how other hooks use the generated client)

### Item 6: CreateTenantRequest.description optional

- `crates/routes_app/src/tenants/tenant_api_schemas.rs`: change `description: String` → `description: Option<String>` with `#[serde(default)]`
- Handler in `routes_tenants.rs`: pass `description.unwrap_or_default()` to SPI call
- Remove or adjust validation for description (optional field, no min length)

**Regenerate**: `cargo run --package xtask openapi && cd ts-client && npm run generate`

### Gate check
```bash
cargo test -p services -p routes_app 2>&1 | grep -E "test result|FAILED"
```

---

## Batch 3: Project Doc Updates (Sub-agent or manual)

Update `ai-docs/claude-plans/20260306-multi-tenant-2/` docs for implemented changes. Functional updates only, matching existing language style.

### TECHDEBT.md — remove resolved items
- Remove: "Standalone `created_by` Not Set During ResourceAdmin -> Ready" (fixed by Item 3)
- Remove: "Dashboard Auth Endpoints Missing from OpenAPI Spec" (fixed by Item 4)
- Remove: "`/info` Not Behind `optional_auth_middleware`" (fixed by Item 2)
- Remove: "ID Token Session Keys Unused" (dead code removed in Batch 0)
- Update: "Multi-Tenant-Aware Logout (D63)" — session key format is now `{client_id}:access_token` (not `access_token:{client_id}`)
- Update: "Navigation Item Visibility (F7)" — deployment value is now `'multi_tenant'` (not `'multi-tenant'`)

### multi-tenant-flow-ctx.md — update for accuracy
- Deployment modes table: `multi-tenant` → `multi_tenant`
- Session architecture section: key format is now `{client_id}:access_token`
- Any references to `access_token:{client_id}` → `{client_id}:access_token`
- Grep other docs in the folder for `"multi-tenant"`, `access_token:{client_id}`, `update_status`, `update_created_by` and update references

### Crate-level CLAUDE.md updates
- `crates/routes_app/src/middleware/CLAUDE.md` — session key format section, remove id_token_key reference
- `crates/services/CLAUDE.md` — update TenantService methods: `set_client_ready` replaces `update_status` + `update_created_by`

---

## Execution Strategy

```
Batch 0 (sub-agent): Token namespace flip + TestSessionBuilder + dead code cleanup
    → cargo test -p services -p routes_app
    ↓
Batch 1 (sub-agent): Item 1 (BODHI_DEPLOYMENT rename) + Item 2 (/info optional_auth)
    → cargo test -p services -p routes_app
    ↓
Batch 2 (sub-agent): Item 3 (set_client_ready) + Item 4 (OpenAPI + hooks) + Item 6 (description optional)
    → cargo test -p services -p routes_app
    ↓
Batch 3 (sub-agent): Doc updates + full validation
    → make test.backend + cd crates/bodhi && npm test + make ci.ts-client-check
```

---

## Final Validation (Batch 3 sub-agent runs these)

```bash
make test.backend
cd crates/bodhi && npm test
make ci.ts-client-check
```
