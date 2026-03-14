/**
 * Tool name encoding/decoding utilities for MCP (Model Context Protocol) servers.
 *
 * Tool names are encoded as: mcp__{mcpSlug}__{toolName}
 * where mcpSlug is the unique instance slug and toolName is the tool name.
 */

/**
 * Encode a tool name with MCP instance slug and tool name.
 * Format: mcp__{mcpSlug}__{toolName}
 */
export function encodeMcpToolName(mcpSlug: string, toolName: string): string {
  return `mcp__${mcpSlug}__${toolName}`;
}

/**
 * Decode a tool name to extract MCP instance slug and tool name.
 */
export function decodeMcpToolName(toolName: string): { mcpSlug: string; toolName: string } | null {
  const match = toolName.match(/^mcp__(.+?)__(.+)$/);
  if (!match) return null;
  return { mcpSlug: match[1], toolName: match[2] };
}

/**
 * Check if a string is a valid encoded MCP tool name.
 */
export function isEncodedMcpToolName(toolName: string): boolean {
  return /^mcp__.+__.+$/.test(toolName);
}
