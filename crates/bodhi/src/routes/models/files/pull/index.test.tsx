import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import PullPage from '@/routes/models/files/pull/index';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfo, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import {
  mockModelPullDownloadsDefault,
  mockModelPullDownloadsInternalError,
} from '@/test-utils/msw-v2/handlers/modelfiles';

vi.mock('@/routes/models/files/pull/-components/PullForm', () => ({
  PullForm: () => <div data-testid="pull-form">Pull Form</div>,
}));

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

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
beforeEach(() => {
  vi.resetAllMocks();
  navigateMock.mockClear();
});

describe('PullPage', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockModelPullDownloadsDefault()
    );
  });

  it('renders pull form and downloads table with error details in expanded row', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('pull-form')).toBeInTheDocument();

    expect(screen.getByText('test/repo1')).toBeInTheDocument();
    expect(screen.getByText('model1.gguf')).toBeInTheDocument();
    expect(screen.getByText('pending')).toBeInTheDocument();

    expect(screen.getByText('test/repo2')).toBeInTheDocument();
    expect(screen.getByText('model2.gguf')).toBeInTheDocument();
    expect(screen.getByText('completed')).toBeInTheDocument();

    expect(screen.getByText('test/repo3')).toBeInTheDocument();
    expect(screen.getByText('model3.gguf')).toBeInTheDocument();
    expect(screen.getByText('error')).toBeInTheDocument();

    // Error detail is collapsed until the row is expanded.
    expect(screen.queryByText('Error:')).not.toBeInTheDocument();
    expect(screen.queryByText('Download failed')).not.toBeInTheDocument();

    const rows = screen.getAllByRole('row');
    const errorRow = rows.find((row) => row.textContent?.includes('test/repo3'));
    const expandButton = errorRow?.querySelector('button');
    expect(expandButton).toBeInTheDocument();
    await user.click(expandButton!);

    expect(screen.getByText('Error:')).toBeInTheDocument();
    expect(screen.getByText('Download failed')).toBeInTheDocument();
  });

  it('displays progress information for pending downloads', async () => {
    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('50.0%')).toBeInTheDocument();
    expect(screen.getByText('488.3 KB / 976.6 KB')).toBeInTheDocument();

    const progressCells = screen.getAllByText('-');
    expect(progressCells).toHaveLength(2); // completed and error downloads
  });

  it('handles API error', async () => {
    server.use(...mockModelPullDownloadsInternalError());

    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Error loading downloads')).toBeInTheDocument();
  });
});

describe('PullPage access control', () => {
  it('should redirect to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn({ role: 'resource_user' }));
    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/' });
  });

  it('should redirect to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }), ...mockUserLoggedOut());
    await act(async () => {
      render(<PullPage />, { wrapper: createWrapper() });
    });
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
  });
});
