# PACKAGE.md - server_core/test_utils

*For overview, see [CLAUDE.md](CLAUDE.md)*

## File Index

### mod.rs
Module declarations and re-exports for `http`, `server`, `state` modules.

### http.rs
Two trait extensions for HTTP testing:

**ResponseTestExt** (impl for `axum::Response`):
- `json<T: DeserializeOwned>()` — collect body bytes, parse as JSON via `serde_json::from_reader`
- `json_obj<T: Deserialize>()` — similar but with `from_str` (different lifetime handling)
- `text()` — collect body as `String` via `String::from_utf8_lossy`
- `sse<T: DeserializeOwned>()` — split by lines, parse each `data:` prefixed line as JSON, skip empty lines
- `direct_sse()` — split by lines, return each as String (no parsing)

**RequestTestExt** (impl for `http::request::Builder`):
- `json<T: Serialize>(value)` — sets Content-Type to application/json, serializes body
- `json_str(value: &str)` — sets Content-Type, uses raw string as body

### state.rs
- `router_state_stub` fixture: `#[fixture] #[awt] async fn` returning `Arc<dyn AppService>` from `AppServiceStub`
- `ServerFactoryStub`: implements `ServerFactory` trait, stores `Mutex<Vec<Box<dyn Server>>>`, `create_server()` pops from vec

### server.rs
- `mock_server` fixture: creates `MockServer` with `expect_start().times(1).return_once(|| async { Ok(()) }.boxed())`
- `bin_path` fixture: creates temp dir with `{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}` file structure, returns `TempDir`

## Integration Points

- `state.rs` imports `services::test_utils::{app_service_stub, AppServiceStub}`
- `server.rs` imports `llama_server_proc::MockServer` and `services::test_utils::temp_dir`
- Used by server_core unit tests in `src/test_shared_rw.rs` and live tests in `tests/test_live_shared_rw.rs`
- Downstream crates (`routes_app`, `server_app`) import via `server_core::test_utils::*`

## Commands

```bash
cargo test -p server_core --features test-utils
```
