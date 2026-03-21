# Plan: Uniform Service Crate Restructuring

## Context

The `crates/services/src/` crate has inconsistent organization: some services are directories with `_service` suffix, some are single `.rs` files, naming varies, and domain groupings don't align with the UI/routes layers. This refactor creates a uniform feature-based directory structure, preparing for a future merge of `crates/objs` into `crates/services`.

## Target Directory Structure

```
crates/services/src/
├── lib.rs
├── macros.rs, env_wrapper.rs, objs.rs, app_service.rs, token.rs  (stay at root)
│
├── settings/              ← setting_service/
├── toolsets/              ← tool_service/ + exa_service/
├── mcps/                  ← mcp_service/
├── apps/                  ← app_instance_service.rs
├── app_access_requests/   ← access_request_service/
├── models/                ← hub_service/ + data_service.rs + progress_tracking.rs
├── auth/                  ← auth_service/ + session_service/
├── tokens/                ← NEW thin service wrapping db::TokenRepository
├── ai_apis/               ← ai_api_service.rs
├── utils/                 ← cache, network, keyring, concurrency, queue
├── db/                    (UNCHANGED)
└── test_utils/            (UNCHANGED)
```

## Standard Internal Convention

Each feature directory follows:
- `mod.rs` — only module declarations + `pub use` re-exports (no implementation code)
- `error.rs` — error types (when they exist, not forced)
- `<feature>_service.rs` — service trait + default impl
- `<feature>_objs.rs` — domain objects (only where inline structs need extraction)
- `test_<feature>_service.rs` — tests (sibling file pattern per CLAUDE.md)

## Import Fixup Strategy (applies to all phases)

- **Intra-module**: prefer `super::` (e.g., `execution.rs` → `super::exa_service::ExaService`)
- **Cross-module**: prefer `crate::FlatType` via lib.rs re-exports (e.g., `crate::HubService`)
- **Downstream crates**: no changes needed — they use `services::FlatType` preserved by `pub use` re-exports
- **Search for breakage**: `grep -rn 'crate::old_module_name' crates/services/src/` after each migration

---

## Phase 1: Simple Directory Renames (3 services)

**Scope**: Rename 3 existing directories. No merges, no file promotions.

### 1a: `setting_service/` → `settings/`
- `git mv setting_service settings`
- Rename internal files: `service.rs` → `setting_service.rs`, `tests.rs` → `test_setting_service.rs`, `test_service_db.rs` → `test_setting_service_db.rs`
- Update `mod.rs` with new file names
- Update `lib.rs`: `mod setting_service` → `mod settings`; `pub use setting_service::*` → `pub use settings::*`
- Fix any `crate::setting_service::` qualified paths

### 1b: `mcp_service/` → `mcps/`
- `git mv mcp_service mcps`
- Rename: `service.rs` → `mcp_service.rs`, `tests.rs` → `test_mcp_service.rs`
- Update `mod.rs`, `lib.rs`
- Fix `crate::mcp_service::` paths

### 1c: `access_request_service/` → `app_access_requests/`
- `git mv access_request_service app_access_requests`
- Rename: `service.rs` → `access_request_service.rs`
- Update `mod.rs`, `lib.rs`
- Fix internal path: `crate::access_request_service::error::` → `super::error::`

### Gate Check
```bash
cargo test -p services
```

---

## Phase 2: Directory Merges (2 services)

**Scope**: Merge multiple sources into single feature directories.

### 2a: Merge `exa_service/` into `tool_service/`, rename to `toolsets/`
- Move `exa_service/service.rs` → `tool_service/exa_service.rs`
- Move `exa_service/tests.rs` → `tool_service/test_exa_service.rs`
- Remove `exa_service/` directory
- Rename `tool_service/` → `toolsets/`
- Rename: `service.rs` → `tool_service.rs`, `tests.rs` → `test_tool_service.rs`
- Update `toolsets/mod.rs` to include exa modules
- Update `lib.rs`: remove `exa_service` module, rename `tool_service` → `toolsets`
- Fix `crate::exa_service::ExaService` refs in `execution.rs` / `service.rs` → `super::exa_service::ExaService`

### 2b: Merge `hub_service/` + `data_service.rs` + `progress_tracking.rs` → `models/`
- Create `models/` directory
- Move `hub_service/service.rs` → `models/hub_service.rs`
- Move `hub_service/tests.rs` → `models/test_hub_service.rs`
- Move `data_service.rs` → `models/data_service.rs`
- Move `progress_tracking.rs` → `models/progress_tracking.rs`
- Remove `hub_service/` directory
- Create `models/mod.rs` with all declarations + re-exports
- Update `lib.rs`: remove individual modules, add `mod models; pub use models::*`
- Errors stay inline (small per-service error enums)

### Gate Check
```bash
cargo test -p services
```

---

## Phase 3: Complex Merge + File Promotions (3 services)

**Scope**: The highest-complexity phase — auth merge with struct extraction, plus 2 file-to-directory promotions.

### 3a: Merge `auth_service/` + `session_service/` → `auth/` + extract `auth_objs.rs`
- Create `auth/` directory
- Move `auth_service/service.rs` → `auth/auth_service.rs`
- Move `auth_service/tests.rs` → `auth/test_auth_service.rs`
- Move all `session_service/` files into `auth/` (rename `error.rs` → `session_error.rs`)
- Move `test_session_service.rs` from root into `auth/`
- Extract from `auth_service.rs`:
  - `error.rs` — `AuthServiceError` enum
  - `auth_objs.rs` — 6 public structs: `ClientRegistrationResponse`, `RegisterAccessRequestConsentResponse`, `AppClientInfo`, `RegisterClientRequest`, `AppClientToolset`, `UserListResponse`
- Create `auth/mod.rs`
- Update `lib.rs`: remove `auth_service`, `session_service`, `test_session_service`; add `mod auth; pub use auth::*`
- Fix all `crate::auth_service::`, `crate::session_service::` paths

### 3b: `app_instance_service.rs` → `apps/`
- Create `apps/` directory
- Move `app_instance_service.rs` → `apps/app_instance_service.rs`
- Move `test_app_instance_service.rs` → `apps/test_app_instance_service.rs`
- Create `apps/mod.rs`
- Update `lib.rs`

### 3c: `ai_api_service.rs` → `ai_apis/`
- Create `ai_apis/` directory
- Move `ai_api_service.rs` → `ai_apis/ai_api_service.rs`
- Create `ai_apis/mod.rs`
- Update `lib.rs`

### Gate Check
```bash
cargo test -p services
```

---

## Phase 4: Utility Grouping + New Service (2 items)

**Scope**: Group utility services, create new thin token service.

### 4a: Utility services → `utils/`
- Create `utils/` directory
- Move: `cache_service.rs`, `network_service.rs`, `keyring_service.rs`, `concurrency_service.rs`, `queue_service.rs`, `test_queue_service.rs`
- Create `utils/mod.rs`
- Update `lib.rs`: remove individual modules, add `mod utils; pub use utils::*`

### 4b: Create `tokens/` thin service (NEW CODE)
- Create `tokens/` directory
- Create `tokens/token_service.rs` — `TokenService` trait with `#[mockall::automock]` wrapping `db::TokenRepository` methods (create, list, get, update)
- Create `tokens/test_token_service.rs`
- Create `tokens/mod.rs`
- Add to `lib.rs` and `app_service.rs` (DI registry)

### Gate Check
```bash
cargo test -p services
```

---

## Phase 5: Final Cleanup + Full Verification

**Scope**: lib.rs cleanup, downstream verification, documentation.

### 5a: Final lib.rs cleanup
- Ensure consistent module ordering and grouping
- Verify all `mod.rs` files contain only declarations + re-exports
- Verify no `_service/` directories remain

### 5b: Downstream verification
```bash
cargo check -p auth_middleware && cargo check -p routes_app
cargo check -p server_app && cargo check -p lib_bodhiserver
```
- Fix any downstream qualified import paths (`services::old_module::Type`)

### 5c: Full regression
```bash
make test.backend
```

### 5d: Update documentation
- Update `crates/services/CLAUDE.md` and `PACKAGE.md`
- Update root `CLAUDE.md` if needed

### Gate Check
```bash
make test.backend
```

---

## Critical Files Reference

| File | Role |
|------|------|
| `crates/services/src/lib.rs` | Module declarations + re-exports — updated every phase |
| `crates/services/src/app_service.rs` | DI registry — validates entire re-export chain compiles |
| `crates/services/src/auth_service/service.rs` | Struct + error extraction in Phase 3 |
| `crates/services/src/access_request_service/service.rs` | Has qualified `crate::auth_service::` paths to fix |
| `crates/services/src/tool_service/execution.rs` | Has `crate::exa_service::` cross-module ref to fix |

## Key Risks

1. **Phase 3 auth merge** — highest complexity: two error files (resolved via `error.rs` + `session_error.rs`), struct extraction from 800+ line file, test file moving from lib.rs scope into auth/ scope
2. **Phase 4b tokens** — new code, needs to integrate with `AppService` DI registry
3. **Phase 5 downstream** — any crate using qualified `services::old_module::Type` paths will break (most use flat `services::Type` imports, so risk is low)
