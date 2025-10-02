import { useCreateToken, useListTokens, useUpdateToken } from '@/hooks/useApiTokens';
import { createWrapper } from '@/tests/wrapper';
import { ApiTokenResponse, PaginatedApiTokenResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it } from 'vitest';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockTokens,
  mockCreateToken,
  mockUpdateToken,
  mockTokensError,
  mockCreateTokenError,
  mockUpdateTokenError,
} from '@/test-utils/msw-v2/handlers/tokens';

const mockTokenResponse: ApiTokenResponse = {
  token: 'test-token-123',
};

const mockListResponse: PaginatedApiTokenResponse = {
  data: [
    {
      id: 'token-1',
      name: 'Test Token 1',
      status: 'active',
      token_hash: 'hash123',
      token_prefix: 'bodhiapp_test01',
      scopes: 'scope_token_user',
      user_id: 'user-123',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    },
  ],
  total: 1,
  page: 1,
  page_size: 10,
};

const mockUpdatedToken = {
  id: 'token-1',
  name: 'Updated Token',
  status: 'inactive' as const,
  token_hash: 'hash123',
  token_prefix: 'bodhiapp_test01',
  scopes: 'scope_token_user',
  user_id: 'user-123',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:01Z',
};

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useListTokens', () => {
  it('fetches tokens with default pagination', async () => {
    server.use(...mockTokens(mockListResponse));

    const { result } = renderHook(() => useListTokens(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockListResponse);
  });

  it('fetches tokens with custom pagination', async () => {
    server.use(
      ...mockTokens({
        ...mockListResponse,
        page: 2,
        page_size: 20,
      })
    );

    const { result } = renderHook(() => useListTokens(2, 20), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual({
      ...mockListResponse,
      page: 2,
      page_size: 20,
    });
  });

  it('handles error response', async () => {
    server.use(
      ...mockTokensError({
        message: 'Test Error',
        type: 'internal_server_error',
      })
    );

    const { result } = renderHook(() => useListTokens(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });
});

describe('useCreateToken', () => {
  it('creates a token successfully', async () => {
    server.use(...mockCreateToken(mockTokenResponse));

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      const { data } = await result.current.mutateAsync({ name: 'Test Token', scope: 'scope_token_user' });
      expect(data).toEqual(mockTokenResponse);
    });
  });

  it('creates a token without name', async () => {
    server.use(...mockCreateToken(mockTokenResponse));

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      const { data } = await result.current.mutateAsync({ scope: 'scope_token_user' });
      expect(data).toEqual(mockTokenResponse);
    });
  });

  it.each([
    { scope: 'scope_token_user' as const, tokenValue: 'test-token-user', label: 'User' },
    { scope: 'scope_token_power_user' as const, tokenValue: 'test-token-poweruser', label: 'PowerUser' },
  ])('creates token with $label scope and passes it correctly to API', async ({ scope, tokenValue }) => {
    server.use(...mockCreateToken({ token: tokenValue }));

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      const { data } = await result.current.mutateAsync({
        name: `${scope} Token`,
        scope,
      });
      expect(data).toEqual({ token: tokenValue });
    });
  });

  it('invalidates tokens query on successful creation', async () => {
    const wrapper = createWrapper();

    // Setup initial list tokens mock
    server.use(...mockTokens(mockListResponse));

    // Setup list tokens hook and wait for initial data
    const { result: listResult } = renderHook(() => useListTokens(), {
      wrapper,
    });
    await waitFor(() => {
      expect(listResult.current.isSuccess).toBe(true);
      expect(listResult.current.data).toEqual(mockListResponse);
    });

    // Store the initial fetch time
    const initialDataUpdatedAt = listResult.current.dataUpdatedAt;

    // Update mock to return different data after token creation
    server.use(
      ...mockTokens({
        ...mockListResponse,
        data: [
          ...mockListResponse.data,
          {
            id: 'new-token',
            name: 'New Token',
            status: 'active',
            scopes: 'scope_token_user',
            token_hash: 'newhash456',
            token_prefix: 'jwt-token-id-new',
            user_id: 'user-123',
            created_at: '2024-01-02T00:00:00Z',
            updated_at: '2024-01-02T00:00:00Z',
          },
        ],
        total: 2,
      })
    );

    // Setup create token mock
    server.use(...mockCreateToken(mockTokenResponse));

    // Create new token
    const { result: createResult } = renderHook(() => useCreateToken(), {
      wrapper,
    });
    await act(async () => {
      await createResult.current.mutateAsync({ name: 'New Token', scope: 'scope_token_user' });
    });

    // Verify list query was invalidated and refetched with new data
    await waitFor(() => {
      expect(listResult.current.dataUpdatedAt).toBeGreaterThan(initialDataUpdatedAt);
      expect(listResult.current.data?.data.length).toBe(2);
      expect(listResult.current.data?.total).toBe(2);
    });
  });

  it('handles error response', async () => {
    server.use(
      ...mockCreateTokenError({
        code: 'validation_error',
        message: 'Invalid token name',
        type: 'invalid_request_error',
      })
    );

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ name: 'Test Token', scope: 'scope_token_user' });
      } catch (error) {
        const axiosError = error as AxiosError<OpenAiApiError>;
        expect(axiosError.response?.status).toBe(400);
        expect(axiosError.response?.data.error?.message).toBe('Invalid token name');
      }
    });
  });
});

describe('useUpdateToken', () => {
  it('updates a token successfully', async () => {
    server.use(...mockUpdateToken('token-1', mockUpdatedToken));

    const { result } = renderHook(() => useUpdateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      result.current.mutate({
        id: 'token-1',
        status: 'inactive',
        name: 'Updated Token',
      });
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.data).toEqual(mockUpdatedToken);
  });

  it('handles error response', async () => {
    server.use(
      ...mockUpdateTokenError('token-1', {
        code: 'validation_error',
        message: 'Invalid token status',
        type: 'invalid_request_error',
      })
    );

    const { result } = renderHook(() => useUpdateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      result.current.mutate({ id: 'token-1', name: 'Test Token 1', status: 'inactive' });
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<OpenAiApiError>;
    expect(error.response?.data.error?.message).toBe('Invalid token status');
  });
});
