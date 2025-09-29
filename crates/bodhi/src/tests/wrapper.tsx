import { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@/hooks/useQuery';

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

/**
 * Mock window.location for tests
 * @param href - The URL to mock as window.location.href
 */
export const mockWindowLocation = (href: string) => {
  const url = new URL(href);
  let currentHref = href;

  Object.defineProperty(window, 'location', {
    value: {
      get href() {
        return currentHref;
      },
      set href(newHref: string) {
        currentHref = newHref;
      },
      protocol: url.protocol,
      host: url.host,
    } as any, // eslint-disable-line @typescript-eslint/no-explicit-any
    writable: true,
    configurable: true,
  });
};
