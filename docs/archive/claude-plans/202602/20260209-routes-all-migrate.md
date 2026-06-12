# Plan: Merge routes_all into routes_app + Migrate Tests to Full Router

## Context

The `routes_all` crate is a thin composition layer (~4 files) that imports ~100 handlers from `routes_app` and composes them into a full Router with middleware. This separation forces route tests to use ad-hoc routers that bypass the real auth middleware chain. By merging `routes_all` into `routes_app`, all tests can use the fully-composed router with real authentication, making tests more realistic and eliminating the extra crate.

**Current state**: There are stashed changes from a prior attempt to add test infrastructure to `routes_all`. We will stash/discard those and start fresh with this new approach.

## Decisions Made

| Decision | Choice |
|----------|--------|
| Router API | Single `build_routes()` in routes_app |
| Middleware | Everything moves into routes_app |
| Dependencies | Accept added deps (utoipa-swagger-ui, include_dir, hyper-util) |
| Test strategy | Migrate ALL existing tests to use `build_routes()` |
| Test location | Co-located with domain modules (routes_oai/tests/, etc.) |
| Test utils | Extend routes_app/src/test_utils/ |
| Proxy location | Move into routes_app |
| Auth in tests | All tests use real auth (sessions or API tokens) |
| Edge case tests | Keep as separate unit tests using ad-hoc routers |
| Module placement | New `routes_app/src/routes.rs` for build_routes() |
| Consumers | Update server_app, lib_bodhiserver in this plan |
| Commit strategy | Smaller commits, module-by-module |
| Stashed tests | View for reference, start fresh |
| Migration order | Start with routes_oai (highest value) |

## Key Files

### routes_all (source, to be absorbed)
- `crates/routes_all/src/routes.rs` — `build_routes()`, `apply_ui_router()`, `make_ui_endpoint!`
- `crates/routes_all/src/routes_proxy.rs` — `proxy_router()`, `proxy_handler()`
- `crates/routes_all/src/lib.rs` — re-exports
- `crates/routes_all/src/test_utils/mod.rs` — empty (stashed changes have content)
- `crates/routes_all/Cargo.toml` — deps to merge

### routes_app (destination)
- `crates/routes_app/src/lib.rs` — module structure, re-exports
- `crates/routes_app/Cargo.toml` — needs new deps
- `crates/routes_app/src/test_utils/mod.rs` — will gain router test helpers

### Consumers (to update)
- `crates/server_app/Cargo.toml` — depends on routes_all
- `crates/server_app/src/serve.rs` — `use routes_all::build_routes;` (line 8)
- `crates/lib_bodhiserver/Cargo.toml` — depends on routes_all (unused import)
- `Cargo.toml` (workspace) — members list includes routes_all

## Execution Plan

### Commit 1: Merge routes_all code into routes_app

**Goal**: Move `build_routes()`, `proxy_router()`, and `make_ui_endpoint!` macro into routes_app. No test changes yet.

1. **Stash/discard** any uncommitted changes from the prior routes_all test work
2. **Add dependencies** to `crates/routes_app/Cargo.toml`:
   - `include_dir`, `tower-http` (with cors, trace features), `hyper-util`, `utoipa-swagger-ui`, `tracing` (if not already present)
   - Dev: `tower-sessions`, `time`, `maplit`, `pretty_assertions`, `anyhow_trace`
3. **Create `crates/routes_app/src/routes.rs`**: Copy `build_routes()`, `apply_ui_router()` from `routes_all/src/routes.rs`
   - Update imports to use local modules instead of `routes_app::*` imports
   - Move `make_ui_endpoint!` macro here
4. **Create `crates/routes_app/src/routes_proxy.rs`**: Copy from `routes_all/src/routes_proxy.rs`
5. **Update `crates/routes_app/src/lib.rs`**:
   - Add `mod routes;` and `mod routes_proxy;`
   - Add `pub use routes::*;` and `pub use routes_proxy::*;`
6. **Move inline tests** from `routes_all/src/routes.rs` (the `apply_ui_router` tests) into routes_app's routes.rs
7. **Verify**: `cargo check -p routes_app`

### Commit 2: Update consumers to import from routes_app

1. **Update `crates/server_app/Cargo.toml`**:
   - Remove `routes_all` from both `[dependencies]` and `[dev-dependencies]`
   - Ensure `routes_app` is in dependencies (it should already be via server_core or directly)
2. **Update `crates/server_app/src/serve.rs`**:
   - Change `use routes_all::build_routes;` → `use routes_app::build_routes;`
3. **Update `crates/lib_bodhiserver/Cargo.toml`**:
   - Remove `routes_all` dependency
4. **Verify**: `cargo check -p server_app -p lib_bodhiserver`

### Commit 3: Remove routes_all crate from workspace

1. **Remove** `crates/routes_all/` directory entirely
2. **Update workspace `Cargo.toml`**: Remove `"crates/routes_all"` from members list
3. **Remove** any `routes_all` entries from workspace `[dependencies]` section
4. **Verify**: `cargo check` (full workspace), then `cargo test -p routes_app`

### Commit 4: Add router test infrastructure to routes_app

**Goal**: Create the test helpers that enable full-router testing with real auth.

Add to `crates/routes_app/src/test_utils/mod.rs` (or new `router.rs` sub-module):

- **`build_test_router() -> (Router, Arc<SqliteSessionService>, TempDir)`**
  - Creates `AppServiceStubBuilder` with session service, secret service, db service, hub service, data service
  - Calls `build_routes(ctx, app_service, None)`
  - Returns the router + session service handle for injecting sessions

- **`create_authenticated_session(session_service, roles) -> Result<String>`**
  - Creates JWT with specified roles using `build_token()`
  - Saves session Record to session store
  - Returns session ID string

- **`session_request(method, uri, session_id) -> Request<Body>`**
  - Builds a request with `Cookie: bodhiapp_session_id={id}` + `Sec-Fetch-Site: same-origin`

- **`unauth_request(method, uri) -> Request<Body>`**
  - Builds a request with no auth headers

**Verify**: `cargo check -p routes_app`

### Commit 5: Migrate routes_oai tests to full router

**Goal**: Migrate `routes_oai/tests/chat_test.rs` and `models_test.rs` to use `build_test_router()` with real auth.

For each test:
1. Replace ad-hoc `Router::new().route(...)` with `build_test_router()`
2. Replace `X-BodhiApp-*` header injection with `create_authenticated_session()` + `session_request()`
3. Keep handler-level edge case tests (impossible states) as separate unit tests using ad-hoc routers

**Verify**: `cargo test -p routes_app -- routes_oai`

### Commit 6: Migrate routes_ollama tests

Same pattern as Commit 5 for `routes_ollama/tests/handlers_test.rs`.

**Verify**: `cargo test -p routes_app -- routes_ollama`

### Commit 7: Migrate routes_models tests

Migrate `routes_models/tests/aliases_test.rs`, `metadata_test.rs`, `pull_test.rs`.

**Verify**: `cargo test -p routes_app -- routes_models`

### Commit 8: Migrate routes_api_models tests

Migrate `routes_api_models/tests/api_models_test.rs`.

**Verify**: `cargo test -p routes_app -- routes_api_models`

### Commit 9: Migrate routes_users tests

Migrate `routes_users/tests/management_test.rs`, `access_request_test.rs`, `user_info_test.rs`.

**Verify**: `cargo test -p routes_app -- routes_users`

### Commit 10: Migrate routes_auth tests

Migrate `routes_auth/tests/login_test.rs`, `request_access_test.rs`.

**Verify**: `cargo test -p routes_app -- routes_auth`

### Commit 11: Migrate routes_toolsets tests

Migrate `routes_toolsets/tests/toolsets_test.rs`.

**Verify**: `cargo test -p routes_app -- routes_toolsets`

### Commit 12: Migrate standalone route tests

Migrate `routes_api_token_test.rs`, `routes_settings_test.rs`, `routes_setup_test.rs`.

**Verify**: `cargo test -p routes_app`

### Commit 13: Add auth tier smoke tests + public endpoint tests

Add new comprehensive tests (inspired by the stashed routes_all tests but fresh):
- **Auth tier tests**: For every protected endpoint, test unauthenticated→401, wrong role→403, correct role→passes auth
- **Public endpoint tests**: Verify ping, health, app_info, setup, logout work without auth

Place these in `crates/routes_app/src/routes.rs` as `#[cfg(test)] mod tests` or in a new `routes_app/tests/test_auth_tiers.rs`.

**Verify**: `cargo test -p routes_app`, then `make test.backend`

## Sub-Agent Execution Strategy

Each commit is implemented by a dedicated sub-agent. The orchestrating agent:
1. Provides full context from prior commits
2. Launches the sub-agent
3. Verifies compilation and tests pass
4. Commits if green
5. Passes context to next sub-agent

Commits 1-4 are sequential (each depends on prior). Commits 5-12 can potentially be parallelized but are better done sequentially to catch patterns early. Commit 13 comes last.

## Verification

After each commit:
- `cargo check -p routes_app` — compilation
- `cargo test -p routes_app` — run tests
- For commits 1-3: `cargo check` (full workspace)

After all commits:
- `make test.backend` — full backend test suite
- Verify `crates/routes_all/` no longer exists
- Verify no remaining references to `routes_all` in codebase
