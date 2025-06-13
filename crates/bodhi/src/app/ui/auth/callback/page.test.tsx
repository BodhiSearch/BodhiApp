import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import OAuthCallbackPage, { OAuthCallbackContent } from './page';

vi.mock('@/hooks/useOAuth', () => {
  const mockOAuthCallback = vi.fn();
  const mockExtractOAuthParams = vi.fn();

  return {
    useOAuthCallback: vi.fn(() => ({
      mutate: mockOAuthCallback,
      isLoading: false,
      isError: false,
      error: null,
    })),
    extractOAuthParams: mockExtractOAuthParams,
  };
});

// Mock window.location
const mockLocation = {
  href: 'https://example.com/ui/auth/callback?code=test_code&state=test_state',
};

Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

// Get access to the mocked functions
const { useOAuthCallback, extractOAuthParams } = vi.mocked(await import('@/hooks/useOAuth'));

describe('OAuthCallbackPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the callback page', () => {
    extractOAuthParams.mockReturnValue({
      code: 'test_code',
      state: 'test_state',
    });
    useOAuthCallback.mockReturnValue({
      mutate: vi.fn(),
      isLoading: false,
      isError: false,
      error: null,
    });

    render(<OAuthCallbackPage />, { wrapper: createWrapper() });

    expect(screen.getByTestId('oauth-callback-page')).toBeInTheDocument();
  });

  it('processes OAuth callback by sending all parameters to backend', async () => {
    const mockMutate = vi.fn();
    extractOAuthParams.mockReturnValue({
      code: 'test_code',
      state: 'test_state',
      session_state: 'sess123',
      custom_param: 'value',
    });
    useOAuthCallback.mockReturnValue({
      mutate: mockMutate,
      isLoading: false,
      isError: false,
      error: null,
    });

    render(<OAuthCallbackContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(extractOAuthParams).toHaveBeenCalledWith(mockLocation.href);
      expect(mockMutate).toHaveBeenCalledWith({
        code: 'test_code',
        state: 'test_state',
        session_state: 'sess123',
        custom_param: 'value',
      });
    });

    expect(screen.getByText('Completing Authentication')).toBeInTheDocument();
    // The component shows loading state instead of description text when processing
    expect(screen.getByTestId('auth-card-loading')).toBeInTheDocument();
  });

  it('sends error parameters to backend without frontend validation', async () => {
    const mockMutate = vi.fn();
    extractOAuthParams.mockReturnValue({
      error: 'access_denied',
      error_description: 'User denied access',
    });
    useOAuthCallback.mockReturnValue({
      mutate: mockMutate,
      isLoading: false,
      isError: false,
      error: null,
    });

    render(<OAuthCallbackContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(extractOAuthParams).toHaveBeenCalledWith(mockLocation.href);
      expect(mockMutate).toHaveBeenCalledWith({
        error: 'access_denied',
        error_description: 'User denied access',
      });
    });

    expect(screen.getByText('Completing Authentication')).toBeInTheDocument();
  });

  it('handles backend error response', async () => {
    const mockMutate = vi.fn();
    extractOAuthParams.mockReturnValue({
      code: 'invalid_code',
      state: 'test_state',
    });

    // Mock error from backend
    useOAuthCallback.mockReturnValue({
      mutate: mockMutate,
      isLoading: false,
      isError: true,
      error: { response: { data: { error: { message: 'Invalid authorization code' } } } },
    });

    // Simulate the onError callback being called
    const mockOnError = vi.fn();
    useOAuthCallback.mockImplementation((options) => {
      // Simulate calling onError after mutation
      setTimeout(() => options?.onError?.('Invalid authorization code'), 0);
      return {
        mutate: mockMutate,
        isLoading: false,
        isError: true,
        error: { response: { data: { error: { message: 'Invalid authorization code' } } } },
      };
    });

    render(<OAuthCallbackContent />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Authentication Failed')).toBeInTheDocument();
      expect(screen.getByText('Invalid authorization code')).toBeInTheDocument();
    });

    expect(screen.getByRole('link', { name: 'Try Again' })).toHaveAttribute('href', '/ui/login');
  });
});
