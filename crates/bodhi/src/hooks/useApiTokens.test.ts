import {
  ListTokensResponse,
  TokenResponse,
  useCreateToken,
  useListTokens,
  useUpdateToken,
} from '@/hooks/useApiTokens';
import { API_TOKENS_ENDPOINT } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { ApiError } from '@/types/models';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest';

const mockTokenResponse: TokenResponse = {
  offline_token: 'test-token-123',
};

const mockListResponse: ListTokensResponse = {
  data: [
    {
      id: 'token-1',
      name: 'Test Token 1',
      status: 'active',
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
  status: 'inactive',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:01Z',
};

const server = setupServer(
  rest.post(`*${API_TOKENS_ENDPOINT}`, (_, res, ctx) => {
    return res(ctx.status(201), ctx.json(mockTokenResponse));
  }),
  rest.get(`*${API_TOKENS_ENDPOINT}`, (req, res, ctx) => {
    const page = req.url.searchParams.get('page') || '1';
    const pageSize = req.url.searchParams.get('page_size') || '10';
    return res(
      ctx.status(200),
      ctx.json({
        ...mockListResponse,
        page: parseInt(page),
        page_size: parseInt(pageSize),
      })
    );
  }),
  rest.put(`*${API_TOKENS_ENDPOINT}/token-1`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockUpdatedToken));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('useListTokens', () => {
  it('fetches tokens with default pagination', async () => {
    const { result } = renderHook(() => useListTokens(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockListResponse);
  });

  it('fetches tokens with custom pagination', async () => {
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
      rest.get(`*${API_TOKENS_ENDPOINT}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ error: { message: 'Test Error' } })
        );
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
    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      const { data } = await result.current.mutateAsync({ name: 'Test Token' });
      expect(data).toEqual(mockTokenResponse);
    });
  });

  it('creates a token without name', async () => {
    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      const { data } = await result.current.mutateAsync({});
      expect(data).toEqual(mockTokenResponse);
    });
  });

  it('invalidates tokens query on successful creation', async () => {
    const wrapper = createWrapper();

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
      rest.get(`*${API_TOKENS_ENDPOINT}`, (req, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({
            ...mockListResponse,
            data: [
              ...mockListResponse.data,
              {
                id: 'new-token',
                name: 'New Token',
                status: 'active',
                created_at: '2024-01-02T00:00:00Z',
                updated_at: '2024-01-02T00:00:00Z',
              },
            ],
            total: 2,
          })
        );
      })
    );

    // Create new token
    const { result: createResult } = renderHook(() => useCreateToken(), {
      wrapper,
    });
    await act(async () => {
      await createResult.current.mutateAsync({ name: 'New Token' });
    });

    // Verify list query was invalidated and refetched with new data
    await waitFor(() => {
      expect(listResult.current.dataUpdatedAt).toBeGreaterThan(
        initialDataUpdatedAt
      );
      expect(listResult.current.data?.data.length).toBe(2);
      expect(listResult.current.data?.total).toBe(2);
    });
  });

  it('handles error response', async () => {
    server.use(
      rest.post(`*${API_TOKENS_ENDPOINT}`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: 'Bad Request',
            message: 'Invalid token name',
          })
        );
      })
    );

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ name: 'Test Token' });
      } catch (error) {
        const axiosError = error as AxiosError<ApiError>;
        expect(axiosError.response?.status).toBe(400);
        expect(axiosError.response?.data.message).toBe('Invalid token name');
      }
    });
  });
});

describe('useUpdateToken', () => {
  it('updates a token successfully', async () => {
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
      rest.put(`*${API_TOKENS_ENDPOINT}/token-1`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: 'Bad Request',
            message: 'Invalid token status',
          })
        );
      })
    );

    const { result } = renderHook(() => useUpdateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      result.current.mutate({ id: 'token-1', status: 'inactive' });
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<ApiError>;
    expect(error.response?.data.message).toBe('Invalid token status');
  });
});
