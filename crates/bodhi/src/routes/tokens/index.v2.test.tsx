import { TokenPage } from '@/routes/tokens/index';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockTokens } from '@/test-utils/msw-v2/handlers/tokens';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
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
    <ShellSlotsProvider>
      <SlotsConsumer />
      {children}
    </ShellSlotsProvider>
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

  it('opens the detail rail with real fields on row select', async () => {
    const user = userEvent.setup();
    await renderReady();

    await user.click(screen.getByTestId('token-row-token-1'));
    const rail = await screen.findByTestId('token-detail-rail');
    // prefix (Token ID) + scope + the created date are shown in the rail Details
    expect(within(rail).getByText('bodhiapp_prod001')).toBeInTheDocument();
    expect(within(rail).getByText('scope_token_power_user')).toBeInTheDocument();
  });
});
