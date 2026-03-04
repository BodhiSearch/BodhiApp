# server_core/test_utils — CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` — Implementation details

## Purpose

Test infrastructure for server_core's HTTP and LLM server management. Three modules: HTTP response/request helpers, RouterState fixture, and mock server fixtures.

## Key Utilities

### ResponseTestExt (`http.rs`)
Trait extension on `axum::Response` for parsing test responses:
- `json<T>()` — deserialize body as JSON
- `json_obj<T>()` — deserialize with different lifetime bounds
- `text()` — extract body as String
- `sse<T>()` — parse SSE stream, deserialize each `data:` line as JSON
- `direct_sse()` — parse SSE stream as raw strings (line by line)

### RequestTestExt (`http.rs`)
Trait extension on `http::request::Builder`:
- `json<T>(value)` — serialize body + set Content-Type header
- `json_str(value)` — raw string body + set Content-Type header

### router_state_stub (`state.rs`)
Fixture returning `Arc<dyn AppService>` from `AppServiceStub`. Uses `services::test_utils::app_service_stub`.

### ServerFactoryStub (`state.rs`)
`ServerFactory` implementation that pops mock servers from a `Mutex<Vec<Box<dyn Server>>>`. Methods: `new(instance)`, `new_with_instances(vec)`.

### mock_server (`server.rs`)
Fixture creating `MockServer` with `start()` expectation pre-configured.

### bin_path (`server.rs`)
Fixture creating temp directory with llama server binary path structure: `{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}`.

## Cross-Crate Dependencies
- Uses `services::test_utils::AppServiceStub` and `app_service_stub` fixture
- Uses `llama_server_proc::MockServer`
