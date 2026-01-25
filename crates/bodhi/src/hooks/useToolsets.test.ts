/**
 * useToolsets Hook Tests - Instance-based Architecture
 *
 * Tests for UUID-based multi-instance toolset hooks with MSW v2 mocks
 */
import { ListToolsetsResponse, ToolsetResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  useToolsets,
  useToolset,
  useCreateToolset,
  useUpdateToolset,
  useDeleteToolset,
  useToolsetTypes,
} from '@/hooks/useToolsets';
import {
  mockListToolsets,
  mockListToolsetsError,
  mockGetToolset,
  mockGetToolsetError,
  mockCreateToolset,
  mockCreateToolsetError,
  mockUpdateToolset,
  mockUpdateToolsetError,
  mockDeleteToolset,
  mockListTypes,
  mockType,
} from '@/test-utils/msw-v2/handlers/toolsets';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const mockToolsetsResponse: ListToolsetsResponse = {
  toolsets: [
    {
      id: 'uuid-test-toolset',
      name: 'my-exa-search',
      scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6', scope: 'scope_toolset-builtin-exa-web-search',
      description: 'Test toolset',
      enabled: true,
      has_api_key: true,
      app_enabled: true,
      tools: [
        {
          type: 'function',
          function: {
            name: 'search',
            description: 'Search the web',
            parameters: {
              type: 'object',
              properties: {
                query: { type: 'string', description: 'Search query' },
              },
              required: ['query'],
            },
          },
        },
      ],
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    },
  ],
};

const mockToolsetResponse: ToolsetResponse = mockToolsetsResponse.toolsets[0];

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useToolsets', () => {
  it('fetches toolsets successfully', async () => {
    server.use(mockListToolsets(mockToolsetsResponse.toolsets));

    const { result } = renderHook(() => useToolsets(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.toolsets).toHaveLength(1);
    expect(result.current.data?.toolsets[0].id).toBe('uuid-test-toolset');
    expect(result.current.data?.toolsets[0].name).toBe('my-exa-search');
  });

  it('handles error response', async () => {
    server.use(
      mockListToolsetsError({
        message: 'Failed to fetch toolsets',
        status: 500,
      })
    );

    const { result } = renderHook(() => useToolsets(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<OpenAiApiError>;
    expect(error.response?.status).toBe(500);
  });

  it('caches toolsets data', async () => {
    server.use(mockListToolsets(mockToolsetsResponse.toolsets));

    const { result, rerender } = renderHook(() => useToolsets(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const firstData = result.current.data;

    rerender();

    expect(result.current.data).toBe(firstData);
  });
});

describe('useToolset', () => {
  it('fetches toolset by ID successfully', async () => {
    server.use(mockGetToolset(mockToolsetResponse));

    const { result } = renderHook(() => useToolset('uuid-test-toolset'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.id).toBe('uuid-test-toolset');
    expect(result.current.data?.name).toBe('my-exa-search');
    expect(result.current.data?.enabled).toBe(true);
  });

  it('handles error response', async () => {
    server.use(
      mockGetToolsetError({
        message: 'Toolset not found',
        status: 404,
      })
    );

    const { result } = renderHook(() => useToolset('invalid-id'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<OpenAiApiError>;
    expect(error.response?.status).toBe(404);
  });
});

describe('useCreateToolset', () => {
  it('creates toolset successfully', async () => {
    const newToolset: ToolsetResponse = {
      ...mockToolsetResponse,
      id: 'uuid-new-toolset',
      name: 'my-new-toolset',
    };

    server.use(mockCreateToolset(newToolset));

    const { result } = renderHook(() => useCreateToolset(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6', scope: 'scope_toolset-builtin-exa-web-search',
        name: 'my-new-toolset',
        description: 'New toolset',
        enabled: true,
        api_key: 'test-api-key',
      });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.id).toBe('uuid-new-toolset');
    expect(result.current.data?.data.name).toBe('my-new-toolset');
  });

  it('calls onSuccess callback on successful creation', async () => {
    const onSuccess = vi.fn();
    server.use(mockCreateToolset(mockToolsetResponse));

    const { result } = renderHook(() => useCreateToolset({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6', scope: 'scope_toolset-builtin-exa-web-search',
        name: 'my-exa-search',
        enabled: true,
        api_key: 'test-api-key',
      });
    });

    expect(onSuccess).toHaveBeenCalledWith(mockToolsetResponse);
  });

  it('handles error response and calls onError callback', async () => {
    const onError = vi.fn();
    server.use(
      mockCreateToolsetError({
        message: 'Name already exists',
        status: 400,
      })
    );

    const { result } = renderHook(() => useCreateToolset({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current
        .mutateAsync({
          scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6', scope: 'scope_toolset-builtin-exa-web-search',
          name: 'duplicate-name',
          enabled: true,
          api_key: 'test-api-key',
        })
        .catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Name already exists');
  });
});

describe('useUpdateToolset', () => {
  it('updates toolset successfully', async () => {
    const updatedToolset: ToolsetResponse = {
      ...mockToolsetResponse,
      description: 'Updated description',
      updated_at: new Date().toISOString(),
    };

    server.use(mockUpdateToolset(updatedToolset));

    const { result } = renderHook(() => useUpdateToolset(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        description: 'Updated description',
        enabled: true,
        api_key: { action: 'Keep' },
      });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.description).toBe('Updated description');
  });

  it('calls onSuccess callback with updated toolset', async () => {
    const onSuccess = vi.fn();
    server.use(mockUpdateToolset(mockToolsetResponse));

    const { result } = renderHook(() => useUpdateToolset({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        enabled: true,
        api_key: { action: 'Keep' },
      });
    });

    expect(onSuccess).toHaveBeenCalledWith(mockToolsetResponse);
  });

  it('handles error response and calls onError callback', async () => {
    const onError = vi.fn();
    server.use(
      mockUpdateToolsetError({
        message: 'Failed to update toolset',
        status: 500,
      })
    );

    const { result } = renderHook(() => useUpdateToolset({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current
        .mutateAsync({
          id: 'uuid-test-toolset',
          name: 'my-exa-search',
          enabled: true,
          api_key: { action: 'Keep' },
        })
        .catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Failed to update toolset');
  });
});

describe('useDeleteToolset', () => {
  it('deletes toolset successfully', async () => {
    server.use(mockDeleteToolset());

    const { result } = renderHook(() => useDeleteToolset(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: 'uuid-test-toolset' });
    });

    expect(result.current.isSuccess).toBe(true);
  });

  it('calls onSuccess callback on successful deletion', async () => {
    const onSuccess = vi.fn();
    server.use(mockDeleteToolset());

    const { result } = renderHook(() => useDeleteToolset({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: 'uuid-test-toolset' });
    });

    expect(onSuccess).toHaveBeenCalled();
  });
});

describe('useToolsetTypes', () => {
  it('fetches toolset types successfully', async () => {
    server.use(mockListTypes([mockType]));

    const { result } = renderHook(() => useToolsetTypes(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.types).toHaveLength(1);
    expect(result.current.data?.types[0].scope_uuid).toBe('4ff0e163-36fb-47d6-a5ef-26e396f067d6');
    expect(result.current.data?.types[0].scope).toBe('scope_toolset-builtin-exa-web-search');
    expect(result.current.data?.types[0].name).toBe('Exa Web Search');
  });
});
