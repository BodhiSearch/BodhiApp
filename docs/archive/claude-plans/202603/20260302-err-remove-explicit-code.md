# Plan: Remove Explicit `code` Attributes from `#[error_meta]`

## Context

The `errmeta_derive` proc macro auto-derives error codes as `{enum_name_snake_case}-{variant_name_snake_case}`. Many variants redundantly specify explicit `code = "..."` that either matches the derived value or preserves legacy codes from before enum renames. This creates duplication and confusion. Goal: remove ALL explicit `code` attributes, letting derivation handle code generation consistently.

**Total: 38 explicit `code = "..."` removals across 9 files in 5 crates.**

---

## Phase 1: `services` crate (4 removals, 0 test changes)

All 4 are Category A (explicit = derived), no code value changes.

### `crates/services/src/shared_objs/error_wrappers.rs`
- `ObjValidationError::ForwardAllRequiresPrefix`: remove `code = "obj_validation_error-forward_all_requires_prefix"`

### `crates/services/src/models/model_objs.rs`
- `ModelValidationError::ForwardAllRequiresPrefix`: remove `code = "model_validation_error-forward_all_requires_prefix"`

### `crates/services/src/db/error.rs`
- `DbError::PrefixExists` (line 50): remove `code = "db_error-prefix_exists"`
- `DbError::StrumParse` (line 41): remove `code="db_error-strum_parse"`. **Special handling**: also change `#[error(transparent)]` to `#[error("{0}")]` because `strum::ParseError` doesn't implement `AppError`, so transparent delegation would fail. Remove now-unnecessary `args_delegate = false`.

### `crates/services/src/shared_objs/token.rs`
- `TokenError::JsonWebToken` (line 46): remove `code = "token_error-json_web_token"`

**Verify**: `cargo test -p services`

---

## Phase 2: `auth_middleware` crate (1 removal, code changes)

### `crates/auth_middleware/src/auth_middleware/middleware.rs`
- `AuthError::TowerSession` (line 111): remove `code = "auth_error-tower_sessions"`. New derived code: `"auth_error-tower_session"` (singular). **Special handling**: change `#[error(transparent)]` to `#[error("{0}")]` (same reason as `StrumParse`). Remove `args_delegate = false`.

**Verify**: `cargo test -p auth_middleware` (no tests assert on this code)

---

## Phase 3: `routes_app` crate (30 removals, 6+ test file updates)

### `crates/routes_app/src/auth/error.rs` (3 removals)
- `LoginError::SessionError` (line 17): remove `code = "login_error-session_error"`. **Special handling**: change `#[error(transparent)]` to `#[error("{0}")]`. Remove `args_delegate = false`.
- `LoginError::ParseError` (line 28): remove `code = "login_error-parse_error"`. **Special handling**: change `#[error(transparent)]` to `#[error("{0}")]`. Remove `args_delegate = false`.
- `LoginError::SessionDelete` (line 42): remove `code = "login_error-session_delete"`. No special handling needed (already uses `#[error("Failed to delete session: {0}.")]`, not transparent). Keep `args_delegate = false` as-is (it's a no-op but harmless).

### `crates/routes_app/src/apps/error.rs` (10 removals, code changes)
Remove `code` from all 10 non-transparent `AppsRouteError` variants. Error codes change from `app_access_request_error-*` to `apps_route_error-*`:

| Variant | Old code | New derived code |
|---|---|---|
| `NotFound` | `app_access_request_error-not_found` | `apps_route_error-not_found` |
| `Expired` | `app_access_request_error-expired` | `apps_route_error-expired` |
| `AlreadyProcessed` | `app_access_request_error-already_processed` | `apps_route_error-already_processed` |
| `MissingRedirectUrl` | `app_access_request_error-missing_redirect_url` | `apps_route_error-missing_redirect_url` |
| `AppClientNotFound` | `app_access_request_error-app_client_not_found` | `apps_route_error-app_client_not_found` |
| `InvalidToolType` | `app_access_request_error-invalid_tool_type` | `apps_route_error-invalid_tool_type` |
| `ToolInstanceNotOwned` | `app_access_request_error-tool_instance_not_owned` | `apps_route_error-tool_instance_not_owned` |
| `ToolInstanceNotConfigured` | `app_access_request_error-tool_instance_not_configured` | `apps_route_error-tool_instance_not_configured` |
| `InsufficientPrivileges` | `app_access_request_error-insufficient_privileges` | `apps_route_error-insufficient_privileges` |
| `PrivilegeEscalation` | `app_access_request_error-privilege_escalation` | `apps_route_error-privilege_escalation` |

### `crates/routes_app/src/users/error.rs` (9 removals, code changes)
Remove `code` from all 9 `UsersRouteError` variants. Codes change from `user_route_error-*`/`access_request_error-*` to `users_route_error-*`:

| Variant | Old code | New derived code |
|---|---|---|
| `ListFailed` | `user_route_error-list_failed` | `users_route_error-list_failed` |
| `RoleChangeFailed` | `user_route_error-role_change_failed` | `users_route_error-role_change_failed` |
| `RemoveFailed` | `user_route_error-remove_failed` | `users_route_error-remove_failed` |
| `AlreadyPending` | `access_request_error-already_pending` | `users_route_error-already_pending` |
| `AlreadyHasAccess` | `access_request_error-already_has_access` | `users_route_error-already_has_access` |
| `PendingRequestNotFound` | `access_request_error-pending_request_not_found` | `users_route_error-pending_request_not_found` |
| `RequestNotFound` | `access_request_error-request_not_found` | `users_route_error-request_not_found` |
| `InsufficientPrivileges` | `access_request_error-insufficient_privileges` | `users_route_error-insufficient_privileges` |
| `FetchFailed` | `access_request_error-fetch_failed` | `users_route_error-fetch_failed` |

### `crates/routes_app/src/models/error.rs` (8 removals, code changes)
Remove `code` from 8 non-transparent `ModelRouteError` variants. Codes change from `model_error-*`/`create_alias_error-*`/`pull_error-*`/`metadata_error-*` to `model_route_error-*`:

| Variant | Old code | New derived code |
|---|---|---|
| `MetadataFetchFailed` | `model_error-metadata_fetch_failed` | `model_route_error-metadata_fetch_failed` |
| `AliasMismatch` | `create_alias_error-alias_mismatch` | `model_route_error-alias_mismatch` |
| `FileAlreadyExists` | `pull_error-file_already_exists` | `model_route_error-file_already_exists` |
| `InvalidRepoFormat` | `metadata_error-invalid_repo_format` | `model_route_error-invalid_repo_format` |
| `ListAliasesFailed` | `metadata_error-list_aliases_failed` | `model_route_error-list_aliases_failed` |
| `AliasNotFound` | `metadata_error-alias_not_found` | `model_route_error-alias_not_found` |
| `ExtractionFailed` | `metadata_error-extraction_failed` | `model_route_error-extraction_failed` |
| `EnqueueFailed` | `metadata_error-enqueue_failed` | `model_route_error-enqueue_failed` |

### Test files to update in `routes_app`

| Test file | Old code | New code |
|---|---|---|
| `src/apps/test_access_request.rs` | `app_access_request_error-tool_instance_not_owned` | `apps_route_error-tool_instance_not_owned` |
| | `app_access_request_error-tool_instance_not_configured` | `apps_route_error-tool_instance_not_configured` |
| | `app_access_request_error-privilege_escalation` | `apps_route_error-privilege_escalation` |
| `src/users/test_management_crud.rs` | `user_route_error-list_failed` | `users_route_error-list_failed` |
| | `user_route_error-remove_failed` | `users_route_error-remove_failed` |
| | `user_route_error-role_change_failed` | `users_route_error-role_change_failed` |
| `src/users/test_access_request_user.rs` | `access_request_error-already_has_access` | `users_route_error-already_has_access` |
| | `access_request_error-already_pending` | `users_route_error-already_pending` |
| | `access_request_error-pending_request_not_found` | `users_route_error-pending_request_not_found` |
| `src/users/test_access_request_admin.rs` | `access_request_error-insufficient_privileges` | `users_route_error-insufficient_privileges` |
| | `access_request_error-request_not_found` | `users_route_error-request_not_found` |
| `src/models/test_pull.rs` | `pull_error-file_already_exists` | `model_route_error-file_already_exists` |
| `src/models/test_metadata.rs` | `metadata_error-enqueue_failed` | `model_route_error-enqueue_failed` |
| | `metadata_error-invalid_repo_format` | `model_route_error-invalid_repo_format` |
| | `metadata_error-alias_not_found` | `model_route_error-alias_not_found` |

**Verify**: `cargo test -p routes_app`

---

## Phase 4: `lib_bodhiserver` crate (1 removal, no code change)

### `crates/lib_bodhiserver/src/error.rs`
- `BootstrapError::Parse` (line 32): remove `code = "bootstrap_error-parse"`. **Special handling**: change `#[error(transparent)]` to `#[error("{0}")]`. Remove `args_delegate = false`.

**Verify**: `cargo test -p lib_bodhiserver`

---

## Phase 5: `bodhi/src-tauri` crate (1 removal, code changes)

### `crates/bodhi/src-tauri/src/native_init.rs`
- `NativeError::Tauri` (line 31): remove `code = "tauri"`. New derived code: `"native_error-tauri"`. No `#[error(transparent)]` to change (already uses custom message). Keep `args_delegate = false` as-is.

**Verify**: `cargo check -p bodhi --features native`

---

## Phase 6: Full Backend Validation

Run `make test.backend` to verify all changes across all crates.

---

## Phase 7: OpenAPI & TypeScript Client Regeneration

```bash
cargo run --package xtask openapi
cd ts-client && npm run generate && npm run build
```

---

## Phase 8: Frontend Updates

### `crates/bodhi/src/app/ui/pull/PullForm.tsx` (line 36)
- Change `'pull_error-file_already_exists'` → `'model_route_error-file_already_exists'`

### `crates/bodhi/src/hooks/useModels.test.ts` (lines 606, 645, 660)
- Change all `'pull_error-file_already_exists'` → `'model_route_error-file_already_exists'`

### `crates/bodhi/src/test-utils/msw-v2/handlers/modelfiles.ts` (lines 333, 368)
- Change all `'pull_error-file_already_exists'` → `'model_route_error-file_already_exists'`

**Verify**: `cd crates/bodhi && npm run test`

---

## Phase 9: E2E Rebuild

```bash
make build.ui-rebuild
make test.napi
```

---

## Key Design Decision: `#[error(transparent)]` → `#[error("{0}")]`

5 variants have both `#[error(transparent)]` and explicit `code`. The derive macro determines transparency from `#[error(transparent)]` (parse.rs:172). Without explicit `code`, transparent variants delegate to inner `.code()` — but the inner types (`strum::ParseError`, `tower_sessions::session::Error`, `ParseError`) don't implement `AppError`, causing compile errors.

**Fix**: Change `#[error(transparent)]` to `#[error("{0}")]` for these 5 variants. Effect:
- Display output: identical (both format via `{0}`)
- `source()`: minor difference (transparent delegates to inner's source; `{0}` returns inner as source) — no practical impact
- `args()`: key changes from `"error"` to `"var_0"` in API response `param` field — acceptable for internal error types

**Affected variants**: `DbError::StrumParse`, `AuthError::TowerSession`, `LoginError::SessionError`, `LoginError::ParseError`, `BootstrapError::Parse`
