import type { BodhiErrorResponse } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it } from 'vitest';

import { useQuery, useMutationQuery } from '@/hooks/useQuery';
import { setupMswV2, server, http, HttpResponse } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const TEST_ENDPOINT = '/bodhi/v1/test-resource';
const TEST_MUTATION_ENDPOINT = '/bodhi/v1/test-mutation';

interface TestData {
  id: string;
  name: string;
}

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useQuery', () => {
  it('fetches data successfully from endpoint', async () => {
    const mockData: TestData = { id: '1', name: 'Test Item' };
    server.use(
      http.get(`*${TEST_ENDPOINT}`, () => {
        return HttpResponse.json(mockData);
      })
    );

    const { result } = renderHook(() => useQuery<TestData>('test-key', TEST_ENDPOINT), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockData);
  });

  it('normalizes string key to array', async () => {
    const mockData: TestData = { id: '2', name: 'Test' };
    server.use(
      http.get(`*${TEST_ENDPOINT}`, () => {
        return HttpResponse.json(mockData);
      })
    );

    const { result } = renderHook(() => useQuery<TestData>('string-key', TEST_ENDPOINT), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockData);
  });

  it('handles error responses with AxiosError typing', async () => {
    server.use(
      http.get(`*${TEST_ENDPOINT}`, () => {
        return HttpResponse.json(
          { error: { code: 'not_found', message: 'Resource not found', type: 'not_found_error' } },
          { status: 404 }
        );
      })
    );

    const { result } = renderHook(() => useQuery<TestData>('error-key', TEST_ENDPOINT), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<BodhiErrorResponse>;
    expect(error.response?.status).toBe(404);
  });
});

describe('useMutationQuery', () => {
  it('POST method works correctly', async () => {
    const responseData: TestData = { id: 'new-1', name: 'Created Item' };
    server.use(
      http.post(`*${TEST_MUTATION_ENDPOINT}`, () => {
        return HttpResponse.json(responseData, { status: 201 });
      })
    );

    const { result } = renderHook(() => useMutationQuery<TestData, { name: string }>(TEST_MUTATION_ENDPOINT, 'post'), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      const response = await result.current.mutateAsync({ name: 'Created Item' });
      expect(response.data).toEqual(responseData);
    });
  });

  it('PUT method works correctly', async () => {
    const responseData: TestData = { id: '1', name: 'Updated Item' };
    server.use(
      http.put(`*${TEST_MUTATION_ENDPOINT}`, () => {
        return HttpResponse.json(responseData);
      })
    );

    const { result } = renderHook(
      () => useMutationQuery<TestData, { id: string; name: string }>(TEST_MUTATION_ENDPOINT, 'put'),
      { wrapper: createWrapper() }
    );

    await act(async () => {
      const response = await result.current.mutateAsync({ id: '1', name: 'Updated Item' });
      expect(response.data).toEqual(responseData);
    });
  });

  it('handles error responses', async () => {
    server.use(
      http.post(`*${TEST_MUTATION_ENDPOINT}`, () => {
        return HttpResponse.json(
          { error: { code: 'validation_error', message: 'Invalid data', type: 'invalid_request_error' } },
          { status: 400 }
        );
      })
    );

    const { result } = renderHook(() => useMutationQuery<TestData, { name: string }>(TEST_MUTATION_ENDPOINT, 'post'), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ name: 'Bad Item' });
      } catch (error) {
        const axiosError = error as AxiosError<BodhiErrorResponse>;
        expect(axiosError.response?.status).toBe(400);
        expect(axiosError.response?.data.error?.message).toBe('Invalid data');
      }
    });
  });

  it('transforms body correctly with transformBody option', async () => {
    const responseData: TestData = { id: '1', name: 'Transformed' };
    server.use(
      http.post(`*${TEST_MUTATION_ENDPOINT}`, async ({ request }) => {
        const body = (await request.json()) as { transformed_name: string };
        return HttpResponse.json({ id: '1', name: body.transformed_name });
      })
    );

    const { result } = renderHook(
      () =>
        useMutationQuery<TestData, { id: string; name: string }>(TEST_MUTATION_ENDPOINT, 'post', undefined, {
          transformBody: ({ name }) => ({ transformed_name: name }),
        }),
      { wrapper: createWrapper() }
    );

    await act(async () => {
      const response = await result.current.mutateAsync({ id: '1', name: 'Transformed' });
      expect(response.data.name).toBe('Transformed');
    });
  });

  it('resolves dynamic endpoint from variables', async () => {
    const responseData: TestData = { id: 'item-42', name: 'Dynamic Item' };
    server.use(
      http.post(`*${TEST_MUTATION_ENDPOINT}/item-42`, () => {
        return HttpResponse.json(responseData);
      })
    );

    const { result } = renderHook(
      () =>
        useMutationQuery<TestData, { id: string; name: string }>(
          ({ id }) => `${TEST_MUTATION_ENDPOINT}/${id}`,
          'post',
          undefined,
          {
            transformBody: ({ name }) => ({ name }),
          }
        ),
      { wrapper: createWrapper() }
    );

    await act(async () => {
      const response = await result.current.mutateAsync({ id: 'item-42', name: 'Dynamic Item' });
      expect(response.data).toEqual(responseData);
    });
  });
});
