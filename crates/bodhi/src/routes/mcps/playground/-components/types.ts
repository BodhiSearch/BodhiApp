import { type McpToolCallResult } from '@/hooks/mcps/useMcpClient';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type InputSchema = { type?: string; properties?: Record<string, any>; required?: string[] };

export type ResultTab = 'response' | 'raw' | 'request';

export interface ExecutionResult {
  response: McpToolCallResult;
  toolName: string;
  params: Record<string, unknown>;
}

const getDefaultValue = (propSchema: { type?: string }): unknown => {
  switch (propSchema.type) {
    case 'boolean':
      return false;
    case 'number':
    case 'integer':
      return '';
    case 'array':
      return [];
    case 'object':
      return {};
    default:
      return '';
  }
};

export const buildDefaultParams = (schema: InputSchema | undefined): Record<string, unknown> => {
  if (!schema?.properties) return {};
  const params: Record<string, unknown> = {};
  for (const [key, prop] of Object.entries(schema.properties)) {
    params[key] = getDefaultValue(prop);
  }
  return params;
};
