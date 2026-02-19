import NewMcpServerPage from '@/app/ui/mcp-servers/new/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCreateMcpServer,
  mockDiscoverMcp,
  mockDiscoverMcpError,
  mockStandaloneDynamicRegister,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { http, HttpResponse } from 'msw';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: pushMock }),
  usePathname: () => '/ui/mcp-servers/new',
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('NewMcpServerPage - OAuth Auto-DCR', () => {
  it('auto-triggers DCR and populates registration type dropdown on OAuth selection', async () => {
    // Mock successful discovery
    server.use(
      mockDiscoverMcp({
        authorization_endpoint: 'https://mcp.asana.com/authorize',
        token_endpoint: 'https://mcp.asana.com/token',
        registration_endpoint: 'https://mcp.asana.com/register',
        scopes_supported: ['mcp:tools', 'mcp:read'],
      })
    );

    const user = userEvent.setup();

    await act(async () => {
      render(<NewMcpServerPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-server-page')).toBeInTheDocument();
    });

    // Fill in server URL
    const urlInput = screen.getByTestId('mcp-server-url-input');
    await user.clear(urlInput);
    await user.type(urlInput, 'https://mcp.asana.com/mcp');

    // Expand auth config section
    const authToggle = screen.getByTestId('auth-config-section-toggle');
    await user.click(authToggle);

    // Select OAuth from auth type dropdown
    const authTypeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(authTypeSelect);

    // Wait for dropdown to open and click OAuth option
    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Wait for auto-DCR to complete
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-auth-endpoint-input')).toHaveValue('https://mcp.asana.com/authorize');
    });

    // Check that registration type dropdown shows "Dynamic Registration"
    const registrationTypeSelect = screen.getByTestId('oauth-registration-type-select');

    // Open the dropdown to verify which option is selected
    await user.click(registrationTypeSelect);

    await waitFor(() => {
      // The "Dynamic Registration" option should be present and selected (aria-selected="true")
      const dynamicRegOption = screen.getByRole('option', { name: /dynamic registration/i });
      expect(dynamicRegOption).toBeInTheDocument();
      expect(dynamicRegOption).toHaveAttribute('aria-selected', 'true');
    });

    // Verify all OAuth endpoints are populated
    expect(screen.getByTestId('auth-config-auth-endpoint-input')).toHaveValue('https://mcp.asana.com/authorize');
    expect(screen.getByTestId('auth-config-token-endpoint-input')).toHaveValue('https://mcp.asana.com/token');
    expect(screen.getByTestId('auth-config-registration-endpoint-input')).toHaveValue('https://mcp.asana.com/register');
    expect(screen.getByTestId('auth-config-scopes-input')).toHaveValue('mcp:tools mcp:read');
  });

  it('creates MCP server with OAuth DCR config on save', async () => {
    let capturedRequest: Record<string, unknown> | null = null;

    server.use(
      mockDiscoverMcp({
        authorization_endpoint: 'https://mcp.asana.com/authorize',
        token_endpoint: 'https://mcp.asana.com/token',
        registration_endpoint: 'https://mcp.asana.com/register',
        scopes_supported: ['mcp:tools', 'mcp:read'],
      }),
      mockStandaloneDynamicRegister({
        client_id: 'test-client-id',
        client_secret: 'test-client-secret',
        token_endpoint_auth_method: 'client_secret_post',
      }),
      http.post('/bodhi/v1/mcps/servers', async ({ request }) => {
        capturedRequest = (await request.json()) as Record<string, unknown>;
        return HttpResponse.json(
          {
            id: 'new-server-uuid',
            url: 'https://mcp.asana.com/mcp',
            name: 'asana',
            description: undefined,
            enabled: true,
            created_by: 'admin',
            updated_by: 'admin',
            enabled_mcp_count: 0,
            disabled_mcp_count: 0,
            created_at: '2025-01-01T00:00:00Z',
            updated_at: '2025-01-01T00:00:00Z',
          },
          { status: 201 }
        );
      })
    );

    const user = userEvent.setup();

    await act(async () => {
      render(<NewMcpServerPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-server-page')).toBeInTheDocument();
    });

    // Fill in server URL
    const urlInput = screen.getByTestId('mcp-server-url-input');
    await user.clear(urlInput);
    await user.type(urlInput, 'https://mcp.asana.com/mcp');

    // Tab out to trigger name auto-population
    await user.tab();

    // Expand auth config section
    const authToggle = screen.getByTestId('auth-config-section-toggle');
    await user.click(authToggle);

    // Select OAuth
    const authTypeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(authTypeSelect);
    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Wait for auto-DCR discovery to complete and populate endpoint fields
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-auth-endpoint-input')).toHaveValue('https://mcp.asana.com/authorize');
    });

    // Verify registration endpoint is populated
    expect(screen.getByTestId('auth-config-registration-endpoint-input')).toHaveValue('https://mcp.asana.com/register');

    // Click save - triggers DCR then server creation
    const saveButton = screen.getByTestId('mcp-server-save-button');
    await user.click(saveButton);

    // Verify the request was sent with OAuth auth config
    await waitFor(() => {
      expect(capturedRequest).not.toBeNull();
    });

    const req = capturedRequest as Record<string, unknown>;
    expect(req.url).toBe('https://mcp.asana.com/mcp');
    expect(req.name).toBe('asana');
    const authConfig = req.auth_config as Record<string, unknown>;
    expect(authConfig).toBeDefined();
    expect(authConfig.type).toBe('oauth');
    expect(authConfig.registration_type).toBe('dynamic-registration');
    expect(authConfig.client_id).toBe('test-client-id');
    expect(authConfig.client_secret).toBe('test-client-secret');

    // Should redirect to MCP servers list
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/mcp-servers');
    });
  });

  it('silently switches to pre-registered on auto-DCR failure', async () => {
    server.use(
      mockDiscoverMcpError({
        status: 404,
        message: 'Discovery endpoint not found',
      })
    );

    const user = userEvent.setup();

    await act(async () => {
      render(<NewMcpServerPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-server-page')).toBeInTheDocument();
    });

    // Fill in server URL
    const urlInput = screen.getByTestId('mcp-server-url-input');
    await user.clear(urlInput);
    await user.type(urlInput, 'https://mcp.asana.com/mcp');

    // Expand auth config section
    const authToggle = screen.getByTestId('auth-config-section-toggle');
    await user.click(authToggle);

    // Select OAuth
    const authTypeSelect = screen.getByTestId('auth-config-type-select');
    await user.click(authTypeSelect);
    await waitFor(() => {
      expect(screen.getByRole('option', { name: /oauth/i })).toBeInTheDocument();
    });
    await user.click(screen.getByRole('option', { name: /oauth/i }));

    // Wait for auto-DCR to fail and fall back to pre-registered (shows client ID field)
    await waitFor(() => {
      expect(screen.getByTestId('auth-config-client-id-input')).toBeInTheDocument();
    });

    // Registration type should fall back to Pre-Registered (no error shown)
    const registrationTypeSelect = screen.getByTestId('oauth-registration-type-select');
    expect(registrationTypeSelect).toHaveTextContent('Pre-Registered');
  });
});
