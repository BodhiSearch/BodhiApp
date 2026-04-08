# server_core — CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` — Implementation details and file index
- `src/test_utils/CLAUDE.md` — Test utility infrastructure
- `src/test_utils/PACKAGE.md` — Test utility implementation details

## Purpose

HTTP infrastructure for LLM inference. Provides SSE streaming, LLM server process management (SharedContext), server argument merging, and inference service implementations (standalone + multitenant).

## Architecture Position

```
services / llama_server_proc
              ↓
         [server_core]  ← YOU ARE HERE
              ↓
    routes_app / server_app / lib_bodhiserver
```

Depends on `services` (domain types, business logic) and `llama_server_proc` (LLM process management).

## Module Structure

- `src/shared_rw.rs` — `SharedContext` trait, `DefaultSharedContext`, `ServerFactory` trait, `ServerState`, `ServerStateListener`
- `src/direct_sse.rs` — `DirectEvent`, `DirectSse` — application-generated SSE with keep-alive
- `src/fwd_sse.rs` — `RawSSE`, `fwd_sse()` — proxy forwarded SSE from LLM server
- `src/server_args_merge.rs` — LLM server argument merging (setting → variant → alias)
- `src/error.rs` — `ContextError` enum aggregating service errors
- `src/standalone_inference.rs` — `StandaloneInferenceService` — local LLM inference with keep-alive timer
- `src/multitenant_inference.rs` — `MultitenantInferenceService` — remote-only inference (local unsupported)

## Critical Design Details

### SharedContext (`src/shared_rw.rs`)
Manages LLM server process lifecycle. Key trait methods:
- `reload(server_args)` — stop current + start new LLM server
- `stop()` — stop current server
- `forward_request(endpoint, request, alias)` — forward to loaded LLM server
- `is_loaded()` — check if server is running
- `set_exec_variant(variant)` — change execution variant (CPU/CUDA/etc)
- `add_state_listener()` / `notify_state_listeners()` — observer pattern

`DefaultSharedContext` uses `RwLock<Option<Box<dyn Server>>>` for the LLM process. `ServerFactory` trait abstracts server creation for testability.

### Inference Services
Two `InferenceService` implementations (trait defined in `services::inference`):
- **StandaloneInferenceService**: wraps `SharedContext` for local models. Has keep-alive timer that auto-stops server after inactivity. `forward_local()` loads model via SharedContext, `forward_remote()` proxies to API.
- **MultitenantInferenceService**: remote-only. `forward_local()` returns `InferenceError::Unsupported`. No SharedContext needed.

`proxy_to_remote` uses `http::Method` for type-safe HTTP method dispatch (was `&str`). Body is only sent for `Method::POST` requests; GET/DELETE receive `None`.

### SSE Streaming
Two distinct implementations for different use cases:
- **DirectSse** (`direct_sse.rs`): Application-generated events. `DirectEvent` builder with `data()` and `finalize()`. BytesMut-optimized. Optional keep-alive.
- **RawSSE / fwd_sse** (`fwd_sse.rs`): Proxies raw string streams from LLM server. Uses `tokio::sync::mpsc::Receiver<String>`.

### Server Arguments Merging (`src/server_args_merge.rs`)
Three-tier precedence: Setting → Variant → Alias. Handles complex patterns: negative numbers (`--temp -0.5`), key-value pairs, scaled values, JSON arrays.

### ContextError (`src/error.rs`)
Aggregates errors from multiple services with transparent delegation:
- `HubService(HubServiceError)`, `Builder(BuilderError)`, `Server(ServerError)`, `SerdeJson(SerdeJsonError)`, `DataServiceError(DataServiceError)`, `ObjValidationError(ObjValidationError)`
- Non-transparent: `Unreachable(String)`, `ExecNotExists(String)` — both InternalServer

## Testing

### Two-Tier Strategy
1. **Unit tests** (`src/test_shared_rw.rs`): Mock-based SharedContext tests using `MockServer` and `ServerFactoryStub`. Use `#[serial(BodhiServerContext)]`.
2. **Live integration tests** (`tests/test_live_shared_rw.rs`): Real llama.cpp binary + GGUF models. No HTTP server. Use `#[serial(live)]`.

### Live Test Prerequisites
- Pre-built llama.cpp binary at `crates/llama_server_proc/bin/{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}`
- Test model files in `tests/data/live/huggingface/hub/` (Llama-68M, HuggingFace cache layout with symlinks)

### Test Utilities (`src/test_utils/`)
- `state.rs`: `router_state_stub` fixture (returns `Arc<dyn AppService>`), `ServerFactoryStub`
- `http.rs`: `ResponseTestExt` (json, text, sse, direct_sse parsing), `RequestTestExt` (json body builder)
- `server.rs`: `mock_server` fixture, `bin_path` fixture
