# Next.js Testing Guide

This guide outlines the testing approach for Next.js components in our Next.js v14 project.

## Overview

Our Next.js application uses a comprehensive testing strategy that provides better, more maintainable tests. The approach offers:

- **Next.js App Router testing** with proper configuration
- **Reduced boilerplate** through reusable utilities
- **Better separation** between unit and integration tests
- **Simplified mocking** for navigation hooks

## Key Improvements

✅ **Next.js App Router testing** with proper configuration and mocking
✅ **Simplified testing utilities** that are actually practical to use
✅ **Better mock patterns** that reduce repetitive code
✅ **Clear separation** between unit tests (mocked) and integration tests

## Testing Utilities

### 1. Enhanced Wrapper (`createWrapper`)

The basic wrapper includes Next.js testing configuration:

```tsx
import { createWrapper } from '@/tests/wrapper';

render(<Component />, { wrapper: createWrapper() });
```

### 2. Next.js Navigation Testing

For testing Next.js navigation behavior:

```tsx
import { render, screen } from '@testing-library/react';
import { useRouter } from 'next/navigation';

// Mock Next.js navigation
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    back: vi.fn(),
  }),
  usePathname: () => '/current-path',
}));
```

### 3. Mock Navigation (`createMockNavigation`)

For unit testing components in isolation:

```tsx
import { createMockNavigation } from '@/tests/router-utils';

const mockNav = createMockNavigation();
mockNav.setCurrentPath('/dashboard');

vi.mock('next/navigation', () => ({
  useRouter: () => mockNav.mockRouter,
  usePathname: () => mockNav.mockPathname(),
}));
```

## Testing Patterns

### Pattern 1: Integration Tests (Recommended for navigation logic)

```tsx
describe('Navigation Integration', () => {
  it('should navigate between pages correctly', async () => {
    const user = userEvent.setup();
    const mockPush = vi.fn();

    vi.mock('next/navigation', () => ({
      useRouter: () => ({ push: mockPush }),
      usePathname: () => '/',
    }));

    render(<NavigationComponent />);

    await user.click(screen.getByText('About'));
    expect(mockPush).toHaveBeenCalledWith('/about');
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

### Before (Complex Pattern)
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

### After (Simplified Pattern)
```tsx
// Clean, focused testing with utilities
const mockRouter = createMockNavigation();
mockRouter.setCurrentPath('/test');

render(<Component />, { wrapper: createWrapper() });
```

## Best Practices

### 1. Choose the Right Testing Approach

- **Integration tests**: Use Next.js navigation mocks for testing routing logic and navigation flows
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

### Issue: Next.js Navigation Hooks Not Working in Tests
**Solution**: Mock `next/navigation` hooks properly with vi.mock.

### Issue: Link Component Not Working in Tests
**Solution**: Use Next.js Link component mocking or test utilities.

### Issue: Cannot Test Navigation State Changes
**Solution**: Use the navigation utilities to inspect and control navigation state.

### Issue: Tests Are Slow Due to Complex Setup
**Solution**: Use `createMockNavigation` for simple unit tests that don't need full navigation.

## Examples

See the following files for complete examples:
- `src/tests/examples/navigation.test.tsx` - Complete Navigation component testing
- `src/tests/navigation-utils.test.tsx` - Testing utility examples
- `src/tests/navigation-utils.tsx` - Utility implementations
