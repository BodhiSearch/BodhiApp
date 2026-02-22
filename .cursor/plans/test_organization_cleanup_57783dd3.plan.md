---
name: Test Organization Cleanup
overview: Migrate all routes_app tests/ directories to the test_*.rs sibling pattern (using routes_mcp as reference), split large test files by thematic concern, extract inline tests from objs/mcp.rs and services/queue_service.rs, apply rstest optimizations, consolidate fixtures to test_utils, and update CLAUDE.md conventions. Execute crate-level work in parallel via specialized sub-agents.
todos:
  - id: phase1-objs
    content: "Phase 1: Extract objs/src/mcp.rs inline tests into test_mcp_validation.rs and test_mcp_types.rs. Apply rstest #[case] to validation tests. Verify with cargo test -p objs."
    status: completed
  - id: phase2-services
    content: "Phase 2: Extract services/src/queue_service.rs inline tests into test_queue_service.rs. Verify with cargo test -p services."
    status: completed
  - id: phase3-api-models
    content: "Phase 3.1: Migrate routes_api_models tests/ to test_api_models_{crud,validation,prefix,sync,auth}.rs. Delete tests/ dir. Apply rstest optimizations."
    status: completed
  - id: phase3-auth
    content: "Phase 3.2: Migrate routes_auth tests/ to test_login_{initiate,callback,logout,resource_admin}.rs. Delete tests/ dir."
    status: completed
  - id: phase3-toolsets
    content: "Phase 3.3: Migrate routes_toolsets tests/ to test_toolsets_{crud,types,auth}.rs. Delete tests/ dir."
    status: completed
  - id: phase3-users
    content: "Phase 3.4: Migrate routes_users tests/ to test_access_request_{dto,user,admin,auth}.rs, test_management_{crud,auth}.rs, test_user_info.rs. Delete tests/ dir."
    status: completed
  - id: phase3-api-token
    content: "Phase 3.5: Migrate routes_api_token tests/ to test_api_token_{crud,security,auth}.rs. Delete tests/ dir."
    status: completed
  - id: phase3-models
    content: "Phase 3.6: Migrate routes_models tests/ to test_aliases_{crud,auth}.rs, test_pull.rs, test_metadata.rs. Delete tests/ dir."
    status: completed
  - id: phase3-oai
    content: "Phase 3.7: Migrate routes_oai tests/ to test_chat.rs, test_models.rs, test_live_chat.rs, test_live_utils.rs. Delete tests/ dir."
    status: completed
  - id: phase3-setup
    content: "Phase 3.8: Migrate routes_setup tests/ to test_setup.rs, test_setup_auth.rs. Delete tests/ dir."
    status: completed
  - id: phase3-apps
    content: "Phase 3.9: Migrate routes_apps tests/ to test_access_request.rs, test_access_request_auth.rs. Delete tests/ dir."
    status: completed
  - id: phase3-ollama
    content: "Phase 3.10: Migrate routes_ollama tests/ to test_handlers.rs. Delete tests/ dir."
    status: completed
  - id: phase3-settings
    content: "Phase 3.11: Migrate routes_settings tests/ to test_settings.rs. Delete tests/ dir."
    status: completed
  - id: phase4-fixtures
    content: "Phase 4: Consolidate shared test fixtures into routes_app test_utils/ with module-namespaced submodules."
    status: completed
  - id: phase5-validation
    content: "Phase 5: Full validation - cargo test -p objs, cargo test -p services, cargo test -p routes_app, make test.backend, cargo fmt --all."
    status: completed
  - id: phase6-claude-md
    content: "Phase 6: Update root CLAUDE.md and relevant crate CLAUDE.md files with test organization conventions."
    status: completed
isProject: false
---

# Test Organization Cleanup

## Reference Pattern

`[routes_mcp/](crates/routes_app/src/routes_mcp/)` is the canonical reference. It has NO `tests/` directory. Each handler file declares its own test files:

```rust
// In servers.rs
#[cfg(test)]
#[path = "test_servers.rs"]
mod test_servers;
```

Test files live as siblings: `test_servers.rs`, `test_mcps.rs`, `test_auth_configs.rs`, `test_oauth_utils.rs`, `test_oauth_flow.rs`.

## Pattern Conventions

**Test declaration anchor**: Pattern A (source file anchor) by default. Each handler file declares its own `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`. Use Pattern B (mod.rs anchor) when a test file covers multiple handlers or shared concerns.

**Naming**: `test_<handler>_<feature>.rs` when splitting by CRUD/feature. `test_<handler>.rs` when file is focused.

**Auth tests**: Always extract to a dedicated `test_<module>_auth.rs` declared from `mod.rs` (since auth tests cover the module as a whole, not a single handler).

**Threshold**: Use `test_*.rs` when combined file exceeds 500 lines. Inline `#[cfg(test)] mod tests {}` acceptable for small files under threshold. Going forward, prefer `test_*.rs` for new code.

**rstest optimizations**: Apply `#[case]` for parameterized tests, `#[fixture]` for reusable setup, `#[values]` for combinatorial testing. Do this during the split, not as a separate pass.

**Fixture extraction**: Move reusable fixtures to crate-level `test_utils/` with module-namespaced submodules. Only transfer/organize in this plan -- downstream reuse across crates is deferred.

---

## Phase 1: objs crate

### 1.1 `objs/src/mcp.rs` (728 lines: 529 impl + 198 tests)

Split tests into two files by concern:

- `**test_mcp_validation.rs`** -- 20 validation tests (slug, description, server name/URL, auth config name)
- `**test_mcp_types.rs`** -- 7 tests (RegistrationType serde/display/from_str/default + OAuth endpoint URL validation)

Declaration in `mcp.rs`:

```rust
#[cfg(test)]
#[path = "test_mcp_validation.rs"]
mod test_mcp_validation;

#[cfg(test)]
#[path = "test_mcp_types.rs"]
mod test_mcp_types;
```

Apply rstest `#[case]` to parameterize the validation accept/reject tests.

**Verify**: `cargo test -p objs`

---

## Phase 2: services crate

### 2.1 `services/src/queue_service.rs` (514 lines: 420 impl + 93 tests)

Extract 93-line test block to `test_queue_service.rs`.

Declaration in `queue_service.rs`:

```rust
#[cfg(test)]
#[path = "test_queue_service.rs"]
mod test_queue_service;
```

**Verify**: `cargo test -p services`

---

## Phase 3: routes_app crate (11 modules to migrate)

All 11 route modules currently use `tests/` subdirectory pattern. Migrate every one to `test_*.rs` sibling pattern. Delete `tests/` directories and their `mod.rs` files after migration.

Modules are independent and can be processed in parallel by sub-agents. Each sub-agent handles one module: rename/move test files, update mod.rs declarations, apply rstest optimizations where applicable, extract fixtures to module-local helpers.

### 3.1 `routes_api_models/` -- SPLIT (1,489 lines)

Source: `[tests/api_models_test.rs](crates/routes_app/src/routes_api_models/tests/api_models_test.rs)` (1,489 lines)

Split into thematic files:

- `**test_api_models_crud.rs`** -- List, Create (success + UUID generation), Get, Update, Delete handlers (~580 lines)
- `**test_api_models_validation.rs`** -- Create validation errors, forward_all tests, creds enum validation, API key masking (~250 lines)
- `**test_api_models_prefix.rs`** -- Prefix lifecycle, duplicate prefix, forward_all requires prefix (~350 lines)
- `**test_api_models_sync.rs`** -- Sync models handler, direct DB operations (~170 lines)
- `**test_api_models_auth.rs`** -- Reject unauthenticated, reject insufficient role, allow power user (~80 lines)

Declare from `api_models.rs` (crud, validation, prefix, sync) and `mod.rs` (auth). Also move `test_types.rs` -- it already uses the target pattern via mod.rs.

Delete: `tests/api_models_test.rs`, `tests/mod.rs`

### 3.2 `routes_auth/` -- SPLIT (1,024 lines)

Source: `[tests/login_test.rs](crates/routes_app/src/routes_auth/tests/login_test.rs)` (1,024 lines)

Split into thematic files:

- `**test_login_initiate.rs`** -- Auth initiate handler tests (5 tests, ~260 lines)
- `**test_login_callback.rs**` -- Auth callback handler success + error cases, PKCE (7 tests, ~420 lines)
- `**test_login_logout.rs**` -- Logout handler test (~40 lines)
- `**test_login_resource_admin.rs**` -- Resource admin callback flow with helpers (~190 lines)

Declare from `login.rs`. No separate auth test file needed (these ARE the auth tests).

Delete: `tests/login_test.rs`, `tests/mod.rs`

### 3.3 `routes_toolsets/` -- SPLIT (987 lines)

Source: `[tests/toolsets_test.rs](crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs)` (987 lines)

Split into thematic files:

- `**test_toolsets_crud.rs**` -- List, Create, Get, Update, Delete, Execute (17 tests, ~600 lines)
- `**test_toolsets_types.rs**` -- List types, Enable/Disable type (3 tests, ~180 lines)
- `**test_toolsets_auth.rs**` -- Reject unauthenticated, role checks, API token rejection (~120 lines)

Declare crud and types from `toolsets.rs`, auth from `mod.rs`.

Delete: `tests/toolsets_test.rs`, `tests/mod.rs`

### 3.4 `routes_users/` -- SPLIT + MIGRATE (3 files: 814 + 572 + 313 = 1,699 total)

Sources:

- `[tests/access_request_test.rs](crates/routes_app/src/routes_users/tests/access_request_test.rs)` (814 lines)
- `[tests/management_test.rs](crates/routes_app/src/routes_users/tests/management_test.rs)` (572 lines)
- `[tests/user_info_test.rs](crates/routes_app/src/routes_users/tests/user_info_test.rs)` (313 lines)

**access_request_test.rs** (814 lines) -- Split:

- `**test_access_request_dto.rs`** -- DTO/serialization tests (~55 lines)
- `**test_access_request_user.rs`** -- User-facing endpoints: request access, request status (~230 lines)
- `**test_access_request_admin.rs`** -- Admin endpoints: approve, reject, list pending/all (~400 lines)
- `**test_access_request_auth.rs`** -- Auth tier tests (~80 lines)

Declare dto/user/admin from `access_request.rs`, auth from `mod.rs`.

**management_test.rs** (572 lines) -- Split:

- `**test_management_crud.rs`** -- User list, role change, removal, session clearing (~490 lines)
- `**test_management_auth.rs`** -- Auth tier tests (~80 lines)

Declare crud from `management.rs`, auth from `mod.rs`.

**user_info_test.rs** (313 lines) -- Rename:

- `**test_user_info.rs`** -- All tests (under threshold, keep as one file)

Declare from `user_info.rs`.

Delete: `tests/` directory entirely.

### 3.5 `routes_api_token/` -- SPLIT (770 lines)

Source: `[tests/api_token_test.rs](crates/routes_app/src/routes_api_token/tests/api_token_test.rs)` (770 lines)

Split into thematic files:

- `**test_api_token_crud.rs`** -- List, Create (success/errors), Update (~560 lines)
- `**test_api_token_security.rs`** -- Privilege escalation tests (~90 lines)
- `**test_api_token_auth.rs`** -- Auth tier tests (~60 lines)

Declare crud and security from `route_api_token.rs`, auth from `mod.rs`.

Delete: `tests/api_token_test.rs`, `tests/mod.rs`

### 3.6 `routes_models/` -- MIGRATE (3 files: 554 + 392 + 288 = 1,234 total)

Sources:

- `[tests/aliases_test.rs](crates/routes_app/src/routes_models/tests/aliases_test.rs)` (554 lines)
- `[tests/pull_test.rs](crates/routes_app/src/routes_models/tests/pull_test.rs)` (392 lines)
- `[tests/metadata_test.rs](crates/routes_app/src/routes_models/tests/metadata_test.rs)` (288 lines)

**aliases_test.rs** (554 lines) -- Split:

- `**test_aliases_crud.rs`** -- CRUD + pagination + sorting + copy (~470 lines)
- `**test_aliases_auth.rs`** -- Auth tier tests (~80 lines)

Declare crud from `aliases.rs`, auth from `mod.rs`.

**pull_test.rs** (392 lines) -- Rename to `**test_pull.rs`** (under threshold). Declare from `pull.rs`.

**metadata_test.rs** (288 lines) -- Rename to `**test_metadata.rs`** (under threshold). Declare from `metadata.rs`.

Delete: `tests/` directory entirely.

### 3.7 `routes_oai/` -- MIGRATE (4 files: 374 + 365 + 652 + 128 = 1,519 total)

Sources:

- `[tests/chat_test.rs](crates/routes_app/src/routes_oai/tests/chat_test.rs)` (374 lines)
- `[tests/models_test.rs](crates/routes_app/src/routes_oai/tests/models_test.rs)` (365 lines)
- `[tests/test_live_chat.rs](crates/routes_app/src/routes_oai/tests/test_live_chat.rs)` (652 lines)
- `[tests/test_live_utils.rs](crates/routes_app/src/routes_oai/tests/test_live_utils.rs)` (128 lines)

Rename to siblings:

- `**test_chat.rs`** (374 lines) -- Declare from `chat.rs`
- `**test_models.rs`** (365 lines) -- Declare from `models.rs`
- `**test_live_chat.rs`** (652 lines) -- Declare from `chat.rs` (second test module)
- `**test_live_utils.rs`** (128 lines) -- Declare from `mod.rs` (shared utility)

Delete: `tests/` directory entirely.

### 3.8 `routes_setup/` -- MIGRATE (565 lines)

Source: `[tests/setup_test.rs](crates/routes_app/src/routes_setup/tests/setup_test.rs)` (565 lines)

Split into:

- `**test_setup.rs`** -- Main setup tests (~480 lines)
- `**test_setup_auth.rs**` -- Auth tier tests (~80 lines)

Declare test_setup from `route_setup.rs`, auth from `mod.rs`.

Delete: `tests/setup_test.rs`, `tests/mod.rs`

### 3.9 `routes_apps/` -- MIGRATE (514 lines)

Source: `[tests/access_request_test.rs](crates/routes_app/src/routes_apps/tests/access_request_test.rs)` (514 lines)

Split into:

- `**test_access_request.rs**` -- Approve/deny handlers (~430 lines)
- `**test_access_request_auth.rs**` -- Auth tier tests if present (~80 lines)

Declare from `handlers.rs` and `mod.rs`.

Delete: `tests/access_request_test.rs`, `tests/mod.rs`

### 3.10 `routes_ollama/` -- MIGRATE (194 lines)

Source: `[tests/handlers_test.rs](crates/routes_app/src/routes_ollama/tests/handlers_test.rs)` (194 lines)

Rename to `**test_handlers.rs**` (under threshold, single file). Declare from `handlers.rs`.

Delete: `tests/handlers_test.rs`, `tests/mod.rs`

### 3.11 `routes_settings/` -- MIGRATE (487 lines)

Source: `[tests/settings_test.rs](crates/routes_app/src/routes_settings/tests/settings_test.rs)` (487 lines)

Rename to `**test_settings.rs**` (under threshold, single file). Declare from `route_settings.rs`.

Delete: `tests/settings_test.rs`, `tests/mod.rs`

---

## Phase 4: Fixture Consolidation (routes_app)

After all migrations, consolidate shared test fixtures into `[test_utils/](crates/routes_app/src/test_utils/)`:

- Move common router setup helpers that are duplicated across modules
- Create module-namespaced submodules (e.g., `test_utils/api_models.rs` for api_models-specific fixtures)
- Keep module-specific helpers in the test files themselves

This is organizational only -- no cross-crate reuse changes.

---

## Phase 5: Validation

After all crate changes:

1. `cargo test -p objs`
2. `cargo test -p services`
3. `cargo test -p routes_app`
4. `make test.backend` (full backend validation)
5. `cargo fmt --all`

---

## Phase 6: Update CLAUDE.md

Update root [CLAUDE.md](CLAUDE.md) and relevant crate CLAUDE.md files with the test organization convention:

**Convention to document**:

- Prefer `test_*.rs` sibling files for tests when combined file exceeds 500 lines
- Use `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;` in source file (Pattern A)
- Use mod.rs declarations for cross-handler test files (auth tier tests, shared concerns)
- Split large test files by thematic concern: `test_<handler>_<feature>.rs`
- For CRUD routes: `test_<handler>_crud.rs`, `test_<handler>_auth.rs`, `test_<handler>_<feature>.rs`
- Auth tier tests always in dedicated `test_<module>_auth.rs`
- Reference implementation: `routes_mcp/` module
- Apply rstest: `#[case]` for parameterized, `#[fixture]` for reusable setup, `#[values]` for combinatorial

---

## Execution Strategy

Work is crate-independent and can be parallelized:

- **Sub-agent 1**: Phase 1 (objs/mcp.rs) + Phase 2 (services/queue_service.rs)
- **Sub-agents 2-5**: Phase 3 routes_app modules (batch 2-3 modules per sub-agent)
- **Final**: Phase 4 (fixture consolidation), Phase 5 (validation), Phase 6 (CLAUDE.md)

Each sub-agent:

1. Reads the source test file
2. Creates new `test_*.rs` files with proper module structure
3. Updates source file / mod.rs with declarations
4. Removes old `tests/` files
5. Applies rstest optimizations in the split files
6. Runs `cargo test -p <crate>` to verify

---

## Scope Summary


| Crate      | Modules              | Files Migrated | Files Created  | Split?          |
| ---------- | -------------------- | -------------- | -------------- | --------------- |
| objs       | 1 (mcp.rs)           | 0 (inline)     | 2              | Yes             |
| services   | 1 (queue_service.rs) | 0 (inline)     | 1              | No              |
| routes_app | 11 modules           | 20 test files  | ~35 test files | 6 modules split |
| **Total**  | **13**               | **20**         | **~38**        |                 |


**Directories deleted**: 11 `tests/` subdirectories in routes_app modules.