# Frontend Testing

This document provides comprehensive guidance for frontend testing patterns, frameworks, and quality assurance approaches in the Bodhi App React frontend.

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
- **Happy DOM** - Lightweight DOM environment for testing
- **jsdom** - Alternative DOM environment when needed

### Additional Testing Libraries
- **@testing-library/user-event** - User interaction simulation
- **@testing-library/jest-dom** - Custom Jest matchers for DOM testing
- **jest-axe** - Accessibility testing utilities
- **@vitest/coverage-v8** - Code coverage reporting

## Component Testing Patterns

### Basic Component Test
```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ComponentName } from './ComponentName';

describe('ComponentName', () => {
  it('renders correctly with required props', () => {
    render(<ComponentName prop1="test" />);
    expect(screen.getByText('test')).toBeInTheDocument();
  });

  it('handles user interactions', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    
    render(<ComponentName onClick={handleClick} />);
    
    await user.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledOnce();
  });

  it('displays loading state correctly', () => {
    render(<ComponentName isLoading={true} />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });
});
```

### Testing with React Query
```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, waitFor } from '@testing-library/react';

const createTestQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: { retry: false },
    mutations: { retry: false },
  },
});

const renderWithQueryClient = (component: React.ReactElement) => {
  const testQueryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={testQueryClient}>
      {component}
    </QueryClientProvider>
  );
};

describe('DataComponent', () => {
  it('displays loading state initially', () => {
    renderWithQueryClient(<DataComponent />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('displays data after loading', async () => {
    renderWithQueryClient(<DataComponent />);
    
    await waitFor(() => {
      expect(screen.getByText('Data loaded')).toBeInTheDocument();
    });
  });
});
```

### Next.js Navigation Testing Patterns

#### Navigation Test Utilities
```typescript
// src/tests/navigation-utils.tsx
import { vi } from 'vitest';

export function createMockRouter() {
  return {
    push: vi.fn(),
    replace: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
    prefetch: vi.fn(),
  };
}

export function createMockNavigation() {
  const mockRouter = createMockRouter();
  const pathname = '/';

  return { mockRouter, pathname };
}

// Custom assertion utility
export function expectNavigation(navigate: any, expectedPath: string) {
  expect(navigate).toHaveBeenCalledWith(expectedPath);
}
```

#### Router Component Testing
```typescript
import { renderWithRouter, expectNavigation } from '@/tests/router-utils';
import { NavigationComponent } from './NavigationComponent';

describe('NavigationComponent', () => {
  it('navigates to correct route on click', async () => {
    const user = userEvent.setup();
    const { router } = renderWithRouter(<NavigationComponent />);
    
    await user.click(screen.getByText('Go to Models'));
    
    expect(router.state.location.pathname).toBe('/ui/models');
  });

  it('highlights active route', () => {
    renderWithRouter(<NavigationComponent />, ['/ui/models']);
    
    expect(screen.getByText('Models')).toHaveClass('active');
  });
});
```

### Form Testing Patterns

#### Form Validation Testing
```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CreateModelForm } from './CreateModelForm';

describe('CreateModelForm', () => {
  it('submits form with valid data', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    
    render(<CreateModelForm onSubmit={onSubmit} />);
    
    await user.type(screen.getByLabelText('Model Name'), 'Test Model');
    await user.type(screen.getByLabelText('Alias'), 'test:latest');
    await user.click(screen.getByRole('button', { name: 'Create' }));
    
    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Test Model',
      alias: 'test:latest'
    });
  });

  it('displays validation errors for invalid input', async () => {
    const user = userEvent.setup();
    
    render(<CreateModelForm onSubmit={vi.fn()} />);
    
    // Submit without filling required fields
    await user.click(screen.getByRole('button', { name: 'Create' }));
    
    expect(screen.getByText('Name is required')).toBeInTheDocument();
    expect(screen.getByText('Alias is required')).toBeInTheDocument();
  });

  it('resets form on reset button click', async () => {
    const user = userEvent.setup();
    
    render(<CreateModelForm onSubmit={vi.fn()} />);
    
    const nameInput = screen.getByLabelText('Model Name');
    await user.type(nameInput, 'Test Model');
    
    await user.click(screen.getByRole('button', { name: 'Reset' }));
    
    expect(nameInput).toHaveValue('');
  });
});
```

## API Mocking with MSW

### Mock Setup
```typescript
// src/tests/mocks/handlers.ts
import { rest } from 'msw';

export const handlers = [
  // Models API
  rest.get('/bodhi/v1/models', (req, res, ctx) => {
    const page = req.url.searchParams.get('page') || '1';
    const pageSize = req.url.searchParams.get('page_size') || '10';
    
    return res(
      ctx.json({
        data: [
          { id: '1', name: 'Test Model', alias: 'test:latest' }
        ],
        total: 1,
        page: parseInt(page),
        page_size: parseInt(pageSize)
      })
    );
  }),

  rest.post('/bodhi/v1/models', (req, res, ctx) => {
    return res(
      ctx.status(201),
      ctx.json({
        id: '2',
        name: 'New Model',
        alias: 'new:latest'
      })
    );
  }),

  // Error scenarios
  rest.get('/bodhi/v1/models/error', (req, res, ctx) => {
    return res(
      ctx.status(500),
      ctx.json({
        error: {
          message: 'Internal server error',
          type: 'server_error'
        }
      })
    );
  }),
];
```

### Test Server Setup
```typescript
// src/tests/setup.ts
import { setupServer } from 'msw/node';
import { handlers } from './mocks/handlers';

export const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### Dynamic Mock Responses
```typescript
describe('ModelsList with API errors', () => {
  it('displays error message when API fails', async () => {
    // Override default handler for this test
    server.use(
      rest.get('/bodhi/v1/models', (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Failed to load models',
              type: 'server_error'
            }
          })
        );
      })
    );

    renderWithQueryClient(<ModelsList />);
    
    await waitFor(() => {
      expect(screen.getByText('Failed to load models')).toBeInTheDocument();
    });
  });
});
```

## Accessibility Testing

### Basic Accessibility Tests
```typescript
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

describe('ComponentName Accessibility', () => {
  it('should not have accessibility violations', async () => {
    const { container } = render(<ComponentName />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('supports keyboard navigation', async () => {
    const user = userEvent.setup();
    render(<ComponentName />);
    
    await user.tab();
    expect(screen.getByRole('button')).toHaveFocus();
    
    await user.keyboard('{Enter}');
    expect(screen.getByText('Button clicked')).toBeInTheDocument();
  });

  it('provides proper ARIA labels', () => {
    render(<ComponentName />);
    
    expect(screen.getByRole('button')).toHaveAttribute('aria-label', 'Close dialog');
    expect(screen.getByRole('dialog')).toHaveAttribute('aria-labelledby');
  });
});
```

### Screen Reader Testing
```typescript
describe('Screen Reader Support', () => {
  it('announces loading states', () => {
    render(<DataComponent isLoading={true} />);
    
    expect(screen.getByText('Loading data')).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Loading data');
  });

  it('announces form errors', async () => {
    const user = userEvent.setup();
    render(<FormComponent />);
    
    await user.click(screen.getByRole('button', { name: 'Submit' }));
    
    expect(screen.getByRole('alert')).toHaveTextContent('Name is required');
  });
});
```

## Performance Testing

### Component Performance Tests
```typescript
describe('Performance Tests', () => {
  it('renders large lists efficiently', () => {
    const items = Array.from({ length: 1000 }, (_, i) => ({
      id: i,
      name: `Item ${i}`
    }));

    const start = performance.now();
    render(<LargeList items={items} />);
    const end = performance.now();

    expect(end - start).toBeLessThan(100); // Should render in under 100ms
  });

  it('handles rapid state updates without performance issues', async () => {
    const user = userEvent.setup();
    render(<SearchComponent />);
    
    const searchInput = screen.getByRole('textbox');
    
    const start = performance.now();
    await user.type(searchInput, 'rapid typing test');
    const end = performance.now();
    
    expect(end - start).toBeLessThan(500); // Should handle typing smoothly
  });
});
```

## Test Commands and Configuration

### Frontend Test Commands
```bash
cd crates/bodhi

# Run tests once (CI mode)
npm run test

# Run tests with coverage
npm run test -- --coverage

# Run specific test file
npm run test -- ComponentName.test.tsx

# Run tests matching pattern
npm run test -- --grep "form validation"
```

### Vitest Configuration
```typescript
// crates/bodhi/vitest.config.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'happy-dom',
    setupFiles: ['./src/tests/setup.ts'],
    globals: true,
    coverage: {
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/tests/',
        '**/*.d.ts',
        '**/*.config.*',
      ],
      thresholds: {
        global: {
          branches: 80,
          functions: 80,
          lines: 80,
          statements: 80,
        },
      },
    },
  },
});
```

## Test Case Design Best Practices

### Test Structure and Organization

#### Preferred Test Case Design
- **Focus on content components only** - Test `ComponentContent` rather than wrapper components like `ComponentPage`
- **Separate success and error scenarios** - Never merge unrelated test steps in a single test
- **Avoid `unmount()` calls** - Using `unmount()` indicates merging of unrelated test steps
- **One responsibility per test** - Each test should verify a single behavior or scenario
- **Meaningful test separation** - Keep related but distinct scenarios (success vs error) in separate tests

#### Test Case Examples
```typescript
// ✅ Good: Focused, single responsibility
it('renders admin setup page with all required content', () => {
  // Test static content and UI elements
});

it('handles successful OAuth flow and redirects to auth URL', async () => {
  // Test only the success path
});

it('handles OAuth errors and allows retry', async () => {
  // Test only the error path and retry functionality
});

// ❌ Bad: Merged unrelated scenarios
it('handles OAuth flow with success and error scenarios', async () => {
  // Test success path
  // unmount() - indicates merging unrelated steps
  // Test error path - should be separate test
});
```

#### Component Testing Focus
- **Test content components** - Focus on `LoginContent`, `ResourceAdminContent`, etc.
- **Skip wrapper components** - Avoid testing `LoginPage`, `ResourceAdminPage` unless they have specific logic
- **Test user-visible behavior** - Focus on what users see and interact with
- **Maintain test isolation** - Each test should be independent and not rely on previous test state

### Maintainable Test Suites

#### Substantial vs Granular Tests
- **Prefer fewer, substantial tests** over many fine-grained tests
- **Group related assertions** within logical test boundaries
- **Separate unrelated concerns** into different tests
- **Avoid excessive test fragmentation** that makes maintenance difficult

#### Test Naming Conventions
- Use descriptive names that explain the behavior being tested
- Include the expected outcome in the test name
- Be specific about the scenario being tested

```typescript
// ✅ Good test names
it('renders admin setup page with all required content and functionality')
it('handles successful OAuth flow and redirects to auth URL')
it('handles OAuth errors and allows retry')
it('displays error message with proper styling')

// ❌ Poor test names
it('works correctly')
it('handles errors')
it('OAuth test')
```

## Common Issues and Solutions

### Test Design Anti-patterns
- **Issue**: Using `unmount()` between test scenarios
- **Solution**: Separate unrelated test scenarios into different tests

- **Issue**: Merging success and error paths in one test
- **Solution**: Keep success and error scenarios in separate test cases

- **Issue**: Testing wrapper components without specific logic
- **Solution**: Focus testing on content components that contain the actual functionality

### Next.js Navigation Testing Issues
- **Issue**: Navigation not working in tests
- **Solution**: Mock `next/navigation` hooks properly in tests

### State Management Testing
- **Issue**: React Query cache persisting between tests
- **Solution**: Create new QueryClient for each test

### Async Testing Pitfalls
- **Issue**: Tests finishing before async operations complete
- **Solution**: Use `waitFor` and proper async/await patterns

### Mock Cleanup
- **Issue**: Mocks affecting other tests
- **Solution**: Reset mocks in `afterEach` hooks

## Related Documentation

- **[Backend Testing](backend-testing.md)** - Backend testing approaches and database testing
- **[Frontend React](frontend-react.md)** - React component patterns and development
- **[Development Conventions](development-conventions.md)** - Testing conventions and file organization
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Complete testing implementation guide

---

*For backend testing patterns, see [Backend Testing](backend-testing.md). For complete testing implementation examples, see [TESTING_GUIDE.md](TESTING_GUIDE.md).*
