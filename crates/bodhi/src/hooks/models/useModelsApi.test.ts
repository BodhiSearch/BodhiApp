import type { ApiAliasResponse, ApiModelRequest, OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useCreateApiModel, useUpdateApiModel, useDeleteApiModel, maskApiKey } from '@/hooks/models';
import {
  mockCreateApiModel,
  mockCreateApiModelError,
  mockUpdateApiModel,
  mockUpdateApiModelError,
  mockDeleteApiModel,
  mockDeleteApiModelError,
} from '@/test-utils/msw-v2/handlers/api-models';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const mockApiModelResponse: ApiAliasResponse = {
  source: 'api',
  id: 'test-api-model-123',
  api_format: 'openai',
  base_url: 'https://api.openai.com/v1',
  has_api_key: true,
  models: ['gpt-4'],
  prefix: null,
  forward_all_with_prefix: false,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const mockCreateRequest: ApiModelRequest = {
  api_format: 'openai',
  base_url: 'https://api.openai.com/v1',
  api_key: { action: 'set', value: 'sk-test-key-12345' },
  models: ['gpt-4'],
};

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useCreateApiModel', () => {
  it('creates an API model successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    server.use(
      ...mockCreateApiModel({
        ...mockApiModelResponse,
      })
    );

    const { result } = renderHook(
      () =>
        useCreateApiModel({
          onSuccess,
        }),
      { wrapper: createWrapper() }
    );

    await act(async () => {
      await result.current.mutateAsync(mockCreateRequest);
    });

    expect(onSuccess).toHaveBeenCalled();
    expect(result.current.data?.data.id).toBe('test-api-model-123');
    expect(result.current.data?.data.api_format).toBe('openai');
  });

  it('handles error response', async () => {
    server.use(
      ...mockCreateApiModelError({
        code: 'validation_error',
        message: 'Invalid base URL',
        type: 'invalid_request_error',
        status: 400,
      })
    );

    const { result } = renderHook(() => useCreateApiModel(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync(mockCreateRequest);
      } catch (error) {
        const axiosError = error as AxiosError<OpenAiApiError>;
        expect(axiosError.response?.status).toBe(400);
        expect(axiosError.response?.data.error?.message).toBe('Invalid base URL');
      }
    });
  });
});

describe('useUpdateApiModel', () => {
  it('updates an API model successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    const modelId = 'test-api-model-123';
    server.use(
      ...mockUpdateApiModel(modelId, {
        ...mockApiModelResponse,
        models: ['gpt-4', 'gpt-3.5-turbo'],
      })
    );

    const { result } = renderHook(
      () =>
        useUpdateApiModel({
          onSuccess,
        }),
      { wrapper: createWrapper() }
    );

    await act(async () => {
      await result.current.mutateAsync({
        id: modelId,
        data: {
          ...mockCreateRequest,
          models: ['gpt-4', 'gpt-3.5-turbo'],
        },
      });
    });

    expect(onSuccess).toHaveBeenCalled();
    expect(result.current.data?.data.models).toEqual(['gpt-4', 'gpt-3.5-turbo']);
  });
});

describe('useDeleteApiModel', () => {
  it('deletes an API model successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    const modelId = 'test-api-model-123';
    server.use(...mockDeleteApiModel(modelId));

    const { result } = renderHook(
      () =>
        useDeleteApiModel({
          onSuccess,
        }),
      { wrapper: createWrapper() }
    );

    await act(async () => {
      await result.current.mutateAsync(modelId);
    });

    expect(onSuccess).toHaveBeenCalled();
    expect(result.current.isSuccess).toBe(true);
  });
});

describe('maskApiKey', () => {
  it('masks a long API key showing first 3 and last 6 characters', () => {
    expect(maskApiKey('sk-1234567890abcdef')).toBe('sk-...abcdef');
  });

  it('returns *** for short keys (less than 10 characters)', () => {
    expect(maskApiKey('sk-short')).toBe('***');
  });

  it('returns *** for empty string', () => {
    expect(maskApiKey('')).toBe('***');
  });

  it('masks a key with exactly 10 characters', () => {
    expect(maskApiKey('1234567890')).toBe('123...567890');
  });
});
