import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { McpClientTool, McpToolCallResult } from '@/hooks/mcps/useMcpClient';

import { useMcpAgentTools, McpAgentToolsInput } from './useMcpAgentTools';

describe('useMcpAgentTools', () => {
  const mockCallTool = vi.fn<[string, string, Record<string, unknown>], Promise<McpToolCallResult>>();

  const baseTool: McpClientTool = {
    name: 'echo',
    description: 'Echoes input',
    inputSchema: {
      type: 'object',
      properties: {
        message: { type: 'string' },
      },
      required: ['message'],
    },
  };

  const baseInput: McpAgentToolsInput = {
    enabledMcpTools: { 'mcp-uuid-1': ['echo'] },
    allTools: new Map([['mcp-uuid-1', [baseTool]]]),
    slugs: new Map([['mcp-uuid-1', 'my-mcp']]),
    callTool: mockCallTool,
  };

  it('should build AgentTool array from enabled MCP tools', () => {
    const { result } = renderHook(() => useMcpAgentTools(baseInput));

    expect(result.current).toHaveLength(1);
    expect(result.current[0].name).toBe('mcp__my-mcp__echo');
    expect(result.current[0].label).toBe('echo');
    expect(result.current[0].description).toBe('Echoes input');
  });

  it('should filter to only enabled tools', () => {
    const tools: McpClientTool[] = [baseTool, { name: 'disabled-tool', description: 'Not enabled', inputSchema: {} }];

    const input: McpAgentToolsInput = {
      ...baseInput,
      allTools: new Map([['mcp-uuid-1', tools]]),
    };

    const { result } = renderHook(() => useMcpAgentTools(input));

    expect(result.current).toHaveLength(1);
    expect(result.current[0].name).toBe('mcp__my-mcp__echo');
  });

  it('should return empty array when no tools enabled', () => {
    const input: McpAgentToolsInput = {
      ...baseInput,
      enabledMcpTools: {},
    };

    const { result } = renderHook(() => useMcpAgentTools(input));
    expect(result.current).toHaveLength(0);
  });

  it('should skip MCP with unknown slug', () => {
    const input: McpAgentToolsInput = {
      ...baseInput,
      slugs: new Map(),
    };

    const { result } = renderHook(() => useMcpAgentTools(input));
    expect(result.current).toHaveLength(0);
  });

  it('should call mcpClients.callTool on execute and return text content', async () => {
    mockCallTool.mockResolvedValueOnce({
      content: [{ type: 'text', text: 'echoed: hello' }],
      isError: false,
    });

    const { result } = renderHook(() => useMcpAgentTools(baseInput));
    const tool = result.current[0];

    const toolResult = await tool.execute('call-1', { message: 'hello' });

    expect(mockCallTool).toHaveBeenCalledWith('mcp-uuid-1', 'echo', { message: 'hello' });
    expect(toolResult.content).toEqual([
      { type: 'text', text: JSON.stringify([{ type: 'text', text: 'echoed: hello' }]) },
    ]);
  });

  it('should throw on tool execution error', async () => {
    mockCallTool.mockResolvedValueOnce({
      content: 'Tool execution failed: timeout',
      isError: true,
    });

    const { result } = renderHook(() => useMcpAgentTools(baseInput));
    const tool = result.current[0];

    await expect(tool.execute('call-2', { message: 'hello' })).rejects.toThrow('Tool execution failed: timeout');
  });

  it('should handle multiple MCPs with multiple tools', () => {
    const tool2: McpClientTool = {
      name: 'search',
      description: 'Search the web',
      inputSchema: { type: 'object', properties: { query: { type: 'string' } } },
    };

    const input: McpAgentToolsInput = {
      ...baseInput,
      enabledMcpTools: {
        'mcp-uuid-1': ['echo'],
        'mcp-uuid-2': ['search'],
      },
      allTools: new Map([
        ['mcp-uuid-1', [baseTool]],
        ['mcp-uuid-2', [tool2]],
      ]),
      slugs: new Map([
        ['mcp-uuid-1', 'my-mcp'],
        ['mcp-uuid-2', 'web-search'],
      ]),
    };

    const { result } = renderHook(() => useMcpAgentTools(input));

    expect(result.current).toHaveLength(2);
    expect(result.current.map((t) => t.name)).toEqual(['mcp__my-mcp__echo', 'mcp__web-search__search']);
  });
});
