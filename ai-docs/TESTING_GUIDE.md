# React Router Testing Guide

This guide outlines the improved testing approach for React Router components in our Vite+React project.

## Overview

We've migrated from Next.js to React Router and enhanced our testing utilities to provide better, more maintainable tests. The new approach offers:

- **Eliminated React Router v7 warnings** with proper future flags
- **Reduced boilerplate** through reusable utilities
- **Better separation** between unit and integration tests
- **Simplified mocking** for navigation hooks

## Key Improvements

✅ **Fixed React Router v7 warnings** by adding future flags to BrowserRouter
✅ **Simplified testing utilities** that are actually practical to use
✅ **Better mock patterns** that reduce repetitive code
✅ **Clear separation** between unit tests (mocked) and integration tests (real router)

## Testing Utilities

### 1. Enhanced Wrapper (`createWrapper`)

The basic wrapper now includes React Router v7 future flags to eliminate warnings:

```tsx
import { createWrapper } from '@/tests/wrapper';

render(<Component />, { wrapper: createWrapper() });
```

### 2. Router Testing Utilities (`renderWithRouter`)

For integration testing with actual routing behavior:

```tsx
import { renderWithRouter } from '@/tests/router-utils';

const { getCurrentPath } = renderWithRouter(
  <NavigationComponent />,
  {
    initialEntries: ['/docs/getting-started/'],
    routes: [
      { path: '/docs/getting-started/', element: <DocsPage /> },
      { path: '/docs/features/', element: <FeaturesPage /> },
    ],
  }
);

expect(getCurrentPath()).toBe('/docs/getting-started/');
```

### 3. Mock Navigation (`createMockNavigation`)

For unit testing components in isolation:

```tsx
import { createMockNavigation } from '@/tests/router-utils';

const mockNav = createMockNavigation();
mockNav.setCurrentPath('/dashboard');

vi.mock('@/lib/navigation', () => ({
  useRouter: () => mockNav.mockRouter,
  usePathname: () => mockNav.mockPathname(),
}));
```

## Testing Patterns

### Pattern 1: Integration Tests (Recommended for routing logic)

```tsx
describe('Navigation Integration', () => {
  it('should navigate between pages correctly', async () => {
    const user = userEvent.setup();
    
    const { expectCurrentPath } = renderWithRouter(
      <App />,
      {
        initialEntries: ['/'],
        routes: [
          { path: '/', element: <HomePage /> },
          { path: '/about', element: <AboutPage /> },
        ],
      }
    );

    await user.click(screen.getByText('About'));
    expectCurrentPath('/about');
  });
});
```

### Pattern 2: Unit Tests (For component behavior)

```tsx
describe('Navigation Component', () => {
  it('should render active state correctly', () => {
    const mockNav = createMockNavigation();
    mockNav.setMockLocation({ pathname: '/current-page' });

    vi.mock('@/lib/navigation', () => ({
      usePathname: () => mockNav.mockLocation.pathname,
    }));

    render(<Navigation />);
    
    expect(screen.getByText('Current Page')).toHaveAttribute('aria-current', 'page');
  });
});
```

### Pattern 3: Navigation Assertions

```tsx
import { expectNavigation } from '@/tests/router-utils';

const mockRouter = { push: vi.fn(), replace: vi.fn() };
const navAssertions = expectNavigation(mockRouter);

// Trigger navigation
await user.click(screen.getByText('Go to Dashboard'));

// Assert navigation occurred
navAssertions.toHaveNavigatedTo('/dashboard');
```

## Migration Guide

### Before (Old Pattern)
```tsx
// Lots of repetitive mocking
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: vi.fn() }),
  usePathname: () => '/test',
}));

vi.mock('next/link', () => ({
  default: ({ children, ...props }) => <a {...props}>{children}</a>,
}));

render(<Component />, { wrapper: createWrapper() });
```

### After (New Pattern)
```tsx
// Clean, focused testing
const { expectCurrentPath } = renderWithRouter(
  <Component />,
  { initialEntries: ['/test'] }
);

expectCurrentPath('/test');
```

## Best Practices

### 1. Choose the Right Testing Approach

- **Integration tests**: Use `renderWithRouter` for testing routing logic and navigation flows
- **Unit tests**: Use `createMockNavigation` for testing component behavior in isolation

### 2. Test Real User Interactions

```tsx
// Good: Test actual user interactions
await user.click(screen.getByRole('link', { name: 'About' }));
expectCurrentPath('/about');

// Avoid: Testing implementation details
expect(mockRouter.push).toHaveBeenCalledWith('/about');
```

### 3. Use Descriptive Test Names

```tsx
// Good
it('should navigate to features page when features link is clicked')

// Less clear
it('should handle navigation')
```

### 4. Group Related Tests

```tsx
describe('Navigation Component', () => {
  describe('routing behavior', () => {
    // Integration tests with real router
  });
  
  describe('component rendering', () => {
    // Unit tests with mocked navigation
  });
});
```

## Common Issues and Solutions

### Issue: React Router v7 Future Flag Warnings
**Solution**: Use the enhanced wrapper with future flags enabled.

### Issue: Link Component Not Working in Tests
**Solution**: Use `renderWithRouter` instead of manual Link mocking.

### Issue: Cannot Test Navigation State Changes
**Solution**: Use the router utilities to inspect and control navigation state.

### Issue: Tests Are Slow Due to Full Router Setup
**Solution**: Use `createMockNavigation` for simple unit tests that don't need routing.

## Examples

See the following files for complete examples:
- `src/tests/examples/improved-navigation.test.tsx` - Complete Navigation component testing
- `src/tests/router-utils.test.tsx` - Testing utility examples
- `src/tests/router-utils.tsx` - Utility implementations
