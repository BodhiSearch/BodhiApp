import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { createMcpProtocolHandlers } from '@/test-utils/msw-v2/handlers/mcp-protocol';
import { http, HttpResponse, server, setupMswV2 } from '@/test-utils/msw-v2/setup';

import { useMcpClient } from './useMcpClient';

setupMswV2();

const ENDPOINT = '/bodhi/v1/apps/mcps/test-id/mcp';

const ECHO_TOOL = {
  name: 'echo',
  description: 'Echoes input',
  inputSchema: { type: 'object', properties: { message: { type: 'string' } } },
};

describe('useMcpClient', () => {
  it('starts disconnected with no tools', () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));
    expect(result.current.status).toBe('disconnected');
    expect(result.current.tools).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it('connect() transitions to connected and lists the server tools', async () => {
    server.use(...createMcpProtocolHandlers({ endpoint: ENDPOINT, tools: [ECHO_TOOL] }));
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('connected');
    expect(result.current.tools).toHaveLength(1);
    expect(result.current.tools[0]).toMatchObject({ name: 'echo', description: 'Echoes input' });
    expect(result.current.error).toBeNull();
  });

  it('connect() with a null endpoint is a no-op and stays disconnected', async () => {
    const { result } = renderHook(() => useMcpClient(null));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('disconnected');
  });

  it('connect() failure surfaces status=error with a message', async () => {
    const unreachable = '/bodhi/v1/apps/mcps/unreachable/mcp';
    server.use(http.post(unreachable, () => new HttpResponse(null, { status: 500 })));
    const { result } = renderHook(() => useMcpClient(unreachable));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBeTruthy();
    expect(result.current.tools).toEqual([]);
  });

  it('disconnect() returns the hook to the disconnected state and clears tools', async () => {
    server.use(...createMcpProtocolHandlers({ endpoint: ENDPOINT, tools: [ECHO_TOOL] }));
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });
    expect(result.current.status).toBe('connected');

    await act(async () => {
      await result.current.disconnect();
    });

    expect(result.current.status).toBe('disconnected');
    expect(result.current.tools).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it('callTool() returns an error result when not connected', async () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    let callResult;
    await act(async () => {
      callResult = await result.current.callTool('echo', { message: 'hi' });
    });

    expect(callResult).toEqual({ content: 'Not connected to MCP server', isError: true });
  });

  it('callTool() routes through the connected client and returns the result', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [ECHO_TOOL],
        toolCallHandler: (name, args) => ({ text: `${name}:${args.message}` }),
      })
    );
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    let callResult;
    await act(async () => {
      callResult = await result.current.callTool('echo', { message: 'hi' });
    });

    expect(callResult).toMatchObject({ isError: false });
    expect(JSON.stringify(callResult)).toContain('echo:hi');
  });

  it('refreshTools() is a no-op while disconnected', async () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.refreshTools();
    });

    expect(result.current.status).toBe('disconnected');
    expect(result.current.tools).toEqual([]);
  });

  it('refreshTools() re-lists tools and ends back at connected', async () => {
    server.use(...createMcpProtocolHandlers({ endpoint: ENDPOINT, tools: [ECHO_TOOL] }));
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    await act(async () => {
      await result.current.refreshTools();
    });

    await waitFor(() => expect(result.current.status).toBe('connected'));
    expect(result.current.tools).toHaveLength(1);
  });
});
