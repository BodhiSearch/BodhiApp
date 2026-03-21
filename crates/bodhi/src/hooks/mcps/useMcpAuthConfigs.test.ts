import type { CreateAuthConfig, McpAuthConfigResponse } from '@bodhiapp/ts-client';
import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useCreateAuthConfig, useDeleteAuthConfig, useDeleteOAuthToken } from '@/hooks/mcps';
import {
  mockCreateAuthConfig,
  mockCreateAuthConfigError,
  mockDeleteAuthConfig,
  mockDeleteOAuthToken,
  mockAuthConfigHeader,
  mockAuthConfigOAuthPreReg,
} from '@/test-utils/msw-v2/handlers/mcps';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useCreateAuthConfig', () => {
  const createHeaderRequest: CreateAuthConfig = {
    mcp_server_id: 'server-uuid-1',
    name: 'Header Auth',
    type: 'header',
    entries: [{ param_type: 'header', param_key: 'Authorization' }],
  };

  it('creates auth config successfully', async () => {
    server.use(mockCreateAuthConfig(mockAuthConfigHeader));

    const { result } = renderHook(() => useCreateAuthConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createHeaderRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.id).toBe('auth-header-uuid-1');
    expect(result.current.data?.data.type).toBe('header');
  });

  it('calls onSuccess callback on successful creation', async () => {
    const onSuccess = vi.fn();
    server.use(mockCreateAuthConfig(mockAuthConfigOAuthPreReg));

    const createOAuthRequest: CreateAuthConfig = {
      mcp_server_id: 'server-uuid-1',
      name: 'OAuth Pre-Registered',
      type: 'oauth',
      client_id: 'test-client-id',
      authorization_endpoint: 'https://auth.example.com/authorize',
      token_endpoint: 'https://auth.example.com/token',
    };

    const { result } = renderHook(() => useCreateAuthConfig({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createOAuthRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(mockAuthConfigOAuthPreReg);
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockCreateAuthConfigError({ message: 'Invalid auth config', status: 400 }));

    const { result } = renderHook(() => useCreateAuthConfig({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createHeaderRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Invalid auth config');
  });
});

describe('useDeleteAuthConfig', () => {
  it('deletes auth config successfully', async () => {
    server.use(mockDeleteAuthConfig());

    const { result } = renderHook(() => useDeleteAuthConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ configId: 'auth-header-uuid-1' });
    });

    expect(result.current.isSuccess).toBe(true);
  });

  it('calls onSuccess callback on successful deletion', async () => {
    const onSuccess = vi.fn();
    server.use(mockDeleteAuthConfig());

    const { result } = renderHook(() => useDeleteAuthConfig({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ configId: 'auth-header-uuid-1' });
    });

    expect(onSuccess).toHaveBeenCalled();
  });
});

describe('useDeleteOAuthToken', () => {
  it('deletes OAuth token successfully', async () => {
    server.use(mockDeleteOAuthToken());

    const { result } = renderHook(() => useDeleteOAuthToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ tokenId: 'oauth-token-uuid-1' });
    });

    expect(result.current.isSuccess).toBe(true);
  });

  it('calls onSuccess callback on successful deletion', async () => {
    const onSuccess = vi.fn();
    server.use(mockDeleteOAuthToken());

    const { result } = renderHook(() => useDeleteOAuthToken({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ tokenId: 'oauth-token-uuid-1' });
    });

    expect(onSuccess).toHaveBeenCalled();
  });
});
