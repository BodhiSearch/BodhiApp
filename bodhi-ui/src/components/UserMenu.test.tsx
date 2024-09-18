'use client';

import { render, screen, waitFor } from '@testing-library/react';
import {
  beforeAll,
  afterAll,
  beforeEach,
  describe,
  it,
  expect,
  vi,
} from 'vitest';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import UserMenu from './UserMenu';
import { QueryClient, QueryClientProvider } from 'react-query';
import { ReactNode } from 'react';
import { ToastProvider } from "@/components/ui/toast";
import userEvent from '@testing-library/user-event';

// Mock the Next.js router
const routerPushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: routerPushMock,
  }),
}));

// Mock the useToast hook
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: vi.fn(),
  }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => server.resetHandlers());

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnMount: false,
      },
    },
  });
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>{children}</ToastProvider>
    </QueryClientProvider>
  );
};

describe('UserMenu', () => {
  it('renders user email', () => {
    const wrapper = createWrapper();
    render(<UserMenu />, { wrapper });
    expect(screen.getByText('user@example.com')).toBeInTheDocument();
  });

  it('handles successful logout', async () => {
    const user = userEvent.setup();
    const wrapper = createWrapper();
    server.use(
      rest.post('*/app/logout', (req, res, ctx) => {
        return res(
          ctx.set('location', '/ui/test/home'),
          ctx.status(302)
        );
      })
    );

    render(<UserMenu />, { wrapper });

    const userEmail = screen.getByText('user@example.com');
    await user.click(userEmail);

    const logoutButton = await screen.findByText('Logout');
    await user.click(logoutButton);
    expect(routerPushMock).toHaveBeenCalledWith('/ui/test/home');
  });
});
