# Frontend Testing

This document provides comprehensive guidance for frontend testing patterns, frameworks, and quality assurance approaches in the Bodhi App React frontend, based on lessons learned from OAuth testing fixes and current codebase patterns.

## Required Documentation References

**MUST READ for frontend testing:**
- `ai-docs/01-architecture/frontend-react.md` - React component patterns and development
- `ai-docs/01-architecture/development-conventions.md` - Testing conventions and file organization

**FOR COMPLETE TESTING OVERVIEW:**
- `ai-docs/01-architecture/backend-testing.md` - Backend testing approaches
- `ai-docs/01-architecture/TESTING_GUIDE.md` - Complete testing implementation guide

## Testing Philosophy

### Frontend Testing Pyramid
1. **Unit Tests** (70%) - Component logic, hooks, and utility functions
2. **Integration Tests** (20%) - Feature workflows and user interactions
3. **End-to-End Tests** (10%) - Complete user journeys
4. **Accessibility Tests** - WCAG compliance verification

### Quality Goals
- **Unit Tests**: 80%+ coverage for components and hooks
- **Integration Tests**: All critical user flows covered
- **Accessibility**: 100% WCAG 2.1 AA compliance
- **Performance**: Core Web Vitals targets met

## Technology Stack

### Core Testing Tools
- **Vitest** - Fast unit testing framework with Next.js integration
- **Testing Library** - Component testing utilities with accessibility focus
- **MSW (Mock Service Worker)** - API mocking for reliable tests
- **jsdom** - DOM environment for testing (configured in vitest.config.ts)
- **React Query v3.39.3** - Data fetching and state management in tests

### Additional Testing Libraries
- **@testing-library/user-event** - User interaction simulation
- **@testing-library/jest-dom** - Custom Jest matchers for DOM testing
- **jest-axe** - Accessibility testing utilities
- **@vitest/coverage-v8** - Code coverage reporting

## Critical Testing Configuration

### ApiClient Test Environment Setup

**Key Discovery**: The most critical aspect of frontend testing is proper apiClient configuration for test environments. This was discovered during OAuth testing fixes.

```typescript
// crates/bodhi/src/lib/apiClient.ts
const isTest = typeof process !== 'undefined' && process.env.NODE_ENV === 'test';
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});
```

**Why This Matters**:
- **Empty baseURL Problem**: When `baseURL: ''`, axios cannot construct valid URLs from relative paths
- **MSW Interception**: MSW requires valid URLs to intercept requests using wildcard patterns (`*`)
- **AppInitializer Components**: Pages using `AppInitializer` call `useAppInfo()` immediately, which fails without proper baseURL
- **Test Environment Detection**: Using `NODE_ENV === 'test'` ensures production behavior is unchanged

## Hook Testing Patterns

### Hook Consistency Requirements

**Critical Pattern**: All hooks must use consistent patterns to ensure reliable testing. This was discovered during OAuth testing fixes.

#### useMutationQuery vs Direct useMutation

**✅ Correct Pattern - Use useMutationQuery Helper**:
```typescript
// crates/bodhi/src/hooks/useOAuth.ts
export function useOAuthInitiate(options?: {
  onSuccess?: (response: AxiosResponse<AuthInitiateResponse>) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AuthInitiateResponse>, AxiosError<ErrorResponse>, void> {
  return useMutationQuery<AuthInitiateResponse, void>(
    ENDPOINT_AUTH_INITIATE,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth flow';
        options?.onError?.(message);
      },
    },
    {
      validateStatus: (status) => status >= 200 && status < 500, // Accept 401 responses
    }
  );
}
```

**❌ Problematic Pattern - Direct useMutation**:
```typescript
// This pattern was causing OAuth test failures
return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
  async (variables) => {
    const response = await apiClient.post<T>(endpoint, variables);
    return response;
  },
  options
);
```

**Why useMutationQuery is Required**:
- **Consistent axios configuration**: Supports custom `validateStatus` and headers
- **Automatic query invalidation**: Built-in cache invalidation patterns
- **Error handling consistency**: Standardized error response handling
- **Test compatibility**: Works reliably with MSW patterns

### Test Wrapper Utilities

**Standard Test Wrapper Pattern**:
```typescript
// crates/bodhi/src/tests/wrapper.tsx
import { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from 'react-query';

export const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnMount: false,
      },
    },
  });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = 'TestClientWrapper';
  return Wrapper;
};
```

**Hook Testing Example**:
```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { createWrapper } from '@/tests/wrapper';
import { useOAuthInitiate } from './useOAuth';

describe('useOAuthInitiate', () => {
  it('handles successful OAuth initiation', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    const { result } = renderHook(
      () => useOAuthInitiate({ onSuccess: mockOnSuccess, onError: mockOnError }),
      { wrapper: createWrapper() }
    );

    result.current.mutate();

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockOnSuccess).toHaveBeenCalled();
  });
});
```

## Component Testing Patterns

### AppInitializer vs Non-AppInitializer Pages

**Critical Discovery**: Pages using `AppInitializer` require different testing approaches than those that don't.

#### Pages with AppInitializer (Login, Resource-Admin, Chat, Models)

**AppInitializer Pattern**:
```typescript
// These pages call useAppInfo() immediately when rendering
export default function ResourceAdminPage() {
  return (
    <AppInitializer allowedStatus="resource-admin" authenticated={false}>
      <ResourceAdminContent />
    </AppInitializer>
  );
}
```

**Testing Requirements**:
- **Must mock ENDPOINT_APP_INFO**: AppInitializer calls `useAppInfo()` immediately
- **Requires proper baseURL**: Without it, axios fails before MSW can intercept
- **Test content components**: Focus on `*Content` components, not wrapper pages

```typescript
describe('ResourceAdminPage', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );
  });

  it('handles OAuth initiation when login required', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(401), // 401 when login required
          ctx.json({ auth_url: 'https://oauth.example.com/auth?client_id=test' })
        );
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });
});
```

#### Pages without AppInitializer

**Simpler Testing Pattern**:
```typescript
// These pages don't call useAppInfo() automatically
describe('SimpleComponent', () => {
  it('renders correctly', () => {
    render(<SimpleComponent />);
    expect(screen.getByText('Expected content')).toBeInTheDocument();
  });
});
```

### Next.js Navigation Testing

**Standard Navigation Mock Pattern**:
```typescript
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

describe('NavigationComponent', () => {
  beforeEach(() => {
    pushMock.mockClear();
  });

  it('navigates to correct route', async () => {
    render(<NavigationComponent />);

    await userEvent.click(screen.getByText('Go to Models'));

    expect(pushMock).toHaveBeenCalledWith('/ui/models');
  });
});
```

### Form Testing Patterns

#### OAuth Flow Testing
```typescript
describe('OAuth Form Testing', () => {
  it('handles OAuth initiation with proper error handling', async () => {
    // Mock window.location.href for redirect testing
    const mockLocation = { href: '' };
    Object.defineProperty(window, 'location', {
      value: mockLocation,
      writable: true,
    });

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(401), // 401 when login required
          ctx.json({ auth_url: 'https://oauth.example.com/auth?client_id=test' })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('displays error message when OAuth fails', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'OAuth configuration error',
              type: 'internal_server_error',
            },
          })
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });
  });
});
```

## MSW (Mock Service Worker) Configuration

### Critical MSW Setup Requirements

**Key Discovery**: MSW configuration is critical for reliable testing, especially with the baseURL fix.

#### Standard MSW Server Setup
```typescript
// Standard pattern used across all test files
import { setupServer } from 'msw/node';
import { rest } from 'msw';

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
```

#### Wildcard Pattern Usage

**✅ Correct Pattern - Use Wildcard Prefix**:
```typescript
// This works with the baseURL fix
server.use(
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
    return res(ctx.json({ status: 'ready' }));
  }),
  rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
    return res(
      ctx.status(401),
      ctx.json({ auth_url: 'https://oauth.example.com/auth' })
    );
  })
);
```

**Why Wildcards are Required**:
- **BaseURL Handling**: With `baseURL: 'http://localhost:3000'` in tests, full URLs are constructed
- **Flexible Matching**: `*` prefix matches any protocol/host combination
- **Environment Independence**: Works in both test and development environments

#### OAuth-Specific MSW Patterns

**OAuth Initiate Handler**:
```typescript
rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
  return res(
    ctx.status(401), // 401 when login required
    ctx.json({ auth_url: 'https://oauth.example.com/auth?client_id=test' })
  );
})
```

**OAuth Callback Handler**:
```typescript
rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
  return res(
    ctx.status(200),
    ctx.set('Location', '/ui/chat'),
  );
})
```

**Error Response Handler**:
```typescript
rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
  return res(
    ctx.status(500),
    ctx.json({
      error: {
        message: 'OAuth configuration error',
        type: 'internal_server_error',
        code: 'oauth_config_error',
      },
    })
  );
})
```

### Dynamic Mock Responses

**Per-Test Handler Override Pattern**:
```typescript
describe('OAuth Error Handling', () => {
  it('displays error message when OAuth initiation fails', async () => {
    // Override default handler for this specific test
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      }),
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'OAuth configuration error',
              type: 'internal_server_error',
            },
          })
        );
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });
  });
});
```

### Test Environment Setup

**Global Test Setup** (`crates/bodhi/src/tests/setup.ts`):
```typescript
import { vi } from 'vitest';
import '@testing-library/jest-dom';

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => {
  const React = require('react');
  return {
    motion: new Proxy({}, {
      get: (target, prop) => {
        return ({ children, ...rest }: { children?: React.ReactNode }) =>
          React.createElement('div', rest, children);
      }
    }),
    AnimatePresence: ({ children }: { children?: React.ReactNode }) =>
      React.createElement(React.Fragment, null, children),
  };
});

// Mock ResizeObserver
global.ResizeObserver = class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Suppress expected console errors in tests
const originalError = console.error;
beforeAll(() => {
  console.error = (...args) => {
    const errorString = args.join(' ');
    if (
      errorString.includes('Request failed with status code ') ||
      errorString.includes('Network Error')
    ) {
      return; // Suppress expected test errors
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
```

## Test Case Design Best Practices

### Critical Testing Principles

**Lessons from OAuth Testing Fixes**:

#### 1. Focus on Content Components
**✅ Test Content Components**:
```typescript
// Test the actual functionality
describe('LoginContent', () => {
  it('handles OAuth initiation correctly', async () => {
    render(<LoginContent />, { wrapper: createWrapper() });
    // Test actual user interactions
  });
});
```

**❌ Avoid Testing Wrapper Components**:
```typescript
// Don't test wrapper pages unless they have specific logic
describe('LoginPage', () => {
  // This just wraps LoginContent with AppInitializer
  // Focus testing on LoginContent instead
});
```

#### 2. Separate Success and Error Scenarios
**✅ Separate Test Cases**:
```typescript
it('handles successful OAuth flow and redirects to auth URL', async () => {
  // Test only success path
});

it('displays error message when OAuth initiation fails', async () => {
  // Test only error path
});
```

**❌ Don't Merge Unrelated Scenarios**:
```typescript
it('handles OAuth flow with success and error scenarios', async () => {
  // Test success path
  // unmount() - indicates merging unrelated steps
  // Test error path - should be separate test
});
```

#### 3. Avoid unmount() Usage
- **Issue**: Using `unmount()` indicates merging unrelated test scenarios
- **Solution**: Create separate test cases for different scenarios
- **Benefit**: Better test isolation and clearer failure messages

#### 4. Test Naming Conventions
**✅ Descriptive Names**:
```typescript
it('handles OAuth initiation when login required and redirects to auth URL')
it('displays error message when OAuth initiation fails')
it('shows redirecting state during OAuth initiation')
```

**❌ Vague Names**:
```typescript
it('works correctly')
it('handles errors')
it('OAuth test')
```

### Page Testing Patterns

#### AppInitializer Page Testing
```typescript
describe('ResourceAdminPage', () => {
  beforeEach(() => {
    // Always mock ENDPOINT_APP_INFO for AppInitializer pages
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );
  });

  it('redirects to /ui/setup when status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );

    render(<ResourceAdminPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });
});
```

## Common Issues and Solutions

### OAuth Testing Issues

#### Issue: "Invalid base URL" Error
**Problem**: Tests failing with axios "Invalid base URL" error when using MSW
**Root Cause**: Empty baseURL in apiClient prevents axios from constructing valid URLs
**Solution**: Configure baseURL for test environment in apiClient.ts

```typescript
// crates/bodhi/src/lib/apiClient.ts
const isTest = typeof process !== 'undefined' && process.env.NODE_ENV === 'test';
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});
```

#### Issue: MSW Not Intercepting Requests
**Problem**: MSW handlers not catching API requests
**Root Cause**: Incorrect URL patterns or missing wildcard prefixes
**Solution**: Use wildcard patterns with endpoint constants

```typescript
// ✅ Correct
rest.get(`*${ENDPOINT_APP_INFO}`, handler)

// ❌ Incorrect
rest.get('/bodhi/v1/info', handler)
```

#### Issue: AppInitializer Pages Failing
**Problem**: Pages with AppInitializer component failing in tests
**Root Cause**: AppInitializer calls useAppInfo() immediately, requires mocked response
**Solution**: Always mock ENDPOINT_APP_INFO in beforeEach for AppInitializer pages

```typescript
beforeEach(() => {
  server.use(
    rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
      return res(ctx.json({ status: 'resource-admin' }));
    })
  );
});
```

### Hook Testing Issues

#### Issue: Hook Inconsistency
**Problem**: Some hooks use useMutation directly, others use useMutationQuery
**Root Cause**: Inconsistent patterns across the codebase
**Solution**: Standardize on useMutationQuery for all mutation hooks

#### Issue: Custom validateStatus Not Working
**Problem**: Need to accept 401 responses as success for OAuth flows
**Root Cause**: Direct useMutation doesn't support custom axios config
**Solution**: Use useMutationQuery with axiosConfig parameter

```typescript
return useMutationQuery<AuthInitiateResponse, void>(
  ENDPOINT_AUTH_INITIATE,
  'post',
  options,
  {
    validateStatus: (status) => status >= 200 && status < 500, // Accept 401
  }
);
```

### Test Environment Issues

#### Issue: Framer Motion Errors
**Problem**: Animation library causing test failures
**Solution**: Mock framer-motion in test setup

#### Issue: ResizeObserver Errors
**Problem**: Browser API not available in test environment
**Solution**: Mock ResizeObserver globally

#### Issue: Console Error Noise
**Problem**: Expected errors cluttering test output
**Solution**: Suppress expected error messages in test setup

## Test Commands and Configuration

### Frontend Test Commands
```bash
cd crates/bodhi

# Run tests once (CI mode)
npm run test -- --run

# Run tests in watch mode
npm run test

# Run specific test file
npm run test -- --run src/app/ui/login/page.test.tsx

# Run tests matching pattern
npm run test -- --run --grep "OAuth"

# Run tests with coverage
npm run test -- --run --coverage
```

### Vitest Configuration

**Current Configuration** (`crates/bodhi/vitest.config.ts`):
```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom', // Using jsdom, not happy-dom
    setupFiles: ['./src/tests/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,jsx,ts,tsx}'],
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    deps: {
      optimizer: {
        web: {
          include: ['@testing-library/jest-dom'],
        },
      },
    },
  },
});
```

**Key Configuration Notes**:
- **Environment**: Using `jsdom` instead of `happy-dom` for better compatibility
- **Setup Files**: Global test setup in `./src/tests/setup.ts`
- **Alias Support**: `@/` imports work in tests
- **Global APIs**: Vitest globals enabled for `describe`, `it`, `expect`

## Testing Best Practices Summary

### Key Lessons from OAuth Testing Fixes

#### 1. ApiClient Configuration is Critical
- **Always configure baseURL for test environment** to enable MSW interception
- **Use environment detection** (`NODE_ENV === 'test'`) for test-specific configuration
- **Understand the axios + MSW interaction** - empty baseURL prevents valid URL construction

#### 2. Hook Consistency is Essential
- **Standardize on useMutationQuery** for all mutation hooks
- **Use parameterized helpers** with smart fallbacks for backward compatibility
- **Support custom axios configuration** for OAuth flows requiring 401 acceptance

#### 3. MSW Patterns Must Be Consistent
- **Always use wildcard patterns** (`*${ENDPOINT}`) for URL matching
- **Mock required endpoints** for AppInitializer components (ENDPOINT_APP_INFO)
- **Set up server in beforeAll/afterAll** with resetHandlers in afterEach

#### 4. Component Testing Focus
- **Test content components** rather than wrapper pages
- **Separate success and error scenarios** into different test cases
- **Avoid unmount() usage** which indicates merged test scenarios
- **Focus on user-visible behavior** and interactions

#### 5. Test Environment Setup
- **Mock browser APIs** (ResizeObserver, matchMedia) that aren't available in tests
- **Mock animation libraries** (framer-motion) to prevent test failures
- **Suppress expected console errors** to reduce noise in test output

### Testing Checklist

**Before Writing Tests**:
- [ ] Is apiClient configured with baseURL for tests?
- [ ] Are MSW handlers using wildcard patterns?
- [ ] Does the component use AppInitializer (requires ENDPOINT_APP_INFO mock)?
- [ ] Are you testing the content component, not the wrapper?

**During Test Writing**:
- [ ] Are success and error scenarios in separate test cases?
- [ ] Are you avoiding unmount() usage?
- [ ] Are test names descriptive and specific?
- [ ] Are you using the standard createWrapper() pattern?

**After Writing Tests**:
- [ ] Do all tests pass consistently?
- [ ] Are mocks properly reset between tests?
- [ ] Is the test focused on user-visible behavior?
- [ ] Are async operations properly awaited?

### File Organization

**Test File Patterns**:
```
src/app/ui/login/page.test.tsx          # Page tests (with AppInitializer)
src/hooks/useOAuth.test.ts              # Hook tests
src/components/AppInitializer.test.tsx  # Component tests
src/tests/setup.ts                      # Global test setup
src/tests/wrapper.tsx                   # Test wrapper utilities
```

**Import Patterns**:
```typescript
// Standard test imports
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { createWrapper } from '@/tests/wrapper';
import { ENDPOINT_APP_INFO } from '@/hooks/useQuery';
```

## Related Documentation

- **[Backend Testing](backend-testing.md)** - Backend testing approaches and database testing
- **[Frontend React](frontend-react.md)** - React component patterns and development
- **[Development Conventions](development-conventions.md)** - Testing conventions and file organization
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Complete testing implementation guide
- **[API Integration](api-integration.md)** - Frontend-backend integration patterns and hook usage

## Implementation References

**Key Files Referenced in This Guide**:
- `crates/bodhi/src/lib/apiClient.ts:4-8` - Test environment baseURL configuration
- `crates/bodhi/src/hooks/useQuery.ts:77-113` - useMutationQuery implementation with axios config
- `crates/bodhi/src/hooks/useOAuth.ts:33-60` - OAuth hook using useMutationQuery pattern
- `crates/bodhi/src/tests/setup.ts:1-77` - Global test environment setup
- `crates/bodhi/src/tests/wrapper.tsx:4-18` - Standard test wrapper utility
- `crates/bodhi/vitest.config.ts:5-28` - Vitest configuration for frontend tests

**Example Test Files**:
- `crates/bodhi/src/app/ui/login/page.test.tsx` - AppInitializer page testing patterns
- `crates/bodhi/src/app/ui/setup/resource-admin/page.test.tsx` - OAuth flow testing
- `crates/bodhi/src/hooks/useOAuth.test.ts` - Hook testing with MSW
- `crates/bodhi/src/components/AppInitializer.test.tsx` - Component testing patterns

---

*This guide reflects lessons learned from OAuth testing fixes and current codebase patterns. For backend testing patterns, see [Backend Testing](backend-testing.md). For complete testing implementation examples, see [TESTING_GUIDE.md](TESTING_GUIDE.md).*
