import { renderHook, act } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest';
import {
  CREATE_TOKEN_ENDPOINT,
  TokenResponse,
  useCreateToken,
} from './useCreateToken';
import { createWrapper } from '@/tests/wrapper';
import { AxiosError } from 'axios';
import { ApiError } from '@/types/models';

const mockTokenResponse: TokenResponse = {
  offline_token: 'test-token-123',
  name: 'Test Token',
  status: 'active',
  last_used: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const server = setupServer(
  rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
    return res(ctx.status(201), ctx.json(mockTokenResponse));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

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

  it('handles server error', async () => {
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({ message: 'Internal server error' })
        );
      })
    );

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await expect(result.current.mutateAsync({})).rejects.toThrow();
    });
  });

  it('handles validation error', async () => {
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({ message: 'Invalid token name' })
        );
      })
    );

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ name: 'invalid' });
        fail('Expected error to be thrown');
      } catch (error) {
        expect(error instanceof AxiosError).toBe(true);
        const apiError = (error as AxiosError).response?.data as ApiError;
        expect(apiError?.message).toBe('Invalid token name');
      }
    });
  });

  it('handles network error', async () => {
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res) => {
        return res.networkError('Failed to connect');
      })
    );

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await expect(result.current.mutateAsync({})).rejects.toThrow();
    });
  });

  it('sends correct request payload', async () => {
    let requestPayload: TokenResponse | null = null;
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, async (req, res, ctx) => {
        requestPayload = await req.json();
        return res(ctx.status(201), ctx.json(mockTokenResponse));
      })
    );

    const { result } = renderHook(() => useCreateToken(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ name: 'Test Token' });
      expect(requestPayload).toEqual({ name: 'Test Token' });
    });
  });
});
