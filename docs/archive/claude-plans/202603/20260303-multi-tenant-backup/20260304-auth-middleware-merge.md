# Plan: Merge auth_middleware crate into routes_app

## Context

The `auth_middleware` crate was upstream of `routes_app` in the dependency chain, which forced it to define its own error types, utility functions, and test infrastructure independently. Now that we're merging it into `routes_app`, we can eliminate this duplication, share infrastructure from `shared/`, and organize middleware into a clean hierarchical module structure. `server_app` and `lib_bodhiserver` list `auth_middleware` as a dependency but don't use it in source code (purely transitive).

## Target Module Structure

```
routes_app/src/middleware/
  mod.rs                              -- module declarations + selective pub exports
  error.rs                            -- MiddlewareError (shared return type for all middleware)
  auth/
    mod.rs
    auth_middleware.rs                 -- auth_middleware(), optional_auth_middleware()
    error.rs                          -- AuthError (14 variants)
    test_auth_middleware.rs            -- ~970 lines of tests
  apis/
    mod.rs
    api_middleware.rs                  -- api_auth_middleware()
    error.rs                          -- ApiAuthError (5 variants)
    test_api_middleware.rs             -- extracted from inline tests
  access_requests/
    mod.rs
    access_request_middleware.rs       -- access_request_auth_middleware()
    error.rs                          -- AccessRequestAuthError (10 variants)
    access_request_validator.rs        -- AccessRequestValidator trait + Toolset/Mcp impls
    test_access_request_middleware.rs  -- ~571 lines of tests
  redirects/
    mod.rs
    canonical_url_middleware.rs        -- canonical_url_middleware()
    test_canonical_url_middleware.rs   -- extracted from inline tests
  token_service/
    mod.rs
    token_service.rs                  -- DefaultTokenService, CachedExchangeResult
    test_token_service.rs             -- ~1585 lines of tests
```

## Key Design Decisions

| Decision | Choice |
|----------|--------|
| MiddlewareError vs ApiError | **Keep separate** -- MiddlewareError stays as distinct type in `middleware/error.rs` |
| AuthContext import | **From services directly** -- remove re-export shim, all files import `services::AuthContext` |
| Utility functions | **Move to shared/** -- `generate_random_string`, `app_status_or_default`, `ApiErrorResponse` go to `shared/utils.rs` |
| Token service location | **middleware/token_service/** -- co-located with auth middleware (its only consumer) |
| Token service errors | **Uses AuthError via import** from `crate::middleware::auth` |
| Canonical URL middleware | **middleware/redirects/** -- all middleware together |
| Error enums | **3 separate enums**, each in its own `error.rs` within submodule |
| Validators | **Stay in middleware/access_requests/** |
| Re-exports from lib.rs | **No blanket re-export** -- downstream uses `routes_app::middleware::*` explicitly |
| Test utilities | **routes_app/test_utils/middleware/** -- `RequestAuthContextExt`, `AuthServerTestClient` |
| Live integration tests | **routes_app/tests/** directory |
| Inline tests | **Extract to sibling test_*.rs files** (api_auth_middleware.rs and canonical_url_middleware.rs have inline tests) |
| Documentation | **Merge** auth_middleware CLAUDE.md/PACKAGE.md content into routes_app CLAUDE.md/PACKAGE.md |

## Execution Strategy

Two phases, each executed by a single sub-agent. Gate-checks before committing. Sub-agent returns concise summary that feeds into next phase.

---

### Phase 1: Merge auth_middleware into routes_app (sub-agent)

**Commit**: "refactor: merge auth_middleware into routes_app/middleware"

The sub-agent executes steps 1-8 sequentially, running `cargo check -p routes_app` after each step as incremental gate-check.

**Step 1: Scaffold middleware/ module structure + Cargo.toml**
- Create all directories: `middleware/`, `middleware/{auth,apis,access_requests,redirects,token_service}/`
- Create all `mod.rs` files with module declarations (stub implementations)
- Move `middleware_error.rs` -> `middleware/error.rs`, adapt imports
- Add `pub mod middleware;` to `routes_app/src/lib.rs`
- Update `routes_app/Cargo.toml`: add `jsonwebtoken`, `sha2`, `constant_time_eq`, `rand`, `chrono`, `time`; test-utils: `base64`, `derive_builder`

**Step 2: Move auth/ submodule**
- `auth_middleware/auth_middleware.rs` -> `middleware/auth/auth_middleware.rs`
- Extract `AuthError` enum to `middleware/auth/error.rs`
- Move `test_auth_middleware.rs` -> `middleware/auth/test_auth_middleware.rs`
- Replace `use crate::{AuthContext, MiddlewareError}` with `use services::AuthContext` + `use crate::middleware::MiddlewareError`

**Step 3: Move token_service/ submodule**
- `token_service.rs` -> `middleware/token_service/token_service.rs`
- `test_token_service.rs` -> `middleware/token_service/test_token_service.rs`
- Import `AuthError` from `crate::middleware::auth`

**Step 4: Move apis/ submodule**
- `api_auth_middleware.rs` -> `middleware/apis/api_middleware.rs`
- Extract `ApiAuthError` to `middleware/apis/error.rs`
- Extract inline tests to `middleware/apis/test_api_middleware.rs`

**Step 5: Move access_requests/ submodule**
- Extract `AccessRequestAuthError` to `middleware/access_requests/error.rs`
- Extract `AccessRequestValidator` trait + impls to `middleware/access_requests/access_request_validator.rs`
- Move middleware function to `middleware/access_requests/access_request_middleware.rs`
- Move `test_access_request_middleware.rs`

**Step 6: Move redirects/ submodule**
- `canonical_url_middleware.rs` -> `middleware/redirects/canonical_url_middleware.rs`
- Extract inline tests to `middleware/redirects/test_canonical_url_middleware.rs`

**Step 7: Move utilities + test infrastructure + rewire imports**
- Move `generate_random_string()`, `app_status_or_default()`, `ApiErrorResponse` to `shared/utils.rs`
- Move test utils to `test_utils/middleware/{auth_context.rs, auth_server_test_client.rs}`
- Move `auth_middleware/tests/test_live_auth_middleware.rs` -> `routes_app/tests/`
- Rewire all `routes_app` imports:
  - `routes.rs`: `use auth_middleware::*` -> `use crate::middleware::*`
  - `shared/auth_scope_extractor.rs`: `use auth_middleware::AuthContext` -> `use services::AuthContext`
  - `mcps/routes_mcps_auth.rs`, `setup/routes_setup.rs`: update util imports
  - ~62 test files: `auth_middleware::test_utils::*` -> `crate::test_utils::middleware::*`

**Step 8: Full validation + commit**
- `cargo check -p routes_app && cargo test -p routes_app` — fix all failures, iterate until green
- Commit

---

### Phase 2: Remove auth_middleware + update docs (sub-agent)

**Receives**: Phase 1 summary (what was moved, any gotchas)

**Step 1: Remove crate + clean downstream**
- Delete `crates/auth_middleware/` directory
- Workspace `Cargo.toml`: remove from members + dependencies
- `routes_app/Cargo.toml`: remove `auth_middleware` from deps and dev-deps
- `server_app/Cargo.toml`: remove `auth_middleware` dep
- `lib_bodhiserver/Cargo.toml`: remove `auth_middleware` dep
- Grep for remaining references, fix any stragglers
- Gate-check: `cargo check && cargo test -p routes_app && cargo test -p server_app && cargo test -p lib_bodhiserver`

**Step 2: Merge documentation + final commit**
- Merge auth_middleware `CLAUDE.md` middleware sections into `routes_app/CLAUDE.md`
- Merge auth_middleware `PACKAGE.md` into `routes_app/PACKAGE.md`
- Update `crates/CLAUDE.md`: remove auth_middleware row, update routes_app description
- Update root `CLAUDE.md`: update crate dependency chain diagram
- Remove documentation symlinks referencing auth_middleware
- Gate-check: `make test.backend` — full backend green
- Commit: "refactor: remove auth_middleware crate, update docs"

---

## Verification Checklist

1. `cargo test -p routes_app` — all green after Phase 1
2. `cargo check` — full workspace compiles after Phase 2 step 1
3. `cargo test -p server_app && cargo test -p lib_bodhiserver` — downstream green
4. `make test.backend` — full backend suite passes
5. `grep -r "auth_middleware" --include="*.rs" --include="*.toml" crates/` — zero matches (except docs comments)
