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
  annotations: { readOnlyHint: true },
  title: 'Echo',
};

const GREET_PROMPT = {
  name: 'greet',
  title: 'Greet',
  description: 'Greet someone',
  arguments: [{ name: 'who', required: true }],
};

const README_RESOURCE = {
  uri: 'demo://readme',
  name: 'Readme',
  mimeType: 'text/markdown',
};

const TEMPLATE = {
  uriTemplate: 'demo://item/{id}',
  name: 'Item by id',
};

describe('useMcpClient', () => {
  it('starts disconnected with empty lists and zero counts', () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));
    expect(result.current.status).toBe('disconnected');
    expect(result.current.tools).toEqual([]);
    expect(result.current.prompts).toEqual([]);
    expect(result.current.resources).toEqual([]);
    expect(result.current.resourceTemplates).toEqual([]);
    expect(result.current.counts).toEqual({ tools: 0, prompts: 0, resources: 0, resourceTemplates: 0 });
    expect(result.current.error).toBeNull();
  });

  it('connect() populates all four capability lists and maps annotations + title', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [ECHO_TOOL],
        prompts: [GREET_PROMPT],
        resources: [README_RESOURCE],
        resourceTemplates: [TEMPLATE],
      })
    );
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('connected');
    expect(result.current.tools).toHaveLength(1);
    expect(result.current.tools[0]).toMatchObject({
      name: 'echo',
      description: 'Echoes input',
      title: 'Echo',
      annotations: { readOnlyHint: true },
    });
    expect(result.current.prompts).toHaveLength(1);
    expect(result.current.prompts[0]).toMatchObject({ name: 'greet', title: 'Greet' });
    expect(result.current.resources).toHaveLength(1);
    expect(result.current.resources[0]).toMatchObject({ uri: 'demo://readme', mimeType: 'text/markdown' });
    expect(result.current.resourceTemplates).toHaveLength(1);
    expect(result.current.resourceTemplates[0]).toMatchObject({ uriTemplate: 'demo://item/{id}' });
    expect(result.current.counts).toEqual({ tools: 1, prompts: 1, resources: 1, resourceTemplates: 1 });
  });

  it('connect() with a server that only declares tools leaves prompts/resources empty (guarded)', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [ECHO_TOOL],
        // prompts/resources/resourceTemplates omitted -> capability not advertised
      })
    );
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('connected');
    expect(result.current.tools).toHaveLength(1);
    expect(result.current.prompts).toEqual([]);
    expect(result.current.resources).toEqual([]);
    expect(result.current.resourceTemplates).toEqual([]);
    expect(result.current.counts).toEqual({ tools: 1, prompts: 0, resources: 0, resourceTemplates: 0 });
  });

  it('connect() with a null endpoint is a no-op and stays disconnected', async () => {
    const { result } = renderHook(() => useMcpClient(null));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('disconnected');
  });

  it('connect() failure surfaces status=error with a message and clears lists', async () => {
    const unreachable = '/bodhi/v1/apps/mcps/unreachable/mcp';
    server.use(http.post(unreachable, () => new HttpResponse(null, { status: 500 })));
    const { result } = renderHook(() => useMcpClient(unreachable));

    await act(async () => {
      await result.current.connect();
    });

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBeTruthy();
    expect(result.current.tools).toEqual([]);
    expect(result.current.prompts).toEqual([]);
  });

  it('disconnect() returns the hook to the disconnected state and clears every capability list', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [ECHO_TOOL],
        prompts: [GREET_PROMPT],
        resources: [README_RESOURCE],
      })
    );
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
    expect(result.current.prompts).toEqual([]);
    expect(result.current.resources).toEqual([]);
    expect(result.current.resourceTemplates).toEqual([]);
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

  it('getPrompt() returns the assembled messages on success', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [],
        prompts: [GREET_PROMPT],
        promptGetHandler: (name, args) => ({
          description: 'greeting',
          messages: [{ role: 'user', content: { type: 'text', text: `Hi ${args.who}` } }],
        }),
      })
    );
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    let promptResult: Awaited<ReturnType<typeof result.current.getPrompt>> | undefined;
    await act(async () => {
      promptResult = await result.current.getPrompt('greet', { who: 'world' });
    });

    expect(promptResult).toBeDefined();
    expect(promptResult!.isError).toBeUndefined();
    expect(promptResult!.description).toBe('greeting');
    expect(promptResult!.messages).toHaveLength(1);
    expect(promptResult!.messages[0].role).toBe('user');
  });

  it('getPrompt() returns an error envelope when not connected', async () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));
    let promptResult: Awaited<ReturnType<typeof result.current.getPrompt>> | undefined;
    await act(async () => {
      promptResult = await result.current.getPrompt('greet', {});
    });

    expect(promptResult).toMatchObject({ isError: true, messages: [] });
    expect(promptResult!.errorMessage).toBe('Not connected to MCP server');
  });

  it('readResource() returns contents on success', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [],
        resources: [README_RESOURCE],
        resourceReadHandler: (uri) => ({
          contents: [{ uri, mimeType: 'text/markdown', text: `# Readme\nbody for ${uri}` }],
        }),
      })
    );
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    let readResult: Awaited<ReturnType<typeof result.current.readResource>> | undefined;
    await act(async () => {
      readResult = await result.current.readResource('demo://readme');
    });

    expect(readResult).toBeDefined();
    expect(readResult!.isError).toBeUndefined();
    expect(readResult!.contents).toHaveLength(1);
    expect(readResult!.contents[0].text).toContain('Readme');
  });

  it('readResource() returns an error envelope when not connected', async () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));
    let readResult: Awaited<ReturnType<typeof result.current.readResource>> | undefined;
    await act(async () => {
      readResult = await result.current.readResource('demo://readme');
    });

    expect(readResult).toMatchObject({ isError: true, contents: [] });
    expect(readResult!.errorMessage).toBe('Not connected to MCP server');
  });

  it('refresh() is a no-op while disconnected', async () => {
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.status).toBe('disconnected');
    expect(result.current.tools).toEqual([]);
  });

  it('refresh() re-lists every capability and ends back at connected', async () => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: ENDPOINT,
        tools: [ECHO_TOOL],
        prompts: [GREET_PROMPT],
        resources: [README_RESOURCE],
        resourceTemplates: [TEMPLATE],
      })
    );
    const { result } = renderHook(() => useMcpClient(ENDPOINT));

    await act(async () => {
      await result.current.connect();
    });

    await act(async () => {
      await result.current.refresh();
    });

    await waitFor(() => expect(result.current.status).toBe('connected'));
    expect(result.current.tools).toHaveLength(1);
    expect(result.current.prompts).toHaveLength(1);
    expect(result.current.resources).toHaveLength(1);
    expect(result.current.resourceTemplates).toHaveLength(1);
  });
});
