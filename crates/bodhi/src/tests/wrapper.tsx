import { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from 'react-query';

// Create a simple wrapper that only includes essential providers for Next.js
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

// Enhanced wrapper for testing routing behavior (simplified for Next.js)
export interface RouterTestOptions {
  initialPath?: string;
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

  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  Wrapper.displayName = 'TestRouterWrapper';
  return { Wrapper };
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
      <QueryClientProvider client={queryClient}>{content}</QueryClientProvider>
    );
  };

  Wrapper.displayName = 'TestFullWrapper';
  return Wrapper;
};
