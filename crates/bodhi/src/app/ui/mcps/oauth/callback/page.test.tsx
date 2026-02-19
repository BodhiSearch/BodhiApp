import OAuthCallbackPage from '@/app/ui/mcps/oauth/callback/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockOAuthToken, mockOAuthTokenExchange } from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { BODHI_API_BASE } from '@/hooks/useQuery';

const pushMock = vi.fn();
let searchParamsMap: Record<string, string | null> = {};

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: (key: string) => searchParamsMap[key] ?? null,
  }),
  usePathname: () => '/ui/mcps/oauth/callback',
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  searchParamsMap = {};
  sessionStorage.clear();
});

afterEach(() => {
  vi.resetAllMocks();
  sessionStorage.clear();
});

describe('OAuthCallbackPage - Success flow', () => {
  beforeEach(() => {
    searchParamsMap = { code: 'auth-code-123', state: 'state-abc' };
    sessionStorage.setItem(
      'mcp_oauth_form_state',
      JSON.stringify({
        name: 'Test MCP',
        slug: 'test-mcp',
        mcp_server_id: 'server-uuid-1',
        auth_type: 'oauth',
        selected_auth_config_id: 'oauth-config-uuid-1',
      })
    );
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true }),
      mockOAuthTokenExchange(mockOAuthToken)
    );
  });

  it('exchanges token and redirects on success', async () => {
    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-success')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/mcps/new/');
    });
  });

  it('redirects to return_url when present (edit mode)', async () => {
    sessionStorage.setItem(
      'mcp_oauth_form_state',
      JSON.stringify({
        name: 'Test MCP',
        slug: 'test-mcp',
        mcp_server_id: 'server-uuid-1',
        auth_type: 'oauth',
        selected_auth_config_id: 'oauth-config-uuid-1',
        return_url: '/ui/mcps/new/?id=existing-mcp-id',
      })
    );

    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-success')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/mcps/new/?id=existing-mcp-id');
    });
  });
});

describe('OAuthCallbackPage - Error from provider', () => {
  beforeEach(() => {
    searchParamsMap = { error: 'access_denied', error_description: 'User denied access' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true })
    );
  });

  it('shows error from OAuth provider', async () => {
    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });
    expect(screen.getByText('User denied access')).toBeInTheDocument();
    expect(screen.getByTestId('oauth-callback-back')).toBeInTheDocument();
  });
});

describe('OAuthCallbackPage - Missing code', () => {
  beforeEach(() => {
    searchParamsMap = {};
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true })
    );
  });

  it('shows error when no authorization code provided', async () => {
    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });
    expect(screen.getByText('No authorization code received')).toBeInTheDocument();
  });
});

describe('OAuthCallbackPage - Missing state', () => {
  beforeEach(() => {
    searchParamsMap = { code: 'auth-code-123' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true })
    );
  });

  it('shows error when state parameter is missing', async () => {
    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });
    expect(screen.getByText('Missing state parameter. Please start the OAuth flow again.')).toBeInTheDocument();
  });
});

describe('OAuthCallbackPage - Corrupt session data', () => {
  beforeEach(() => {
    searchParamsMap = { code: 'auth-code-123', state: 'state-abc' };
    sessionStorage.setItem('mcp_oauth_form_state', 'not-valid-json{{{');
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true })
    );
  });

  it('shows error when session data is corrupt', async () => {
    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });
    expect(screen.getByText('Corrupt session data. Please start the OAuth flow again.')).toBeInTheDocument();
  });
});

describe('OAuthCallbackPage - Token exchange failure', () => {
  beforeEach(() => {
    searchParamsMap = { code: 'bad-code', state: 'state-abc' };
    sessionStorage.setItem(
      'mcp_oauth_form_state',
      JSON.stringify({
        name: 'Test MCP',
        slug: 'test-mcp',
        mcp_server_id: 'server-uuid-1',
        auth_type: 'oauth',
        selected_auth_config_id: 'oauth-config-uuid-1',
      })
    );
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true }),
      http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/token`, () =>
        HttpResponse.json(
          { error: { message: 'Invalid code', code: 'bad_request', type: 'bad_request' } },
          { status: 400 }
        )
      )
    );
  });

  it('shows error when token exchange fails', async () => {
    await act(async () => {
      render(<OAuthCallbackPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('oauth-callback-error')).toBeInTheDocument();
    });
    expect(screen.getByText('Invalid code')).toBeInTheDocument();
  });
});
