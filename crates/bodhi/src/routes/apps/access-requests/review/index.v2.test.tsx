import { act, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import ReviewAccessRequestPage from '@/routes/apps/access-requests/review/index';
import { MOCK_REQUEST_ID, mockDraftReviewResponse } from '@/test-fixtures/apps';
import { mockAppAccessRequestReview } from '@/test-utils/msw-v2/handlers/apps';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

let mockSearch: Record<string, string | undefined> = { id: MOCK_REQUEST_ID };

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
    useSearch: () => mockSearch,
    useLocation: () => ({ pathname: '/apps/access-requests/review' }),
  };
});

vi.mock('@/hooks/useToastMessages', () => ({
  useToastMessages: () => ({ showSuccess: vi.fn(), showError: vi.fn() }),
}));

setupMswV2();

beforeEach(() => {
  mockSearch = { id: MOCK_REQUEST_ID };
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ role: 'resource_admin' }),
    ...mockAppAccessRequestReview(mockDraftReviewResponse)
  );
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

describe('ReviewAccessRequestPage V2', () => {
  it('renders the consent page with the V2 header and preserves review testids', async () => {
    await act(async () => {
      render(<ReviewAccessRequestPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('review-access-page')).toBeInTheDocument();
    });

    // V2 consent header treatment
    expect(screen.getByText('Decide which of your resources this 3rd-party app can use.')).toBeInTheDocument();
    // V2 root carries the api-keys-screen V2 class
    expect(screen.getByTestId('review-access-page')).toHaveClass('api-keys-screen');
    // real-data testids preserved
    expect(screen.getByTestId('review-app-name')).toBeInTheDocument();
    expect(screen.getByTestId('review-approve-button')).toBeInTheDocument();
    expect(screen.getByTestId('review-deny-button')).toBeInTheDocument();
  });
});
