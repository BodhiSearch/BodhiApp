import { act, render, screen } from '@testing-library/react';
import { rest } from 'msw';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { setupServer } from 'msw/node';
import { createWrapper } from '@/tests/wrapper';
import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import ModelDownloadPage from '@/app/ui/setup/download-models/page';

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, className }: any) => <div className={className}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const server = setupServer();
beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
});

describe('ModelDownloadPage', () => {
  it('should render the page when app is ready without auth', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            authz: false,
            status: 'ready',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: false,
          })
        );
      })
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('Recommended Models')).toBeInTheDocument();
    expect(screen.getByText('Additional Models')).toBeInTheDocument();
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should redirect to /ui/setup if app status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            status: 'setup',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: false,
          })
        );
      })
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/setup');
  });

  it('should render the page when app is ready with auth and user is logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            authz: true,
            status: 'ready',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: true,
            email: 'user@email.com',
            roles: [],
          })
        );
      })
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('Recommended Models')).toBeInTheDocument();
    expect(screen.getByText('Additional Models')).toBeInTheDocument();
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should redirect to /ui/login when app is ready with auth but user is not logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            version: '0.1.0',
            authz: true,
            status: 'ready',
          })
        );
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(
          ctx.json({
            logged_in: false,
            email: null,
            roles: [],
          })
        );
      })
    );

    await act(async () => {
      render(<ModelDownloadPage />, { wrapper: createWrapper() });
    });
    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
}); 