# Task: Migrate integration-tests to lib_bodhiserver_napi and routes_app

## Context from Recent Work

### routes_app Test Infrastructure (February 2026)

Recent migrations consolidated `routes_all` into `routes_app` and established comprehensive test patterns:

**Test Architecture (397 tests in routes_app)**:
- **Router-level tests**: Use `build_test_router()` from `crates/routes_app/src/test_utils/router.rs`
- **Real services**: `TestDbService`, `SqliteSessionService`, `LocalDataService` - work well in-process
- **Mock services**: `MockAuthService`, `MockToolService`, `MockSharedContext` - only when external I/O unavoidable
- **Authentication**: `create_authenticated_session()` + `session_request()` for JWT + cookie-based auth
- **Test execution**: Parallel, fast (~5 seconds for 397 tests)

**Key Test Utilities**:
```rust
// crates/routes_app/src/test_utils/router.rs
pub async fn build_test_router() -> Result<(Router, Arc<dyn AppService>, TempDir)>
pub async fn create_authenticated_session(session_service: &dyn SessionService, roles: &[&str]) -> Result<String>
pub fn session_request(method: &str, path: &str, cookie: &str) -> Request<Body>
pub fn unauth_request(method: &str, path: &str) -> Request<Body>
```

**Test Pattern**:
```rust
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoint(#[case] method: &str, #[case] path: &str) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"]).await?;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
```

**Test Categories in routes_app**:
1. **Auth tier tests** (78 tests) - Verify authentication/authorization middleware for all endpoints
2. **Handler logic tests** (~200 tests) - Verify business logic, error handling, request/response formats
3. **Integration tests** (~119 tests) - Multi-handler flows, OpenAPI spec validation, SSE streaming

**Characteristics**:
- No real server startup (uses `axum_test::TestServer` or `tower::ServiceExt::oneshot`)
- No real OAuth2 (uses in-memory test sessions)
- No real LLM inference (uses `MockSharedContext` for chat/embeddings)
- Tests execute in milliseconds
- Suitable for TDD workflows

### integration-tests Crate Structure (Current State)

**Purpose**: End-to-end validation with real servers, OAuth2, and LLM inference

**Test Inventory (8 test files, 12+ tests)**:
1. `test_live_api_ping.rs` - Server connectivity
2. `test_live_chat_completions_non_streamed.rs` - Full OAuth2 → chat API → model inference
3. `test_live_chat_completions_streamed.rs` - Streaming chat with SSE parsing
4. `test_live_tool_calling_non_streamed.rs` - Non-streaming tool/function calling (multi-turn flow)
5. `test_live_tool_calling_streamed.rs` - Streaming tool calling with SSE parsing
6. `test_live_thinking_disabled.rs` (3 tests) - Reasoning content validation
7. `test_live_lib.rs` (4 tests) - Direct llama.cpp server testing, SharedContext reload
8. `test_live_agentic_chat_with_exa.rs` - Full toolset integration (Exa web search)

**Test Infrastructure (384 lines)**:
- **TestServerHandle** - Real server lifecycle: random port allocation, temp directories, graceful shutdown
- **llama2_7b_setup fixture** - Complete service registry with real SQLite, HuggingFace cache, OAuth2 client
- **live_server fixture** - Running HTTP server via `server_app::ServeCommand`
- **get_oauth_tokens()** - Real Keycloak password flow authentication
- **create_authenticated_session()** - Database-backed session creation
- **Tool calling utilities** - SSE stream parsers, tool definition helpers

**Dependencies**:
```toml
[dev-dependencies]
server_app = { features = ["test-utils"] }      # ServeCommand, ServerShutdownHandle
lib_bodhiserver = { features = ["test-utils"] } # AppServiceBuilder, AppOptionsBuilder
auth_middleware = { features = ["test-utils"] } # AuthServerTestClient, session keys
services = { features = ["test-utils"] }        # OfflineHubService, test_auth_service()
```

**Characteristics**:
- Real Axum server on random port (2000-60000)
- Real OAuth2 with Keycloak test instance (via `.env.test`)
- Real llama.cpp inference with test models (Llama-68M, Qwen3-1.7B)
- Real SQLite for sessions and tokens
- Serial execution with `#[serial_test::serial(live)]`
- 5-minute timeout per test
- Tests execute in seconds to minutes (depending on model inference)

**Key Distinction**: integration-tests validates **complete workflows** that routes_app cannot test with mocks.

### lib_bodhiserver_napi Structure

**Purpose**: Node.js bindings for BodhiApp server functionality

**Current State** (to be explored):
- NAPI bindings for server lifecycle management
- TypeScript/JavaScript testing infrastructure
- Integration with Next.js frontend development workflow
- Potential home for JavaScript-based E2E tests

**Test Location Options**:
- `crates/lib_bodhiserver_napi/tests-js/` - JavaScript/TypeScript tests
- `crates/lib_bodhiserver_napi/__tests__/` - Jest/Vitest convention

## Architectural Insights for Migration Planning

### Test Categorization Framework

Based on the current state, tests naturally fall into three categories:

**Category 1: Router-Level Tests (routes_app)**
- **Characteristics**: In-process, mocked services, fast execution, parallel-safe
- **Testing**: Routing logic, auth middleware, request/response formats, error handling
- **Infrastructure**: `build_test_router()`, test utilities, in-memory services
- **Execution Time**: Milliseconds per test
- **CI/CD**: Always run, no external dependencies

**Category 2: Integration Tests (Rust-based E2E)**
- **Characteristics**: Real server, real OAuth2, real LLM inference, serial execution
- **Testing**: Complete workflows, service coordination, real model inference
- **Infrastructure**: TestServerHandle, OAuth2 test environment, model files
- **Execution Time**: Seconds to minutes per test
- **CI/CD**: Conditional (requires OAuth2 server, model files)

**Category 3: E2E Tests (JavaScript-based, via NAPI)**
- **Characteristics**: Node.js runtime, TypeScript/JavaScript, browser-like environment
- **Testing**: Frontend integration, NAPI bindings, desktop app workflows
- **Infrastructure**: To be designed (possibly similar to TestServerHandle)
- **Execution Time**: Seconds per test (likely faster than Rust E2E)
- **CI/CD**: Desktop app testing, frontend/backend integration

### Natural Boundaries

**routes_app** is appropriate for:
- Tests that validate **routing and middleware** behavior
- Tests that validate **handler business logic** with mocked external dependencies
- Tests that validate **API contracts** (request/response formats, error codes)
- Tests that require **fast feedback loops** for TDD workflows
- Tests that can run in **parallel** without resource conflicts

**integration-tests** (Rust E2E) is appropriate for:
- Tests that validate **real OAuth2 authentication flows** end-to-end
- Tests that require **actual LLM inference** to validate streaming, tool calling, reasoning
- Tests that validate **service coordination** across the full stack
- Tests that validate **llama.cpp integration** and model loading
- Tests that require **real file system operations** (model downloads, cache management)

**lib_bodhiserver_napi/tests-js** (JavaScript E2E) is appropriate for:
- Tests that validate **NAPI bindings** work correctly from Node.js
- Tests that validate **TypeScript/JavaScript API** contracts
- Tests that simulate **frontend usage patterns** (desktop app, Next.js integration)
- Tests that validate **server lifecycle** from JavaScript (start, stop, reload)
- Tests that require **browser-like environments** for realistic frontend integration

### Key Insights from Exploration

**Overlap Analysis**:
- **Minimal overlap** between routes_app and integration-tests
- routes_app tests **cannot replace** integration-tests (no real OAuth2, no real inference)
- integration-tests tests **should not replace** routes_app (too slow, serial execution)
- Tests serve **complementary roles** in the test pyramid

**Current integration-tests Analysis**:
All 12+ tests in integration-tests are **genuine integration tests** that require:
- Real server startup with llama.cpp integration
- Real OAuth2 authentication via Keycloak
- Real model inference (cannot be mocked)
- Real streaming response validation
- Real toolset execution flows

**None of these tests should move to routes_app** - they serve a different purpose.

**Migration Question**: Should any of these tests move to lib_bodhiserver_napi/tests-js?

Consider:
- JavaScript E2E tests could provide **faster feedback** than Rust E2E tests
- JavaScript tests could better simulate **frontend usage patterns**
- JavaScript tests could validate **NAPI bindings** under realistic conditions
- But JavaScript tests might **lose Rust-level validation** of internal state

## Exploratory Questions for Planning

### Question 1: What is the migration goal?

**Possible interpretations**:
1. Move all integration-tests to lib_bodhiserver_napi/tests-js for JavaScript-based E2E testing
2. Distribute tests: some to routes_app, some to lib_bodhiserver_napi, some stay
3. Add JavaScript E2E tests alongside existing Rust integration tests
4. Consolidate test infrastructure into lib_bodhiserver_napi for reuse

**Explore**:
- User's vision for JavaScript vs Rust E2E testing
- Whether lib_bodhiserver_napi should become the primary E2E test location
- If integration-tests crate should be deleted, renamed, or refocused

### Question 2: What does lib_bodhiserver_napi need?

**Explore**:
- Does lib_bodhiserver_napi already have test infrastructure?
- Should it provide TestServerHandle-like utilities for JavaScript?
- How would JavaScript tests handle OAuth2, model files, and temp directories?
- What JavaScript testing framework should be used (Jest, Vitest, Playwright)?

### Question 3: What test infrastructure should be shared?

**Current situation**:
- `routes_app/src/test_utils/` - Router-level testing utilities
- `integration-tests/src/live_server_utils.rs` - E2E testing utilities
- Both use `AppServiceBuilder` from `lib_bodhiserver` with test-utils features

**Explore**:
- Should lib_bodhiserver_napi expose similar test utilities for JavaScript?
- Can TestServerHandle pattern be adapted for JavaScript consumers?
- Should test data (model files, OAuth config) be shared across test crates?
- How should test fixtures be managed (Rust rstest vs JavaScript beforeAll)?

### Question 4: How should test execution be organized?

**Current execution**:
- `make test.backend` runs routes_app (fast) + integration-tests (slow)
- routes_app runs in parallel, integration-tests runs serially
- CI can skip integration-tests if OAuth2 server unavailable

**Explore**:
- Should JavaScript E2E tests be part of `make test.backend` or separate?
- How should CI distinguish between fast unit tests and slow E2E tests?
- Should there be separate `make test.e2e.rust` and `make test.e2e.js` commands?
- What's the developer experience for running different test suites?

## Suggested Exploration Approach

### Phase 0: Understand lib_bodhiserver_napi

**Explore**:
1. Read `crates/lib_bodhiserver_napi/CLAUDE.md` and `PACKAGE.md` thoroughly
2. Check for existing test infrastructure (`tests-js/`, `__tests__/`, `*.test.ts`)
3. Understand NAPI bindings architecture (what's exposed to JavaScript?)
4. Identify if server lifecycle management is already available
5. Check `package.json` for test scripts and testing frameworks

**Deliverable**: Understanding of lib_bodhiserver_napi's current state and capabilities

### Phase 1: Analyze Test Distribution Options

**Explore**:
1. For each integration-tests test file, consider:
   - Could this be a JavaScript E2E test instead of Rust?
   - What would be gained/lost by moving to JavaScript?
   - Does this test validate Rust internals that JavaScript can't see?
   - Would this test provide better frontend integration coverage in JavaScript?

2. Consider architectural patterns:
   - **Option A**: Keep all Rust E2E tests, add JavaScript E2E tests for frontend scenarios
   - **Option B**: Move most E2E tests to JavaScript, keep minimal Rust tests for internal validation
   - **Option C**: Duplicate critical tests (Rust for internals, JavaScript for API contracts)
   - **Option D**: Specialize by concern (Rust for LLM/services, JavaScript for HTTP API)

**Deliverable**: Recommendation on test distribution with pros/cons

### Phase 2: Design Test Infrastructure

Based on Phase 1 decisions, design:
1. JavaScript test utilities (if needed)
2. TestServerHandle equivalent for JavaScript (if needed)
3. OAuth2 test helpers for JavaScript (if needed)
4. Model file and fixture management (if needed)
5. Test execution strategy (parallel/serial, CI integration)

**Deliverable**: Architecture design for new test infrastructure

### Phase 3: Create Migration Plan

Based on Phases 0-2, create a detailed plan with:
1. Specific tests to move (with rationale)
2. New infrastructure to build
3. Dependencies to update
4. Test execution changes
5. CI/CD modifications
6. Documentation updates
7. Verification steps

**Deliverable**: Executable migration plan with clear phases and acceptance criteria

## Key Considerations for Decision-Making

### Performance Trade-offs
- **Rust E2E**: Slower to compile, slower to execute, but validates internal state
- **JavaScript E2E**: Faster to iterate, faster to execute, but treats server as black box
- **Developer Experience**: JavaScript tests may be more familiar to frontend developers

### Test Coverage Strategy
- **Rust E2E**: Essential for validating service coordination, LLM integration
- **JavaScript E2E**: Essential for validating NAPI bindings, frontend integration
- **Overlap**: Some tests might benefit from both perspectives

### Maintenance Burden
- **One test suite**: Simpler, but might not cover all scenarios
- **Two test suites**: More comprehensive, but requires maintaining two patterns
- **Hybrid approach**: Balance coverage vs maintenance cost

### CI/CD Constraints
- OAuth2 server availability
- Model file storage (large binaries in Git LFS?)
- Test execution time budgets
- Parallel vs serial execution requirements

## Anti-Patterns to Avoid

**Don't**:
1. Move tests just for the sake of moving them (preserve working tests)
2. Assume all E2E tests should be JavaScript (Rust E2E validates internal state)
3. Duplicate every test in both Rust and JavaScript (maintenance burden)
4. Ignore test execution time in CI (slow tests impact developer productivity)
5. Break working tests during migration (maintain green test suite)

**Do**:
1. Preserve test coverage during migration (verify before/after test counts)
2. Consider the natural boundaries (what each test type does best)
3. Design for developer experience (where do I add a new test?)
4. Plan for CI/CD execution (fast feedback loops vs comprehensive coverage)
5. Document the testing strategy (why tests live where they live)

## Success Criteria

A successful migration should achieve:
1. **Preserved Coverage**: All existing test scenarios remain covered
2. **Clear Organization**: Obvious where to add new tests
3. **Fast Feedback**: Unit/router tests run in seconds, E2E tests separate
4. **CI/CD Optimized**: Fast tests always run, slow tests conditional
5. **Maintainable**: Test patterns are consistent and well-documented
6. **Comprehensive**: Both Rust internals and JavaScript API validated

## Starting Point

Begin by exploring lib_bodhiserver_napi thoroughly to understand:
- Its current test infrastructure (if any)
- Its NAPI bindings architecture
- Its relationship to integration testing
- Its potential as an E2E test host

Then analyze each integration-tests test to determine:
- Natural home (Rust E2E vs JavaScript E2E)
- Value proposition (what does this test validate?)
- Migration cost (how much work to move it?)
- Maintenance benefit (is the new location better?)

Present findings with architectural recommendations and trade-off analysis before proposing a detailed migration plan.
