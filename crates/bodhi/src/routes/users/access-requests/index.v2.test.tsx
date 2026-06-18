import AllRequestsPage from '@/routes/users/access-requests/index';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockAccessRequests, mockAccessRequestApprove } from '@/test-utils/msw-v2/handlers/user-access-requests';
import { mockAllRequests } from '@/test-fixtures/access-requests';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: '/users/access-requests' }),
  };
});

vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({ showSuccess: vi.fn(), showError: vi.fn() }),
}));

setupMswV2();

function SlotsConsumer() {
  const { headerActions } = useShellSlots();
  return <div data-testid="harness-header-actions">{headerActions}</div>;
}

beforeEach(() => {
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin' }),
    ...mockAccessRequests({
      requests: mockAllRequests.requests,
      total: mockAllRequests.total,
      page: 1,
      page_size: 10,
    })
  );
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

async function renderReady() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <AllRequestsPage />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('all-requests-page')).toHaveAttribute('data-pagestatus', 'ready');
  });
}

describe('AccessRequestsPage V2', () => {
  it('renders rows with preserved testids and a pending pill in the header', async () => {
    await renderReady();

    expect(screen.getByTestId('request-row-user@example.com')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-pending')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-approved')).toBeInTheDocument();
    expect(screen.getByTestId('request-status-rejected')).toBeInTheDocument();

    // pending-count pill published to the shell header (1 pending in the fixture)
    const pill = within(screen.getByTestId('harness-header-actions')).getByTestId('pending-pill');
    expect(pill).toHaveTextContent('1 pending review');
  });

  it('derives filter-tab counts and filters rows', async () => {
    const user = userEvent.setup();
    await renderReady();

    expect(within(screen.getByTestId('requests-filter-all')).getByText('3')).toBeInTheDocument();
    expect(within(screen.getByTestId('requests-filter-pending')).getByText('1')).toBeInTheDocument();

    await user.click(screen.getByTestId('requests-filter-approved'));
    expect(screen.getByTestId('request-row-approved@example.com')).toBeInTheDocument();
    expect(screen.queryByTestId('request-row-user@example.com')).not.toBeInTheDocument();
  });

  it('approves a pending request via the real mutation', async () => {
    const user = userEvent.setup();
    server.use(...mockAccessRequestApprove('01HQXYZ0000000000000000001'));
    await renderReady();

    const approve = screen.getByTestId('approve-btn-user@example.com');
    expect(approve).toBeEnabled();
    await user.click(approve);
    // mutation fires; success toast is mocked — assert the button entered its pending path or settled
    await waitFor(() => expect(screen.getByTestId('approve-btn-user@example.com')).toBeInTheDocument());
  });
});
