# MSW v1 Constraint and openapi-msw Integration Issue

**Date**: 2025-09-26
**Status**: Documented constraint and interim solution implemented
**Context**: MSW v2 migration with type-safe API mocking

## Overview

This document captures the constraint encountered when attempting to integrate `openapi-msw` with our dual MSW v1/v2 setup, the interim solution implemented, and the path forward for future implementation.

## Original Plan

The intended approach was to use `openapi-msw` for type-safe API mocking with full OpenAPI integration:

### Planned Implementation

```typescript
// setup.ts - intended with openapi-msw
import { createOpenApiHttp } from 'openapi-msw';
import { setupServer } from 'msw2/node';
import type { paths, components } from '../generated/openapi-schema';

export const http = createOpenApiHttp<paths>();
export const server = setupServer();
export type { components };

// app-info-typed.ts - intended handler
import { http, type components } from '../setup';

export function createTypedAppInfoHandlers(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.get('/bodhi/v1/info', ({ response }) => {
      return response(200).json({
        status: config.status || 'ready',
        version: config.version || '0.1.0'
      });
    })
  ];
}
```

### Expected Benefits

1. **Full Type Inference**:
   - Path autocompletion (only valid paths for each HTTP method)
   - Path parameters typed from OpenAPI spec
   - Query parameters with `query.get()` and `query.getAll()`
   - Request/response bodies matching OpenAPI schemas

2. **Response Helper**:
   - Status code validation (only valid codes per endpoint)
   - Response shape enforcement (matching OpenAPI schemas)
   - Content type handling (json, text, empty responses)

3. **Developer Experience**:
   - IntelliSense for all API paths
   - Compile-time validation of mock responses
   - No manual type imports or re-exports needed

## MSW v1 Constraint

### Technical Issue

The `openapi-msw` library requires MSW v2 as a peer dependency, but our project maintains MSW v1 for existing tests:

```json
// package.json
{
  "devDependencies": {
    "msw": "^1.3.5",           // Required for existing tests
    "msw2": "npm:msw@^2.10.5", // Alias for new MSW v2 tests
    "openapi-msw": "^2.0.0"    // Requires MSW v2 peer dependency
  }
}
```

### Peer Dependency Conflict

```bash
npm error While resolving: openapi-msw@2.0.0
npm error Found: msw@1.3.5
npm error node_modules/msw
npm error   dev msw@"^1.3.5" from the root project

npm error Could not resolve dependency:
npm error peer msw@"^2.10.5" from openapi-msw@2.0.0
```

### Import Resolution Issue

When `openapi-msw` tries to import from MSW, it finds the v1 installation instead of our `msw2` alias:

```bash
SyntaxError: Named export 'HttpResponse' not found. The requested module 'msw' is a CommonJS module
```

The library expects MSW v2's ESM exports but finds MSW v1's CommonJS exports.

## Interim Solution Implemented

Since `openapi-msw` couldn't be used, we implemented type-safe patterns inspired by the library:

### Current Implementation

```typescript
// setup.ts - actual implementation
import { setupServer } from 'msw2/node';

export type { paths, components } from '../generated/openapi-schema';
export { http, HttpResponse } from 'msw2';
export const server = setupServer();

export function createTypedResponse<T>(status: number, data: T) {
  return HttpResponse.json(data, { status });
}

// app-info-typed.ts - actual handler
import { http, HttpResponse, type components } from '../setup';

export function createTypedAppInfoHandlers(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.get('/bodhi/v1/info', () => {
      const responseData: components['schemas']['AppInfo'] = {
        status: config.status || 'ready',
        version: config.version || '0.1.0'
      };
      return HttpResponse.json(responseData);
    })
  ];
}
```

### What We Achieved

✅ **Generated Types Usage**: `Partial<components['schemas']['AppInfo']>` as single source of truth
✅ **Type Safety**: Full TypeScript coverage using OpenAPI schemas
✅ **Clean Patterns**: Maintainable handler creation inspired by openapi-msw
✅ **Working Tests**: All tests passing with new implementation
✅ **Documentation**: Team usage patterns documented

### What We Missed

❌ **Path Autocompletion**: No compile-time path validation
❌ **Response Helper**: No built-in status code/schema validation
❌ **Query/Path Parameters**: No automatic type inference
❌ **Enhanced DX**: Manual handler setup instead of auto-generated

## Future Path Forward

### When MSW v1 is Removed

Once the project fully migrates to MSW v2 and removes MSW v1:

1. **Install openapi-msw properly**:
   ```bash
   npm uninstall msw
   npm install msw@^2.10.5  # No alias needed
   npm install openapi-msw@^2.0.0
   ```

2. **Implement the original plan**:
   ```typescript
   // setup.ts - future implementation
   import { createOpenApiHttp } from 'openapi-msw';
   import { setupServer } from 'msw/node';
   import type { paths, components } from '../generated/openapi-schema';

   export const http = createOpenApiHttp<paths>();
   export const server = setupServer();
   export type { components };
   ```

3. **Migrate existing handlers**:
   - Replace `http` from `'msw2'` with `http` from openapi-msw
   - Add `{ response }` parameter destructuring
   - Use `response(200).json()` instead of `HttpResponse.json()`

### Migration Steps

1. **Audit MSW v1 usage**: Identify all tests using MSW v1
2. **Migrate remaining tests**: Convert MSW v1 tests to MSW v2 patterns
3. **Remove MSW v1 dependency**: Update package.json
4. **Install openapi-msw**: Add proper MSW v2 peer dependency
5. **Refactor handlers**: Update to use openapi-msw patterns
6. **Validate type safety**: Ensure enhanced type inference works

## Lessons Learned

### Dual Dependency Challenges

Maintaining two versions of the same library (MSW v1 and v2) creates:
- Peer dependency conflicts for libraries expecting specific versions
- Import resolution ambiguity when using npm aliases
- Complex testing setup requiring separate configurations

### Alternative Approaches

1. **Custom Type Wrappers**: Can achieve significant type safety without library dependencies
2. **Generated Type Integration**: OpenAPI-generated types provide excellent foundation
3. **Incremental Migration**: Patterns can be designed to ease future library adoption

### Design Principles

The interim solution demonstrates that effective type-safe testing can be achieved by:
- Using generated types as single source of truth
- Creating consistent patterns inspired by best practices
- Maintaining clean separation between setup and usage
- Documenting patterns for team adoption

## Code Comparison

### Original Plan vs Current Implementation

| Feature | openapi-msw (Planned) | Current Implementation |
|---------|----------------------|----------------------|
| Type Import | `import { http } from openapi-msw` | `import { http } from 'msw2'` |
| Handler Creation | `http.get(path, ({ response }) => ...)` | `http.get(path, () => ...)` |
| Response Creation | `response(200).json(data)` | `HttpResponse.json(data)` |
| Path Validation | ✅ Compile-time | ❌ Runtime only |
| Parameter Types | ✅ Auto-inferred | ❌ Manual |
| Schema Validation | ✅ Built-in | ✅ Via TypeScript types |

### Future Migration Example

```typescript
// Before: Current implementation
export function createTypedAppInfoHandlers(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.get('/bodhi/v1/info', () => {
      const responseData: components['schemas']['AppInfo'] = {
        status: config.status || 'ready',
        version: config.version || '0.1.0'
      };
      return HttpResponse.json(responseData);
    })
  ];
}

// After: With openapi-msw
export function createTypedAppInfoHandlers(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.get('/bodhi/v1/info', ({ response }) => {
      return response(200).json({
        status: config.status || 'ready',
        version: config.version || '0.1.0'
      });
    })
  ];
}
```

## Conclusion

While we couldn't implement the full openapi-msw integration due to MSW v1 constraints, the interim solution successfully achieved:

- Type-safe API mocking using generated OpenAPI types
- Clean, maintainable patterns inspired by openapi-msw
- Full test coverage with improved developer experience
- Clear migration path for future openapi-msw adoption

This approach demonstrates that effective type safety can be achieved incrementally, even when ideal tools aren't immediately available due to project constraints.