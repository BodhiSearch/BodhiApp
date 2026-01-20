/**
 * Tool name encoding/decoding utilities for the multi-instance toolset architecture.
 *
 * Tool names are encoded as: toolset__{toolsetName}__{methodName}
 * where toolsetName is the unique instance name and methodName is the tool method.
 */

/**
 * Encode a tool name with toolset instance name and method.
 * Format: toolset__{toolsetName}__{methodName}
 *
 * @param toolsetName - The unique name of the toolset instance
 * @param methodName - The name of the tool method
 * @returns Encoded tool name
 *
 * @example
 * encodeToolName('my-exa-search', 'search')
 * // Returns: 'toolset__my-exa-search__search'
 */
export function encodeToolName(toolsetName: string, methodName: string): string {
  return `toolset__${toolsetName}__${methodName}`;
}

/**
 * Decode a tool name to extract toolset instance name and method.
 *
 * @param toolName - The encoded tool name
 * @returns Object with toolsetName and method, or null if invalid format
 *
 * @example
 * decodeToolName('toolset__my-exa-search__search')
 * // Returns: { toolsetName: 'my-exa-search', method: 'search' }
 */
export function decodeToolName(toolName: string): { toolsetName: string; method: string } | null {
  const match = toolName.match(/^toolset__(.+?)__(.+)$/);
  if (!match) return null;
  return { toolsetName: match[1], method: match[2] };
}

/**
 * Check if a string is a valid encoded tool name.
 *
 * @param toolName - The string to check
 * @returns True if the string matches the tool name format
 *
 * @example
 * isEncodedToolName('toolset__my-exa-search__search')
 * // Returns: true
 *
 * isEncodedToolName('regular-function-name')
 * // Returns: false
 */
export function isEncodedToolName(toolName: string): boolean {
  return /^toolset__.+__.+$/.test(toolName);
}
