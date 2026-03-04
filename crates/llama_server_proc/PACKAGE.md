# llama_server_proc -- PACKAGE.md

## Module Structure

- `src/lib.rs` -- Module declarations, re-exports, `test-utils` feature gate
- `src/server.rs` -- `Server` trait (async_trait + mockall), `LlamaServer` struct, `LlamaServerArgs` (derive_builder), `LlamaServerArgsBuilder`, process lifecycle, health check, HTTP proxy, Drop impl
- `src/error.rs` -- `ServerError` enum (7 variants), local `ReqwestError` wrapper, `impl_error_from!` bridges, `Result<T>` type alias
- `src/build_envs.rs` -- Compile-time constants: `BUILD_TARGET`, `BUILD_VARIANTS` (lazy_static), `DEFAULT_VARIANT`, `EXEC_NAME`
- `src/test_utils/mod.rs` -- `llama2_7b` fixture, `llama2_7b_str` fixture, `mock_response` helper
- `build.rs` -- Build script: platform detection, GitHub release download, ZIP extraction, file locking

## ServerError Variants

| Variant | ErrorType | Description |
|---|---|---|
| `ServerNotReady` | ServiceUnavailable (503) | Server still starting |
| `StartupError(String)` | InternalServer (500) | Failed to spawn process |
| `IoError(IoError)` | transparent | Filesystem errors |
| `ClientError(ReqwestError)` | transparent | HTTP client errors |
| `HealthCheckError(String)` | InternalServer (500) | Health check failure |
| `TimeoutError(u64)` | InternalServer (500) | Startup timeout |
| `ModelNotFound(String)` | InternalServer (500) | Model file missing |

## Server Trait Methods

- `start(&self)` -- Spawn process, monitor stdout/stderr, wait for health
- `stop(self: Box<Self>)` -- For `Box<dyn Server>` consumers
- `stop_unboxed(self)` -- For direct struct usage
- `get_server_args(&self)` -- Return clone of LlamaServerArgs
- `chat_completions(&self, body)` -- Proxy to `/v1/chat/completions`
- `embeddings(&self, body)` -- Proxy to `/v1/embeddings`
- `tokenize(&self, body)` -- Proxy to `/v1/tokenize`
- `detokenize(&self, body)` -- Proxy to `/v1/detokenize`

## Test Files

- `tests/test_live_server_proc.rs` -- Process lifecycle with bundled Llama-68M
- `tests/test_server_proc.rs` -- Full inference with Qwen3-1.7B from HF cache

## Test Data Layout

`tests/data/live/huggingface/` mirrors HF cache structure:
- `hub/models--afrideva--Llama-68M-Chat-v1-GGUF/` -- Primary live test model (Q8_0)
- `hub/models--TheBloke--TinyLlama-1.1B-Chat-v1.0-GGUF/` -- Additional test model
- Snapshots use symlinks to blobs (same as HF CLI downloads)

## Build System

Platform configurations (from `build.rs`):
- `aarch64-apple-darwin` -- variants: metal, cpu
- `aarch64-unknown-linux-gnu` -- variants: cpu
- `x86_64-unknown-linux-gnu` -- variants: cpu
- `x86_64-pc-windows-msvc` -- variants: cpu (exe extension)

CI env vars: `CI_RELEASE`, `GH_PAT`, `CI_BUILD_TARGET`, `CI_BUILD_VARIANTS`, `CI_DEFAULT_VARIANT`

Uses `fs2` file locking (`bodhi-build.lock`) to prevent concurrent build conflicts.

## Features

- `test-utils` -- Enables `MockServer` (mockall), rstest fixtures, mock_response helper
