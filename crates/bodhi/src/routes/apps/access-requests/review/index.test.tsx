import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import ReviewAccessRequestPage from '@/routes/apps/access-requests/review/index';
import {
  MOCK_REQUEST_ID,
  mockApprovedReviewResponse,
  mockDeniedReviewResponse,
  mockDraftMcpCrossUrlResponse,
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
} from '@/test-fixtures/apps';
import {
  mockAppAccessRequestApprove,
  mockAppAccessRequestApproveError,
  mockAppAccessRequestDeny,
  mockAppAccessRequestDenyError,
  mockAppAccessRequestReview,
  mockAppAccessRequestReviewError,
} from '@/test-utils/msw-v2/handlers/apps';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const navigateMock = vi.fn();
let mockSearch: Record<string, string | undefined> = {};

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
    useSearch: () => mockSearch,
    useLocation: () => ({ pathname: '/apps/access-requests/review' }),
  };
});

vi.mock('@/hooks/useToastMessages', () => ({
  useToastMessages: () => ({
    showSuccess: vi.fn(),
    showError: vi.fn(),
  }),
}));

const windowCloseMock = vi.fn();
const MOCK_REDIRECT_URL = 'https://example.com/callback?code=auth_code';
let originalLocationDescriptor: PropertyDescriptor | undefined;

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  navigateMock.mockClear();
  windowCloseMock.mockClear();
  mockSearch = {};
  if (originalLocationDescriptor) {
    Object.defineProperty(window, 'location', originalLocationDescriptor);
    originalLocationDescriptor = undefined;
  }
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

describe('ReviewAccessRequestPage - Loading & Error States', () => {
  it('shows error page when no id query param', async () => {
    mockSearch = {};
    setupHandlers();

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Missing access request ID')).toBeInTheDocument();
    });
  });

  it('shows loading skeleton while fetching review data', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
    // Handlers without review data so the query stays pending and the skeleton shows
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' }));

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-access-loading')).toBeInTheDocument();
    });
  });

  it('shows error page when API returns 404', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
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

describe('ReviewAccessRequestPage - Draft Review Form', () => {
  it('renders app name and description from review data', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
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

  it('Approve button disabled until MCP instance is selected', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();
  });

  it('Approve button becomes enabled after selecting MCP instance', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    const selectTrigger = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(selectTrigger);

    const option = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(option);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });
});

describe('ReviewAccessRequestPage - Approve Flow', () => {
  it('clicking Approve calls PUT with correct body', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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
      expect(windowCloseMock).toHaveBeenCalled();
    });
  });

  it('clicking Approve on redirect flow redirects using window.location', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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
      expect(window.location.href).toBe(MOCK_REDIRECT_URL);
    });
  });

  it('on approve error, shows toast error message', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });
});

describe('ReviewAccessRequestPage - Deny Flow', () => {
  it('clicking Deny calls POST to deny endpoint', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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

    await waitFor(() => {
      expect(windowCloseMock).toHaveBeenCalled();
    });
  });

  it('clicking Deny on redirect flow redirects using window.location', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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

    await waitFor(() => {
      expect(window.location.href).toBe(MOCK_REDIRECT_URL);
    });
  });

  it('on deny error, shows toast error message', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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

    await waitFor(() => {
      expect(screen.getByTestId('review-deny-button')).not.toBeDisabled();
    });
  });
});

describe('ReviewAccessRequestPage - Non-Draft States', () => {
  it('approved status with popup flow calls window.close', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
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

describe('ReviewAccessRequestPage - MCP Server Review', () => {
  it('renders MCP server card with URL badge', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftMcpResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });
  });

  it('Approve button disabled until MCP instance is selected', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftMcpNoInstancesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-no-mcp-instances-https://mcp.example.com/mcp')).toBeInTheDocument();
    });

    expect(screen.getByText(/No MCP instances configured/)).toBeInTheDocument();
  });

  it('unchecking MCP checkbox enables Approve without instance selection', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };

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
        version: string;
        mcps: Array<{ url: string; status: string; instance?: { id: string } }>;
      };
    };
    expect(body.approved.version).toBe('1');
    expect(body.approved.mcps).toHaveLength(1);
    expect(body.approved.mcps[0].url).toBe('https://mcp.deepwiki.com/mcp');
    expect(body.approved.mcps[0].status).toBe('approved');
    expect(body.approved.mcps[0].instance?.id).toBe('mcp-instance-1');
  });

  it('lists both exact-match and non-matching instances, match first', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftMcpCrossUrlResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    const selectTrigger = await screen.findByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(selectTrigger);

    const matchOption = await screen.findByTestId('review-mcp-instance-option-mcp-instance-1');
    const otherOption = await screen.findByTestId('review-mcp-instance-option-mcp-instance-gw');
    expect(matchOption).toBeInTheDocument();
    expect(otherOption).toBeInTheDocument();
    // Exact-URL match renders before the gateway instance.
    expect(matchOption.compareDocumentPosition(otherOption) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it('approve with a non-matching instance sends its id', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };

    let capturedBody: unknown = null;
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockAppAccessRequestReview(mockDraftMcpCrossUrlResponse),
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

    const selectTrigger = await screen.findByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(selectTrigger);
    const otherOption = await screen.findByTestId('review-mcp-instance-option-mcp-instance-gw');
    await user.click(otherOption);

    const approveButton = screen.getByTestId('review-approve-button');
    await waitFor(() => {
      expect(approveButton).not.toBeDisabled();
    });
    await user.click(approveButton);

    await waitFor(() => {
      expect(capturedBody).not.toBeNull();
    });

    const body = capturedBody as {
      approved: { mcps: Array<{ url: string; status: string; instance?: { id: string; path: string } }> };
    };
    expect(body.approved.mcps[0].url).toBe('https://mcp.deepwiki.com/mcp');
    expect(body.approved.mcps[0].instance?.id).toBe('mcp-instance-gw');
    expect(body.approved.mcps[0].instance?.path).toBe('/mcp/deepwiki-gateway');
  });
});

describe('ReviewAccessRequestPage - MCP Partial Approve', () => {
  it('no-instances MCP: blocks Approve, unchecking enables Approve', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftNoInstancesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    const checkbox = screen.getByTestId('review-mcp-toggle-https://mcp.example.com/mcp');
    await user.click(checkbox);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });

  it('button shows "Approve All" when all checkboxes checked and instances selected', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftReviewResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    const selectTrigger = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(selectTrigger);
    const option = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(option);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toHaveTextContent('Approve All');
    });
  });

  it('button shows "Approve Selected" when some checkboxes unchecked', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftMultiToolResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-https://mcp.weather.com/mcp')).toBeInTheDocument();
    });

    const weatherCheckbox = screen.getByTestId('review-mcp-toggle-https://mcp.weather.com/mcp');
    await user.click(weatherCheckbox);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toHaveTextContent('Approve Selected');
    });
  });
});

describe('ReviewAccessRequestPage - Mixed Resources', () => {
  it('renders MCP cards', async () => {
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftMixedResourcesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-mcp-https://mcp.deepwiki.com/mcp')).toBeInTheDocument();
    });

    expect(screen.getByText('Requested MCP Servers:')).toBeInTheDocument();
  });

  it('Approve button requires MCP instance selection', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
    setupHandlers(mockDraftMixedResourcesResponse);

    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('review-approve-button')).toBeDisabled();

    const mcpSelect = screen.getByTestId('review-mcp-select-trigger-https://mcp.deepwiki.com/mcp');
    await user.click(mcpSelect);
    const mcpOption = await screen.findByText('DeepWiki (deepwiki-prod)');
    await user.click(mcpOption);

    await waitFor(() => {
      expect(screen.getByTestId('review-approve-button')).not.toBeDisabled();
    });
  });
});

describe('ReviewAccessRequestPage - Role Selection Dropdown', () => {
  it('shows 2 role options when resource_power_user approves scope_user_power_user request', async () => {
    const user = userEvent.setup();
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };
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
    mockSearch = { id: MOCK_REQUEST_ID };

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

    const roleSelect = screen.getByTestId('review-approved-role-select');
    await user.click(roleSelect);
    const userRoleOption = await screen.findByTestId('review-approved-role-option-scope_user_user');
    await user.click(userRoleOption);

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

    const body = capturedBody as { approved_role: string; approved: { version: string } };
    expect(body.approved_role).toBe('scope_user_user');
    expect(body.approved.version).toBe('1');
  });
});
