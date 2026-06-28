import { Route as NewTokenRoute } from '@/routes/tokens/new/index';
import { ShellHarness } from '@/test-utils/shell-harness';
import { showSuccessParams } from '@/lib/utils.test';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockCreateToken } from '@/test-utils/msw-v2/handlers/tokens';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const NewTokenPage = NewTokenRoute.options.component!;

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

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: toastMock }),
}));

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
  toastMock.mockClear();
  server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

function renderPage() {
  return act(async () => {
    render(
      <ShellHarness>
        <NewTokenPage />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );
  });
}

describe('NewTokenPage', () => {
  it('renders the create form on a full page', async () => {
    await renderPage();
    await waitFor(() => {
      expect(screen.getByTestId('new-token-page')).toBeInTheDocument();
    });
    expect(screen.getByTestId('token-form')).toBeInTheDocument();
    expect(screen.getByTestId('token-name-input')).toBeInTheDocument();
    expect(screen.getByTestId('generate-token-button')).toBeInTheDocument();
  });

  it('creates a token via the real mutation, shows the reveal dialog, and returns to the list on done', async () => {
    const user = userEvent.setup();
    const createdToken = 'bodhiapp_abc123def456';
    server.use(...mockCreateToken({ token: createdToken }));

    await renderPage();
    await waitFor(() => expect(screen.getByTestId('token-form')).toBeInTheDocument());

    await user.type(screen.getByTestId('token-name-input'), 'My Token');
    await user.click(screen.getByTestId('generate-token-button'));

    await waitFor(() => expect(screen.getByTestId('token-dialog')).toBeInTheDocument());
    expect(toastMock).toHaveBeenCalledWith(showSuccessParams('Success', 'API token successfully generated'));

    await user.click(screen.getByTestId('token-dialog-done'));
    expect(navigateMock).toHaveBeenCalledWith({ to: '/tokens/' });
  });
});
