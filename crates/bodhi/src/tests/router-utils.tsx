import { ReactNode } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { createMemoryRouter, RouterProvider } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from 'react-query';
import { vi, expect } from 'vitest';

// Simplified router testing utilities for React Router v6

export interface RouteConfig {
  path: string;
  element: ReactNode;
}

export interface RouterTestOptions {
  initialEntries?: string[];
  initialIndex?: number;
  routes?: RouteConfig[];
  queryClient?: QueryClient;
}

/**
 * Simple utility to render components with router context
 * This is the most practical approach for most testing scenarios
 */
export function renderWithRouter(
  ui: ReactNode,
  options: RouterTestOptions & RenderOptions = {}
) {
  const {
    initialEntries = ['/'],
    initialIndex = 0,
    routes = [{ path: '*', element: ui }],
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, refetchOnMount: false },
        mutations: { retry: false },
      },
    }),
    ...renderOptions
  } = options;

  const router = createMemoryRouter(routes, {
    initialEntries,
    initialIndex,
    future: {
      v7_startTransition: true,
      v7_relativeSplatPath: true,
    },
  });

  const Wrapper = ({ children }: { children: ReactNode }) => (
    <RouterProvider router={router}>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </RouterProvider>
  );

  const result = render(ui, {
    wrapper: Wrapper,
    ...renderOptions,
  });

  return {
    ...result,
    router,
    // Helper to get current location
    getCurrentPath: () => router.state.location.pathname,
    getCurrentSearch: () => router.state.location.search,
  };
}

/**
 * Creates mock navigation functions for unit testing
 * Use this when you want to test component behavior without actual routing
 */
export function createMockNavigation() {
  const mockRouter = {
    push: vi.fn(),
    replace: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
    pathname: '/',
    query: {},
    asPath: '/',
  };

  const mockPathname = vi.fn(() => '/');
  const mockSearchParams = {
    get: vi.fn(),
    getAll: vi.fn(),
    has: vi.fn(),
    set: vi.fn(),
    delete: vi.fn(),
    toString: vi.fn(() => ''),
  };

  return {
    mockRouter,
    mockPathname,
    mockSearchParams,
    // Helper to set up mocks for a specific path
    setCurrentPath: (path: string) => {
      mockRouter.pathname = path;
      mockRouter.asPath = path;
      mockPathname.mockReturnValue(path);
    },
  };
}

/**
 * Test helper for asserting navigation calls
 * Use this to verify that navigation functions were called correctly
 */
export function expectNavigation(mockRouter: any) {
  return {
    toHaveNavigatedTo: (path: string) => {
      expect(mockRouter.push).toHaveBeenCalledWith(path);
    },
    toHaveReplacedWith: (path: string) => {
      expect(mockRouter.replace).toHaveBeenCalledWith(path);
    },
    toHaveGonenBack: () => {
      expect(mockRouter.back).toHaveBeenCalled();
    },
    toHaveRefreshed: () => {
      expect(mockRouter.refresh).toHaveBeenCalled();
    },
  };
}

// Re-export commonly used testing utilities
export { createWrapper, createFullWrapper } from './wrapper';
