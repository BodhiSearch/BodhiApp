# llama_server_proc -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details and file index

## Purpose

Process management and HTTP proxy for llama.cpp server processes. Handles spawning,
health monitoring, request proxying, and cleanup of external llama.cpp server processes
for local LLM inference.

## Architecture Position

- **Depends on**: `errmeta` (AppError, IoError, impl_error_from!), `errmeta_derive` (ErrorMeta)
- **Consumed by**: `services` (via `Server` trait for LLM inference operations)
- No framework dependencies (no axum, no serde for domain types)

## Non-Obvious Rules

### Server trait dual-stop methods
`stop(self: Box<Self>)` for trait-object consumers (services hold `Box<dyn Server>`) and
`stop_unboxed(self)` for direct struct usage in tests. This works around Rust's limitation
where `self` methods cannot be called on `Box<dyn Trait>`.
See `src/server.rs:85-101`.

### Health check: 300 attempts x 1s = 5 min timeout
`wait_for_server_ready()` polls `/health` endpoint every 1 second for up to 300 attempts.
If the llama.cpp server does not respond in 5 minutes, `ServerError::TimeoutError` is returned.
See `src/server.rs:170-194`.

### Process cleanup via Drop
`LlamaServer::drop()` kills and waits on the child process to prevent orphans.
See `src/server.rs:203-215`.

### Model path sanitization
When model file does not exist, HuggingFace cache paths are sanitized to show
`$HF_HOME/...` instead of the full local path. See `src/server.rs:117-121`.

### Build-time binary resolution
Compile-time constants from `build.rs` via `env!()`:
- `BUILD_TARGET` -- platform triple (e.g., `aarch64-apple-darwin`)
- `BUILD_VARIANTS` -- comma-separated acceleration variants
- `DEFAULT_VARIANT` -- default variant (e.g., `metal`, `cpu`)
- `EXEC_NAME` -- executable name (e.g., `llama-server`)

Binary expected at: `bin/{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}`

### ServerError has its own ReqwestError wrapper
`llama_server_proc` defines its own `ReqwestError` struct in `src/error.rs` (separate from
`services::shared_objs::ReqwestError`) because it cannot depend on `services`. Uses
`impl_error_from!` to bridge `reqwest::Error` -> `ReqwestError` -> `ServerError::ClientError`.

### LlamaServerArgs always passes --embeddings
The `to_args()` method always includes `--embeddings` flag. Server args in `server_args` field
are split on whitespace before being passed as separate CLI arguments.
See `src/server.rs:51-80`.

## Testing

### Two-tier test strategy

1. **Live tests** (`tests/test_live_server_proc.rs`): Use bundled Llama-68M model from
   `tests/data/live/huggingface/`. Validate process lifecycle (start/stop). Self-contained,
   suitable for CI. Require real `llama-server` binary.

2. **Integration tests** (`tests/test_server_proc.rs`): Use Qwen3-1.7B from system HF cache
   (`~/.cache/huggingface/hub/`). Validate full chat completion flows (streaming + non-streaming).
   Require large model download (~1.7GB).

Both tiers resolve the binary using `BUILD_TARGET`/`DEFAULT_VARIANT`/`EXEC_NAME` constants.

### test-utils feature
Enables `MockServer` via mockall, `llama2_7b` fixture, and `mock_response` helper.
See `src/test_utils/mod.rs`.
