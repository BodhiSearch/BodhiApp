# Plan: Full RLS Enforcement with Internal Transaction Management

## Context

The recent RLS refactor added `txn: &DatabaseTransaction` parameters to all mutating repository trait methods, leaking transaction lifecycle to callers. Additionally, read operations bypass RLS entirely (use `&self.db` directly).

**Goals**:
1. Internalize transaction management — no `txn` parameter in any repository trait method
2. Full RLS enforcement — both reads AND writes use `with_tenant_txn` so PostgreSQL RLS policies apply to all operations on tenant-scoped tables
3. Reduce boilerplate via a `with_tenant_txn` helper on `DefaultDbService`

**Accepted trade-off**: Multi-step MCP operations (update+delete OAuth token, delete+create OAuth token) become non-atomic — each repository call has its own txn.

## Step 0: Add `with_tenant_txn` Helper

**File**: `services/src/db/default_service.rs`

Add to `impl DefaultDbService`:

```rust
pub async fn with_tenant_txn<T, F>(&self, tenant_id: &str, f: F) -> Result<T, DbError>
where
    F: for<'a> FnOnce(&'a DatabaseTransaction) -> Pin<Box<dyn Future<Output = Result<T, DbError>> + Send + 'a>>,
    T: Send,
{
    let txn = self.begin_tenant_txn(tenant_id).await?;
    let result = f(&txn).await?;
    txn.commit().await.map_err(DbError::from)?;
    Ok(result)
}
```

Usage pattern at call sites:
```rust
self.with_tenant_txn(tenant_id, |txn| Box::pin(async move {
    model.insert(txn).await.map_err(DbError::from)
})).await
```

## Step 1: Write Methods — Remove `txn` parameter, use `with_tenant_txn`

Remove `txn: &DatabaseTransaction` from all mutation method signatures in these traits, and wrap impl bodies in `self.with_tenant_txn(...)`:

| Trait File | Methods |
|------------|---------|
| `tokens/token_repository.rs` | `create_api_token`, `update_api_token` |
| `toolsets/toolset_repository.rs` | `create_toolset`, `update_toolset`, `delete_toolset`, `set_app_toolset_enabled` |
| `mcps/mcp_server_repository.rs` | `create_mcp_server`, `update_mcp_server`, `clear_mcp_tools_by_server_id` |
| `mcps/mcp_instance_repository.rs` | `create_mcp`, `update_mcp`, `delete_mcp` |
| `mcps/mcp_auth_repository.rs` | All 11 mutation methods |
| `models/api_alias_repository.rs` | `create_api_model_alias`, `update_api_model_alias`, `update_api_model_cache`, `delete_api_model_alias` |
| `models/download_repository.rs` | `create_download_request`, `update_download_request` |
| `models/user_alias_repository.rs` | `create_user_alias`, `update_user_alias`, `delete_user_alias` |
| `models/model_metadata_repository.rs` | `upsert_model_metadata` |
| `users/access_repository.rs` | `insert_pending_request`, `update_request_status` |
| `app_access_requests/access_request_repository.rs` | `create`, `update_approval`, `update_denial`, `update_failure`, `validate_draft_for_update` |

All paths relative to `crates/services/src/`.

**SQLite caveat**: All DB operations within the closure must use `txn`, never `&self.db` (SQLite single-writer lock → deadlock). Already handled in Phase 3 for `api_alias_repository.rs`.

## Step 2: Read Methods — Wrap existing reads in `with_tenant_txn`

Read methods already have `tenant_id` parameters. Change impls to use `with_tenant_txn` internally (no trait signature change needed — just impl body change from `query.one(&self.db)` to `self.with_tenant_txn(tenant_id, |txn| Box::pin(async move { query.one(txn) }))`).

| Trait File | Read Methods to Wrap |
|------------|---------------------|
| `tokens/token_repository.rs` | `list_api_tokens`, `get_api_token_by_id` |
| `toolsets/toolset_repository.rs` | `get_toolset`, `get_toolset_by_slug`, `list_toolsets`, `list_toolsets_by_toolset_type`, `get_toolset_api_key`, `list_app_toolset_configs`, `get_app_toolset_config` |
| `mcps/mcp_server_repository.rs` | `get_mcp_server`, `get_mcp_server_by_url`, `list_mcp_servers`, `count_mcps_by_server_id` |
| `mcps/mcp_instance_repository.rs` | `get_mcp`, `get_mcp_by_slug`, `list_mcps_with_server` |
| `models/api_alias_repository.rs` | `get_api_model_alias`, `list_api_model_aliases`, `get_api_key_for_alias`, `check_prefix_exists` |
| `models/download_repository.rs` | `get_download_request`, `list_download_requests`, `find_download_request_by_repo_filename` |
| `models/user_alias_repository.rs` | `get_user_alias_by_id`, `get_user_alias_by_name`, `list_user_aliases` |
| `models/model_metadata_repository.rs` | `get_model_metadata_by_file`, `batch_get_metadata_by_files`, `list_model_metadata` |
| `users/access_repository.rs` | `get_pending_request`, `list_pending_requests`, `list_all_requests`, `get_request_by_id` |
| `app_access_requests/access_request_repository.rs` | `get`, `get_by_access_request_scope` |

**NOT wrapped** (cross-tenant by design, use `&self.db`):
- `tokens/token_repository.rs` → `get_api_token_by_prefix` (token prefix lookup is cross-tenant; tenant resolved from client_id suffix)

**NOT wrapped** (tables not in RLS list):
- `tenants/tenant_repository.rs` — `tenants` table has no RLS
- `settings/settings_repository.rs` — `settings` table has no RLS

## Step 3: McpAuth Read Methods — Add `tenant_id` parameter

These 9 McpAuthRepository read methods access RLS-protected tables (`mcp_auth_headers`, `mcp_oauth_configs`, `mcp_oauth_tokens`) but don't have `tenant_id`. Add it and wrap in `with_tenant_txn`:

| Method | Current Signature → Add `tenant_id: &str` |
|--------|-------------------------------------------|
| `get_mcp_auth_header` | `(&self, id)` → `(&self, tenant_id, id)` |
| `list_mcp_auth_headers_by_server` | `(&self, mcp_server_id)` → `(&self, tenant_id, mcp_server_id)` |
| `get_decrypted_auth_header` | `(&self, id)` → `(&self, tenant_id, id)` |
| `get_mcp_oauth_config` | `(&self, id)` → `(&self, tenant_id, id)` |
| `list_mcp_oauth_configs_by_server` | `(&self, mcp_server_id)` → `(&self, tenant_id, mcp_server_id)` |
| `get_decrypted_client_secret` | `(&self, id)` → `(&self, tenant_id, id)` |
| `get_mcp_oauth_token` | `(&self, user_id, id)` → `(&self, tenant_id, user_id, id)` |
| `get_latest_oauth_token_by_config` | `(&self, config_id)` → `(&self, tenant_id, config_id)` |
| `get_decrypted_oauth_bearer` | `(&self, id)` → `(&self, tenant_id, id)` |
| `get_decrypted_refresh_token` | `(&self, token_id)` → `(&self, tenant_id, token_id)` |

**Ripple effect** — update callers:
- `McpService` trait + `DefaultMcpService` impl (`services/src/mcps/mcp_service.rs`): add `tenant_id` to these service methods:
  - `get_auth_header`, `list_auth_headers_by_server`
  - `get_oauth_config`, `list_oauth_configs_by_server`
  - `get_oauth_token`
  - Internal methods: `resolve_auth_header_for_mcp`, `refresh_oauth_token_if_needed` etc. (these already have tenant_id available from their caller context)
- `AuthScopedMcpService` (`services/src/app_service/auth_scoped_mcps.rs`): pass `tenant_id` from `AuthContext`
- Route handlers in `routes_app/src/mcps/` that call these service methods
- `MockMcpService` / `MockDbService` mock blocks
- Test files

## Step 4: Service Layer — Remove txn management

Remove `begin_tenant_txn` + `commit` from all service methods. Services just call repository methods directly.

**Files**:
- `services/src/tokens/token_service.rs`
- `services/src/toolsets/tool_service.rs`
- `services/src/mcps/mcp_service.rs`
- `services/src/models/api_model_service.rs`
- `services/src/models/data_service.rs`
- `services/src/models/download_service.rs`
- `services/src/models/progress_tracking.rs`
- `services/src/app_access_requests/access_request_service.rs`
- `services/src/users/access_repository.rs` (if service-level txn exists)

## Step 5: TestDbService + MockDbService — Update delegation

**File**: `services/src/test_utils/db.rs`

- Remove `txn` parameter from all TestDbService delegation methods
- Add `tenant_id` to McpAuth read delegations
- Update `MockDbService` mock! block to match new trait signatures
- TestDbService read methods: delegate to `self.inner` (which now uses `with_tenant_txn` internally)

## Step 6: Test Files — Remove txn boilerplate, add tenant_id where needed

**services crate tests** (remove `begin_tenant_txn`/`commit`, add `tenant_id` to McpAuth read calls):
- `test_token_repository.rs`, `test_toolset_repository.rs`, `test_tool_service.rs`
- `test_mcp_server_repository.rs`, `test_mcp_instance_repository.rs`, `test_mcp_auth_repository.rs`, `test_mcp_service.rs`
- `test_api_alias_repository.rs`, `test_download_repository.rs`, `test_user_alias_repository.rs`, `test_model_metadata_repository.rs`
- `test_progress_tracking.rs`, `test_data_service.rs`
- `test_access_request_repository.rs`, `test_access_request_service.rs`
- `test_rls.rs`
- `test_utils/fixtures.rs`

**routes_app crate tests**:
- `test_tokens_crud.rs`, `test_api_models_sync.rs`, `test_models.rs`, `test_pull.rs`
- `routes_dev.rs`, `test_oauth_utils.rs`, `test_auth_middleware.rs`, `test_token_service.rs`
- `test_access_request.rs`, `test_access_request_middleware.rs`
- `test_access_request_admin.rs`, `test_access_request_user.rs`, `test_management_crud.rs`
- `test_utils/router.rs`

**server_app crate tests**:
- `tests/utils/external_token.rs`, `tests/utils/live_server_utils.rs`

## Step 7: Other affected files

- `server_core/src/multitenant_inference.rs`
- `server_core/src/standalone_inference.rs`
- `lib_bodhiserver/src/app_service_builder.rs`

## What to KEEP (do NOT revert)

- Parameterized `set_config` in `begin_tenant_txn` (SQL injection fix)
- `tenant_id` additions to `update_api_model_cache`, `get_request_by_id`, `update_request_status`, MCP auth write methods, `delete_oauth_config`, `delete_oauth_config_cascade`
- CSRF `evaluate_same_origin` rewrite
- `.expect()` → error propagation in route handlers
- Pagination bounds check, migration for tenant-scoped index
- `update_status_by_id` on TenantService, `tracing::error!` in background task
- `ChangeRoleRequest.role` as `ResourceRole`, `require_tenant_id()` in auth-scoped user access
- Token digest `[0..32]`, Cache TTL, Injected `AiApiService`, `delete_tenant` error on miss
- All new tests (adapt signatures), `TECHDEBT.md`

## Execution Order

Work module-by-module to keep changes compilable:

1. **`with_tenant_txn` helper** on `DefaultDbService`
2. **tokens** — trait + impl (reads + writes) + service + TestDb + Mock + tests
3. **toolsets** — same pattern
4. **models** (api_alias, user_alias, model_metadata, download) — same pattern
5. **users/access** — same pattern
6. **app_access_requests** — same pattern
7. **mcps** (server, instance, auth) — largest: trait + impl + service + add `tenant_id` to auth reads + ripple through McpService trait + AuthScoped
8. **test_utils/db.rs** — TestDbService + MockDbService final cleanup
9. **test_utils/fixtures.rs** — seed helpers
10. **routes_app** — test files + router test utils + any route handler changes for McpAuth
11. **server_app, server_core, lib_bodhiserver** — remaining references
12. Full `cargo test` verification

## Verification

```bash
cargo check -p services -p routes_app -p server_app -p server_core -p lib_bodhiserver 2>&1 | tail -5
cargo test -p services --lib -p routes_app -p server_app 2>&1 | grep -E "test result|FAILED|failures:"
```
