# Code Review: AppRegInfo+AppStatus to AppInstance DB Migration

## Context

Commit `1b350885` replaces the file-based `SecretService` (encrypted YAML) with a SQLite-backed `AppInstanceService` for persisting app registration info and status. This unifies `AppRegInfo` + `AppStatus` into a single `AppInstance` domain object stored in the `apps` table. The motivation is removing dependency on `BODHI_HOME` files to enable future cluster/multi-tenant deployment with Postgres and multiple app instances per organization.

## Review Scope

- **Ref**: HEAD~1..HEAD (commit 1b350885)
- **Files Changed**: 71 files across 8+ crates
- **Crates Affected**: objs, services, auth_middleware, routes_app, server_app, lib_bodhiserver, lib_bodhiserver_napi, bodhi/src-tauri

---

## Findings

### Important Issues

#### 1. Rename stale `AppRegInfoNotFound` variant in `LoginError`
- **File**: `crates/routes_app/src/routes_auth/error.rs:10` and `login.rs:77,229`
- **Issue**: `LoginError::AppRegInfoNotFound` references the removed `AppRegInfo` concept. Used when `get_instance()` returns `None`.
- **Recommendation**: Rename to `AppInstanceNotFound`. Update error message to "Application instance not found." Update all usages in `login.rs`.
- **Error code changes**: `login_error-app_reg_info_not_found` -> `login_error-app_instance_not_found`. Update test assertions.

#### 2. Rename `AppRegInfo` DTO in auth_service to `ClientRegistrationResponse`
- **File**: `crates/services/src/auth_service/service.rs`
- **Issue**: The old `AppRegInfo` struct was moved here as a module-private DTO for Keycloak `register_client()` return value. The name causes confusion with the now-removed domain concept.
- **Recommendation**: Rename to `ClientRegistrationResponse` (or `OAuthClientRegistration`). Update usages in `routes_setup/route_setup.rs` local variables.

#### 3. Rename `encrypted_client_secret` on `AppInstanceRow`
- **File**: `crates/services/src/db/app_instance_repository.rs:6`
- **Issue**: After `get_app_instance()` decrypts the value, `encrypted_client_secret` holds plaintext but retains the "encrypted" name. Misleading for code readers.
- **Recommendation**: Rename to `client_secret`. The repository layer handles encryption/decryption transparently. Update `repository_app_instance.rs` and `app_instance_service.rs` references.

#### 4. Change repository API from `AppInstanceRow` param to individual params
- **File**: `crates/services/src/db/app_instance_repository.rs:19` (`upsert_app_instance`)
- **Issue**: `create_instance()` constructs an `AppInstanceRow` with dummy values (`salt: ""`, `nonce: ""`, `created_at: 0`, `updated_at: 0`) that are overwritten by the repository. The caller constructs a lie.
- **Recommendation**: Change `upsert_app_instance` signature to take `(client_id, client_secret, scope, status)` as individual params. Repository constructs `AppInstanceRow` internally with proper encryption and timestamps.

#### 5. Make `AppStatus` parsing a hard error
- **File**: `crates/services/src/app_instance_service.rs:42,77`
- **Issue**: `row.app_status.parse::<AppStatus>().unwrap_or_default()` silently falls back to `Setup` on unrecognized status strings. Masks data corruption.
- **Recommendation**: Return `AppInstanceError` on parse failure. Add a new variant `InvalidStatus(String)` to `AppInstanceError`. Apply in both `row_to_instance()` and `get_status()`.

#### 6. Inconsistent `MultipleAppInstance` error handling in `get_status()`
- **File**: `crates/services/src/app_instance_service.rs:73-79`
- **Issue**: `get_instance()` (line 66-69) explicitly maps `DbError::MultipleAppInstance` to `AppInstanceError::MultipleAppInstance`. But `get_status()` (line 74) uses `?` which wraps it as `AppInstanceError::Db(DbError::MultipleAppInstance)` — a different error type and code for the same condition.
- **Recommendation**: Extract the `map_err` logic to a shared helper, or use the same explicit mapping in `get_status()`.

#### 7. `update_app_instance_status` silently succeeds on missing row
- **File**: `crates/services/src/db/repository_app_instance.rs:80-88`
- **Issue**: The `UPDATE ... WHERE client_id = ?` query doesn't check rows affected. If `client_id` doesn't exist, it silently succeeds (0 rows updated). The caller (`update_status` in service) does check for `NotFound` via `get_instance()` first, but the repository should also be defensive.
- **Recommendation**: Check `result.rows_affected() == 0` and return `DbError::ItemNotFound` (or a new variant). Alternatively, trust the service-level check and document the contract.

#### 8. Duplicate `MultipleAppInstance` error at two levels
- **File**: `crates/services/src/db/error.rs:59` and `crates/services/src/app_instance_service.rs:15`
- **Issue**: `DbError::MultipleAppInstance` and `AppInstanceError::MultipleAppInstance` have identical error messages and both map to `InternalServer`. The service explicitly converts the DB variant to the service variant in `get_instance()`. This duplication is by design (DB layer detects, service layer exposes), but the `AppInstanceError` variant could delegate to `Db(DbError)` instead.
- **Recommendation**: Remove `AppInstanceError::MultipleAppInstance` and let it propagate as `AppInstanceError::Db(DbError::MultipleAppInstance)`. The error code becomes `db_error-multiple_app_instance` instead of `app_instance_error-multiple_app_instance`. If the distinct code matters, keep both. Decide based on whether API consumers differentiate.

#### 9. Verify error chain propagation
- **Files**: `services/src/app_instance_service.rs`, `services/src/access_request_service/error.rs`, `routes_app/src/routes_auth/error.rs`
- **Issue**: The chain `DbError -> AppInstanceError -> (LoginError | AccessRequestError) -> ApiError` needs verification for correct HTTP status codes.
- **Verified**: `AppInstanceError` maps both `NotFound` and `MultipleAppInstance` to `ErrorType::InternalServer` (500). `Db(DbError)` is transparent, delegating to the inner error's type. `LoginError::AppInstanceError` is transparent via `#[from]`. `AccessRequestError::AppInstance` is transparent via `#[from]`. The chain is correct — a missing app instance returns 500 (internal server error, not a user-fixable problem).

#### 10. Add dedicated unit tests for AppInstanceService and AppInstanceRepository
- **Files**: `crates/services/src/app_instance_service.rs`, `crates/services/src/db/repository_app_instance.rs`
- **Issue**: No dedicated test modules. These new core components are only tested indirectly through higher-layer tests. They handle encryption round-trips, singleton enforcement, and status parsing — all warranting direct tests.
- **Recommendation**: Add test modules covering:
  - Repository CRUD: create, read, update, delete with encryption round-trip verification
  - Singleton enforcement: inserting two different client_ids triggers `MultipleAppInstance`
  - `row_to_instance` conversion edge cases (timestamp conversion, status parsing)
  - `create_instance` -> `get_instance` round-trip
  - `update_status` on non-existent instance
  - `get_status` default when no instance exists

### Nice-to-Have Issues

#### 11. Timestamp fallback uses `Utc::now()` directly
- **File**: `crates/services/src/app_instance_service.rs:46,50`
- **Issue**: `row_to_instance` uses `Utc::now()` as fallback for invalid timestamps, bypassing `TimeService`. This is in a conversion function without access to `TimeService`, but violates the project convention.
- **Recommendation**: Return an error for invalid timestamps instead of falling back to `Utc::now()`. Or accept `TimeService` as a parameter if default is truly needed.

#### 12. NAPI config silently ignores `app_status` without credentials
- **File**: `crates/lib_bodhiserver_napi/src/config.rs`
- **Issue**: If `app_status` is set but `client_id`/`client_secret` are not provided, the status is silently ignored because `AppInstance` requires both. Previously these were independent settings.
- **Recommendation**: Log a warning when `app_status` is set without credentials. Document the behavioral change.

#### 13. Update CLAUDE.md references to SecretService
- **Files**: `crates/services/CLAUDE.md`, `crates/routes_app/CLAUDE.md`
- **Issue**: Both CLAUDE.md files still reference `SecretService`, `secret_service()`, and the old architecture. The services CLAUDE.md has a whole section on "Why PBKDF2 with 1000 Iterations" for the removed SecretService. The routes_app CLAUDE.md lists `secret_service()` in service registry.
- **Recommendation**: Update both CLAUDE.md files to reflect `AppInstanceService` architecture. Remove SecretService references. Update service initialization order.

---

## Fix Order (Layered)

1. **services crate** (findings 3, 4, 5, 6, 7, 8, 10, 11):
   - Rename `encrypted_client_secret` -> `client_secret` on `AppInstanceRow`
   - Change `upsert_app_instance` to take individual params
   - Add `InvalidStatus` variant to `AppInstanceError`; make status parsing a hard error
   - Fix `get_status()` error mapping consistency
   - Consider `update_app_instance_status` rows-affected check
   - Resolve `MultipleAppInstance` duplication
   - Add unit tests
   - Verify: `cargo test -p services`

2. **routes_app crate** (findings 1, 2):
   - Rename `AppRegInfoNotFound` -> `AppInstanceNotFound`
   - Rename `AppRegInfo` -> `ClientRegistrationResponse` in auth_service
   - Update test assertions for changed error codes
   - Verify: `cargo test -p objs -p services -p routes_app`

3. **Full backend**: `make test.backend`

4. **Regenerate ts-client**: `make build.ts-client`

5. **Documentation** (finding 13): Update CLAUDE.md files for services and routes_app crates

---

## Summary

- **Total findings**: 13
- **Critical**: 0
- **Important**: 10 (findings 1-10)
- **Nice-to-have**: 3 (findings 11-13)
- The refactor is architecturally sound — unified domain model, proper encryption at DB layer, atomic state creation, removed file-system dependency. The findings are cleanup items, naming consistency, defensive error handling, and test coverage for the new core components.
