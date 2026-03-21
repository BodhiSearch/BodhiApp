import type { McpServerRequest, McpServerResponse } from '@bodhiapp/ts-client';
import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useCreateMcpServer, useUpdateMcpServer } from '@/hooks/mcps';
import {
  mockCreateMcpServer,
  mockCreateMcpServerError,
  mockUpdateMcpServer,
  mockUpdateMcpServerError,
  mockMcpServerResponse,
} from '@/test-utils/msw-v2/handlers/mcps';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useCreateMcpServer', () => {
  const createRequest: McpServerRequest = {
    url: 'https://new-mcp.example.com/mcp',
    name: 'New MCP Server',
    description: 'A new MCP server',
    enabled: true,
  };

  it('creates MCP server successfully', async () => {
    const newServer: McpServerResponse = {
      ...mockMcpServerResponse,
      id: 'server-uuid-new',
      url: 'https://new-mcp.example.com/mcp',
      name: 'New MCP Server',
    };
    server.use(mockCreateMcpServer(newServer));

    const { result } = renderHook(() => useCreateMcpServer(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.id).toBe('server-uuid-new');
    expect(result.current.data?.data.name).toBe('New MCP Server');
  });

  it('calls onSuccess callback on successful creation', async () => {
    const onSuccess = vi.fn();
    server.use(mockCreateMcpServer(mockMcpServerResponse));

    const { result } = renderHook(() => useCreateMcpServer({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(mockMcpServerResponse);
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockCreateMcpServerError({ message: 'URL already registered', status: 400 }));

    const { result } = renderHook(() => useCreateMcpServer({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('URL already registered');
  });
});

describe('useUpdateMcpServer', () => {
  const updateRequest: McpServerRequest & { id: string } = {
    id: 'server-uuid-1',
    url: 'https://mcp.example.com/mcp',
    name: 'Updated Server',
    description: 'Updated description',
    enabled: true,
  };

  it('updates MCP server successfully', async () => {
    const updatedServer: McpServerResponse = {
      ...mockMcpServerResponse,
      name: 'Updated Server',
      description: 'Updated description',
    };
    server.use(mockUpdateMcpServer(updatedServer));

    const { result } = renderHook(() => useUpdateMcpServer(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(updateRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.name).toBe('Updated Server');
    expect(result.current.data?.data.description).toBe('Updated description');
  });

  it('calls onSuccess callback with updated server', async () => {
    const onSuccess = vi.fn();
    const updatedServer: McpServerResponse = {
      ...mockMcpServerResponse,
      name: 'Updated Server',
    };
    server.use(mockUpdateMcpServer(updatedServer));

    const { result } = renderHook(() => useUpdateMcpServer({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(updateRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(updatedServer);
  });
});
