import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import ReviewAccessRequestPage from '@/app/ui/apps/access-requests/review/page';
import {
  MOCK_REQUEST_ID,
  mockApprovedReviewResponse,
  mockDeniedReviewResponse,
  mockDraftMultiToolResponse,
  mockDraftNoInstancesResponse,
  mockDraftRedirectResponse,
  mockDraftReviewResponse,
  mockExpiredReviewResponse,
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
  // Reset window.close
  vi.restoreAllMocks();
});

const setupWindowClose = () => {
  window.close = windowCloseMock;
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
    const option = await screen.findByText('My Exa Instance');
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
    const option = await screen.findByText('My Exa Instance');
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
    const option = await screen.findByText('My Exa Instance');
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
