import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import ReviewAccessRequestPage from '@/app/ui/apps/access-requests/review/page';
import {
  MOCK_REQUEST_ID,
  mockApprovedReviewResponse,
  mockDeniedReviewResponse,
  mockDraftMcpNoInstancesResponse,
  mockDraftMcpResponse,
  mockDraftMixedResourcesResponse,
  mockDraftMultiToolMixedResponse,
  mockDraftMultiToolResponse,
  mockDraftNoInstancesResponse,
  mockDraftRedirectResponse,
  mockDraftReviewResponse,
  mockDraftReviewResponsePowerUser,
  mockExpiredReviewResponse,
  mockFailedReviewResponse,
} from '@/test-fixtures/app-access-requests';
import {
  mockAppAccessRequestApprove,
  mockAppAccessRequestApproveError,
  mockAppAccessRequestDeny,
  mockAppAccessRequestDenyError,
  mockAppAccessRequestReview,
  mockAppAccessRequestReviewError,
} from '@/test-utils/msw-v2/handlers/app-access-requests';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

// ============================================================================
// Mocks
// ============================================================================

const pushMock = vi.fn();
let mockSearchParams: URLSearchParams | null = null;

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => mockSearchParams,
  usePathname: vi.fn().mockReturnValue('/ui/apps/access-requests/review'),
}));

vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({
    showSuccess: vi.fn(),
    showError: vi.fn(),
  }),
}));

const windowCloseMock = vi.fn();
const MOCK_REDIRECT_URL = 'https://example.com/callback?code=auth_code';
let originalLocationDescriptor: PropertyDescriptor | undefined;

// ============================================================================
// Setup
// ============================================================================

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
  windowCloseMock.mockClear();
  mockSearchParams = null;
  // Restore window.location if it was mocked
  if (originalLocationDescriptor) {
    Object.defineProperty(window, 'location', originalLocationDescriptor);
    originalLocationDescriptor = undefined;
  }
  // Reset window.close
  vi.restoreAllMocks();
});

const setupWindowClose = () => {
  window.close = windowCloseMock;
};

const setupWindowLocation = () => {
  originalLocationDescriptor = Object.getOwnPropertyDescriptor(window, 'location');
  const loc = window.location;
  Object.defineProperty(window, 'location', {
    value: {
      href: loc.href,
      origin: loc.origin,
      protocol: loc.protocol,
      host: loc.host,
      hostname: loc.hostname,
      port: loc.port,
      pathname: loc.pathname,
      search: loc.search,
      hash: loc.hash,
      assign: vi.fn(),
      replace: vi.fn(),
      reload: vi.fn(),
      toString: () => loc.href,
    },
    writable: true,
    configurable: true,
  });
};

const setupHandlers = (reviewData?: Parameters<typeof mockAppAccessRequestReview>[0]) => {
  const handlers = [...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' })];
  if (reviewData) {
    handlers.push(...mockAppAccessRequestReview(reviewData));
  }
  server.use(...handlers);
};

// ============================================================================
// Loading & Error States
// ============================================================================

describe('ReviewAccessRequestPage - Loading & Error States', () => {
  it('shows error page when no id query param', async () => {
    mockSearchParams = new URLSearchParams();
    setupHandlers();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Missing access request ID')).toBeInTheDocument();
    });
  });

  it('shows loading skeleton while fetching review data', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    // Set up handlers but with a delay by not providing review data yet
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }));

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    // Should show loading state
    await waitFor(() => {
      expect(screen.getByTestId('review-access-loading')).toBeInTheDocument();
    });
  });

  it('shows error page when API returns 404', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReviewError(MOCK_REQUEST_ID, {
        status: 404,
        code: 'not_found',
        message: 'Access request not found',
        type: 'not_found_error',
      })
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-access-error')).toBeInTheDocument();
      expect(screen.getByText('Access request not found')).toBeInTheDocument();
    });
  });

  it('shows error page when API returns 500', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReviewError(MOCK_REQUEST_ID, {
        status: 500,
        code: 'internal_error',
        message: 'Internal server error',
        type: 'internal_server_error',
      })
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-access-error')).toBeInTheDocument();
    });
  });
});

// ============================================================================
// Draft Review Form
// ============================================================================

describe('ReviewAccessRequestPage - Draft Review Form', () => {
  it('renders app name and description from review data', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-access-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-app-name')).toHaveTextContent('Test Application');
    expect(screen.getByTestId('review-app-description')).toHaveTextContent('A test third-party application');
  });

  it('renders tool type cards with correct names and descriptions', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-builtin-exa-search')).toBeInTheDocument();
    });

    expect(screen.getByText('Exa Web Search')).toBeInTheDocument();
    expect(screen.getByText('Search the web using Exa AI')).toBeInTheDocument();
  });

  it('renders instance select dropdowns with available instances', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-instance-select-builtin-exa-search')).toBeInTheDocument();
    });
  });

  it('shows "No instances configured" when tool type has no instances', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftNoInstancesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-no-instances-builtin-exa-search')).toBeInTheDocument();
    });

    expect(screen.getByText(/No instances configured/)).toBeInTheDocument();
  });

  it('Approve button disabled when no valid instances exist', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftNoInstancesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();
  });

  it('Approve button disabled until all tool types have instance selected', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Initially disabled because no instance is selected
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();
  });

  it('Approve button becomes enabled after selecting valid instance', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Initially disabled
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Click the select trigger to open dropdown
    const selectTrigger = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(selectTrigger);

    // Select the valid instance
    const option = await screen.findByText('my-exa-instance');
    await user.click(option);

    // Now approve button should be enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });
});

// ============================================================================
// Approve Flow
// ============================================================================

describe('ReviewAccessRequestPage - Approve Flow', () => {
  it('clicking Approve calls PUT with correct body', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponse),
      ...mockAppAccessRequestApprove(MOCK_REQUEST_ID)
    );
    setupWindowClose();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Select instance
    const selectTrigger = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(selectTrigger);
    const option = await screen.findByText('my-exa-instance');
    await user.click(option);

    // Click approve
    const approveButton = screen.getByTestId('review-approve-button');
    await waitFor(() => {
      expect(approveButton).not.toBeDisabled();
    });
    await user.click(approveButton);

    // Should call window.close for popup flow
    await waitFor(() => {
      expect(windowCloseMock).toHaveBeenCalled();
    });
  });

  it('clicking Approve on redirect flow redirects using window.location', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftRedirectResponse),
      ...mockAppAccessRequestApprove(MOCK_REQUEST_ID, { flowType: 'redirect', redirectUrl: MOCK_REDIRECT_URL })
    );
    setupWindowLocation();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Select instance
    const selectTrigger = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(selectTrigger);
    const option = await screen.findByText('my-exa-instance');
    await user.click(option);

    // Click approve
    const approveButton = screen.getByTestId('review-approve-button');
    await waitFor(() => {
      expect(approveButton).not.toBeDisabled();
    });
    await user.click(approveButton);

    // Should redirect using window.location.href
    await waitFor(() => {
      expect(window.location.href).toBe(MOCK_REDIRECT_URL);
    });
  });

  it('on approve error, shows toast error message', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponse),
      ...mockAppAccessRequestApproveError(MOCK_REQUEST_ID, {
        status: 400,
        code: 'bad_request',
        message: 'Invalid approval',
        type: 'bad_request_error',
      })
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Select instance
    const selectTrigger = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(selectTrigger);
    const option = await screen.findByText('my-exa-instance');
    await user.click(option);

    // Click approve
    const approveButton = screen.getByTestId('review-approve-button');
    await waitFor(() => {
      expect(approveButton).not.toBeDisabled();
    });
    await user.click(approveButton);

    // Should not close window on error
    await waitFor(() => {
      // Approve button should be re-enabled after error
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });
});

// ============================================================================
// Deny Flow
// ============================================================================

describe('ReviewAccessRequestPage - Deny Flow', () => {
  it('clicking Deny calls POST to deny endpoint', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponse),
      ...mockAppAccessRequestDeny(MOCK_REQUEST_ID)
    );
    setupWindowClose();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-deny-button')).toBeInTheDocument();
    });

    const denyButton = screen.getByTestId('review-deny-button');
    await user.click(denyButton);

    // Should call window.close for popup flow
    await waitFor(() => {
      expect(windowCloseMock).toHaveBeenCalled();
    });
  });

  it('clicking Deny on redirect flow redirects using window.location', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftRedirectResponse),
      ...mockAppAccessRequestDeny(MOCK_REQUEST_ID, { flowType: 'redirect', redirectUrl: MOCK_REDIRECT_URL })
    );
    setupWindowLocation();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-deny-button')).toBeInTheDocument();
    });

    const denyButton = screen.getByTestId('review-deny-button');
    await user.click(denyButton);

    // Should redirect using window.location.href
    await waitFor(() => {
      expect(window.location.href).toBe(MOCK_REDIRECT_URL);
    });
  });

  it('on deny error, shows toast error message', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponse),
      ...mockAppAccessRequestDenyError(MOCK_REQUEST_ID, {
        status: 409,
        code: 'conflict',
        message: 'Already processed',
        type: 'conflict_error',
      })
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-deny-button')).toBeInTheDocument();
    });

    const denyButton = screen.getByTestId('review-deny-button');
    await user.click(denyButton);

    // Should not close window on error, and button should be re-enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-deny-button')).not.toBeDisabled();
    });
  });
});

// ============================================================================
// Non-Draft States
// ============================================================================

describe('ReviewAccessRequestPage - Non-Draft States', () => {
  it('approved status with popup flow calls window.close', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockApprovedReviewResponse)
    );
    setupWindowClose();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(windowCloseMock).toHaveBeenCalled();
    });
  });

  it('denied status with redirect flow shows status', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDeniedReviewResponse)
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-status-denied')).toBeInTheDocument();
    });
    expect(screen.getByText('Denied')).toBeInTheDocument();
  });

  it('expired status with redirect flow shows status', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockExpiredReviewResponse)
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-status-expired')).toBeInTheDocument();
    });
    expect(screen.getByText('Expired')).toBeInTheDocument();
  });
});

// ============================================================================
// Multi-Tool Type Support
// ============================================================================

describe('ReviewAccessRequestPage - Multi-Tool Types', () => {
  it('renders multiple tool type cards', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMultiToolResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-builtin-exa-search')).toBeInTheDocument();
      expect(screen.getByTestId('review-tool-builtin-weather')).toBeInTheDocument();
    });

    expect(screen.getByText('Exa Web Search')).toBeInTheDocument();
    expect(screen.getByText('Weather Lookup')).toBeInTheDocument();
  });
});

// ============================================================================
// Partial Approve
// ============================================================================

describe('ReviewAccessRequestPage - Partial Approve', () => {
  it('checkbox renders checked by default for each tool type', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-checkbox-builtin-exa-search')).toBeInTheDocument();
    });

    const checkbox = screen.getByTestId('review-tool-checkbox-builtin-exa-search');
    expect(checkbox).toHaveAttribute('data-state', 'checked');
  });

  it('unchecking checkbox grays out card content', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-checkbox-builtin-exa-search')).toBeInTheDocument();
    });

    const checkbox = screen.getByTestId('review-tool-checkbox-builtin-exa-search');
    await user.click(checkbox);

    await waitFor(() => {
      expect(checkbox).toHaveAttribute('data-state', 'unchecked');
    });

    // The content area should have opacity-50 class
    const card = screen.getByTestId('review-tool-builtin-exa-search');
    const contentDiv = card.querySelector('.opacity-50');
    expect(contentDiv).toBeInTheDocument();
  });

  it('unchecking checkbox enables Approve without instance selection', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Initially disabled (no instance selected, checkbox checked)
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Uncheck the checkbox to deny the tool
    const checkbox = screen.getByTestId('review-tool-checkbox-builtin-exa-search');
    await user.click(checkbox);

    // Now approve should be enabled (denied tool skips validation)
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('instance selection preserved across checkbox toggle', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-instance-select-builtin-exa-search')).toBeInTheDocument();
    });

    // Select an instance
    const selectTrigger = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(selectTrigger);
    const option = await screen.findByText('my-exa-instance');
    await user.click(option);

    // Verify instance is selected
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });

    // Uncheck checkbox
    const checkbox = screen.getByTestId('review-tool-checkbox-builtin-exa-search');
    await user.click(checkbox);

    await waitFor(() => {
      expect(checkbox).toHaveAttribute('data-state', 'unchecked');
    });

    // Re-check checkbox
    await user.click(checkbox);

    await waitFor(() => {
      expect(checkbox).toHaveAttribute('data-state', 'checked');
    });

    // Instance should still be selected -- approve button still enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('no-instances tool: checked by default, blocks Approve, unchecking enables Approve', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftNoInstancesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Approve disabled (no instances, checkbox checked by default)
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Checkbox should be checked by default
    const checkbox = screen.getByTestId('review-tool-checkbox-builtin-exa-search');
    expect(checkbox).toHaveAttribute('data-state', 'checked');

    // Uncheck to deny
    await user.click(checkbox);

    // Now approve should be enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('multi-tool partial: approve one tool, deny another via checkbox', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });

    let capturedBody: unknown = null;
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftMultiToolResponse),
      ...mockAppAccessRequestApprove(MOCK_REQUEST_ID, {
        onBody: (body) => {
          capturedBody = body;
        },
      })
    );
    setupWindowClose();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-builtin-exa-search')).toBeInTheDocument();
      expect(screen.getByTestId('review-tool-builtin-weather')).toBeInTheDocument();
    });

    // Select instance for exa-search
    const exaSelect = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(exaSelect);
    const exaOption = await screen.findByText('my-exa-instance');
    await user.click(exaOption);

    // Uncheck weather tool to deny it
    const weatherCheckbox = screen.getByTestId('review-tool-checkbox-builtin-weather');
    await user.click(weatherCheckbox);

    // Approve should be enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });

    // Click approve
    await user.click(screen.getByTestId('review-approve-button'));

    // Verify the body was captured
    await waitFor(() => {
      expect(capturedBody).not.toBeNull();
    });

    // Verify body structure
    const body = capturedBody as {
      approved: {
        toolsets: Array<{ toolset_type: string; status: string; instance?: { id: string } }>;
        mcps: Array<{ url: string; status: string; instance?: { id: string } }>;
      };
    };
    expect(body.approved.toolsets).toHaveLength(2);
    expect(body.approved.mcps).toHaveLength(0);

    const exaApproval = body.approved.toolsets.find((t) => t.toolset_type === 'builtin-exa-search');
    expect(exaApproval?.status).toBe('approved');
    expect(exaApproval?.instance?.id).toBe('instance-1');

    const weatherApproval = body.approved.toolsets.find((t) => t.toolset_type === 'builtin-weather');
    expect(weatherApproval?.status).toBe('denied');
    expect(weatherApproval?.instance).toBeUndefined();
  });

  it('multi-tool mixed: one with instances, one without -- uncheck no-instances tool to enable Approve', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMultiToolMixedResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-builtin-exa-search')).toBeInTheDocument();
      expect(screen.getByTestId('review-tool-builtin-calculator')).toBeInTheDocument();
    });

    // Approve initially disabled (calculator has no instances)
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Uncheck calculator (no instances, must deny)
    const calcCheckbox = screen.getByTestId('review-tool-checkbox-builtin-calculator');
    await user.click(calcCheckbox);

    // Still disabled -- exa-search has no instance selected yet
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Select instance for exa-search
    const exaSelect = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(exaSelect);
    const exaOption = await screen.findByText('my-exa-instance');
    await user.click(exaOption);

    // Now approve should be enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('button shows "Approve All" when all checkboxes checked and instances selected', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    // Select instance
    const selectTrigger = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(selectTrigger);
    const option = await screen.findByText('my-exa-instance');
    await user.click(option);

    // Button should say "Approve All"
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toHaveTextContent('Approve All');
    });
  });

  it('button shows "Approve Selected" when some checkboxes unchecked', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMultiToolResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-builtin-weather')).toBeInTheDocument();
    });

    // Uncheck one tool
    const weatherCheckbox = screen.getByTestId('review-tool-checkbox-builtin-weather');
    await user.click(weatherCheckbox);

    // Button should say "Approve Selected"
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toHaveTextContent('Approve Selected');
    });
  });
});

// ============================================================================
// MCP Server Review
// ============================================================================

describe('ReviewAccessRequestPage - MCP Server Review', () => {
  it('renders MCP server card with URL badge', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMcpResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });

    expect(screen.getByText('https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    expect(screen.getByText('MCP Server')).toBeInTheDocument();
  });

  it('shows instance select for MCP when approved', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMcpResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });
  });

  it('Approve button disabled until MCP instance is selected', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMcpResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();
  });

  it('selecting MCP instance enables Approve button', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMcpResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });

    const selectTrigger = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(selectTrigger);
    const option = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(option);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('shows "No MCP instances" alert when no instances available', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMcpNoInstancesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-no-mcp-instances-https://mcp.example.com/mcp')).toBeInTheDocument();
    });

    expect(screen.getByText(/No MCP instances connected/)).toBeInTheDocument();
  });

  it('unchecking MCP checkbox enables Approve without instance selection', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMcpResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    const checkbox = screen.getByTestId('review-mcp-toggle-https://mcp.deepwiki.com/mcp');
    await user.click(checkbox);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('approve with MCP sends correct body', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });

    let capturedBody: unknown = null;
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftMcpResponse),
      ...mockAppAccessRequestApprove(MOCK_REQUEST_ID, {
        onBody: (body) => {
          capturedBody = body;
        },
      })
    );
    setupWindowClose();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });

    const selectTrigger = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(selectTrigger);
    const option = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(option);

    const approveButton = screen.getByTestId('review-approve-button');
    await waitFor(() => {
      expect(approveButton).not.toBeDisabled();
    });
    await user.click(approveButton);

    await waitFor(() => {
      expect(capturedBody).not.toBeNull();
    });

    const body = capturedBody as {
      approved: {
        toolsets: Array<{ toolset_type: string; status: string; instance?: { id: string } }>;
        mcps: Array<{ url: string; status: string; instance?: { id: string } }>;
      };
    };
    expect(body.approved.toolsets).toHaveLength(0);
    expect(body.approved.mcps).toHaveLength(1);
    expect(body.approved.mcps[0].url).toBe('https://mcp.deepwiki.com/mcp');
    expect(body.approved.mcps[0].status).toBe('approved');
    expect(body.approved.mcps[0].instance?.id).toBe('mcp-instance-1');
  });
});

// ============================================================================
// Mixed Resources (Tools + MCPs)
// ============================================================================

describe('ReviewAccessRequestPage - Mixed Resources', () => {
  it('renders both tool cards and MCP cards', async () => {
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMixedResourcesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-tool-builtin-exa-search')).toBeInTheDocument();
      expect(screen.getByTestId('review-mcp-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });

    expect(screen.getByText('Requested Tools:')).toBeInTheDocument();
    expect(screen.getByText('Requested MCP Servers:')).toBeInTheDocument();
  });

  it('Approve button requires selections for both tools and MCPs', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftMixedResourcesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Select tool instance only
    const toolSelect = screen.getByTestId('review-instance-select-builtin-exa-search');
    await user.click(toolSelect);
    const toolOption = await screen.findByText('my-exa-instance');
    await user.click(toolOption);

    // Still disabled -- MCP instance not selected
    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    // Select MCP instance
    const mcpSelect = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(mcpSelect);
    const mcpOption = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(mcpOption);

    // Now approve should be enabled
    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });
});

// ============================================================================
// Role Selection Dropdown
// ============================================================================

describe('ReviewAccessRequestPage - Role Selection Dropdown', () => {
  it('shows 2 role options when resource_power_user approves scope_user_power_user request', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_power_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponsePowerUser)
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approved-role-section')).toBeInTheDocument();
    });

    const selectTrigger = screen.getByTestId('review-approved-role-select');
    await user.click(selectTrigger);

    await screen.findByTestId('review-approved-role-option-scope_user_power_user');
    expect(screen.getByTestId('review-approved-role-option-scope_user_user')).toBeInTheDocument();
  });

  it('shows only scope_user_user option when resource_user approves scope_user_power_user request', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponsePowerUser)
    );

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approved-role-section')).toBeInTheDocument();
    });

    const selectTrigger = screen.getByTestId('review-approved-role-select');
    await user.click(selectTrigger);

    await screen.findByTestId('review-approved-role-option-scope_user_user');
    expect(screen.queryByTestId('review-approved-role-option-scope_user_power_user')).not.toBeInTheDocument();
  });

  it('shows only scope_user_user option when requested_role is scope_user_user', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approved-role-section')).toBeInTheDocument();
    });

    const selectTrigger = screen.getByTestId('review-approved-role-select');
    await user.click(selectTrigger);

    await screen.findByTestId('review-approved-role-option-scope_user_user');
    expect(screen.queryByTestId('review-approved-role-option-scope_user_power_user')).not.toBeInTheDocument();
  });

  it('approve sends downgraded approved_role when user selects scope_user_user', async () => {
    const user = userEvent.setup();
    mockSearchParams = new URLSearchParams({ id: MOCK_REQUEST_ID });

    let capturedBody: unknown = null;
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_power_user' }),
      ...mockAppAccessRequestReview(mockDraftReviewResponsePowerUser),
      ...mockAppAccessRequestApprove(MOCK_REQUEST_ID, {
        onBody: (body) => {
          capturedBody = body;
        },
      })
    );
    setupWindowClose();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approved-role-section')).toBeInTheDocument();
    });

    // Open role dropdown and select scope_user_user (downgrade)
    const roleSelect = screen.getByTestId('review-approved-role-select');
    await user.click(roleSelect);
    const userRoleOption = await screen.findByTestId('review-approved-role-option-scope_user_user');
    await user.click(userRoleOption);

    // Select MCP instance
    const mcpSelect = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(mcpSelect);
    const mcpOption = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(mcpOption);

    const approveButton = screen.getByTestId('review-approve-button');
    await waitFor(() => {
      expect(approveButton).not.toBeDisabled();
    });
    await user.click(approveButton);

    await waitFor(() => {
      expect(capturedBody).not.toBeNull();
    });

    const body = capturedBody as { approved_role: string };
    expect(body.approved_role).toBe('scope_user_user');
  });
});
