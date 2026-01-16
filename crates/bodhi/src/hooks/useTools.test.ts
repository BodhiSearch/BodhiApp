import { OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it } from 'vitest';

import {
  useAvailableTools,
  useToolConfig,
  useUpdateToolConfig,
  useDeleteToolConfig,
  useSetAppToolEnabled,
  useSetAppToolDisabled,
  ListToolsResponse,
  EnhancedToolConfigResponse,
} from '@/hooks/useTools';
import {
  mockAvailableTools,
  mockToolConfig,
  mockUpdateToolConfig,
  mockDeleteToolConfig,
  mockSetAppToolEnabled,
  mockSetAppToolDisabled,
  mockAvailableToolsError,
  mockToolConfigError,
  mockUpdateToolConfigError,
} from '@/test-utils/msw-v2/handlers/tools';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const mockToolsResponse: ListToolsResponse = {
  tools: [
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

const mockConfigResponse: EnhancedToolConfigResponse = {
  tool_id: 'builtin-exa-web-search',
  app_enabled: true,
  config: {
    tool_id: 'builtin-exa-web-search',
    enabled: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
};

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useAvailableTools', () => {
  it('fetches available tools successfully', async () => {
    server.use(...mockAvailableTools(mockToolsResponse.tools));

    const { result } = renderHook(() => useAvailableTools(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.tools).toHaveLength(1);
    expect(result.current.data?.tools[0].function.name).toBe('builtin-exa-web-search');
  });

  it('handles error response', async () => {
    server.use(
      ...mockAvailableToolsError({
        message: 'Failed to fetch tools',
        status: 500,
      })
    );

    const { result } = renderHook(() => useAvailableTools(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });
});

describe('useToolConfig', () => {
  it('fetches tool config successfully', async () => {
    server.use(
      ...mockToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: mockConfigResponse.config,
      })
    );

    const { result } = renderHook(() => useToolConfig('builtin-exa-web-search'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.tool_id).toBe('builtin-exa-web-search');
    expect(result.current.data?.app_enabled).toBe(true);
    expect(result.current.data?.config.enabled).toBe(true);
  });

  it('does not fetch when toolId is empty', async () => {
    const { result } = renderHook(() => useToolConfig(''), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toBeUndefined();
  });

  it('handles error response', async () => {
    server.use(
      ...mockToolConfigError('invalid-tool', {
        message: 'Tool not found',
        status: 404,
      })
    );

    const { result } = renderHook(() => useToolConfig('invalid-tool'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });
});

describe('useUpdateToolConfig', () => {
  it('updates tool config successfully', async () => {
    server.use(
      ...mockUpdateToolConfig('builtin-exa-web-search', {
        app_enabled: true,
        config: mockConfigResponse.config,
      })
    );

    const { result } = renderHook(() => useUpdateToolConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        toolId: 'builtin-exa-web-search',
        enabled: true,
        api_key: 'test-api-key',
      });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.tool_id).toBe('builtin-exa-web-search');
  });

  it('handles error response', async () => {
    server.use(
      ...mockUpdateToolConfigError('builtin-exa-web-search', {
        message: 'Failed to update tool configuration',
        status: 500,
      })
    );

    const { result } = renderHook(() => useUpdateToolConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          toolId: 'builtin-exa-web-search',
          enabled: true,
        });
      } catch (error) {
        const axiosError = error as AxiosError<OpenAiApiError>;
        expect(axiosError.response?.status).toBe(500);
      }
    });
  });
});

describe('useDeleteToolConfig', () => {
  it('deletes tool config successfully', async () => {
    server.use(...mockDeleteToolConfig('builtin-exa-web-search'));

    const { result } = renderHook(() => useDeleteToolConfig(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ toolId: 'builtin-exa-web-search' });
    });

    expect(result.current.isSuccess).toBe(true);
  });
});

describe('useSetAppToolEnabled', () => {
  it('enables app tool successfully', async () => {
    server.use(...mockSetAppToolEnabled('builtin-exa-web-search'));

    const { result } = renderHook(() => useSetAppToolEnabled(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ toolId: 'builtin-exa-web-search' });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.enabled).toBe(true);
  });
});

describe('useSetAppToolDisabled', () => {
  it('disables app tool successfully', async () => {
    server.use(...mockSetAppToolDisabled('builtin-exa-web-search'));

    const { result } = renderHook(() => useSetAppToolDisabled(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ toolId: 'builtin-exa-web-search' });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.enabled).toBe(false);
  });
});
