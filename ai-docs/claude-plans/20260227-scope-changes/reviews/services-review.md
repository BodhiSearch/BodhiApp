# services Crate Review

## Files Reviewed

- `crates/services/migrations/0009_app_access_requests.up.sql`
- `crates/services/migrations/0014_apps.up.sql`
- `crates/services/src/access_request_service/service.rs`
- `crates/services/src/app_instance_service.rs`
- `crates/services/src/auth_service/service.rs`
- `crates/services/src/auth_service/tests.rs`
- `crates/services/src/db/access_request_repository.rs`
- `crates/services/src/db/app_instance_repository.rs`
- `crates/services/src/db/objs.rs`
- `crates/services/src/db/repository_app_instance.rs`
- `crates/services/src/db/service_access_request.rs`
- `crates/services/src/db/test_access_request_repository.rs`
- `crates/services/src/objs.rs`
- `crates/services/src/setting_service/default_service.rs`
- `crates/services/src/test_app_instance_service.rs`
- `crates/services/src/test_session_service.rs`
- `crates/services/src/test_utils/app.rs`
- `crates/services/src/test_utils/auth.rs`
- `crates/services/src/test_utils/db.rs`
- `crates/services/src/test_utils/envs.rs`
- `crates/services/src/test_utils/objs.rs`

---

## Summary

The changes correctly replace the `resource_scope` column with `requested_role` / `approved_role` semantics across the full stack: migration DDL, DB row struct, repository trait, SQLite implementation, service logic, and test coverage. The design is sound and internally consistent. A few issues of varying priority are noted below.

---

## Findings

### Finding 1: Duplicate `#[rstest]` attribute in `auth_service/tests.rs`

- **Priority**: Important
- **File**: `crates/services/src/auth_service/tests.rs`
- **Location**: Lines 313–314
- **Issue**: The `test_exchange_auth_code_success` test function has two consecutive `#[rstest]` attributes:
  ```rust
  #[rstest]
  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_exchange_auth_code_success() -> anyhow::Result<()> {
  ```
  While Rust (and rstest) is tolerant of duplicate proc-macro applications in this case — because `#[rstest]` is idempotent when no fixtures or `#[case]` are present — the duplicate is an unintentional error that reads as dead noise and could confuse future maintainers. It suggests a copy-paste mistake.
- **Recommendation**: Remove the extra `#[rstest]` at line 313 (keep only one).

---

### Finding 2: `DefaultAccessRequestService` has no unit tests

- **Priority**: Important
- **File**: `crates/services/src/access_request_service/service.rs`
- **Location**: Entire `DefaultAccessRequestService` implementation
- **Issue**: The `DefaultAccessRequestService` implements `create_draft`, `get_request`, `approve_request`, `deny_request`, and `build_review_url`. The new role-based semantics (`requested_role` passed to `create_draft`, `approved_role` passed to `approve_request`) are untested at the service layer. Repository tests in `test_access_request_repository.rs` exercise SQLite persistence, but the service-level logic — flow_type validation, expiry checking, UUID collision fallback to `update_failure`, description generation, `access_request_scope` threading from KC response to DB — has no tests. The `#[cfg(test)]` or `#[path = "..."]` declaration for a test file is absent from `service.rs`, and no file named `test_access_request_service.rs` exists under `src/`.
- **Recommendation**: Add a `test_access_request_service.rs` sibling file and declare it with `#[cfg(test)] #[path = "test_access_request_service.rs"] mod test_access_request_service;` in `service.rs`. At minimum, cover:
  - `create_draft` with invalid flow_type returns `InvalidFlowType`
  - `create_draft` with `flow_type = "redirect"` and missing `redirect_uri` returns `MissingRedirectUri`
  - `get_request` on an expired draft returns `Expired`
  - `approve_request` on a non-draft status returns `AlreadyProcessed`
  - `approve_request` with KC UUID collision (409) transitions to `failed` status via `update_failure` — not an error return
  - Role fields (`requested_role`, `approved_role`) are correctly threaded into the DB row on create and approve

---

### Finding 3: `AppAccessRequestDetail` in `objs.rs` does not expose `requested_role` or `approved_role`

- **Priority**: Important
- **File**: `crates/services/src/objs.rs`
- **Location**: `AppAccessRequestDetail` struct (lines 62–85)
- **Issue**: The API detail struct used when surfacing access request state to callers still does not include `requested_role` or `approved_role` fields. The two fields were added to `AppAccessRequestRow` (the DB row) and to the route-layer response types in `routes_app`, but `AppAccessRequestDetail` in `services/src/objs.rs` — which is an intermediate domain object — still only carries `scopes: Vec<String>` (the KC-side scope string), with no structured role fields. This means any caller that maps from `AppAccessRequestRow` to `AppAccessRequestDetail` will silently discard the new role data unless the mapping layer explicitly populates `scopes` from the role fields.
- **Recommendation**: Verify that the `routes_app` handler that maps `AppAccessRequestRow` → its response DTO populates role fields correctly. If `AppAccessRequestDetail` is used in any response path, either add `requested_role: String` and `approved_role: Option<String>` fields to it, or ensure the existing `scopes` field is populated from the `approved_role`. This is a consistency/completeness concern — the DB holds the new role semantics but the service-layer DTO may silently drop them.

---

### Finding 4: `access_request_scope` unique index uses partial `WHERE` — test coverage for this is good, but `update_approval` does not guard against re-approving with a duplicate scope

- **Priority**: Nice-to-have
- **File**: `crates/services/src/db/service_access_request.rs`
- **Location**: `update_approval` method
- **Issue**: The `update_approval` SQL does not check whether the `access_request_scope` being written is already present in another row. The partial unique index on `app_access_requests(access_request_scope) WHERE access_request_scope IS NOT NULL` correctly enforces uniqueness at the DB level, so a duplicate will surface as a `DbError::SqlxError`. However, the error bubbling from `update_approval` in this scenario is a raw `sqlx_error`, which provides a poor diagnostic. The KC UUID-collision path handles 409 responses before calling `update_approval`, but a concurrent re-approval race (two simultaneous approval calls for different requests both using the same KC-issued scope) would produce an opaque `sqlx_error` at the service layer.
- **Recommendation**: This is acceptable given that KC scope values are meant to be globally unique. No immediate code change is required, but a comment in `update_approval` explaining that the unique index enforces scope deduplication would aid maintainability. Alternatively, the service could check the error message for UNIQUE constraint and convert it to a domain error.

---

### Finding 5: TimeService usage is correct throughout

- **Priority**: Informational
- **Finding**: All `updated_at` computations in `service_access_request.rs` (`update_approval`, `update_denial`, `update_failure`) use `self.time_service.utc_now().timestamp()`. The `create_draft` path in `access_request_service/service.rs` uses `self.time_service.utc_now()` for both `now` and `expires_at`. No direct `Utc::now()` calls exist in the changed files. This is correct.

---

### Finding 6: SQL column ordering in `INSERT` and `RETURNING` clauses is verified correct

- **Priority**: Informational
- **Finding**: The `INSERT` statement in `service_access_request.rs` lists 17 columns and 17 `?` placeholders. The `.bind()` chain provides exactly 17 bindings in the matching order. The `RETURNING` clause lists the same 17 columns. The `AppAccessRequestRow` struct has 17 fields with `#[derive(sqlx::FromRow)]`, so the column-to-field mapping is consistent. All four SQL statements (`create`, `get`, `update_approval`, `update_denial`, `update_failure`, `get_by_access_request_scope`) use the same consistent column list. No column order or count mismatches were found.

---

### Finding 7: `MockDbService` in `test_utils/db.rs` is fully synchronized with the updated `AccessRequestRepository` trait

- **Priority**: Informational
- **Finding**: The `mockall::mock!` block for `MockDbService` in `test_utils/db.rs` declares the `AccessRequestRepository` section with the updated signature of `update_approval` — taking `approved_role: &str` rather than `Option<String>`. The `TestDbService` wrapper in the same file also implements the updated `update_approval` signature correctly by delegating to `self.inner.update_approval(id, user_id, approved, approved_role, access_request_scope)`. Trait/implementation/mock alignment is confirmed.

---

### Finding 8: `AppInstanceService` scope removal is clean

- **Priority**: Informational
- **Finding**: The `scope` field has been removed from the `apps` table DDL in `0014_apps.up.sql`. The `AppInstanceService`, `AppInstanceRow`, `AppInstance`, and `AppInstanceRepository` no longer reference a `scope` field. The `upsert_app_instance` repository method correctly omits scope from both the `INSERT` and `ON CONFLICT DO UPDATE` clauses. No orphaned scope references remain in `app_instance_service.rs`, `repository_app_instance.rs`, `app_instance_repository.rs`, or `test_app_instance_service.rs`.

---

### Finding 9: `AuthService` `register_resource_access` removal is complete

- **Priority**: Informational
- **Finding**: No references to `register_resource_access` or `resource_scope` were found anywhere in the `services` crate or downstream crates (`routes_app`, `auth_middleware`). The removal of the auto-approve branch is complete. The `AuthService` trait now has a clean set of methods: `register_client`, `exchange_auth_code`, `refresh_token`, `exchange_app_token`, `make_resource_admin`, `assign_user_role`, `remove_user`, `list_users`, `register_access_request_consent`, `get_app_client_info`.

---

### Finding 10: `test_access_request_repository.rs` covers the new role fields but does not assert `requested_role` on the returned row in `test_create_draft_request`

- **Priority**: Nice-to-have
- **File**: `crates/services/src/db/test_access_request_repository.rs`
- **Location**: `test_create_draft_request` (lines 14–60)
- **Issue**: The test sets `requested_role: "scope_user_user"` in the input row but does not assert `result.requested_role == "scope_user_user"` in the assertions. Similarly, it does not assert `result.approved_role.is_none()`. While the `test_update_approval` test does assert `result.approved_role == Some("scope_user_user")`, the create test leaves the new fields implicitly verified only through the full round-trip in `test_get_request`.
- **Recommendation**: Add `assert_eq!(result.requested_role, "scope_user_user")` and `assert_eq!(result.approved_role, None)` to `test_create_draft_request` to make the coverage of new fields explicit.

---

## Checklist Summary

| Checklist Item | Status |
|---|---|
| DB migration: column types, NOT NULL constraints, indexes | PASS — `requested_role TEXT NOT NULL`, `approved_role TEXT` (nullable), indexes on `status` and `app_client_id`, partial unique on `access_request_scope` |
| SQL queries: column order in INSERT/RETURNING matches struct | PASS — 17 columns, 17 placeholders, 17 bindings, 17 struct fields |
| TimeService usage (no direct Utc::now()) | PASS — all timestamps use `self.time_service.utc_now()` |
| Error chain: service error → AppError → ApiError | PASS — `AccessRequestError` and `AppInstanceError` both implement `AppError` via `errmeta_derive`; `DbError` propagates correctly |
| Test coverage for new role fields | PARTIAL — repository tests cover `requested_role`/`approved_role` in `update_approval`; `create_draft` test doesn't assert them explicitly; no service-layer unit tests |
| Removed auto-approve branch: no orphaned references | PASS — no references to `register_resource_access` or `resource_scope` found anywhere |
| AccessRequestRepository trait: mock matches implementation | PASS — `MockDbService` and `TestDbService` both implement the updated `update_approval` signature |
