import type {
  OAuthDiscoverMcpRequest,
  DynamicRegisterRequest,
  OAuthLoginRequest,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
} from '@bodhiapp/ts-client';
import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useDiscoverMcp, useStandaloneDynamicRegister, useOAuthLogin, useOAuthTokenExchange } from '@/hooks/mcps';
import {
  mockDiscoverMcp,
  mockDiscoverMcpError,
  mockStandaloneDynamicRegister,
  mockOAuthLogin,
  mockOAuthTokenExchange,
  mockOAuthTokenExchangeError,
  mockOAuthToken,
} from '@/test-utils/msw-v2/handlers/mcps';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useDiscoverMcp', () => {
  const discoverRequest: OAuthDiscoverMcpRequest = {
    mcp_server_url: 'https://mcp.example.com',
  };

  it('discovers MCP OAuth endpoints successfully', async () => {
    const expectedResponse = {
      authorization_endpoint: 'https://auth.example.com/authorize',
      token_endpoint: 'https://auth.example.com/token',
      registration_endpoint: 'https://auth.example.com/register',
      scopes_supported: ['mcp:tools', 'mcp:read'],
      resource: 'https://mcp.example.com',
      authorization_server_url: 'https://auth.example.com',
    };
    server.use(mockDiscoverMcp(expectedResponse));

    const { result } = renderHook(() => useDiscoverMcp(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(discoverRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.authorization_endpoint).toBe('https://auth.example.com/authorize');
    expect(result.current.data?.data.token_endpoint).toBe('https://auth.example.com/token');
  });

  it('calls onSuccess callback with discovery response', async () => {
    const onSuccess = vi.fn();
    server.use(mockDiscoverMcp());

    const { result } = renderHook(() => useDiscoverMcp({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(discoverRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        authorization_endpoint: 'https://auth.example.com/authorize',
        token_endpoint: 'https://auth.example.com/token',
      })
    );
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockDiscoverMcpError({ message: 'Server not found', status: 404 }));

    const { result } = renderHook(() => useDiscoverMcp({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(discoverRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Server not found');
  });
});

describe('useStandaloneDynamicRegister', () => {
  const registerRequest: DynamicRegisterRequest = {
    registration_endpoint: 'https://auth.example.com/register',
    redirect_uri: 'http://localhost:3000/callback',
    scopes: 'mcp:tools mcp:read',
  };

  it('registers dynamic client successfully', async () => {
    server.use(
      mockStandaloneDynamicRegister({
        client_id: 'dcr-client-id',
        client_secret: 'dcr-client-secret',
        token_endpoint_auth_method: 'client_secret_post',
      })
    );

    const { result } = renderHook(() => useStandaloneDynamicRegister(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(registerRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.client_id).toBe('dcr-client-id');
    expect(result.current.data?.data.client_secret).toBe('dcr-client-secret');
  });

  it('calls onSuccess callback with registration response', async () => {
    const onSuccess = vi.fn();
    const expectedResponse = {
      client_id: 'dcr-client-id',
      client_secret: 'dcr-client-secret',
      token_endpoint_auth_method: 'client_secret_post',
    };
    server.use(mockStandaloneDynamicRegister(expectedResponse));

    const { result } = renderHook(() => useStandaloneDynamicRegister({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(registerRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(expectedResponse);
  });
});

describe('useOAuthLogin', () => {
  const loginRequest: OAuthLoginRequest & { id: string } = {
    id: 'oauth-config-uuid-1',
    redirect_uri: 'http://localhost:3000/callback',
  };

  it('initiates OAuth login successfully', async () => {
    server.use(mockOAuthLogin());

    const { result } = renderHook(() => useOAuthLogin(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(loginRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.authorization_url).toContain('https://auth.example.com/authorize');
  });

  it('calls onSuccess callback with login response', async () => {
    const onSuccess = vi.fn();
    server.use(mockOAuthLogin());

    const { result } = renderHook(() => useOAuthLogin({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(loginRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        authorization_url: expect.stringContaining('https://auth.example.com/authorize'),
      })
    );
  });
});

describe('useOAuthTokenExchange', () => {
  const exchangeRequest: OAuthTokenExchangeRequest & { id: string } = {
    id: 'oauth-config-uuid-1',
    code: 'auth-code-123',
    redirect_uri: 'http://localhost:3000/callback',
    state: 'state-abc123',
  };

  it('exchanges OAuth token successfully', async () => {
    server.use(mockOAuthTokenExchange(mockOAuthToken));

    const { result } = renderHook(() => useOAuthTokenExchange(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(exchangeRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.id).toBe('oauth-token-uuid-1');
    expect(result.current.data?.data.has_refresh_token).toBe(true);
  });

  it('calls onSuccess callback with token response', async () => {
    const onSuccess = vi.fn();
    server.use(mockOAuthTokenExchange(mockOAuthToken));

    const { result } = renderHook(() => useOAuthTokenExchange({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(exchangeRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(mockOAuthToken);
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockOAuthTokenExchangeError({ message: 'Invalid authorization code', status: 400 }));

    const { result } = renderHook(() => useOAuthTokenExchange({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(exchangeRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Invalid authorization code');
  });
});
