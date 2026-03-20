'use client';

import HomePage from '@/app/page';
import { ROUTE_DEFAULT, ROUTE_RESOURCE_ADMIN } from '@/lib/constants';
import { mockAppInfoReady, mockAppInfoResourceAdmin, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { render, waitFor } from '@testing-library/react';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

// Add this configuration before starting the server
beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('HomePage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('redirects to /setup when status is setup', async () => {
    server.use(...mockAppInfoSetup());

    render(<HomePage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/setup');
    });
  });

  it(`redirects to ${ROUTE_DEFAULT} when status is ready`, async () => {
    server.use(...mockAppInfoReady());

    render(<HomePage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
    });
  });

  it(`redirects to ${ROUTE_RESOURCE_ADMIN} when status is resource-admin`, async () => {
    server.use(...mockAppInfoResourceAdmin());

    render(<HomePage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_RESOURCE_ADMIN);
    });
  });
});
