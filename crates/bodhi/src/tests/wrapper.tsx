import { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from 'react-query';
import {
  BrowserRouter,
  createMemoryRouter,
  RouterProvider,
} from 'react-router-dom';

// Create a simple wrapper that only includes essential providers
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
    <BrowserRouter
      future={{ v7_startTransition: true, v7_relativeSplatPath: true }}
    >
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </BrowserRouter>
  );

  Wrapper.displayName = 'TestClientWrapper';
  return Wrapper;
};

// Enhanced wrapper for testing routing behavior
export interface RouterTestOptions {
  initialEntries?: string[];
  initialIndex?: number;
  routes?: Array<{
    path: string;
    element: ReactNode;
  }>;
}

export const createRouterWrapper = (options: RouterTestOptions = {}) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnMount: false,
      },
    },
  });

  const { initialEntries = ['/'], initialIndex = 0, routes = [] } = options;

  // Create memory router for controlled testing
  const router = createMemoryRouter(
    routes.length > 0
      ? routes.map((route) => ({
          path: route.path,
          element: route.element,
        }))
      : [
          {
            path: '*',
            element: <div data-testid="router-outlet" />,
          },
        ],
    {
      initialEntries,
      initialIndex,
      future: {
        v7_startTransition: true,
        v7_relativeSplatPath: true,
      },
    }
  );

  const Wrapper = ({ children }: { children: ReactNode }) => (
    <RouterProvider router={router}>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </RouterProvider>
  );

  Wrapper.displayName = 'TestRouterWrapper';
  return { Wrapper, router };
};

// Create a more comprehensive wrapper for tests that need all providers
export const createFullWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnMount: false,
      },
    },
  });

  const Wrapper = ({ children }: { children: ReactNode }) => {
    // Dynamically import providers to avoid issues with mocked modules
    let content = children;

    try {
      // Try to import and use ChatSettingsProvider if available
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { ChatSettingsProvider } = require('@/hooks/use-chat-settings');
      content = <ChatSettingsProvider>{content}</ChatSettingsProvider>;
    } catch (e) {
      // If mocked or not available, skip this provider
    }

    try {
      // Try to import and use NavigationProvider if available
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { NavigationProvider } = require('@/hooks/use-navigation');
      content = <NavigationProvider>{content}</NavigationProvider>;
    } catch (e) {
      // If mocked or not available, skip this provider
    }

    return (
      <BrowserRouter>
        <QueryClientProvider client={queryClient}>
          {content}
        </QueryClientProvider>
      </BrowserRouter>
    );
  };

  Wrapper.displayName = 'TestFullWrapper';
  return Wrapper;
};
