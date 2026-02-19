---
name: Codebase File Size Cleanup
overview: Systematically split 30+ oversized Rust, TSX, and test files across the monorepo to a 500-700 line target by extracting implementation concerns and separating tests, following the established module folder pattern crate-by-crate.
todos:
  - id: phase1-objs
    content: "Phase 1: objs crate - Split gguf/capabilities.rs tests into test_capabilities.rs. Verify with `cargo test -p objs`."
    status: completed
  - id: phase2-services-db
    content: "Phase 2a: services crate - Split db/service.rs (2,462 lines) into 8 repository files + split db/tests.rs into 4 test files. Update db/mod.rs. Verify with `cargo test -p services`."
    status: completed
  - id: phase2-services-auth
    content: "Phase 2b: services crate - Convert auth_service.rs to module folder (service.rs + tests.rs). Verify with `cargo test -p services`."
    status: completed
  - id: phase2-services-hub
    content: "Phase 2c: services crate - Convert hub_service.rs to module folder (service.rs + tests.rs). Verify with `cargo test -p services`."
    status: completed
  - id: phase2-services-exa
    content: "Phase 2d: services crate - Convert exa_service.rs to module folder (service.rs + tests.rs). Verify with `cargo test -p services`."
    status: completed
  - id: phase2-services-tool
    content: "Phase 2e: services crate - Split tool_service/service.rs (797 lines) by concern. Verify with `cargo test -p services`."
    status: completed
  - id: phase2-services-setting
    content: "Phase 2f: services crate - Split setting_service/service.rs (776 lines) by concern. Verify with `cargo test -p services`."
    status: completed
  - id: phase3-auth-token
    content: "Phase 3a: auth_middleware crate - Convert token_service.rs to module folder (service.rs + tests.rs). Verify with `cargo test -p auth_middleware`."
    status: completed
  - id: phase3-auth-middleware
    content: "Phase 3b: auth_middleware crate - Convert auth_middleware.rs to module folder (middleware.rs + tests.rs). Verify with `cargo test -p auth_middleware`."
    status: completed
  - id: phase3-auth-access
    content: "Phase 3c: auth_middleware crate - Convert access_request_auth_middleware.rs to module folder (middleware.rs + tests.rs). Verify with `cargo test -p auth_middleware`."
    status: completed
  - id: phase4-routes-mod-rs
    content: "Phase 4a: routes_app crate - Extract implementation from routes_setup/mod.rs, routes_api_token/mod.rs, routes_settings/mod.rs into named files. Verify with `cargo test -p routes_app`."
    status: completed
  - id: phase4-routes-openapi
    content: "Phase 4b: routes_app crate - Separate openapi.rs tests into test_openapi.rs. Separate routes.rs tests into test_routes.rs. Separate types.rs tests into test_types.rs. Verify with `cargo test -p routes_app`."
    status: completed
  - id: phase5-lib-bodhiserver
    content: "Phase 5: lib_bodhiserver crate - Separate tests from app_service_builder.rs and app_dirs_builder.rs into test files. Verify with `cargo test -p lib_bodhiserver`."
    status: completed
  - id: phase6-server-core
    content: "Phase 6: server_core crate - Separate tests from shared_rw.rs into test_shared_rw.rs. Verify with `cargo test -p server_core`."
    status: completed
  - id: phase7-errmeta
    content: "Phase 7: errmeta_derive crate - Split proc-macro helpers from lib.rs into submodules. Verify with `cargo test -p errmeta_derive`."
    status: completed
  - id: phase8-backend-validation
    content: "Phase 8: Run `make test.backend` and `cargo fmt --all` to validate all Rust changes."
    status: completed
  - id: phase9-frontend
    content: "Phase 9: Frontend - Extract components from models/page.tsx, mcps/new/page.tsx, access-requests/review/page.tsx, AliasForm.tsx. Move co-located tests. Verify with `npm run test && npm run format`."
    status: completed
  - id: phase10-final
    content: "Phase 10: Final validation - `make test.backend`, `npm run test`, `make build.ui-rebuild`, formatting."
    status: completed
isProject: false
---

# Codebase File Size Cleanup Plan

Target: All files should be in the 500-700 line range. Files exceeding this need splitting via established conventions.

## Conventions Established

- **Rust module folder pattern**: `mod.rs` is purely declarations and re-exports (under ~10 lines of code). Implementation goes in named files.
- **Naming**: `route_<handler>.rs` / `test_route_<handler>.rs` for route modules; `<repository>.rs` / `test_<repository>.rs` for db modules.
- **Test placement**: Inline `#[cfg(test)]` if combined file stays under 500 lines; otherwise, separate test file.
- **Standalone files exceeding 500 total lines**: Convert to module folder with `mod.rs` + implementation + tests, using the lighter `mod.rs` + `test_*.rs` sibling approach when only tests push it over.
- **Implementation-only files (no tests) 500-700 lines**: Defer unless over 700 lines.
- **Frontend extraction**: Co-located in the same page folder (not in a components/ subfolder).
- **E2E tests**: Skip restructuring for now.
- **Execution order**: `objs` -> `services` -> `auth_middleware` -> `routes_app` -> `lib_bodhiserver` -> `server_core` -> frontend. Test after each crate.

---

## Phase 1: `objs` crate

### 1.1 `objs/src/gguf/capabilities.rs` (715 lines: 449 impl + 266 tests)

Already in a module folder. Implementation at 449 lines is fine. Separate tests to keep under 500:

- Create `crates/objs/src/gguf/test_capabilities.rs` with the test code (lines 450-715)
- Update `capabilities.rs` to have `#[cfg(test)] mod tests;` pointing to `test_capabilities`
- Alternatively, since the `gguf/` folder already exists with a `mod.rs`, add `#[cfg(test)] mod test_capabilities;` to the `mod.rs`

**Verify**: `cargo test -p objs`

### 1.2 `objs/src/lib.rs` - `impl_error_from!` macro (~10 lines)

This macro definition is borderline. At ~10 lines it's within the "minor deviations under 10 lines" tolerance. **Leave as-is.**

---

## Phase 2: `services` crate

### 2.1 `services/src/db/service.rs` (2,462 lines) -- HIGHEST PRIORITY

Split into one file per repository trait implementation. The `db/` folder already has `mod.rs`.

**New files:**

- `db/db_core.rs` -- `DbCore` impl + shared helpers (`seed_toolset_configs`, `get_by_col`, `parse_user_alias_row`) (~148 lines)
- `db/model_repository.rs` -- `ModelRepository` impl (~742 lines)
- `db/access_repository.rs` -- `AccessRepository` impl (~259 lines)
- `db/token_repository.rs` -- `TokenRepository` impl (~218 lines)
- `db/toolset_repository.rs` -- `ToolsetRepository` impl (~360 lines)
- `db/user_alias_repository.rs` -- `UserAliasRepository` impl (~164 lines)
- `db/access_request_repository.rs` -- `AccessRequestRepository` impl (~260 lines)
- `db/mcp_repository.rs` -- `McpRepository` impl (~421 lines)

**Update `db/mod.rs`**: Add `mod` declarations for each new file. Keep `DbService` trait definition and `SqliteDbService` struct definition and blanket impl in `db/mod.rs` (or move to `db/db_core.rs`).

### 2.2 `services/src/db/tests.rs` (1,524 lines)

Split into per-repository test files:

- `db/test_model_repository.rs` (~847 lines) -- This is still large but it's a separate test file for the largest repository. Acceptable per user's "one file, separate test file" decision.
- `db/test_access_repository.rs` (~119 lines)
- `db/test_token_repository.rs` (~193 lines)
- `db/test_access_request_repository.rs` (~504 lines)

Update `db/mod.rs` with `#[cfg(test)] mod test_model_repository;` etc.

### 2.3 `services/src/auth_service.rs` (1,434 lines: 859 impl + 753 tests)

User chose: separate tests only. Convert to module folder:

- `auth_service/mod.rs` -- module declarations and re-exports
- `auth_service/service.rs` -- implementation (859 lines, exceeds 700 but user explicitly chose to keep implementation as-is)
- `auth_service/tests.rs` -- tests (753 lines)

### 2.4 `services/src/hub_service.rs` (940 lines: 494 impl + 446 tests)

User chose: split hub_service into module folder. Convert:

- `hub_service/mod.rs` -- module declarations and re-exports
- `hub_service/service.rs` -- implementation (494 lines, under 500 -- good)
- `hub_service/tests.rs` -- tests (446 lines)

### 2.5 `services/src/exa_service.rs` (843 lines: 409 impl + 434 tests)

Same pattern as hub_service. Convert to module folder:

- `exa_service/mod.rs`
- `exa_service/service.rs` (409 lines)
- `exa_service/tests.rs` (434 lines)

### 2.6 `services/src/tool_service/service.rs` (797 lines, no tests)

Implementation-only, over 700 lines. Already in a module folder. Needs implementation split by concern. Defer decision on exact split until implementation, but likely:

- `tool_service/service.rs` -- core CRUD operations
- `tool_service/execution.rs` -- tool execution delegation

### 2.7 `services/src/setting_service/service.rs` (776 lines, no tests)

Implementation-only, over 700 lines. Already in a module folder. Likely split:

- `setting_service/service.rs` -- core settings management
- `setting_service/env_settings.rs` -- environment variable integration and helper methods

### 2.8 `services/src/mcp_service/service.rs` (691 lines, no tests)

Under 700 lines, implementation-only, already in module folder. **Defer.**

**Verify**: `cargo test -p services` after each sub-phase, then `cargo test` for all.

---

## Phase 3: `auth_middleware` crate

### 3.1 `auth_middleware/src/token_service.rs` (1,555 lines: 546 impl + 1,192 tests)

User chose: split code vs tests. Convert to module folder:

- `token_service/mod.rs` -- module declarations and re-exports
- `token_service/service.rs` -- implementation (546 lines)
- `token_service/tests.rs` -- tests (1,192 lines -- large but it's a test file)

### 3.2 `auth_middleware/src/auth_middleware.rs` (1,234 lines: 369 impl + 960 tests)

Same pattern:

- `auth_middleware_impl/mod.rs` -- Note: can't name folder same as file, so use descriptive name, or rename the module. The existing module is declared as `mod auth_middleware;` in `lib.rs`. Convert to folder:
- `auth_middleware/mod.rs` (the folder replaces the file)
- `auth_middleware/middleware.rs` -- implementation (369 lines)
- `auth_middleware/tests.rs` -- tests (960 lines)

### 3.3 `auth_middleware/src/access_request_auth_middleware.rs` (825 lines: 253 impl + 572 tests)

Convert to module folder:

- `access_request_auth_middleware/mod.rs`
- `access_request_auth_middleware/middleware.rs` (253 lines)
- `access_request_auth_middleware/tests.rs` (572 lines)

**Verify**: `cargo test -p auth_middleware`, then `cargo test`

---

## Phase 4: `routes_app` crate

### 4.1 `routes_setup/mod.rs` (210 lines) -- Extract implementation from mod.rs

Convert to proper module folder pattern:

- `routes_setup/mod.rs` -- only `mod` declarations and `pub use` re-exports
- `routes_setup/route_setup.rs` -- error enum, types, handler functions (all current implementation)

### 4.2 `routes_api_token/mod.rs` (346 lines) -- Extract implementation from mod.rs

- `routes_api_token/mod.rs` -- only `mod` declarations and `pub use` re-exports
- `routes_api_token/route_api_token.rs` -- error enum, types, handler functions

### 4.3 `routes_settings/mod.rs` (248 lines) -- Extract implementation from mod.rs

- `routes_settings/mod.rs` -- only `mod` declarations and `pub use` re-exports
- `routes_settings/route_settings.rs` -- error enum, types, handler functions

### 4.4 `routes_app/src/shared/openapi.rs` (1,287 lines: 734 impl + 718 tests)

User chose: separate tests only.

- Keep `openapi.rs` as-is for implementation (734 lines, slightly over but declarative)
- Create `shared/test_openapi.rs` or `shared/tests/openapi_test.rs` for tests
- Update mod.rs or openapi.rs with `#[cfg(test)] mod test_openapi;`

### 4.5 `routes_app/src/routes.rs` (672 lines: 505 impl + 166 tests)

User chose: separate tests.

- Keep `routes.rs` for implementation (505 lines)
- Create `test_routes.rs` for tests (166 lines)
- Update with `#[cfg(test)] mod test_routes;` -- but since routes.rs is not in a folder, the test file needs to be declared from `lib.rs` or routes.rs itself. Use `#[cfg(test)] #[path = "test_routes.rs"] mod test_routes;`

### 4.6 `routes_api_models/types.rs` (762 lines: 411 impl + 351 tests)

Already in module folder. Separate tests:

- Keep `types.rs` for implementation (411 lines)
- Create `test_types.rs` for tests, declared from `mod.rs`: `#[cfg(test)] mod test_types;`

### 4.7 Test files in routes_app (already separate, in 500-700+ range)

These test files are in dedicated `tests/` folders and are test-only. They are large but acceptable as test files:

- `routes_api_models/tests/api_models_test.rs` (1,310 lines) -- very large test file, but splitting individual test files is low ROI
- `routes_auth/tests/login_test.rs` (949 lines)
- `routes_toolsets/tests/toolsets_test.rs` (874 lines)
- `routes_mcps/tests/mcps_test.rs` (891 lines)

**Decision**: Defer splitting pure test files -- they're in dedicated test folders already and splitting adds complexity with minimal benefit.

**Verify**: `cargo test -p routes_app`, then `cargo test`

---

## Phase 5: `lib_bodhiserver` crate

### 5.1 `app_service_builder.rs` (655 lines: 509 impl + 146 tests)

509 impl is right at boundary. Separate tests:

- Keep `app_service_builder.rs` (509 lines)
- Create `test_app_service_builder.rs` (146 lines)

### 5.2 `app_dirs_builder.rs` (602 lines: 280 impl + 322 tests)

280 impl is well under, but total exceeds 500. Separate tests:

- Keep `app_dirs_builder.rs` (280 lines)
- Create `test_app_dirs_builder.rs` (322 lines)

**Verify**: `cargo test -p lib_bodhiserver`, then `cargo test`

---

## Phase 6: `server_core` crate

### 6.1 `shared_rw.rs` (615 lines: 354 impl + 261 tests)

Separate tests:

- Keep `shared_rw.rs` (354 lines)
- Create `test_shared_rw.rs` (261 lines)

**Verify**: `cargo test -p server_core`, then `cargo test`

---

## Phase 7: `errmeta_derive` crate

`errmeta_derive/src/lib.rs` (1,018 lines) is a proc-macro crate. Proc-macros must export from `lib.rs`, but helper logic can go in submodules. Split macro parsing/generation helpers into modules, keeping only the `#[proc_macro_derive]` entry points in `lib.rs`. Defer if complexity is high.

---

## Phase 8: Backend validation

Run `make test.backend` to verify all Rust changes together.
Run `cargo fmt --all` for formatting.

---

## Phase 9: Frontend (`crates/bodhi/src/`)

### 9.1 `app/ui/models/page.tsx` (630 lines)

Extract co-located components:

- `app/ui/models/SourceBadge.tsx` -- SourceBadge component
- `app/ui/models/ModelTableRow.tsx` -- renderRow logic as a component
- `app/ui/models/ModelActions.tsx` -- actionUi logic as a component

### 9.2 `app/ui/mcps/new/page.tsx` (600 lines)

Extract co-located components:

- `app/ui/mcps/new/McpServerSelector.tsx` -- MCP server combobox
- `app/ui/mcps/new/ToolSelection.tsx` -- tool selection UI section

### 9.3 `app/ui/apps/access-requests/review/page.tsx` (523 lines)

Extract co-located components:

- `app/ui/apps/access-requests/review/ToolTypeCard.tsx`
- `app/ui/apps/access-requests/review/McpServerCard.tsx`

### 9.4 `app/ui/models/AliasForm.tsx` (464 lines)

Under 500 but close. Extract if natural:

- `app/ui/models/FormFieldWithTooltip.tsx` -- reusable form field component

### 9.5 `hooks/use-chat.tsx` (439 lines) and `hooks/use-chat-completions.ts` (369 lines)

Under 500. **Leave as-is.**

**Verify**: `cd crates/bodhi && npm run test && npm run format`

---

## Phase 10: Final validation

1. `make test.backend` -- all Rust tests
2. `cd crates/bodhi && npm run test` -- frontend tests
3. `make build.ui-rebuild` -- rebuild embedded UI
4. `cargo fmt --all && cd crates/bodhi && npm run format` -- formatting

---

## Summary of Changes


| Crate           | Files to Split | New Files Created | Priority |
| --------------- | -------------- | ----------------- | -------- |
| objs            | 1              | 1                 | Low      |
| services        | 6              | ~18               | High     |
| auth_middleware | 3              | ~9                | High     |
| routes_app      | 6              | ~8                | Medium   |
| lib_bodhiserver | 2              | 2                 | Low      |
| server_core     | 1              | 1                 | Low      |
| errmeta_derive  | 1              | ~2                | Low      |
| frontend (TSX)  | 4              | ~8                | Medium   |
| **Total**       | **~24**        | **~49**           |          |


