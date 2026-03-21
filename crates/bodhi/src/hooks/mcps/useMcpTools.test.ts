import type { FetchMcpToolsRequest, McpToolsResponse, McpExecuteResponse } from '@bodhiapp/ts-client';
import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useFetchMcpTools, useRefreshMcpTools, useExecuteMcpTool } from '@/hooks/mcps';
import {
  mockFetchMcpTools,
  mockFetchMcpToolsError,
  mockRefreshMcpTools,
  mockExecuteMcpTool,
  mockExecuteMcpToolError,
  mockMcpTool,
} from '@/test-utils/msw-v2/handlers/mcps';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useFetchMcpTools', () => {
  const fetchRequest: FetchMcpToolsRequest = {
    mcp_server_id: 'server-uuid-1',
  };

  it('fetches tools successfully', async () => {
    const toolsResponse: McpToolsResponse = { tools: [mockMcpTool] };
    server.use(mockFetchMcpTools([mockMcpTool]));

    const { result } = renderHook(() => useFetchMcpTools(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(fetchRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.tools).toHaveLength(1);
    expect(result.current.data?.data.tools[0].name).toBe('read_wiki_structure');
  });

  it('calls onSuccess callback with tools response', async () => {
    const onSuccess = vi.fn();
    const expectedResponse: McpToolsResponse = { tools: [mockMcpTool] };
    server.use(mockFetchMcpTools([mockMcpTool]));

    const { result } = renderHook(() => useFetchMcpTools({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(fetchRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(expectedResponse);
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockFetchMcpToolsError({ message: 'Server unreachable', status: 502 }));

    const { result } = renderHook(() => useFetchMcpTools({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(fetchRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Server unreachable');
  });
});

describe('useRefreshMcpTools', () => {
  it('refreshes tools successfully', async () => {
    server.use(mockRefreshMcpTools([mockMcpTool]));

    const { result } = renderHook(() => useRefreshMcpTools(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: 'mcp-uuid-1' });
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.tools).toHaveLength(1);
  });

  it('calls onSuccess callback with refreshed tools', async () => {
    const onSuccess = vi.fn();
    const expectedResponse: McpToolsResponse = { tools: [mockMcpTool] };
    server.use(mockRefreshMcpTools([mockMcpTool]));

    const { result } = renderHook(() => useRefreshMcpTools({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: 'mcp-uuid-1' });
    });

    expect(onSuccess).toHaveBeenCalledWith(expectedResponse);
  });
});

describe('useExecuteMcpTool', () => {
  const executeRequest = {
    id: 'mcp-uuid-1',
    toolName: 'read_wiki_structure',
    params: { repo_name: 'test-repo' },
  };

  it('executes tool successfully', async () => {
    const executeResponse: McpExecuteResponse = { result: { data: 'wiki structure' } };
    server.use(mockExecuteMcpTool(executeResponse));

    const { result } = renderHook(() => useExecuteMcpTool(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(executeRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.result).toEqual({ data: 'wiki structure' });
  });

  it('calls onSuccess callback with execution result', async () => {
    const onSuccess = vi.fn();
    const executeResponse: McpExecuteResponse = { result: { data: 'wiki structure' } };
    server.use(mockExecuteMcpTool(executeResponse));

    const { result } = renderHook(() => useExecuteMcpTool({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(executeRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(executeResponse);
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockExecuteMcpToolError({ message: 'Tool not allowed', status: 400 }));

    const { result } = renderHook(() => useExecuteMcpTool({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(executeRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Tool not allowed');
  });
});
