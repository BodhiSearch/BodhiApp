import { TokenPage } from '@/routes/tokens/index';
import { ShellChromeProvider, useShellSlots } from '@/components/shell';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockTokens, mockUpdateTokenStatus } from '@/test-utils/msw-v2/handlers/tokens';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

setupMswV2();

const TOKENS = [
  {
    id: 'token-1',
    name: 'Production API',
    token_prefix: 'bodhiapp_prod001',
    scopes: 'scope_token_power_user',
    user_id: 'user-1',
    status: 'active' as const,
    created_at: '2024-01-03T10:00:00Z',
    updated_at: '2024-01-04T12:00:00Z',
  },
  {
    id: 'token-2',
    name: 'Dev API',
    token_prefix: 'bodhiapp_dev002',
    scopes: 'scope_token_user',
    user_id: 'user-1',
    status: 'inactive' as const,
    created_at: '2024-01-01T08:00:00Z',
    updated_at: '2024-01-02T09:00:00Z',
  },
];

beforeEach(() => {
  navigateMock.mockClear();
  server.use(
    ...mockAppInfo({ status: 'ready' }, { stub: true }),
    ...mockUserLoggedIn({}, { stub: true }),
    ...mockTokens({ data: TOKENS, total: 2 }, { stub: true })
  );
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

/** Mimics the root shell: renders the slots a screen publishes via useShellChrome so the
    header-action button + detail rail are present, exactly as __root's <AppShell> would. */
function SlotsConsumer() {
  const { headerActions, rail } = useShellSlots();
  return (
    <>
      <div data-testid="harness-header-actions">{headerActions}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

function ShellHarness({ children }: { children: React.ReactNode }) {
  return (
    <ShellChromeProvider>
      <SlotsConsumer />
      {children}
    </ShellChromeProvider>
  );
}

async function renderReady() {
  await act(async () => {
    render(
      <ShellHarness>
        <TokenPage />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('tokens-page')).toHaveAttribute('data-pagestatus', 'ready');
  });
}

describe('TokenPage V2', () => {
  it('shows shimmer badges and a body skeleton while the tokens query is pending', async () => {
    // hold the tokens list pending (app-info still resolves) so we can observe the loading window
    server.use(...mockTokens({ data: TOKENS, total: 2 }, { delayMs: 200, stub: true }));

    render(
      <ShellHarness>
        <TokenPage />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => {
      expect(screen.getByTestId('tokens-filter-all')).toBeInTheDocument();
    });
    expect(screen.getByTestId('tokens-page')).toHaveAttribute('data-pagestatus', 'loading');

    // category badges shimmer instead of showing (0); list body shows the skeleton, not the empty state
    expect(screen.getAllByLabelText('Loading count').length).toBeGreaterThan(0);
    expect(within(screen.getByTestId('tokens-filter-all')).queryByText('2')).not.toBeInTheDocument();
    expect(screen.getByTestId('loading-skeleton')).toBeInTheDocument();
    expect(screen.queryByTestId('tokens-empty')).not.toBeInTheDocument();

    // once loaded, the shimmer is replaced by real counts and the skeleton disappears
    await waitFor(() => {
      expect(screen.getByTestId('tokens-page')).toHaveAttribute('data-pagestatus', 'ready');
    });
    expect(screen.queryByLabelText('Loading count')).not.toBeInTheDocument();
    expect(within(screen.getByTestId('tokens-filter-all')).getByText('2')).toBeInTheDocument();
    expect(screen.queryByTestId('loading-skeleton')).not.toBeInTheDocument();
  });

  it('renders V2 rows with preserved testids and filter tabs', async () => {
    await renderReady();

    expect(screen.getByTestId('token-name-token-1')).toHaveTextContent('Production API');
    expect(screen.getByTestId('token-scope-token-1')).toHaveTextContent('scope_token_power_user');
    expect(screen.getByTestId('token-status-switch-token-1')).toBeInTheDocument();

    // filter-tab counts derived from the fetched page
    expect(within(screen.getByTestId('tokens-filter-all')).getByText('2')).toBeInTheDocument();
    expect(within(screen.getByTestId('tokens-filter-active')).getByText('1')).toBeInTheDocument();
    expect(within(screen.getByTestId('tokens-filter-inactive')).getByText('1')).toBeInTheDocument();
  });

  it('filters rows by status tab', async () => {
    const user = userEvent.setup();
    await renderReady();

    await user.click(screen.getByTestId('tokens-filter-active'));
    expect(screen.getByTestId('token-name-token-1')).toBeInTheDocument();
    expect(screen.queryByTestId('token-name-token-2')).not.toBeInTheDocument();
  });

  it('reveals the search input from the search button and filters rows', async () => {
    const user = userEvent.setup();
    await renderReady();

    // search is collapsed behind a button until clicked
    expect(screen.queryByPlaceholderText(/search tokens/i)).not.toBeInTheDocument();
    await user.click(screen.getByTestId('tokens-search-toggle'));

    await user.type(screen.getByPlaceholderText(/search tokens/i), 'dev');
    expect(screen.getByTestId('token-name-token-2')).toBeInTheDocument();
    expect(screen.queryByTestId('token-name-token-1')).not.toBeInTheDocument();
  });

  it('collapses the search on blur when empty, but stays open when it has text', async () => {
    const user = userEvent.setup();
    await renderReady();

    // open + type → blurring keeps it open (has text)
    await user.click(screen.getByTestId('tokens-search-toggle'));
    const input = screen.getByPlaceholderText(/search tokens/i);
    await user.type(input, 'prod');
    input.blur();
    expect(screen.getByPlaceholderText(/search tokens/i)).toBeInTheDocument();

    // clear it, then blur → collapses (removed)
    await user.clear(input);
    input.blur();
    await waitFor(() => {
      expect(screen.queryByPlaceholderText(/search tokens/i)).not.toBeInTheDocument();
    });
  });

  it('opens the detail rail with real fields on row select', async () => {
    const user = userEvent.setup();
    await renderReady();

    await user.click(screen.getByTestId('token-row-token-1'));
    const rail = await screen.findByTestId('token-detail-rail');
    // prefix (Token ID) + scope + the created date are shown in the rail Details
    expect(within(rail).getByText('bodhiapp_prod001')).toBeInTheDocument();
    expect(within(rail).getByText('scope_token_power_user')).toBeInTheDocument();
  });

  it('renders each row as an accessible link and activating it opens the rail', async () => {
    const user = userEvent.setup();
    await renderReady();

    const row = screen.getByTestId('token-row-token-1');
    const link = within(row).getByTestId('row-link');
    expect(link.tagName).toBe('A');
    expect(link).toHaveAccessibleName('Open token Production API');

    await user.click(link);
    expect(await screen.findByTestId('token-detail-rail')).toBeInTheDocument();
  });

  it('toggling the status switch does not open the rail (control stays above the link)', async () => {
    server.use(...mockUpdateTokenStatus('token-1', 'inactive'));
    const user = userEvent.setup();
    await renderReady();

    await user.click(screen.getByTestId('token-status-switch-token-1'));
    expect(screen.queryByTestId('token-detail-rail')).not.toBeInTheDocument();
  });
});

// Auth/init behavior is render-agnostic. The create flow lives in new/index.test.tsx.
describe('TokenPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(
        <ShellChromeProvider>
          <TokenPage />
        </ShellChromeProvider>,
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
        <ShellChromeProvider>
          <TokenPage />
        </ShellChromeProvider>,
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
        <ShellChromeProvider>
          <TokenPage />
        </ShellChromeProvider>,
        { wrapper: createWrapper() }
      );
    });

    await waitFor(() => {
      expect(screen.getByTestId('tokens-page')).toHaveAttribute('data-pagestatus', 'ready');
    });
  });
});
