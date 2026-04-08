import { useMemo } from 'react';

import { Type } from '@mariozechner/pi-ai';
import type { AgentTool, AgentToolResult } from '@mariozechner/pi-agent-core';

import { encodeMcpToolName, decodeMcpToolName } from '@/lib/mcps';
import type { McpClientTool, McpToolCallResult } from '@/hooks/mcps/useMcpClient';

export interface McpAgentToolsInput {
  enabledMcpTools: Record<string, string[]>;
  allTools: Map<string, McpClientTool[]>;
  slugs: Map<string, string>;
  callTool: (mcpId: string, toolName: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
}

export function useMcpAgentTools({ enabledMcpTools, allTools, slugs, callTool }: McpAgentToolsInput): AgentTool[] {
  const slugToId = useMemo(() => {
    const map = new Map<string, string>();
    for (const [id, slug] of slugs) {
      map.set(slug, id);
    }
    return map;
  }, [slugs]);

  return useMemo(() => {
    const tools: AgentTool[] = [];

    for (const [mcpId, toolNames] of Object.entries(enabledMcpTools)) {
      const slug = slugs.get(mcpId);
      if (!slug) continue;

      const clientTools = allTools.get(mcpId) ?? [];

      for (const mcpTool of clientTools) {
        if (!toolNames.includes(mcpTool.name)) continue;

        const encodedName = encodeMcpToolName(slug, mcpTool.name);

        tools.push({
          name: encodedName,
          label: mcpTool.name,
          description: mcpTool.description ?? '',
          parameters: Type.Unsafe(mcpTool.inputSchema),
          execute: async (_toolCallId: string, params: Record<string, unknown>): Promise<AgentToolResult<unknown>> => {
            const decoded = decodeMcpToolName(encodedName);
            if (!decoded) {
              throw new Error(`Failed to decode tool name: ${encodedName}`);
            }

            const resolvedMcpId = slugToId.get(decoded.mcpSlug);
            if (!resolvedMcpId) {
              throw new Error(`Unknown MCP slug: ${decoded.mcpSlug}`);
            }

            const result = await callTool(resolvedMcpId, decoded.toolName, params);

            if (result.isError) {
              throw new Error(typeof result.content === 'string' ? result.content : JSON.stringify(result.content));
            }

            return {
              content: [
                {
                  type: 'text',
                  text: typeof result.content === 'string' ? result.content : JSON.stringify(result.content),
                },
              ],
              details: result.content,
            };
          },
        });
      }
    }

    return tools;
  }, [enabledMcpTools, allTools, slugs, slugToId, callTool]);
}
