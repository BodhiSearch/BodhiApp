import type { McpClientTool, McpToolAnnotations } from './useMcpClient';

interface SdkTool {
  name: string;
  description?: string;
  title?: string;
  inputSchema?: unknown;
  annotations?: McpToolAnnotations;
}

export function mapSdkToolsToClient(tools: SdkTool[] | undefined | null): McpClientTool[] {
  return (tools || []).map((t) => ({
    name: t.name,
    description: t.description,
    title: t.title,
    inputSchema: (t.inputSchema ?? {}) as Record<string, unknown>,
    annotations: t.annotations,
  }));
}
