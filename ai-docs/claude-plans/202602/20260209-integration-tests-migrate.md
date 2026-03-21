# Migrate integration-tests to server_app (and non-llama tests elsewhere)

## Context

The `crates/integration-tests/` crate contains 15 E2E tests requiring real llama.cpp inference, OAuth2, and external APIs. These tests are isolated in a standalone crate with heavy infrastructure. The goal is to move them to their natural homes:

- **server_app** — All tests that use the full `ServeCommand` + HTTP server (11 tests). This is the natural home since `server_app` owns `ServeCommand` and `ServerShutdownHandle`.
- **server_core** — SharedContext reload tests (3 tests, no HTTP server needed)
- **llama_server_proc** — Raw LlamaServer start/stop test (1 test, no HTTP server needed)
- **DROPPED** — `test_live_api_ping` (trivial, already covered by routes_app router tests)

### Why server_app (not routes_app)

Previous attempt to migrate to routes_app using Tower oneshot tests with `DefaultSharedContext` **failed** — llama.cpp server lifecycle didn't work without a full HTTP server. The integration tests work by starting a **real HTTP server** via `ServeCommand::get_server_handle()` which lives in `server_app`. Moving these tests there is a natural fit.

### Dependency Note

`server_app` does NOT depend on `lib_bodhiserver` (it's the other way: `lib_bodhiserver` → `server_app`). The integration-tests used `lib_bodhiserver::{setup_app_dirs, AppOptionsBuilder, AppServiceBuilder}` to create an `AppService`. For the server_app tests, we inline a minimal setup that directly builds `DefaultSettingService`, SQLite databases, and `DefaultAppService::new(...)` without depending on `lib_bodhiserver`.

### Model Files

No GGUF files will be checked into git. All tests will use `ggml-org/Qwen3-1.7B-GGUF` from `~/.cache/huggingface` (must be pre-downloaded). Models are referenced by `repo/filename:quant` format (e.g., `ggml-org/Qwen3-1.7B-GGUF:Q8_0`), no alias DB entries needed.

### Previous Attempt Cleanup

Phase 1 of the old plan committed `c59e11b0` added unused live test infrastructure to `routes_app`. This must be reverted first.

---

## Phase 0: Revert routes_app Changes

Revert all changes from the previous migration attempt.

**Committed changes to revert** (from `c59e11b0`):
1. Delete `crates/routes_app/src/test_utils/live_router.rs`
2. Delete `crates/routes_app/src/test_utils/tool_call.rs`
3. Remove `mod live_router; mod tool_call; pub use live_router::*; pub use tool_call::*;` from `crates/routes_app/src/test_utils/mod.rs`
4. Remove from `crates/routes_app/Cargo.toml`: optional deps `dirs`, `dotenv`; dev-deps `dirs`, `dotenv`, `serial_test`; feature entries `"dirs"`, `"dotenv"`
5. Delete `crates/routes_app/tests/.env.test.example`

**Uncommitted changes to revert**:
6. Remove `fs_extra` from `crates/routes_app/Cargo.toml` (optional deps, dev-deps, feature list)
7. Remove debug line from `crates/routes_app/src/routes_oai/chat.rs`: `tracing::info!(...)`
8. Remove `mod chat_live_test;` from `crates/routes_app/src/routes_oai/tests/mod.rs`

**Untracked files to delete**:
9. Delete `crates/routes_app/src/routes_oai/tests/chat_live_test.rs`
10. Delete `crates/routes_app/tests/.env.test`
11. Delete `crates/routes_app/tests/data/` directory

**Verify**: `cargo check -p routes_app --tests`
**Commit**: "revert(routes_app): remove failed live test infrastructure from previous migration attempt"

---

## Phase 1: Migrate single chat non-streamed test to server_app

Start with a single test. Validate incrementally: first GET `/v1/models` → 200 OK, then POST `/v1/chat/completions`.

### 1.1 Add dev-dependencies to `crates/server_app/Cargo.toml`

```toml
# Crate dev-deps needed (server_app already has: anyhow, rstest, tempfile, serde_json, rand):
auth_middleware = { workspace = true, features = ["test-utils"] }
llama_server_proc = { workspace = true, features = ["test-utils"] }

dirs = { workspace = true }
dotenv = { workspace = true }
cookie = { workspace = true }
fs_extra = { workspace = true }
maplit = { workspace = true }
serial_test = { workspace = true }
time = { workspace = true }
tower-sessions = { workspace = true }
```

Note: `reqwest` is already a regular dep of `server_app`. `services` is already a dev-dep with `test-utils`.

### 1.2 Create `.env.test` infrastructure

Copy from `crates/integration-tests/tests/resources/`:
- `crates/server_app/tests/resources/.env.test` — OAuth2 config (gitignored, must exist locally)
- `crates/server_app/tests/resources/.env.test.example` — template (committed)

### 1.3 Create test utilities

**`crates/server_app/tests/utils/mod.rs`** — re-exports

**`crates/server_app/tests/utils/live_server_utils.rs`** — inline minimal setup:

Key differences from integration-tests version:
- **No `lib_bodhiserver` imports** — instead inline the setup:
  1. Create temp dir with bodhi home structure (dirs for aliases, dbs, logs)
  2. Build `DefaultSettingService::new_with_defaults(...)` directly from `services` crate
  3. Set `BODHI_HOME`, `HF_HOME`, `BODHI_EXEC_LOOKUP_PATH`, `BODHI_ENCRYPTION_KEY` on the env wrapper
  4. Create SQLite databases at `setting_service.app_db_path()` and `session_db_path()`
  5. Build `OfflineHubService(HfHubService)`, `test_auth_service`, `DefaultSecretService`
  6. Build `SqliteDbService`, `SqliteSessionService`
  7. Build `DefaultAppService::new(...)` with all services wired together
- **No alias data files** — models referenced by `repo/filename:quant` format directly, no alias DB entries needed
- **`HF_HOME`** → real `~/.cache/huggingface` (via `dirs::home_dir()`)
- **`BODHI_EXEC_LOOKUP_PATH`** → `env!("CARGO_MANIFEST_DIR")/../llama_server_proc/bin`

Reuse from integration-tests:
- `live_server` fixture pattern (random port, `ServeCommand::get_server_handle`)
- `TestServerHandle` struct
- `get_oauth_tokens()`, `create_authenticated_session()`, `create_session_cookie()`

### 1.5 Create first test file

**`crates/server_app/tests/test_live_chat_completions_non_streamed.rs`**

Copy from `crates/integration-tests/tests/test_live_chat_completions_non_streamed.rs`, keeping the exact same test logic. The `mod utils;` reference connects to the new utils.

### 1.6 Incremental verification

```bash
# Step 1: Compile
cargo check -p server_app --tests

# Step 2: Run the test (requires OAuth2 server and pre-downloaded model)
cargo test -p server_app test_live_chat_completions_non_streamed -- --nocapture
```

**Commit**: "test(server_app): migrate chat completions non-streamed live test from integration-tests"

---

## Phase 2: Move remaining HTTP-based live tests to server_app

Copy remaining test files from integration-tests to `crates/server_app/tests/`:

| Source File | Target File |
|---|---|
| `test_live_chat_completions_streamed.rs` | same name in `crates/server_app/tests/` |
| `test_live_tool_calling_non_streamed.rs` | same name |
| `test_live_tool_calling_streamed.rs` | same name |
| `test_live_thinking_disabled.rs` | same name |
| `test_live_agentic_chat_with_exa.rs` | same name |

Also copy `crates/integration-tests/tests/utils/tool_call.rs` → `crates/server_app/tests/utils/tool_call.rs` (SSE parsing helpers for streaming/tool call tests).

Each file: copy from integration-tests, keep `mod utils;` reference. No path changes needed since the relative structure is the same.

**Verify**: `cargo check -p server_app --tests`
**Commit**: "test(server_app): migrate all remaining live tests from integration-tests"

---

## Phase 3: Move non-HTTP tests

### 3a: Move SharedContext tests to server_core

**File**: `crates/server_core/tests/test_live_shared_rw.rs` (new)

Move 3 tests from `integration-tests/tests/test_live_lib.rs`:
- `test_live_shared_rw_reload`
- `test_live_shared_rw_reload_with_model_as_symlink`
- `test_live_shared_rw_reload_with_actual_file`

**Key changes**: Instead of using Llama-68M from checked-in test data, point to `~/.cache/huggingface/hub` with Qwen3-1.7B model files. The `OfflineHubService(HfHubService)` wraps the real HF cache. For tests that pass explicit model paths (`with_model_as_symlink`, `with_actual_file`), resolve the Qwen3 model file from the HF cache directory.

Add dev-deps to `crates/server_core/Cargo.toml` if needed: `serial_test`, `anyhow`, `dirs`.

**Verify**: `cargo test -p server_core test_live_ -- --nocapture`

### 3b: Move LlamaServer test to llama_server_proc

**File**: `crates/llama_server_proc/tests/test_live_server_proc.rs` (new)

Move `test_live_llama_server_load_exec_with_server` from `test_live_lib.rs`.

**Key change**: Use Qwen3-1.7B model from `~/.cache/huggingface/hub` instead of checked-in Llama-68M.

Add dev-deps if needed: `anyhow`, `rstest`, `serial_test`, `dirs`.

**Verify**: `cargo test -p llama_server_proc test_live_ -- --nocapture`

**Commit**: "test(server_core, llama_server_proc): migrate SharedContext and LlamaServer live tests"

---

## Phase 4: Remove integration-tests crate

1. Remove `"crates/integration-tests"` from root `Cargo.toml` workspace members
2. Delete `crates/integration-tests/` entirely (including the tracked 70MB Llama-68M blob and other test data)
3. Update references in: Makefile, CI workflows (`.github/workflows/`)
4. Run `cargo build --workspace && cargo test --workspace` (non-live tests only)

**Commit**: "chore: remove integration-tests crate after migration to server_app, server_core, llama_server_proc"

---

## Phase 5: Documentation Update

Update affected docs:
- `crates/server_app/CLAUDE.md` — mention live test infrastructure
- `crates/server_core/CLAUDE.md` — mention live SharedContext tests
- `crates/llama_server_proc/CLAUDE.md` — mention live test
- Root `CLAUDE.md` — remove integration-tests references, update Architecture section

**Commit**: "docs: update documentation for integration-tests migration"

---

## Key Files

| File | Action |
|---|---|
| `crates/routes_app/` (several files) | REVERT Phase 0 changes |
| `crates/server_app/Cargo.toml` | Add live test dev-deps |
| `crates/server_app/tests/utils/mod.rs` | NEW: test utility re-exports |
| `crates/server_app/tests/utils/live_server_utils.rs` | NEW: inline minimal AppService setup |
| `crates/server_app/tests/utils/tool_call.rs` | NEW: SSE parsing helpers |
| `crates/server_app/tests/resources/.env.test.example` | NEW: env template (committed) |
| `crates/server_app/tests/test_live_chat_completions_non_streamed.rs` | NEW: Phase 1 test |
| `crates/server_app/tests/test_live_chat_completions_streamed.rs` | NEW: Phase 2 |
| `crates/server_app/tests/test_live_tool_calling_non_streamed.rs` | NEW: Phase 2 |
| `crates/server_app/tests/test_live_tool_calling_streamed.rs` | NEW: Phase 2 |
| `crates/server_app/tests/test_live_thinking_disabled.rs` | NEW: Phase 2 |
| `crates/server_app/tests/test_live_agentic_chat_with_exa.rs` | NEW: Phase 2 |
| `crates/server_core/tests/test_live_shared_rw.rs` | NEW: 3 tests |
| `crates/llama_server_proc/tests/test_live_server_proc.rs` | NEW: 1 test |
| `Cargo.toml` (root) | Remove integration-tests member |
| `crates/integration-tests/` | DELETE entirely |

## Reusable Code (from integration-tests)

| Utility | Source | Target |
|---|---|---|
| `TestServerHandle` struct | `tests/utils/live_server_utils.rs:164` | `server_app/tests/utils/live_server_utils.rs` |
| `live_server` fixture | `tests/utils/live_server_utils.rs:140` | same |
| `get_oauth_tokens()` | `tests/utils/live_server_utils.rs:172` | same |
| `create_authenticated_session()` | `tests/utils/live_server_utils.rs:217` | same |
| `create_session_cookie()` | `tests/utils/live_server_utils.rs:244` | same |
| `get_weather_tool()` | `tests/utils/tool_call.rs` | `server_app/tests/utils/tool_call.rs` |
| `parse_streaming_tool_calls()` | `tests/utils/tool_call.rs` | same |
| `parse_streaming_content()` | `tests/utils/tool_call.rs` | same |

## Reusable Code (from services/server_core)

| Utility | Location |
|---|---|
| `DefaultSettingService::new_with_defaults()` | `crates/services/src/setting_service/mod.rs` |
| `OfflineHubService`, `HfHubService` | `crates/services/` |
| `test_auth_service()` | `crates/services/src/test_utils/` |
| `DefaultSecretService` | `crates/services/` |
| `SqliteDbService`, `SqliteSessionService` | `crates/services/` |
| `DefaultAppService::new(...)` | `crates/services/` |
| `ServeCommand::get_server_handle()` | `crates/server_app/src/serve.rs:89` |

## Risks / Pause Points

- **Inline setup complexity**: Building `DefaultSettingService` directly without `setup_app_dirs` helper requires understanding the env wrapper and settings system. If this proves too complex, consider extracting a minimal `test_setup_app_dirs()` into `services/test_utils`.
- **OAuth2 environment**: Tests require real Keycloak access via `.env.test`. Tests will panic if env vars aren't set. Consider graceful skip with clear error message.
- **Qwen3 model not cached**: Tests need pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` in `~/.cache/huggingface`. Add assert with helpful message if missing.
- **70MB blob removal**: Deleting `crates/integration-tests/` removes the tracked Llama-68M blob from git history. This is intentional per the "no GGUF files in git" requirement.

## Verification

After all phases:
```bash
# 1. Full workspace build
cargo build --workspace

# 2. Non-live test suite (should all pass, no regressions)
make test.backend

# 3. Live tests (requires OAuth2 + pre-downloaded model)
cargo test -p server_app test_live_ -- --nocapture
cargo test -p server_core test_live_ -- --nocapture
cargo test -p llama_server_proc test_live_ -- --nocapture
```
