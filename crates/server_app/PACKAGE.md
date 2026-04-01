# PACKAGE.md - server_app

See [CLAUDE.md](CLAUDE.md) for architectural overview and key decisions.

## Source File Index

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module declarations and public re-exports |
| `src/server.rs` | `Server`, `ServerHandle`, `build_server_handle()`, `ShutdownCallback` trait, `ServerError` |
| `src/serve.rs` | `ServeCommand`, `ServerShutdownHandle`, `ServeError`, `ShutdownInferenceCallback`, `KeepAliveSettingListener` |
| `src/shutdown.rs` | `shutdown_signal()` cross-platform signal handling |
| `src/listener_variant.rs` | `VariantChangeListener` for `BODHI_EXEC_VARIANT` changes |
| `src/error.rs` | `TaskJoinError` |
| `src/test_utils/mod.rs` | Feature-gated test utilities (currently empty) |

## Error Types

| Enum | Variants | Location |
|------|----------|----------|
| `TaskJoinError` | wraps `tokio::task::JoinError` | `src/error.rs` |
| `ServerError` | `Io(IoError)` | `src/server.rs` |
| `ServeError` | `Setting(SettingServiceError)`, `Join(TaskJoinError)`, `Server(ServerError)`, `Unknown` | `src/serve.rs` |

## Live Integration Test Infrastructure

### Test Utility Files

| File | Purpose |
|------|---------|
| `tests/utils/live_server_utils.rs` | `setup_minimal_app_service()`, `setup_test_app_service()`, `live_server` fixture, `TestServerHandle`, `start_test_live_server()`, `create_test_session_for_live_server()`, OAuth helpers |
| `tests/utils/external_token.rs` | `ExternalTokenSimulator` -- seeds cache to bypass Keycloak |
| `tests/utils/tool_call.rs` | `get_weather_tool()`, `parse_streaming_tool_calls()`, `parse_streaming_content()` |
| `tests/utils/test_mcp_server.rs` | `TestMcpServer` (builder, configurable tools/resources/prompts/auth), `TestMcpServerHandler` (rmcp ServerHandler), `ReceivedToolCall` (observable call log) |
| `tests/utils/mod.rs` | Re-exports |
| `tests/resources/.env.test` | OAuth2 credentials (gitignored) |
| `tests/resources/.env.test.example` | Template for required env vars |

### TestServerHandle

Returned by `live_server` rstest fixture (`tests/utils/live_server_utils.rs`):
- `temp_cache_dir: TempDir` -- BODHI_HOME in temp directory
- `host: String`, `port: u16` -- server address (127.0.0.1:51135)
- `handle: ServerShutdownHandle` -- for programmatic shutdown
- `app_service: Arc<dyn AppService>` -- for accessing services in tests

### TestLiveServer

Returned by `start_test_live_server()` for tests not needing Keycloak:
- `_temp_dir: TempDir`, `base_url: String`, `app_service: Arc<dyn AppService>`, `handle: ServerShutdownHandle`

### ExternalTokenSimulator

Bypasses Keycloak by seeding the token validation cache directly:
- `new(app_service)` -- uses default client ID
- `new_with_client_id(app_service, client_id)` -- custom client ID
- `create_token_with_role(role, azp)` -- creates bearer token with seeded cache entry
- Works because `extract_claims()` doesn't verify JWT signatures and the token service checks cache first

### Common Live Test Pattern

1. Destructure `live_server` fixture into `TestServerHandle` components
2. `get_oauth_tokens(app_service)` -- password grant to Keycloak
3. `create_authenticated_session(app_service, access_token, refresh_token)` -- store in session DB
4. `create_session_cookie(session_id)` -- build cookie for reqwest
5. HTTP requests with cookie + `Sec-Fetch-Site: same-origin` header
6. `handle.shutdown().await` before final assertions

### Commands

```bash
# Unit tests
cargo test -p server_app

# Specific live test
cargo test -p server_app --test test_live_tool_calling_non_streamed
cargo test -p server_app --test test_live_mcp
cargo test -p server_app --test test_oauth_external_token

# Build
cargo build -p server_app
```
