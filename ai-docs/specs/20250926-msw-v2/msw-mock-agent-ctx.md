# MSW Mock Conversion Agent Context

## Project Goal
Convert all MSW v2 handlers from stubs to one-time mocks using closure state. Each handler invocation creates independent state that responds only once.

## Core Implementation Patterns

### 1. Standard Handler Pattern
```typescript
export function mockHandler(config = {}) {
  let hasBeenCalled = false;  // Closure state - unique per invocation

  return [
    typedHttp.method(ENDPOINT, async ({ response }) => {
      if (hasBeenCalled) return;  // Pass through if already called
      hasBeenCalled = true;       // Mark as called BEFORE responding

      // ... response logic
      return response(status).json(data);
    }),
  ];
}
```

### 2. Parameterized Handler Pattern
```typescript
export function mockGetResource(id: string, config = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT, async ({ params, response }) => {
      if (hasBeenCalled) return;
      if (params.id !== id) return;  // Still check parameter match
      hasBeenCalled = true;

      // ... response logic
    }),
  ];
}
```

### 3. Catch-All Handlers (Special Case)
```typescript
// These should NOT have closure state - they're fallback handlers
export function mockResourceNotFound() {
  return [
    typedHttp.get(ENDPOINT, async ({ response }) => {
      // NO hasBeenCalled check - always responds as fallback
      return response(404).json({ error: "Not found" });
    }),
  ];
}
```

## Test Migration Patterns

### Single Call Tests (No Change)
```typescript
server.use(...mockHandler(config));
await api.call(); // Works once
```

### Multiple Call Tests (Needs Update)
```typescript
// Before (broken after conversion)
server.use(...mockHandler(config));
await api.call(); // Works
await api.call(); // Fails - passes through

// After (fixed)
server.use(
  ...mockHandler(config),
  ...mockHandler(config)
);
await api.call(); // Works
await api.call(); // Works
```

### Sequential Different Responses
```typescript
server.use(
  ...mockHandlerError({ status: 500 }),  // First call fails
  ...mockHandler({ data: success })      // Retry succeeds
);
```

## Agent Guidelines

### Implementation Steps
1. **Read this context** to understand established patterns
2. **Analyze target file** to identify all exported mock functions
3. **Apply closure pattern** to each function (except catch-alls)
4. **Build and test** to identify failures
5. **Fix failing tests** by adding multiple mock invocations
6. **Document results** in log file
7. **Update this context** with new insights

### Critical Rules
- Add `hasBeenCalled` state to ALL handlers except catch-alls
- Mark as called BEFORE returning response (not after)
- Preserve all existing parameter validation logic
- Don't change function signatures or exports
- Fix tests by adding more mock invocations, not by changing logic

### Build Commands
```bash
cd crates/bodhi
npm run build    # Verify TypeScript compilation
npm run test     # Run full test suite
```

## Files and Conversion Status

| File | Status | Agent | Functions | Tests Fixed | Notes |
|------|--------|-------|-----------|-------------|-------|
| models.ts | ✅ Complete | 1 | 19 | 1 test file | Most complex, sets patterns |
| settings.ts | ✅ Complete | 2 | 13 | 1 test file | Catch-all pattern identified |
| tokens.ts | ✅ Complete | 3 | 13 | 0 test files | Clean conversion, no test failures |
| user.ts | ✅ Complete | 4 | 11 | 2 test files | Clean conversion, multiple-render patterns handled |
| info.ts | ✅ Complete | 5 | 6 | 1 test file | Clean conversion with single test fix |
| modelfiles.ts | ✅ Complete | 6 | 13 | 0 test files | Clean conversion, simple endpoints |
| access-requests.ts | ✅ Complete | 7 | 19 | 3 test files | Parameterized handlers with ID validation, multiple-call patterns |
| api-models.ts | Pending | 8 | TBD | TBD | - |

## Common Issues and Solutions

### Multiple Renders in Tests
**Issue**: Tests that render components multiple times will fail because each render makes API calls, but mocks only respond once.

**Solution**: Add fresh mock invocations before subsequent renders:
```typescript
// After first render and unmount
server.use(...mockModelsDefault());

// Second render
render(<Component />, { wrapper: createWrapper() });
```

**Pattern**: Look for tests with multiple `render()` calls or `unmount()` followed by `render()`.

### Parameterized Handlers
**Insight**: For handlers that check parameters (like ID/alias matching), preserve the parameter check logic BEFORE the closure state check:
```typescript
export function mockGetModel(alias: string, config = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT, async ({ response, params }) => {
      // Parameter check FIRST
      if (params.alias !== alias) return;

      // THEN closure check
      if (hasBeenCalled) return;
      hasBeenCalled = true;

      // Response logic
    })
  ];
}
```

## Insights for Future Agents

### Agent 1 Learnings (models.ts)
1. **All Functions Need Conversion**: Even wrapper functions that call base functions need individual closure state - don't rely on delegation
2. **Parameter Matching**: Preserve existing parameter validation logic before closure checks
3. **Test Failures**: Multiple renders are the most common test failure pattern - search for tests with responsive layout checking or multiple component instances
4. **Build First**: Always verify TypeScript compilation passes before running tests
5. **Error Messages**: 404 errors in test output usually indicate missing mock responses due to closure state
6. **Function Count**: models.ts had 19 functions - larger than expected, plan accordingly for time
7. **Systematic Approach**: Apply closure pattern to ALL exported functions, even simple wrapper functions

### Agent 2 Learnings (settings.ts)
1. **Catch-All Handler Pattern**: Identified special handlers that should NOT have closure state - functions ending in "NotFound" that provide fallback 404 responses
2. **React Query Invalidation Pattern**: Tests that check query invalidation after mutations need multiple mock responses (initial load + refetch after mutation)
3. **Wrapper Function Strategy**: For functions that delegate to base functions, inline the implementation with closure state rather than relying on delegation
4. **Parameter Ordering**: For parameterized handlers, parameter validation must come BEFORE closure state check to maintain proper routing logic
5. **Test Pattern Recognition**: Look for tests with "invalidates...query" in the description - these likely need duplicate mock invocations
6. **Error Message Analysis**: Timing-related test failures ("expected X to be greater than Y") usually indicate missing refetch responses
7. **Function Categories**: Settings had 13 regular functions + 3 catch-all functions = 16 total, but only 13 needed conversion

### Agent 3 Learnings (tokens.ts)
1. **Clean Conversion Possible**: Some handler files convert smoothly without test failures - tokens.ts had no catch-all handlers and simpler usage patterns
2. **Wrapper Function Delegation**: For convenience functions that simply call base functions with parameters, delegation works fine since base functions have closure state
3. **Parameterized Handler Success**: The pattern of parameter check BEFORE closure check works consistently across different endpoint types
4. **Test Usage Patterns**: Token handlers appear to be used in single-call scenarios per test, avoiding the multiple-render pitfalls seen in models.ts
5. **Function Distribution**: Tokens had 13 functions total - 3 core handlers + 5 convenience methods + 5 error handlers, all needing conversion
6. **Error Handler Patterns**: Error handlers follow same closure pattern as success handlers - no special treatment needed
7. **Build Stability**: Proper TypeScript compilation continues to be a reliable indicator of successful conversion

### Agent 4 Learnings (user.ts)
1. **Multiple-Render Pattern Mastery**: Successfully handled complex test cases with 3 renders (mobile/tablet/desktop responsive testing) by adding complete mock sets before each subsequent render
2. **Parameterized Handler Validation**: User role change and removal handlers with user_id parameters work correctly with parameter check BEFORE closure check pattern
3. **Convenience Function Delegation Success**: Confirmed Agent 3's approach - convenience functions that delegate to base functions with closure state work without modification
4. **Complete Mock Set Requirement**: When tests have multiple renders, ALL mocks (app info, user info, domain-specific) must be refreshed, not just the primary endpoint
5. **Test Failure Analysis**: Systematic approach to identifying root causes - 404 errors indicate missing mocks, 500 errors are expected for error test cases
6. **Function Categories**: User handlers had 11 functions - 3 user info + 2 users list + 4 convenience methods + 2 role change + 2 user removal, all with clear responsibilities
7. **Build-Test-Fix Cycle**: TypeScript compilation success followed by targeted test runs enables efficient identification and resolution of multiple-render issues
8. **Test Quality Improvement**: Fixed tests now properly simulate real-world usage patterns where components may re-render due to responsive design changes

### Agent 5 Learnings (info.ts)
1. **Simple Delegation Pattern Success**: Info.ts confirmed the pattern from Agent 3 - files with simple convenience functions that delegate cleanly can be converted efficiently by inlining implementations
2. **Query Invalidation Pattern Recognition**: Tests that trigger mutations followed by query invalidation need additional mock invocations to handle the refetch cycle (setup mutation → invalidation → refetch)
3. **Minimal Test Impact Achievable**: With only 6 functions and straightforward usage patterns, achieved conversion with just 1 test fix, demonstrating efficiency gains from established patterns
4. **No Catch-All Handlers Confirmed**: Info endpoints follow simple success/error patterns without complex fallback routing, making conversion straightforward
5. **Inlined Convenience Functions Work Well**: Rather than delegating convenience functions to base functions, inlining implementations with independent closure state provides clearer behavior
6. **Function Count Predictability**: Info handlers followed expected simple pattern: 1 main success handler + 3 variants + 1 main error handler + 1 error variant = 6 total
7. **Query Dependency Chain Understanding**: Tests involving setup mutations must account for dependent queries (app info, user info) that get invalidated and refetched automatically

### Agent 6 Learnings (modelfiles.ts)
1. **Clean Conversion Pattern Mastery**: Modelfiles.ts was the smoothest conversion yet - 13 functions converted with zero test failures, demonstrating pattern maturity
2. **Simple Endpoint Pattern Success**: Files with straightforward CRUD operations (list, post, get status) without complex parameter validation convert very cleanly
3. **Inlined Implementation Strategy Confirmed**: Following Agent 5's approach, inlined all convenience functions rather than delegating, ensuring independent closure state per function
4. **No Catch-All Handlers Expected**: Simple REST endpoints don't require complex fallback routing, making conversion predictable
5. **Test Usage Pattern Recognition**: Like tokens.ts, modelfiles endpoints are used in single-call scenarios per test, avoiding multiple-render complexity
6. **Function Distribution Predictable**: Modelfiles had 13 functions across 3 endpoint groups: model files list (4), pull downloads (5), pull POST (4)
7. **Home Stretch Efficiency**: With 6/8 files complete and patterns established, conversion efficiency has dramatically improved (zero test fixes needed)
8. **API Endpoint Complexity Correlation**: Simpler REST endpoints (modelfiles, tokens, info) require no test fixes; complex entity management (models, users) requires test fixes for multiple-render patterns

### Agent 7 Learnings (access-requests.ts)
1. **Parameterized Handler ID Validation**: Access-requests endpoints with ID parameters require proper parameter validation BEFORE closure checks - the pattern `if (params.id !== id.toString()) return;` must come first
2. **Multiple Status Check Patterns**: Request access workflows involve multiple API calls to check status before/after actions, requiring duplicate mock invocations in test helper functions
3. **Test Callback Pattern Recognition**: Tests that expect button state changes (enabled/disabled/enabled again) typically trigger additional API calls for status validation
4. **Helper Function Mock Multiplication**: Shared test helper functions like `createNoRequestHandlers` need multiple instances of the same mock to handle various test scenarios that reuse the helper
5. **Error Test Pattern Success**: Tests that intentionally trigger API errors work correctly with closure state - they just need sufficient error mock invocations to handle component retry patterns
6. **Complex Workflow Testing**: Access request flows (submit → status check → UI update → status recheck) require careful analysis of component behavior to provide adequate mock coverage
7. **ID Parameter Requirement**: All access-request approve/reject handlers require ID parameters - missing IDs cause runtime errors in tests
8. **Test Error Analysis Improvement**: Error logs showing expected HTTP status codes (404, 500) are actually signs of successful test behavior, not failures - important to distinguish expected errors from missing mock coverage