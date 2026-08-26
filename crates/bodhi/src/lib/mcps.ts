export function encodeMcpToolName(mcpSlug: string, toolName: string): string {
  return `mcp__${mcpSlug}__${toolName}`;
}

export function decodeMcpToolName(toolName: string): { mcpSlug: string; toolName: string } | null {
  const match = toolName.match(/^mcp__(.+?)__(.+)$/);
  if (!match) return null;
  return { mcpSlug: match[1], toolName: match[2] };
}

export function isEncodedMcpToolName(toolName: string): boolean {
  return /^mcp__.+__.+$/.test(toolName);
}
