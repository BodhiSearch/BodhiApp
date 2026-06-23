import type { McpClientTool } from './useMcpClient';

interface SdkTool {
  name: string;
  description?: string;
  inputSchema?: unknown;
}

export function mapSdkToolsToClient(tools: SdkTool[] | undefined | null): McpClientTool[] {
  return (tools || []).map((t) => ({
    name: t.name,
    description: t.description,
    inputSchema: (t.inputSchema ?? {}) as Record<string, unknown>,
  }));
}
