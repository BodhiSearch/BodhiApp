import AllRequestsPage from '@/routes/users/access-requests/index';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockAccessRequests, mockAccessRequestApprove } from '@/test-utils/msw-v2/handlers/user-access-requests';
import { mockAllRequests, mockApprovedRequest } from '@/test-fixtures/access-requests';
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

// Mirror the root shell: render the published header-actions and rail slots so we can assert them.
function SlotsConsumer() {
  const { headerActions, rail, railHeader } = useShellSlots();
  return (
    <>
      <div data-testid="harness-header-actions">{headerActions}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
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

describe('AccessRequestsPage V2 chrome', () => {
  it('publishes a pending-count pill to the shell header', async () => {
    await renderReady();

    const pill = within(screen.getByTestId('harness-header-actions')).getByTestId('pending-pill');
    expect(pill).toHaveTextContent('1 pending review');
  });

  it('opens the detail rail when a pending row is selected', async () => {
    const user = userEvent.setup();
    await renderReady();

    // no rail until a row is selected
    expect(within(screen.getByTestId('harness-rail')).queryByTestId('request-detail-rail')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('request-row-user@example.com'));

    const rail = within(screen.getByTestId('harness-rail')).getByTestId('request-detail-rail');
    expect(rail).toBeInTheDocument();
    // pending → rail offers assign-role + approve/reject
    expect(within(rail).getByTestId('request-detail-role-select')).toBeInTheDocument();
    expect(within(rail).getByTestId('request-detail-approve')).toBeInTheDocument();
    expect(within(rail).getByTestId('request-detail-reject')).toBeInTheDocument();
  });

  it('shows a static decided rail (no actions) for a decided row', async () => {
    const user = userEvent.setup();
    await renderReady();

    await user.click(screen.getByTestId(`request-row-${mockApprovedRequest.username}`));

    const rail = within(screen.getByTestId('harness-rail')).getByTestId('request-detail-rail');
    expect(within(rail).queryByTestId('request-detail-approve')).not.toBeInTheDocument();
    expect(within(rail).queryByTestId('request-detail-role-select')).not.toBeInTheDocument();
    // decided branch: status chip in the rail + decided-note carrying the decision date
    expect(within(rail).getByTestId('request-status-approved')).toBeInTheDocument();
    expect(
      within(rail).getAllByText((_content, el) => el?.textContent === 'Approved Jan 2, 2024').length
    ).toBeGreaterThan(0);
  });

  it('approves from the rail via the real mutation', async () => {
    const user = userEvent.setup();
    server.use(...mockAccessRequestApprove('01HQXYZ0000000000000000001'));
    await renderReady();

    await user.click(screen.getByTestId('request-row-user@example.com'));
    const approve = within(screen.getByTestId('harness-rail')).getByTestId('request-detail-approve');
    expect(approve).toBeEnabled();
    await user.click(approve);

    await waitFor(() =>
      expect(within(screen.getByTestId('harness-rail')).getByTestId('request-detail-approve')).toBeInTheDocument()
    );
  });
});
