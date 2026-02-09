  Task: Migrate integration-tests to lib_bodhiserver_napi and routes_app                                   
                                                                                         
  Context from Previous Migration

  We just completed a successful migration that:
  - Consolidated routes_all into routes_app
  - Created 102 new router-level auth tests using build_test_router()
  - Established dual-pattern testing: router-level (auth + integration) vs handler-level (isolated business
   logic)
  - Used sub-agents for each phase with clear verification steps

  Key lessons learned:
  - Real services (TestDbService, SqliteSessionService, LocalDataService) work well in tests
  - Mock services (MockAuthService, MockToolService, MockSharedContext) should only be used when external
  I/O is unavoidable
  - Router-level tests use the fully-composed router with real middleware
  - Session auth requires: JWT in session store + Cookie + Sec-Fetch-Site: same-origin + Host headers

  Current State Analysis Needed

  Before starting, explore and document:

  1. integration-tests crate structure
    - Current dependencies (server_app? lib_bodhiserver? routes_all?)
    - Test organization and patterns
    - How tests start servers and manage lifecycle
    - Authentication patterns used (OAuth2 tokens, sessions, cookies)
    - What services are mocked vs real
  2. lib_bodhiserver_napi structure
    - Purpose and current capabilities
    - NAPI bindings architecture
    - Test infrastructure (if any)
    - How it relates to integration testing
  3. Migration goal clarification
    - What does "migrate to lib_bodhiserver_napi and routes_app" mean?
    - Should integration-tests be deleted or refactored?
    - Should tests move into lib_bodhiserver_napi/tests/?
    - Should tests use routes_app test infrastructure?

  Exploration Phase (Before Planning)

  Launch exploration agents to answer:

  Question 1: What is lib_bodhiserver_napi?

  - Read crates/lib_bodhiserver_napi/CLAUDE.md and PACKAGE.md
  - Understand its purpose and current test coverage
  - Check if it already has integration tests or test infrastructure
  - Identify any existing patterns for testing server lifecycle

  Question 2: How do integration-tests currently work?

  - Read crates/integration-tests/CLAUDE.md thoroughly
  - Map out the test categories (live server, chat completions, OAuth flow, etc.)
  - Document the TestServerHandle pattern and fixtures
  - Identify dependencies on server_app, routes_all, and other crates

  Question 3: What's the dependency relationship?

  - Grep for imports: what does integration-tests import from server_app, routes_all, etc.?
  - What does lib_bodhiserver_napi import?
  - Can lib_bodhiserver_napi depend on routes_app/test-utils?
  - Should there be a new test-utils feature in lib_bodhiserver_napi?

  Question 4: What test patterns overlap with routes_app?

  - Compare integration-tests test patterns with routes_app router-level tests
  - Identify which tests are truly "integration" (real server, OAuth, LLM) vs "router-level" (could use
  build_test_router)
  - Determine if some integration-tests should actually move to routes_app/tests/

  Decision Points (After Exploration)

  Present findings and ask user:

  1. Test categorization: Which tests belong where?
    - True integration tests (real server, llama.cpp, OAuth2 server) → where?
    - Router-level tests (HTTP API, auth middleware, no real LLM) → routes_app/tests/?
    - NAPI binding tests → lib_bodhiserver_napi/tests/?
  2. Crate fate: What happens to integration-tests crate?
    - Keep it for true end-to-end tests only?
    - Delete it and distribute tests to other crates?
    - Rename/refactor it?
  3. Test infrastructure: Where should server lifecycle utilities live?
    - Keep TestServerHandle in integration-tests?
    - Move to lib_bodhiserver_napi/test-utils?
    - Use routes_app test infrastructure where possible?
  4. Dependencies: How should test dependencies flow?
    - Should lib_bodhiserver_napi depend on routes_app/test-utils?
    - Should integration-tests depend on lib_bodhiserver_napi/test-utils?
    - What about circular dependencies?

  Proposed Phased Approach (Flexible)

  Phase 0: Research & Decision (1-2 sub-agents)

  - Exploration agents gather information
  - Present findings with architectural options
  - User makes decisions on test categorization and crate structure

  Phase 1: Test Categorization (potential sub-agents)

  - Audit all integration-tests tests
  - Categorize each test: integration vs router-level vs NAPI
  - Identify tests that can be deleted (redundant with routes_app tests)
  - Document dependencies for each test

  Phase 2: Infrastructure Setup (potential sub-agents)

  - Create test-utils in lib_bodhiserver_napi if needed
  - Set up any new test infrastructure
  - Establish patterns for NAPI testing vs integration testing

  Phase 3: Migration Execution (multiple sub-agents)

  - Move/rewrite tests according to categorization
  - Update dependencies
  - Ensure all tests pass in new locations

  Phase 4: Cleanup (sub-agent)

  - Remove redundant tests
  - Update documentation
  - Remove or refactor integration-tests crate as decided

  Test-Driven Principles

  - Each phase should maintain a green test suite
  - Don't delete tests until replacements pass
  - Verify test count before/after each change
  - Use cargo test --workspace as the ultimate verification

  Sub-Agent Strategy

  Each sub-agent should:
  1. Explore first: Read current state, understand context
  2. Plan: Propose specific changes for their scope
  3. Implement: Make changes incrementally
  4. Verify: Run tests, check compilation
  5. Report: Document what was done, what was learned, blockers encountered

  Non-Prescriptive Guidance

  Do explore:
  - Whether some "integration" tests are actually just router-level tests in disguise
  - If NAPI bindings need different test patterns than Rust integration tests
  - How test data management (model files, OAuth config) should work across crates
  - Whether fixtures like live_server and llama2_7b_setup belong in a shared test-utils

  Do NOT assume:
  - That all tests must move
  - That lib_bodhiserver_napi must become the new home for everything
  - That the current integration-tests structure is optimal
  - That test patterns from one crate directly apply to another

  Consider:
  - Test execution time (integration tests are slow, router tests are fast)
  - CI/CD implications (some tests need OAuth servers, others don't)
  - Developer experience (where do I add a new integration test?)
  - Maintenance burden (how many test patterns to maintain?)

  Starting Point

  Begin with Phase 0: Launch 2-3 exploration agents in parallel to answer the key questions above. Based on
   their findings, we'll create a detailed migration plan with clear decision points for user approval
  before executing.
