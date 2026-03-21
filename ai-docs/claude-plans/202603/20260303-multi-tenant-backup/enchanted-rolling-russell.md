# Multi-Tenant Review Fix Plan

## Context

Commit `3ba6997f0` ("RLS first cut") added PostgreSQL RLS infrastructure (migrations, `begin_tenant_txn`, tenant_id columns) but never wired `begin_tenant_txn` into production code paths. A comprehensive code review identified 48 findings (8 P0, 18 P1, 14 P2, 8 P3). This plan addresses all validated P0 and P1 findings, dismisses false positives, and defers items requiring broader redesign to TECHDEBT.md.

Multi-tenant deployment serves concurrent requests from different tenant domains against the same PostgreSQL instance. RLS enforcement via `SET LOCAL app.current_tenant_id` is transaction-scoped and concurrency-safe.

## Key Decisions

| Decision | Choice |
|----------|--------|
| P0-2 SQL injection fix | Parameterized `SELECT set_config('app.current_tenant_id', $1, true)` via `Statement::from_sql_and_values` |
| P0-6 CSRF fix | Check `Sec-Fetch-Site` for ALL hosts, not just localhost |
| Transaction ownership | Domain service layer calls `begin_tenant_txn`, passes `&DatabaseTransaction` to repo methods |
| Cross-service atomicity | Accepted non-atomic for multi-service calls from route handlers (non-financial) |
| Standalone-only repos | Wire uniformly (downloads, metadata, user_alias) even though SQLite RLS is no-op |
| P1-8 model metadata | Intentionally global/shared -- NOT A BUG |
| P1-16 DI bypass | Fix now -- inject `Arc<dyn AiApiService>` |
| Tests | Real SQLite DB for app-layer; attempt PostgreSQL RLS tests, else TECHDEBT.md |

## Dismissed Findings (Verified Not-a-Bug)

| Finding | Rationale |
|---------|-----------|
| P0-3c: mcp_auth_repository missing tenant_id | Already filters by proper keys (verified in code) |
| P0-3d: access_repository missing tenant_id | Already filters by tenant_id in WHERE clauses (verified) |
| P1-8: model metadata hardcoded empty tenant_id | Intentionally global -- GGUF file metadata is intrinsic to files, not tenants |
| P1-10: Non-atomic approve+role | Accepted -- non-financial transaction, documented |
| P1-18: MCP server+auth non-atomic | Accepted -- same rationale |

## Deferred to TECHDEBT.md

- **P1-2**: AuthScopedAccessRequestService + app access request flow redesign for multi-tenant (client_id/tenant_id population on login)
- **P1-15**: `std::sync::RwLock` in StandaloneInferenceService (standalone-only, locks not held across .await)
- App access request client_id/tenant_id population on login
- PostgreSQL RLS integration tests (if not implementable in this pass)

---

## Implementation Phases (9 Sequential Sub-Agents)

**Architecture**: Mutating repository trait methods gain a `txn: &DatabaseTransaction` parameter. Service layer calls `begin_tenant_txn(tenant_id)`, passes the txn, then commits. Read-only methods keep using `&self.db`.

---

### Phase 1: DB Core + Tokens + Toolsets (P0-2, P0-1 partial, P1-6)

**Scope**: Fix SQL injection first, then wire begin_tenant_txn into simpler repos (tokens, toolsets).

**Files**:
- `crates/services/src/db/default_service.rs:54` -- Replace `format!("SET LOCAL ...")` with parameterized `SELECT set_config('app.current_tenant_id', $1, true)` via `Statement::from_sql_and_values`
- `crates/services/src/db/test_rls.rs` -- Add test with special chars in tenant_id
- `crates/services/src/tokens/token_repository.rs` -- Add `txn: &DatabaseTransaction` param to `create_api_token`, `update_api_token`
- `crates/services/src/tokens/token_service.rs` -- Call `begin_tenant_txn(tenant_id)`, pass txn to repo. **P1-6**: Replace `unwrap_or_else` fallback (line 100) with error: `ok_or(TokenServiceError::TenantNotFound)?`
- `crates/services/src/toolsets/toolset_repository.rs` -- Add txn param to `create_toolset`, `update_toolset`, `delete_toolset`, `set_app_toolset_enabled`
- `crates/services/src/toolsets/tool_service.rs` -- Wire begin_tenant_txn
- `crates/services/src/test_utils/db.rs` -- Update TestDbService impls for TokenRepository + ToolsetRepository

**Tests**: Existing tests + new tenant isolation test for token create/update + special-char tenant_id test
**Gate**: `cargo test -p services --lib -- "token|toolset|rls"`
**Commit**: `fix(services): P0-2/P0-1/P1-6 parameterize begin_tenant_txn, wire into token+toolset mutations`

---

### Phase 2: MCP Domain (P0-1, P0-5, P1-1)

**Scope**: MCP is the most complex domain -- 3 repos, service layer, auth-scoped wrapper all need changes.

**Files**:
- `crates/services/src/mcps/mcp_server_repository.rs` -- Add txn param to mutations
- `crates/services/src/mcps/mcp_instance_repository.rs` -- Add txn param to mutations
- `crates/services/src/mcps/mcp_auth_repository.rs` -- Add txn param. **P0-5**: Add `tenant_id` param to `create_mcp_oauth_token` (currently `String::new()`). **P1-1**: Add `tenant_id` param to `delete_mcp_oauth_config`, `delete_oauth_config_cascade`
- `crates/services/src/mcps/mcp_service.rs` -- Wire begin_tenant_txn for all mutations. Fix `store_oauth_token` (line 1214) to pass actual tenant_id. Fix `update_auth_header` (line 1056) to preserve `existing.tenant_id` instead of `String::new()`
- `crates/services/src/mcps/auth_scoped_mcps.rs` -- **P1-1**: `delete_auth_config`, `list_auth_configs`, `get_auth_config`, `get_oauth_config` must use `require_tenant_id()` and pass through
- `crates/services/src/test_utils/db.rs` -- Update mocks

**Tests**: Existing MCP tests + new tests for tenant-scoped auth config delete, OAuth token with correct tenant_id
**Gate**: `cargo test -p services --lib -- mcp`
**Commit**: `fix(services): P0-1/P0-5/P1-1 wire begin_tenant_txn into MCP mutations, fix OAuth/auth-config tenant scoping`

---

### Phase 3: Models/Downloads Domain (P0-1, P0-3, P0-8, P1-7)

**Scope**: Model aliases, downloads, user aliases, metadata repos + pagination panic fix.

**Files**:
- `crates/services/src/models/api_alias_repository.rs` -- Add txn param. **P0-3**: Add `tenant_id` param to `update_api_model_cache`
- `crates/services/src/models/download_repository.rs` -- Add txn param (standalone-only but wired uniformly)
- `crates/services/src/models/user_alias_repository.rs` -- Add txn param
- `crates/services/src/models/model_metadata_repository.rs` -- Add txn param
- `crates/services/src/models/api_model_service.rs` -- Wire begin_tenant_txn. **P0-8**: Bounds check at line 269: `if start >= total { return empty page }`. **P1-7**: Document hardcoded `has_api_key: true` with TODO comment
- `crates/services/src/models/data_service.rs` -- Wire begin_tenant_txn in LocalDataService mutations
- `crates/services/src/models/download_service.rs` -- Wire begin_tenant_txn
- `crates/services/src/test_utils/db.rs` -- Update mocks

**Tests**: Test pagination bounds (P0-8). Test `update_api_model_cache` with tenant_id
**Gate**: `cargo test -p services --lib -- models`
**Commit**: `fix(services): P0-1/P0-3/P0-8/P1-7 wire begin_tenant_txn into model mutations, fix pagination panic`

---

### Phase 4: Users + App Access + Tenants + Misc Services (P0-1, P0-3, P0-4, P1-4, P1-5, P1-11, P1-12, P1-17)

**Scope**: Users/access repos, app access request repos, tenant service fixes, ChangeRoleRequest type, migration.

**Files**:
- `crates/services/src/users/access_repository.rs` -- Add txn param. **P0-3/P0-4**: Add `tenant_id` param to `get_request_by_id` and `update_request_status`
- `crates/services/src/app_service/auth_scoped_user_access_requests.rs` -- **P1-12**: Replace `tenant_id_or_empty()` with `require_tenant_id()` for mutations. Pass tenant_id to `get_request_by_id`
- `crates/services/src/app_access_requests/access_request_repository.rs` -- Add txn param to mutations
- `crates/services/src/app_access_requests/access_request_service.rs` -- Wire begin_tenant_txn
- `crates/services/src/tenants/tenant_repository.rs` -- **P1-17**: Check `rows_affected == 0` in `delete_tenant`, return error
- `crates/services/src/tenants/tenant_service.rs` -- **P1-5**: Add `update_status_by_id(tenant_id, status)` for multi-tenant use
- `crates/services/src/users/user_objs.rs` -- **P1-11**: Change `role: String` to `role: ResourceRole`
- New migration `m20250101_000015_fix_access_request_scope_index.rs` -- **P1-4**: Drop old index, create `ON app_access_requests(tenant_id, access_request_scope) WHERE access_request_scope IS NOT NULL`
- `crates/services/src/db/sea_migrations/mod.rs` -- Register migration
- `crates/services/src/test_utils/db.rs` -- Update mocks

**Tests**: Test `get_request_by_id` returns None for wrong tenant. Test `update_request_status` scoped. Test delete_tenant returns error on miss.
**Gate**: `cargo test -p services`
**Commit**: `fix(services): P0-1/P0-3/P0-4/P1-4/P1-5/P1-11/P1-12/P1-17 users+tenants+access fixes`

---

### Phase 5: server_core + lib_bodhiserver (P1-16)

**Scope**: Inject `Arc<dyn AiApiService>` into inference services + wire at construction site.

**Files**:
- `crates/server_core/src/standalone_inference.rs` -- Add `ai_api_service: Arc<dyn AiApiService>` field. Change `proxy_to_remote` to use injected service
- `crates/server_core/src/multitenant_inference.rs` -- Same: inject `Arc<dyn AiApiService>`
- `crates/lib_bodhiserver/src/app_service_builder.rs` -- Pass `AiApiService` when constructing inference services
- `crates/bodhi/src-tauri/src/` -- Same if applicable

**Gate**: `cargo test -p server_core -p server_app -p lib_bodhiserver`
**Commit**: `fix(server_core): P1-16 inject AiApiService into inference services`

---

### Phase 6: routes_app -- All Middleware + Handler Fixes (P0-4, P0-6, P0-7, P1-3, P1-9, P1-11, P1-13, P1-14, cascades)

**Scope**: All routes_app fixes in one batch -- CSRF, panics, role hierarchy, cache, signature cascades.

**Files**:
- `crates/routes_app/src/middleware/auth/auth_middleware.rs:59-71` -- **P0-6**: Rewrite `evaluate_same_origin` to check `Sec-Fetch-Site` for all hosts:
  - `Some("same-origin")` or `Some("same-site")` -> true
  - `Some("cross-site")` or `Some("none")` -> false
  - `None` (non-browser clients) -> true for non-localhost, false for localhost
- `crates/routes_app/src/middleware/token_service/token_service.rs` -- **P1-13**: `[0..12]` -> `[0..32]`. **P1-14**: Add TTL to cached exchange results
- `crates/routes_app/src/users/routes_users_access_request.rs:300` -- **P0-7**: `.expect(...)` -> error propagation. Pass tenant_id for **P0-4**
- `crates/routes_app/src/apps/routes_apps.rs:296` -- **P0-7**: Same `.expect` fix
- `crates/routes_app/src/models/routes_models_pull.rs:227` -- **P1-9**: `.expect(...)` -> `if let Err(e) { tracing::error!(...) }`
- `crates/routes_app/src/users/routes_users.rs` -- **P1-3**: Add `has_access_to()` check in `users_change_role`. **P1-11**: Use `ValidatedJson<ChangeRoleRequest>` since role is now `ResourceRole`
- `crates/routes_app/src/mcps/` -- Cascade tenant_id for auth config ops
- `crates/routes_app/src/models/` -- Cascade tenant_id for cache update
- `crates/routes_app/src/middleware/access_request_middleware.rs` -- Fix empty string tenant_id (P0-3)

**Tests**: Parameterized rstest for evaluate_same_origin. Test privilege escalation (manager assigning admin). Test cache key length + TTL.
**Gate**: `cargo test -p routes_app`
**Commit**: `fix(routes_app): P0-4/P0-6/P0-7/P1-3/P1-9/P1-13/P1-14 middleware+handler fixes`

---

### Phase 7: Full Backend Validation + TypeScript Regen

**Scope**: Run full backend tests, regenerate OpenAPI + TypeScript types.

```bash
make test.backend
cargo run --package xtask openapi
cd ts-client && npm run generate
```

Fix any failures. Verify diffs (P1-11 changes `ChangeRoleRequest.role` from string to enum).

**Commit**: `chore: regenerate OpenAPI spec and TypeScript client`

---

### Phase 8: Frontend + UI Rebuild + E2E

**Scope**: Update frontend for type changes, build UI, run E2E.

**Files**: Any frontend code referencing `ChangeRoleRequest.role` as free-form string

```bash
cd crates/bodhi && npm run build && npm run test:all
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright:sqlite
```

**Commit**: `fix(bodhi): update frontend for ChangeRoleRequest type change`

---

### Phase 9: TECHDEBT.md + Documentation

Create/update `TECHDEBT.md` with deferred items:
- P1-2: AuthScopedAccessRequestService + app access request flow redesign
- P1-15: std::sync::RwLock in StandaloneInferenceService
- App access request client_id/tenant_id population on login
- PostgreSQL RLS integration tests

**Commit**: `docs: add TECHDEBT.md for deferred multi-tenant items`

---

## Verification

1. `make test.backend` -- all Rust tests pass
2. `make build.ts-client` -- TypeScript client in sync
3. `cd crates/bodhi && npm run test:all` -- frontend tests pass
4. `make build.ui-rebuild` -- UI builds successfully
5. `make test.napi` -- E2E Playwright tests pass
6. Manual: verify `begin_tenant_txn` is called in all mutating code paths (grep for `\.exec(&self\.db)` in repository files -- should find ZERO hits for mutating operations)

## Summary

| Phase | Findings | Crate | Description |
|-------|----------|-------|-------------|
| 1 | P0-2, P0-1, P1-6 | services | DB core + tokens + toolsets |
| 2 | P0-1, P0-5, P1-1 | services | MCP domain |
| 3 | P0-1, P0-3, P0-8, P1-7 | services | Models/downloads domain |
| 4 | P0-1, P0-3, P0-4, P1-4, P1-5, P1-11, P1-12, P1-17 | services | Users + tenants + misc |
| 5 | P1-16 | server_core + lib | Inference DI fix |
| 6 | P0-4, P0-6, P0-7, P1-3, P1-9, P1-13, P1-14 | routes_app | All middleware + handlers |
| 7 | -- | all | Backend validation + TS regen |
| 8 | P1-11 cascade | frontend/e2e | Frontend + E2E |
| 9 | deferred | docs | TECHDEBT documentation |

**Total: 9 sub-agents, 22 P0+P1 findings fixed, 5 dismissed, 4 deferred**
