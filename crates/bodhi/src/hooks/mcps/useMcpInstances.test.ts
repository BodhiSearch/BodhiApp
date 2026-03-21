import type { Mcp, McpRequest } from '@bodhiapp/ts-client';
import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useCreateMcp, useUpdateMcp, useDeleteMcp } from '@/hooks/mcps';
import {
  mockCreateMcp,
  mockCreateMcpError,
  mockUpdateMcp,
  mockUpdateMcpError,
  mockDeleteMcp,
  mockMcp,
} from '@/test-utils/msw-v2/handlers/mcps';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useCreateMcp', () => {
  const createRequest: McpRequest = {
    name: 'New MCP',
    slug: 'new-mcp',
    mcp_server_id: 'server-uuid-1',
    enabled: true,
  };

  it('creates MCP instance successfully', async () => {
    const newMcp: Mcp = { ...mockMcp, id: 'mcp-uuid-new', slug: 'new-mcp', name: 'New MCP' };
    server.use(mockCreateMcp(newMcp));

    const { result } = renderHook(() => useCreateMcp(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.id).toBe('mcp-uuid-new');
    expect(result.current.data?.data.slug).toBe('new-mcp');
  });

  it('calls onSuccess callback on successful creation', async () => {
    const onSuccess = vi.fn();
    server.use(mockCreateMcp(mockMcp));

    const { result } = renderHook(() => useCreateMcp({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(mockMcp);
  });

  it('calls onError callback on failure', async () => {
    const onError = vi.fn();
    server.use(mockCreateMcpError({ message: 'Slug already exists', status: 400 }));

    const { result } = renderHook(() => useCreateMcp({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(createRequest).catch(() => {});
    });

    expect(onError).toHaveBeenCalledWith('Slug already exists');
  });
});

describe('useUpdateMcp', () => {
  const updateRequest: McpRequest & { id: string } = {
    id: 'mcp-uuid-1',
    name: 'Updated MCP',
    slug: 'updated-mcp',
    enabled: true,
  };

  it('updates MCP instance successfully', async () => {
    const updatedMcp: Mcp = { ...mockMcp, name: 'Updated MCP', slug: 'updated-mcp' };
    server.use(mockUpdateMcp(updatedMcp));

    const { result } = renderHook(() => useUpdateMcp(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(updateRequest);
    });

    expect(result.current.isSuccess).toBe(true);
    expect(result.current.data?.data.name).toBe('Updated MCP');
  });

  it('calls onSuccess callback with updated MCP', async () => {
    const onSuccess = vi.fn();
    const updatedMcp: Mcp = { ...mockMcp, name: 'Updated MCP' };
    server.use(mockUpdateMcp(updatedMcp));

    const { result } = renderHook(() => useUpdateMcp({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(updateRequest);
    });

    expect(onSuccess).toHaveBeenCalledWith(updatedMcp);
  });
});

describe('useDeleteMcp', () => {
  it('deletes MCP instance successfully', async () => {
    server.use(mockDeleteMcp());

    const { result } = renderHook(() => useDeleteMcp(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: 'mcp-uuid-1' });
    });

    expect(result.current.isSuccess).toBe(true);
  });

  it('calls onSuccess callback on successful deletion', async () => {
    const onSuccess = vi.fn();
    server.use(mockDeleteMcp());

    const { result } = renderHook(() => useDeleteMcp({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: 'mcp-uuid-1' });
    });

    expect(onSuccess).toHaveBeenCalled();
  });
});
