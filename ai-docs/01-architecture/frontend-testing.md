# Frontend Testing

> **AI Coding Assistant Guide**: This document provides concise frontend testing conventions and patterns for the Bodhi App React frontend. Focus on key concepts and established patterns rather than detailed implementation examples.

## ⚠️ Critical Testing Requirements

### Framer Motion Components
**IMPORTANT**: Components using `framer-motion` (motion.div, motion.button, etc.) require proper mocking to prevent test failures and React warnings.

**Universal Mock Implementation**: Framer Motion is mocked using module aliasing in `vitest.config.ts` which provides a single, consistent mock across all tests.

**Mock Architecture**:
- **Module Aliasing**: `framer-motion` is aliased to `/src/tests/mocks/framer-motion.tsx` in vitest configuration
- **Intelligent Prop Filtering**: Only removes motion-specific props, preserves all HTML functionality (onClick, data-testid, etc.)
- **Comprehensive Coverage**: Supports all common HTML elements and framer-motion hooks

**No Local Mocks Needed**: The module aliasing approach eliminates the need for individual test file mocks. All framer-motion imports are automatically resolved to the universal mock.

**Mock Features**:
- Preserves HTML event handlers (onClick, onSubmit, etc.)
- Maintains accessibility attributes (data-testid, aria-*, etc.)
- Supports CSS classes and styling
- Includes mock implementations for framer-motion hooks (useAnimation, useMotionValue, etc.)

**Symptoms of Missing Mock**:
- React warnings about unrecognized props (`whileHover`, `whileTap`, etc.)
- Test timeouts or failures when components use animations
- Components not rendering properly in tests

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

- write test such that baseURL hardcoding is not needed in apiClient.
```typescript
// crates/bodhi/src/lib/apiClient.ts
const apiClient = axios.create({
  baseURL: '',
  maxRedirects: 0,
});
```

**Base URL Testing Pattern**:
- Use wild card patter with path for msw to mock
- Hardcoding base url will cause problems in production

## Test Utilities and Standardization

### Standardized Test Wrapper

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

### Standardized Window.Location Mocking

**Critical Discovery**: Consistent `window.location` mocking is essential for reliable OAuth and navigation testing.

**✅ Standardized Utility - Use This Pattern**:
```typescript
// crates/bodhi/src/tests/wrapper.tsx
/**
 * Mock window.location for tests
 * @param href - The URL to mock as window.location.href
 */
export const mockWindowLocation = (href: string) => {
  ...
};
```

**Usage Pattern in Tests**:
```typescript
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';

describe('ComponentWithNavigation', () => {
  beforeEach(() => {
    mockWindowLocation('http://localhost:3000/ui/login');
    // Reset for each test to prevent race conditions
  });

  it('handles external URL redirect', async () => {
    // Test will use the mocked location
    render(<ComponentWithNavigation />, { wrapper: createWrapper() });
    
    // Component can read and write to window.location.href
    expect(window.location.href).toBe('http://localhost:3000/ui/login');
  });
});
```

**Why This Utility is Required**:
- **Consistent Behavior**: Same mocking approach across all test files
- **Read/Write Support**: Supports both reading and writing to `window.location.href`
- **Race Condition Prevention**: Proper setup in `beforeEach` prevents test interference
- **URL Parsing**: Automatically extracts protocol and host from provided URL
- **Configurable**: Supports both external and same-origin URL testing scenarios

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
    mockWindowLocation('http://localhost:3000/ui/setup/resource-admin');
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
          ctx.status(201), // 201 when creating new OAuth session
          ctx.json({ location: 'https://oauth.example.com/auth?client_id=test' })
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

### OAuth Flow Testing Patterns

**Current OAuth Implementation**: The OAuth flow now uses JSON responses instead of HTTP redirects, with proper status codes and button state management.

#### OAuth Initiate Testing
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
beforeEach(() => {
  server.resetHandlers();
  // Other test setup
});
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
      ctx.status(201),
      ctx.json({ location: 'https://oauth.example.com/auth' })
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
    ctx.status(201), // 201 Created for new OAuth session
    ctx.json({ location: 'https://oauth.example.com/auth?client_id=test' })
  );
})
```

**OAuth Callback Handler**:
```typescript
rest.post(`*${ENDPOINT_AUTH_CALLBACK}`, (_, res, ctx) => {
  return res(
    ctx.status(200),
    ctx.json({ location: '/ui/chat' })
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
  beforeEach(() => {
    mockWindowLocation('http://localhost:3000/ui/setup/resource-admin');
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );
  });

  it('displays error message when OAuth initiation fails', async () => {
    // Override default handler for this specific test
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

// Framer Motion is mocked via module aliasing in vitest.config.ts
// No runtime mocking needed - the alias resolves 'framer-motion' imports to our mock module

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
**Problem**: Animation library causing test failures and React warnings
**Solution**: Universal mock using module aliasing in vitest.config.ts - eliminates need for individual test file mocks

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
      'framer-motion': path.resolve(__dirname, './src/tests/mocks/framer-motion.tsx'),
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
- **Framer Motion Mock**: Module aliasing resolves `framer-motion` to universal mock
- **Global APIs**: Vitest globals enabled for `describe`, `it`, `expect`

## Testing Best Practices Summary

### Key Lessons from OAuth Testing Fixes

#### 1. ApiClient Configuration is Critical
- **Always configure baseURL for test environment** to enable MSW interception
- **Use environment detection** (`NODE_ENV === 'test'`) for test-specific configuration
- **Understand the axios + MSW interaction** - empty baseURL prevents valid URL construction

#### 2. Standardized Test Utilities are Essential
- **Use mockWindowLocation utility** from `@/tests/wrapper` for all location mocking
- **Call mockWindowLocation in beforeEach** to prevent race conditions between tests
- **Use createWrapper utility** for consistent React Query setup

#### 3. Hook Consistency is Essential
- **Standardize on useMutationQuery** for all mutation hooks
- **Use parameterized helpers** with smart fallbacks for backward compatibility
- **Support custom axios configuration** for OAuth flows requiring 401 acceptance

#### 4. MSW Patterns Must Be Consistent
- **Always use wildcard patterns** (`*${ENDPOINT}`) for URL matching
- **Mock required endpoints** for AppInitializer components (ENDPOINT_APP_INFO)
- **Set up server in beforeAll/afterAll** with resetHandlers in beforeEach

#### 5. Component Testing Focus
- **Test content components** rather than wrapper pages
- **Separate success and error scenarios** into different test cases
- **Avoid unmount() usage** which indicates merged test scenarios
- **Focus on user-visible behavior** and interactions

#### 5. Test Environment Setup
- **Mock browser APIs** (ResizeObserver, matchMedia) that aren't available in tests
- **Universal framer-motion mock** using module aliasing for consistent behavior across all tests
- **Suppress expected console errors** to reduce noise in test output

#### 6. OAuth Flow Testing Specifics
- **Test button state management** - disabled during flow, enabled only on error
- **Test both same-origin and external URL handling** with proper mocking
- **Verify all parameters are sent to backend** in callback testing
- **Test status code differences** (201 for new sessions, 200 for authenticated users)

### Testing Checklist

**Before Writing Tests**:
- [ ] Is apiClient configured with baseURL for tests?
- [ ] Are MSW handlers using wildcard patterns?
- [ ] Does the component use AppInitializer (requires ENDPOINT_APP_INFO mock)?
- [ ] Are you testing the content component, not the wrapper?

**During Test Writing**:
- [ ] Are you using mockWindowLocation in beforeEach?
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
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
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
- `crates/bodhi/src/tests/wrapper.tsx:1-43` - Standardized test utilities including mockWindowLocation
- `crates/bodhi/src/tests/mocks/framer-motion.tsx` - Universal framer-motion mock module
- `crates/bodhi/vitest.config.ts:5-28` - Vitest configuration with module aliasing for framer-motion

**Example Test Files**:
- `crates/bodhi/src/app/ui/login/page.test.tsx` - AppInitializer page testing patterns
- `crates/bodhi/src/app/ui/setup/resource-admin/page.test.tsx` - OAuth flow testing
- `crates/bodhi/src/hooks/useOAuth.test.ts` - Hook testing with MSW
- `crates/bodhi/src/components/AppInitializer.test.tsx` - Component testing patterns
- `crates/bodhi/src/app/ui/auth/callback/page.test.tsx` - OAuth callback testing with parameter handling

---

*This guide reflects lessons learned from OAuth testing fixes and standardized test utilities. For backend testing patterns, see [Backend Testing](backend-testing.md). For complete testing implementation examples, see [TESTING_GUIDE.md](TESTING_GUIDE.md).*
