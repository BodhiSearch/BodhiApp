# server_app -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Test infrastructure details, file index, live test patterns

## Purpose

Standalone HTTP server orchestration: server lifecycle (start/shutdown), listener registration, and service bootstrap. Near-leaf crate consumed by `lib_bodhiserver` and `bodhi/src-tauri`.

## Architecture Position

**Upstream**: `services`, `server_core`, `routes_app`, `llama_server_proc`
**Downstream**: None (consumed by `lib_bodhiserver`, `bodhi/src-tauri`)

## Key Components

### Server Handle (`src/server.rs`)
- `Server` -- TCP listener binding, ready notification, graceful shutdown via `axum::serve().with_graceful_shutdown()`
- `ServerHandle` -- bundles `Server` + `shutdown` sender + `ready_rx` receiver
- `build_server_handle(host, port)` -- factory function
- `ShutdownCallback` trait -- custom cleanup hook called during shutdown

### ServeCommand (`src/serve.rs`)
- `ServeCommand::ByParams { host, port }` -- only variant
- `aexecute()` -- starts server and blocks until Ctrl+C/SIGTERM
- `get_server_handle()` -- starts server and returns `ServerShutdownHandle` for programmatic control
- `ServerShutdownHandle` -- holds `JoinHandle` + shutdown `Sender`, provides `shutdown()` and `shutdown_on_ctrlc()`
- `ShutdownInferenceCallback` -- stops inference service on shutdown
- `KeepAliveSettingListener` -- forwards `BODHI_KEEP_ALIVE_SECS` changes to `InferenceService`

### VariantChangeListener (`src/listener_variant.rs`)
- Listens for `BODHI_EXEC_VARIANT` setting changes, calls `InferenceService::set_variant()`
- Skips if key doesn't match or value unchanged

### Error Types (`src/error.rs`, `src/serve.rs`)
- `TaskJoinError` -- wraps `tokio::task::JoinError`
- `ServerError::Io` -- wraps `IoError` for TCP binding failures
- `ServeError` -- `Setting | Join | Server | Unknown`

### Signal Handling (`src/shutdown.rs`)
- `shutdown_signal()` -- cross-platform: Ctrl+C + SIGTERM (Unix) or Ctrl+Break (Windows)

## Bootstrap Sequence (in `get_server_handle`)

1. Build `ServerHandle` with lifecycle channels
2. Register `VariantChangeListener` and `KeepAliveSettingListener` via `setting_service.add_listener()`
3. Create optional static router from `include_dir::Dir`
4. Build routes via `routes_app::build_routes()`
5. Spawn server task with `ShutdownInferenceCallback`
6. Await ready signal, report server URL

## IMPORTANT: No `listener_keep_alive.rs`

The separate `ServerKeepAlive` / `listener_keep_alive.rs` file was removed. Keep-alive logic is now:
- `KeepAliveSettingListener` in `src/serve.rs` -- delegates to `InferenceService::set_keep_alive()`
- The `InferenceService` trait (in `server_core`) owns timer management internally

## Live Integration Tests

### Design Philosophy
Full-stack tests: real HTTP server on TCP, real services, real OAuth2 via Keycloak. No mocks. Serial execution (`#[serial_test::serial(live)]`) due to shared port 51135.

### Why Inline AppService (No lib_bodhiserver dependency)
`setup_minimal_app_service` in `tests/utils/live_server_utils.rs` manually constructs `DefaultAppService` because `lib_bodhiserver` depends on `server_app` (circular). Uses `OfflineHubService` wrapping real `HfHubService` to prevent network downloads.

### OAuth2 Flow for Tests
1. Loads credentials from `tests/resources/.env.test`
2. Pre-configured resource client with Direct Access Grants in Keycloak
3. Password grant to get access/refresh tokens
4. Injects tokens into session DB, creates session cookie

### OAuth Tests Without Keycloak (`tests/utils/external_token.rs`)
- `ExternalTokenSimulator` seeds `MokaCacheService` directly with `CachedExchangeResult`
- `setup_test_app_service()` uses fake auth URL -- no Keycloak dependency
- `start_test_live_server()` -- live TCP server for tests not needing real OAuth
- `create_test_session_for_live_server(app_service, roles)` -- mints JWT with specified roles

### Test Files
| File | Purpose |
|------|---------|
| `tests/test_live_tool_calling_non_streamed.rs` | Tool calling (single + multi-turn) |
| `tests/test_live_tool_calling_streamed.rs` | Streaming tool calls |
| `tests/test_live_mcp.rs` | MCP integration |
| `tests/test_live_multi_tenant.rs` | Multi-tenant server lifecycle |
| `tests/test_oauth_external_token.rs` | OAuth via ExternalTokenSimulator |

### Environment Requirements
- Model: `ggml-org/Qwen3-1.7B-GGUF` in `~/.cache/huggingface/hub/`
- llama.cpp binary at `crates/llama_server_proc/bin/`
- `tests/resources/.env.test` with Keycloak credentials

### server_app vs routes_app Testing Boundary
- **server_app**: Multi-turn workflows, server lifecycle, real HTTP/TCP, OAuth code flow
- **routes_app**: Single-turn endpoint tests via `tower::oneshot()`, no TCP listener
