import RootPage from '@/routes/index';
import { ROUTE_DEFAULT, ROUTE_RESOURCE_ADMIN } from '@/lib/constants';
import { mockAppInfoReady, mockAppInfoResourceAdmin, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { render, waitFor } from '@testing-library/react';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

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
    useSearch: () => ({}),
  };
});

// Add this configuration before starting the server
beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('HomePage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    navigateMock.mockClear();
  });

  it('redirects to /setup when status is setup', async () => {
    server.use(...mockAppInfoSetup());

    render(<RootPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup' });
    });
  });

  it(`redirects to ${ROUTE_DEFAULT} when status is ready`, async () => {
    server.use(...mockAppInfoReady());

    render(<RootPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: ROUTE_DEFAULT });
    });
  });

  it(`redirects to ${ROUTE_RESOURCE_ADMIN} when status is resource-admin`, async () => {
    server.use(...mockAppInfoResourceAdmin());

    render(<RootPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: ROUTE_RESOURCE_ADMIN });
    });
  });
});
