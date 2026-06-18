import { TokenPage } from '@/routes/tokens/index';
import { ShellSlotsProvider } from '@/components/shell';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockTokens } from '@/test-utils/msw-v2/handlers/tokens';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
  };
});

vi.mock('@/hooks/use-toast', () => ({ useToast: () => ({ toast: vi.fn() }) }));

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
});

afterEach(() => {
  vi.resetAllMocks();
});

// Auth/init behavior is render-agnostic. The V2 list render + interactions are
// covered in index.v2.test.tsx; the create flow lives in new/index.test.tsx.
describe('TokenPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <TokenPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/' });
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <TokenPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
    });
  });

  it('renders the tokens page when ready and logged in', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      ...mockTokens({ data: [], total: 0 }, { stub: true })
    );

    await act(async () => {
      render(
        <ShellSlotsProvider>
          <TokenPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });

    await waitFor(() => {
      expect(screen.getByTestId('tokens-page')).toHaveAttribute('data-pagestatus', 'ready');
    });
  });
});
