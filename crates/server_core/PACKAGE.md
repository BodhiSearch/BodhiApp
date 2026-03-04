# PACKAGE.md - server_core Crate Implementation Index

*For architecture and critical rules, see [CLAUDE.md](CLAUDE.md)*

## Module Structure

### Core Files
- `src/lib.rs` — module declarations and re-exports. Exports: `DirectEvent`, `DirectSse`, `RawSSE`, `fwd_sse`, `SharedContext`, `DefaultSharedContext`, `MockSharedContext`, `ServerFactory`, `DefaultServerFactory`, `ServerState`, `ServerStateListener`, `MockServerStateListener`, `ContextError`, `merge_server_args`, `StandaloneInferenceService`, `MultitenantInferenceService`, `LlmEndpoint`
- `src/shared_rw.rs` — `SharedContext` trait (6 async methods), `DefaultSharedContext` (hub_service + setting_service + factory + RwLock server), `ServerFactory` trait, `DefaultServerFactory`, `ServerState` enum, `ServerStateListener` trait
- `src/direct_sse.rs` — `DirectEvent` (BytesMut-based builder: `new()`, `data()`, `finalize()`), `DirectSse<S>` (wraps TryStream, optional KeepAlive)
- `src/fwd_sse.rs` — `RawSSE<S>` (wraps String Stream), `fwd_sse(Receiver<String>) -> Response`
- `src/server_args_merge.rs` — `merge_server_args()` function
- `src/error.rs` — `ContextError` enum (7 variants, 2 with `impl_error_from!`)
- `src/standalone_inference.rs` — `StandaloneInferenceService` implementing `InferenceService`. Has `SharedContext` + keep-alive timer (`RwLock<i64>` seconds, `RwLock<Option<JoinHandle>>`)
- `src/multitenant_inference.rs` — `MultitenantInferenceService` implementing `InferenceService`. Remote-only; `forward_local()` returns `Unsupported`

### Test Files
- `src/test_shared_rw.rs` — unit tests for SharedContext (mock-based)
- `src/test_utils/mod.rs` — re-exports http, server, state modules
- `src/test_utils/http.rs` — `ResponseTestExt` trait (5 methods: json, json_obj, text, sse, direct_sse), `RequestTestExt` trait (json, json_str)
- `src/test_utils/state.rs` — `router_state_stub` fixture, `ServerFactoryStub` (Mutex<Vec<Box<dyn Server>>>)
- `src/test_utils/server.rs` — `mock_server` fixture (MockServer with start expectation), `bin_path` fixture (creates temp dir with llama binary path)

### Live Integration Tests
- `tests/test_live_shared_rw.rs` — 3 test cases exercising real llama.cpp binary
- `tests/data/live/huggingface/hub/` — HuggingFace cache layout (Llama-68M): `blobs/`, `snapshots/` (symlinks), `refs/main`

## SharedContext Trait API

```
trait SharedContext: Debug + Send + Sync {
  async fn set_exec_variant(&self, variant: &str) -> Result<()>
  async fn reload(&self, server_args: Option<LlamaServerArgs>) -> Result<()>
  async fn stop(&self) -> Result<()>
  async fn is_loaded(&self) -> bool
  async fn forward_request(&self, endpoint: LlmEndpoint, request: Value, alias: Alias) -> Result<reqwest::Response>
  async fn add_state_listener(&self, listener: Arc<dyn ServerStateListener>)
  async fn notify_state_listeners(&self, state: ServerState)
}
```

Has `#[mockall::automock]` when test or test-utils feature enabled.

## ContextError Variants

| Variant | Source | ErrorType |
|---------|--------|-----------|
| `HubService(HubServiceError)` | transparent | delegated |
| `Builder(BuilderError)` | transparent | delegated |
| `Server(ServerError)` | transparent | delegated |
| `SerdeJson(SerdeJsonError)` | transparent + `impl_error_from!` | delegated |
| `DataServiceError(DataServiceError)` | transparent | delegated |
| `ObjValidationError(ObjValidationError)` | transparent + `impl_error_from!` | delegated |
| `Unreachable(String)` | direct | InternalServer |
| `ExecNotExists(String)` | direct | InternalServer |

## Live Test Details

Test cases in `tests/test_live_shared_rw.rs`:
- `test_live_shared_rw_reload` — reload with no model args (stop-only path)
- `test_live_shared_rw_reload_with_model_as_symlink` — load via symlinked snapshot path
- `test_live_shared_rw_reload_with_actual_file` — load via direct blob path

Setup uses `OfflineHubService` from `services::test_utils` wrapping `HfHubService` for local-only operations.

## Feature Flags

- `test-utils` — enables: `llama_server_proc/test-utils`, `services/test-utils`, `anyhow`, `mockall`, `rstest`, `http-body-util`, `tempfile`

## Commands

```bash
cargo test -p server_core                    # All tests (mock-based)
cargo test -p server_core test_live_shared_rw  # Live integration tests (requires binary)
cargo build -p server_core --features test-utils  # Build with test utilities
```
