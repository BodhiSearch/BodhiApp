# MSW v2 Type-Safe Testing Setup

This directory contains the MSW v2 integration for type-safe API mocking, inspired by patterns from the openapi-msw library.

## Overview

Since we maintain dual MSW v1/v2 compatibility and openapi-msw has peer dependency conflicts with this setup, we've implemented our own type-safe patterns using:

- **Generated OpenAPI Types**: Single source of truth from `../generated/openapi-schema.ts`
- **MSW v2 via Alias**: Uses `msw2` package alias to avoid conflicts with existing MSW v1
- **Type-Safe Response Helpers**: Inspired by openapi-msw patterns

## Key Benefits

1. **Type Safety**: Full TypeScript support using generated OpenAPI types
2. **Single Source of Truth**: All types come from the OpenAPI specification
3. **Developer Experience**: IntelliSense and compile-time validation
4. **Maintainability**: Changes to OpenAPI spec automatically update test types

## Usage Example

```typescript
// Import the setup and types
import { http, type components } from '../test-utils/msw-v2/setup';

// Create type-safe handlers
export function createAppInfoHandlers(config: Partial<components['schemas']['AppInfo']> = {}) {
  return [
    http.get('/bodhi/v1/info', () => {
      const responseData: components['schemas']['AppInfo'] = {
        status: config.status || 'ready',
        version: config.version || '0.1.0',
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

// Use in tests
import { server } from '../test-utils/msw-v2/setup';
import { createAppInfoHandlers } from './handlers/app-info-typed';

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

it('handles setup status', async () => {
  server.use(...createAppInfoHandlers({ status: 'setup' }));
  // Your test code here
});
```

## File Structure

```
src/test-utils/msw-v2/
├── README.md                    # This documentation
├── setup.ts                     # Main setup file with exports
└── handlers/
    └── app-info-typed.ts        # Example typed handler
```

## Key Patterns

### 1. Handler Creation

- Use `Partial<components['schemas']['TypeName']>` for configuration
- Explicitly type response data with the full schema type
- Return `HttpResponse.json(responseData)` for consistency

### 2. Type Import

- Import types from the setup file: `import { type components } from '../setup'`
- Use generated types directly without re-exporting or creating aliases

### 3. Response Helper

- The `createTypedResponse` helper is available for custom status codes
- Use `HttpResponse.json(data)` directly for standard 200 responses

## Migration from MSW v1

When migrating handlers from MSW v1:

1. Change imports from `'msw'` to `'../test-utils/msw-v2/setup'`
2. Update handler syntax from `rest.get()` to `http.get()`
3. Use generated types instead of manual type definitions
4. Replace `res(ctx.json())` with `HttpResponse.json()`

## Future Improvements

When we can fully migrate to MSW v2:

1. Consider adopting openapi-msw directly for enhanced features
2. Evaluate path parameter type inference
3. Add query parameter validation
4. Implement request body validation
