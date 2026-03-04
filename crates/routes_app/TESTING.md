# routes_app -- TESTING.md

## Test File Organization

All modules use `test_*.rs` sibling files (not `tests/` subdirectories):
- Pattern A: handler file declares `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`
- Pattern B: auth tier tests declared from `mod.rs` as `test_<module>_auth.rs`
- Split large test files by concern: `test_<handler>_crud.rs`, `test_<handler>_auth.rs`
- Reference implementation: `src/mcps/` module

## Route-Level Unit Tests vs Server Integration Tests

Tests in `routes_app` are **route-level unit tests** (single request/response via `router.oneshot()`):
- Use `build_test_router()` for fully-composed router with real services
- Authenticate with `create_authenticated_session()` (session cookie injection)
- Single-turn only: one request, one response

Multi-turn workflows (OAuth flow, tool calling sequences, session persistence across requests) belong in `server_app` integration tests.

## Service Mocking Strategy

**Default: prefer `build_test_router()` with real services.**

### What `build_test_router()` provides (real implementations)

- SQLite DB (in-memory via tempfile)
- SessionService (SQL-backed session store)
- DataService (file-based alias storage)
- HubService (HuggingFace cache scanning)

Stubbed by default (external boundaries):
- `MockInferenceService` (no LLM binary needed)
- `StubNetworkService` (fixed IP)
- `StubQueue` (no-op task queue)

Setup pattern: see `src/test_utils/router.rs:30`

### When to mock a service

Use `MockMcpService`, `MockToolService`, etc. only when:
1. Service makes external HTTP calls (e.g., `McpService::discover_oauth_metadata()`)
2. Need specific error path impractical to trigger via real DB
3. Testing handler mapping in complete isolation

Build minimal router with just routes under test, inject auth via `RequestAuthContextExt::with_auth_context()`.

### OAuth flow tests with session persistence

When two API calls must share session state (e.g., OAuth login -> token exchange), build custom router with real `DefaultSessionService` but mocked external MCP service. Return `session_service` alongside router for session inspection.

### Decision table

| Scenario | Pattern |
|----------|---------|
| CRUD persisted in DB | `build_test_router()` |
| Handler with no external calls | `build_test_router()` (preferred) |
| Service calls external HTTP | Mock the service |
| Specific service error path | Mock the service |
| LLM inference/streaming | `MockInferenceService` via `AppServiceStubBuilder` |
| OAuth two-call flow | Custom router + real `DefaultSessionService` |
| Full LLM stack (real llama.cpp) | `build_live_test_router()` |

### Auth context injection in tests

Use `RequestAuthContextExt::with_auth_context()` from `auth_middleware` test-utils:

Factory methods: `AuthContext::test_session()`, `test_session_with_token()`, `test_session_no_role()`, `test_api_token()`, `test_external_app()`, `test_external_app_no_role()`.
