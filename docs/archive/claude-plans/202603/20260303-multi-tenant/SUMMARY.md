# Multi-Tenant Refactor — Combined Review Summary

## CRUD Layer Refactor (commit f354b09a0)

Well-executed across 85 files. No critical issues. Successfully replaced `WithRejection<Json<...>>` with `ValidatedJson`, consolidated `Create*`/`Update*` pairs into unified `*Request` types, moved request/response types from routes_app to services, removed `form.validate()` from service layer.

### Remaining Issues

| ID | Sev | Issue |
|----|-----|-------|
| M-4 | Minor | `CopyAliasRequest` derives `Validate` with no validation annotations — trivially succeeds |
| M-6 | Minor | `UserAliasResponse::from` hardcodes empty `model_params` instead of using alias's model_params |
| S-4 | Low | `EDIT_SETTINGS_ALLOWED` and `LLM_SETTINGS` are identical constants |
| T-2 | Low | Stale "form" references in test function names (`test_update_token_with_form`, `tok_form_update`) |
| U-6 | Low | Long line in `routes_apps.rs:206` entity-to-response conversion |
| S-6 | Info | `ValidationRejection` and `ValidatedJson` lack doc comments |

### Documented Exceptions (Informational)

- **Token domain**: Returns `TokenDetail` response types directly from services (Phase 1 legacy)
- **ApiModel domain**: Service returns `ApiModelOutput` directly — `has_api_key` requires service-level logic
- **`OAuthTokenResponse`** and **`LocalModelResponse`**: `From<Entity>` impls stay in routes_app — presentation-only types

### Type Split Rule

**In `services/*_objs.rs`**: Request types, response types mirroring entities (excluding tenant_id/user_id, secrets → `has_<secret>`), paginated wrappers, domain enums, `From<Entity> for Response` impls.

**In `routes_app/*_api_schemas.rs`**: Presentation-only types (HTTP formatting), `IntoResponse` impls, query param types, action DTOs, workflow composition types, auth/session presentation types.

**One-sentence rule**: If used by service-layer code or a direct Entity projection → services. If it only shapes HTTP responses with presentation logic → routes_app.

---

## RLS / Transaction Refactor

Architecture: Repository methods internally call `begin_tenant_txn(tenant_id)` via `with_tenant_txn` helper and commit — txn not exposed to callers. Both reads AND writes on tenant-scoped tables go through tenant-scoped transactions.

### All P0 Findings Fixed (8/8)

- **P0-1**: All mutating repos use `begin_tenant_txn` internally
- **P0-2**: SQL injection fixed — parameterized `set_config($1, true)` in `begin_tenant_txn`
- **P0-3**: `tenant_id` added to `update_api_model_cache`, `get_request_by_id`, `update_request_status`, all MCP auth read methods
- **P0-4**: `update_request_status` and `get_request_by_id` filter by tenant_id
- **P0-5**: `store_oauth_token` and `update_auth_header` use actual tenant_id (was `String::new()`)
- **P0-6**: `evaluate_same_origin` checks `Sec-Fetch-Site` for all hosts
- **P0-7**: `.expect()` in route handlers replaced with error propagation
- **P0-8**: Pagination bounds check — returns empty page when `start >= total`

### P1 Findings Fixed (9/18)

- **P1-1**: `tenant_id` added to `delete_oauth_config`, `delete_oauth_config_cascade`
- **P1-3**: `has_access_to()` role hierarchy check added to `users_change_role`
- **P1-4**: Tenant-scoped unique index on `app_access_requests(tenant_id, access_request_scope)` — merged into original migration
- **P1-5**: `update_status_by_id` added to TenantService
- **P1-9** (partial): `.expect()` removed from `routes_models_pull.rs` (replaced with `tracing::error!`)
- **P1-11**: `ChangeRoleRequest.role` changed from `String` to `ResourceRole`
- **P1-13**: Token digest prefix `[0..12]` → `[0..32]`
- **P1-14**: 5-minute TTL added to cached exchange results
- **P1-16**: `Arc<dyn AiApiService>` injected into both inference services
- **P1-17**: `delete_tenant` returns error when `rows_affected == 0`

### Open — P1 (8 findings)

**P1-2: Missing `AuthScopedAccessRequestService` wrapper**
`access_request_service()` is a bare passthrough — no AuthScoped wrapper injecting tenant_id/user_id. Not a data isolation bug (repos enforce tenant scoping), but an architectural inconsistency.

**P1-6: Silent tenant fallback in token creation**
`token_service.rs:102` — `unwrap_or_else(|| tenant_id.to_string())` when tenant not found. Masks data integrity issues; should return error.

**P1-7: `has_api_key` hardcoded to `true` in list view**
`api_model_service.rs:279` — TODO comment added but not fixed. List view shows all models as having API keys. Detail view checks correctly.

**P1-8: Model metadata uses empty `tenant_id`**
`batch_get_metadata_by_files("", ...)` in routes. Design decision needed — GGUF metadata is intrinsic to files, not tenants. Table has `tenant_id` column but data is intentionally global.

**P1-9: `.expect()` in `mcp_service.rs` (4 calls)**
Lines 750, 754, 801, 807 — JSON serialization of `Vec<McpTool>` and `Vec<String>`. Infallible in practice but not idiomatic. Low risk.

**P1-10: Non-atomic approve + role assignment**
`routes_users_access_request.rs` — three sequential operations (update status, assign role, clear sessions). If step 2 fails, request marked "approved" with no role. Accepted trade-off: repo-level txn means no cross-repo atomicity.

**P1-12: `tenant_id_or_empty()` in read methods**
`auth_scoped_user_access_requests.rs` — mutations use `require_tenant_id()` but 4 read methods still use `tenant_id_or_empty()`. Should be consistent.

**P1-15: `std::sync::RwLock` in async context**
`standalone_inference.rs` — uses `std::sync::RwLock` (not `tokio::sync::RwLock`). Standalone-only, locks not held across `.await`. Low risk but should migrate.

### Dismissed

- **P1-10**: Accepted non-atomic (non-financial workflow)
- **P1-18**: MCP server+auth creation non-atomic — same rationale. Repo-level txn trades cross-repo atomicity for simpler code.

---

## P2 Findings (Follow-up)

| # | Domain | Issue |
|---|--------|-------|
| P2-1 | Settings | `created_at`/`updated_at` set to `UNIX_EPOCH` then discarded by repo |
| P2-2 | Tokens | Redundant timestamp assignment in `create_token` (overwritten by repo) |
| P2-3 | Tenants | `Tenant` struct serializes `client_secret` in plaintext |
| P2-4 | Tenants | `get_tenant()` fetches ALL rows to check count (O(n)) |
| P2-5 | Models | In-memory pagination fetches all aliases then slices |
| P2-6 | Models | `data_service.rs` silently swallows API alias errors |
| P2-7 | MCPs | `delete_oauth_token` error mapped to Validation (400) |
| P2-8 | Apps/Access | `tenant_id` filtering via `is_empty()` sentinel instead of `Option` |
| P2-9 | Apps/Access | Error matching by string content (`contains("409")`) |
| P2-10 | Middleware | `extract_id_from_path` does not validate ID format |
| P2-11 | Middleware | `DbError` variant in `AuthError` may leak SQL details |
| P2-12 | Inference | No unit tests for inference services |
| P2-13 | Inference | `forward_local` resets keep-alive timer even on error |
| P2-14 | DB Core | `reset_all_tables`, `TENANT_TABLES`, test lists must stay in sync |

## P3 Findings (Optional)

| # | Domain | Issue |
|---|--------|-------|
| P3-1 | Settings | Duplicate multi-tenant guard logic |
| P3-2 | Tokens | `token_hash` lacks `#[serde(skip_serializing)]` |
| P3-3 | Tenants | `TenantRow` is `pub` but should be `pub(crate)` |
| P3-4 | Models | `ApiModelsRouteError` defined but never used |
| P3-5 | Toolsets | Tests use `Utc::now()` instead of FrozenTimeService |
| P3-6 | Frontend | Duplicated validation schemas (~45 lines) |
| P3-7 | Inference | Duplicated code in `ModelLoadStrategy` variants |
| P3-8 | DB Core | `MultipleTenant` should be `MultipleTenants` |
