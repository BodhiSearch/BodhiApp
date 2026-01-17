import { EnhancedToolsetConfigResponse, ListToolsetsResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it } from 'vitest';

import {
  useAvailableToolsets,
  useDeleteToolsetConfig,
  useSetAppToolsetDisabled,
  useSetAppToolsetEnabled,
  useToolsetConfig,
  useUpdateToolsetConfig,
} from '@/hooks/useToolsets';
import {
  mockAvailableToolsets,
  mockAvailableToolsetsError,
  mockDeleteToolsetConfig,
  mockSetAppToolsetDisabled,
  mockSetAppToolsetEnabled,
  mockToolsetConfig,
  mockToolsetConfigError,
  mockUpdateToolsetConfig,
  mockUpdateToolsetConfigError,
} from '@/test-utils/msw-v2/handlers/toolsets';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const mockToolsetsResponse: ListToolsetsResponse = {
  toolsets: [
    {
      type: 'function',
      function: {
        name: 'builtin-exa-web-search',
        description: 'Search the web using Exa AI for real-time information',
        parameters: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'Search query' },
          },
          required: ['query'],
        },
      },
      app_enabled: true,
      user_config: {
        enabled: true,
        has_api_key: true,
      },
    },
  ],
};

const mockConfigResponse: EnhancedToolsetConfigResponse = {
  toolset_id: 'builtin-exa-web-search',
  app_enabled: true,
  config: {
    toolset_id: 'builtin-exa-web-search',
    enabled: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
};

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useAvailableToolsets', () => {
  it('fetches available toolsets successfully', async () => {
    server.use(...mockAvailableToolsets(mockToolsetsResponse.toolsets));

    const { result } = renderHook(() => useAvailableToolsets(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.toolsets).toHaveLength(1);
    expect(result.current.data?.toolsets[0].function.name).toBe('builtin-exa-web-search');
  });

  it('handles error response', async () => {
    server.use(
      ...mockAvailableToolsetsError({
        message: 'Failed to fetch toolsets',
        status: 500,
      })
    );

    const { result } = renderHook(() => useAvailableToolsets(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });
});

describe('useToolsetConfig', () => {
  it('fetches toolset config successfully', async () => {
    server.use(
      ...mockToolsetConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: mockConfigResponse.config,
      })
    );

    const { result } = renderHook(() => useToolsetConfig('builtin-exa-web-search'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.toolset_id).toBe('builtin-exa-web-search');
    expect(result.current.data?.app_enabled).toBe(true);
    expect(result.current.data?.config.enabled).toBe(true);
  });

  it('does not fetch when toolsetId is empty', async () => {
    const { result } = renderHook(() => useToolsetConfig(''), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toBeUndefined();
  });

  it('handles error response', async () => {
    server.use(
      ...mockToolsetConfigError('invalid-toolset', {
        message: 'Toolset not found',
        status: 404,
      })
    );

    const { result } = renderHook(() => useToolsetConfig('invalid-toolset'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });
});

describe('useUpdateToolsetConfig', () => {
  it('updates toolset config successfully', async () => {
    server.use(
      ...mockUpdateToolsetConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: mockConfigResponse.config,
      })
    );

    const { result } = renderHook(() => useUpdateToolsetConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        toolsetId: 'builtin-exa-web-search',
        enabled: true,
        api_key: 'test-api-key',
      });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.toolset_id).toBe('builtin-exa-web-search');
  });

  it('handles error response', async () => {
    server.use(
      ...mockUpdateToolsetConfigError('builtin-exa-web-search', {
        message: 'Failed to update toolset configuration',
        status: 500,
      })
    );

    const { result } = renderHook(() => useUpdateToolsetConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          toolsetId: 'builtin-exa-web-search',
          enabled: true,
        });
      } catch (error) {
        const axiosError = error as AxiosError<OpenAiApiError>;
        expect(axiosError.response?.status).toBe(500);
      }
    });
  });
});

describe('useDeleteToolsetConfig', () => {
  it('deletes toolset config successfully', async () => {
    server.use(...mockDeleteToolsetConfig('builtin-exa-web-search'));

    const { result } = renderHook(() => useDeleteToolsetConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ toolsetId: 'builtin-exa-web-search' });
    });

    expect(result.current.isSuccess).toBe(true);
  });
});

describe('useSetAppToolsetEnabled', () => {
  it('enables app toolset successfully', async () => {
    server.use(...mockSetAppToolsetEnabled('builtin-exa-web-search'));

    const { result } = renderHook(() => useSetAppToolsetEnabled(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ toolsetId: 'builtin-exa-web-search' });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.enabled).toBe(true);
  });
});

describe('useSetAppToolsetDisabled', () => {
  it('disables app toolset successfully', async () => {
    server.use(...mockSetAppToolsetDisabled('builtin-exa-web-search'));

    const { result } = renderHook(() => useSetAppToolsetDisabled(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ toolsetId: 'builtin-exa-web-search' });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.enabled).toBe(false);
  });
});
