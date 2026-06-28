import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { createMockMcp } from '@/test-fixtures/mcps';
import { createMcpProtocolHandlers } from '@/test-utils/msw-v2/handlers/mcp-protocol';
import { http, HttpResponse, server, setupMswV2 } from '@/test-utils/msw-v2/setup';

import { useMcpClients } from './useMcpClients';

setupMswV2();

const PATH_1 = '/bodhi/v1/apps/mcps/mcp-uuid-1/mcp';
const PATH_2 = '/bodhi/v1/apps/mcps/mcp-uuid-2/mcp';

const mcp1 = createMockMcp({ id: 'mcp-uuid-1', path: PATH_1, slug: 'first' });
const mcp2 = createMockMcp({ id: 'mcp-uuid-2', path: PATH_2, slug: 'second' });

function handlersFor(path: string, toolName: string) {
  // Omit inputSchema so the handler supplies a valid `{ type: 'object' }` (the MCP SDK validates it).
  return createMcpProtocolHandlers({ endpoint: path, tools: [{ name: toolName }] });
}

describe('useMcpClients', () => {
  it('connectAll connects each eligible MCP and exposes its tools', async () => {
    server.use(...handlersFor(PATH_1, 'tool-a'), ...handlersFor(PATH_2, 'tool-b'));
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1, mcp2]);
    });

    expect(result.current.clients.get('mcp-uuid-1')?.status).toBe('connected');
    expect(result.current.clients.get('mcp-uuid-2')?.status).toBe('connected');
    expect(result.current.allTools.get('mcp-uuid-1')?.[0].name).toBe('tool-a');
    expect(result.current.allTools.get('mcp-uuid-2')?.[0].name).toBe('tool-b');
    expect(result.current.isConnecting).toBe(false);
  });

  it('skips MCPs that are disabled or have no path', async () => {
    server.use(...handlersFor(PATH_1, 'tool-a'));
    const disabled = createMockMcp({ id: 'mcp-disabled', path: '/x', enabled: false });
    const noPath = createMockMcp({ id: 'mcp-nopath', path: undefined });
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1, disabled, noPath]);
    });

    expect(result.current.clients.has('mcp-uuid-1')).toBe(true);
    expect(result.current.clients.has('mcp-disabled')).toBe(false);
    expect(result.current.clients.has('mcp-nopath')).toBe(false);
  });

  it('diffing: a second connectAll with the same set is a no-op (stable connection preserved)', async () => {
    server.use(...handlersFor(PATH_1, 'tool-a'));
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1]);
    });
    const firstState = result.current.clients.get('mcp-uuid-1');

    await act(async () => {
      await result.current.connectAll([mcp1]);
    });

    // unchanged endpoint → same state object reference (no reconnect churn)
    expect(result.current.clients.get('mcp-uuid-1')).toBe(firstState);
  });

  it('diffing: removing an MCP from the set disconnects only that one', async () => {
    server.use(...handlersFor(PATH_1, 'tool-a'), ...handlersFor(PATH_2, 'tool-b'));
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1, mcp2]);
    });
    expect(result.current.clients.size).toBe(2);

    await act(async () => {
      await result.current.connectAll([mcp1]);
    });

    expect(result.current.clients.has('mcp-uuid-1')).toBe(true);
    expect(result.current.clients.has('mcp-uuid-2')).toBe(false);
    expect(result.current.allTools.has('mcp-uuid-2')).toBe(false);
  });

  it('a connection failure is isolated to that MCP (status=error), others still connect', async () => {
    // PATH_1 connects; PATH_2's handshake 500s.
    server.use(
      ...handlersFor(PATH_1, 'tool-a'),
      http.post(PATH_2, () => new HttpResponse(null, { status: 500 }))
    );
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1, mcp2]);
    });

    expect(result.current.clients.get('mcp-uuid-1')?.status).toBe('connected');
    expect(result.current.clients.get('mcp-uuid-2')?.status).toBe('error');
    expect(result.current.clients.get('mcp-uuid-2')?.error).toBeTruthy();
  });

  it('callTool routes by mcpId and errors for an unknown id', async () => {
    server.use(...handlersFor(PATH_1, 'tool-a'));
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1]);
    });

    let ok;
    let missing;
    await act(async () => {
      ok = await result.current.callTool('mcp-uuid-1', 'tool-a', {});
      missing = await result.current.callTool('nope', 'tool-a', {});
    });

    expect(ok).toMatchObject({ isError: false });
    expect(missing).toEqual({ content: 'Not connected to MCP: nope', isError: true });
  });

  it('disconnectAll clears all connections', async () => {
    server.use(...handlersFor(PATH_1, 'tool-a'), ...handlersFor(PATH_2, 'tool-b'));
    const { result } = renderHook(() => useMcpClients());

    await act(async () => {
      await result.current.connectAll([mcp1, mcp2]);
    });

    await act(async () => {
      await result.current.disconnectAll();
    });

    await waitFor(() => expect(result.current.clients.size).toBe(0));
    expect(result.current.allTools.size).toBe(0);
  });
});
