/**
 * MSW v2 handlers for tools endpoints
 */
import {
  TOOLS_ENDPOINT,
  ToolListItem,
  ListToolsResponse,
  EnhancedToolConfigResponse,
  AppToolConfigResponse,
} from '@/hooks/useTools';

import { http, HttpResponse, INTERNAL_SERVER_ERROR } from '../setup';

// ============================================================================
// Default Test Data
// ============================================================================

const DEFAULT_TOOL_DEFINITION = {
  type: 'function' as const,
  function: {
    name: 'builtin-exa-web-search',
    description: 'Search the web using Exa AI for real-time information',
    parameters: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Search query',
        },
        num_results: {
          type: 'number',
          description: 'Number of results to return (default: 5)',
        },
      },
      required: ['query'],
    },
  },
};

const DEFAULT_TOOL_LIST_ITEM: ToolListItem = {
  ...DEFAULT_TOOL_DEFINITION,
  app_enabled: true,
  user_config: undefined,
};

// ============================================================================
// Tools List Handlers
// ============================================================================

/**
 * Mock handler for GET /tools (list all available tools)
 */
export function mockAvailableTools(
  tools: ToolListItem[] = [DEFAULT_TOOL_LIST_ITEM],
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(TOOLS_ENDPOINT, () => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: ListToolsResponse = { tools };
      return HttpResponse.json(responseData);
    }),
  ];
}

// ============================================================================
// Tool Config Handlers
// ============================================================================

/**
 * Mock handler for GET /tools/:toolId/config
 */
export function mockToolConfig(
  toolId: string = 'builtin-exa-web-search',
  config: Partial<EnhancedToolConfigResponse> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(`${TOOLS_ENDPOINT}/:toolId/config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: EnhancedToolConfigResponse = {
        tool_id: toolId,
        app_enabled: config.app_enabled ?? true,
        config: {
          tool_id: toolId,
          enabled: config.config?.enabled ?? false,
          created_at: config.config?.created_at ?? '2024-01-01T00:00:00Z',
          updated_at: config.config?.updated_at ?? '2024-01-01T00:00:00Z',
        },
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Mock handler for PUT /tools/:toolId/config (update tool config)
 */
export function mockUpdateToolConfig(
  toolId: string = 'builtin-exa-web-search',
  responseConfig: Partial<EnhancedToolConfigResponse> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.put(`${TOOLS_ENDPOINT}/:toolId/config`, async ({ params, request }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const body = (await request.json()) as { enabled: boolean; api_key?: string };
      const responseData: EnhancedToolConfigResponse = {
        tool_id: toolId,
        app_enabled: responseConfig.app_enabled ?? true,
        config: {
          tool_id: toolId,
          enabled: body.enabled,
          created_at: responseConfig.config?.created_at ?? '2024-01-01T00:00:00Z',
          updated_at: new Date().toISOString(),
        },
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Mock handler for DELETE /tools/:toolId/config (delete tool config / clear API key)
 */
export function mockDeleteToolConfig(toolId: string = 'builtin-exa-web-search', { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    http.delete(`${TOOLS_ENDPOINT}/:toolId/config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return new HttpResponse(null, { status: 204 });
    }),
  ];
}

// ============================================================================
// App-Level Config Handlers (Admin)
// ============================================================================

/**
 * Mock handler for PUT /tools/:toolId/app-config (enable app tool)
 */
export function mockSetAppToolEnabled(toolId: string = 'builtin-exa-web-search', { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    http.put(`${TOOLS_ENDPOINT}/:toolId/app-config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: AppToolConfigResponse = {
        tool_id: toolId,
        enabled: true,
        updated_by: 'admin-user',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: new Date().toISOString(),
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Mock handler for DELETE /tools/:toolId/app-config (disable app tool)
 */
export function mockSetAppToolDisabled(toolId: string = 'builtin-exa-web-search', { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    http.delete(`${TOOLS_ENDPOINT}/:toolId/app-config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: AppToolConfigResponse = {
        tool_id: toolId,
        enabled: false,
        updated_by: 'admin-user',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: new Date().toISOString(),
      };
      return HttpResponse.json(responseData);
    }),
  ];
}

// ============================================================================
// Error Handlers
// ============================================================================

/**
 * Mock error for tools list endpoint
 */
export function mockAvailableToolsError(
  {
    message = 'Failed to fetch tools',
    code = INTERNAL_SERVER_ERROR.code,
    type = INTERNAL_SERVER_ERROR.type,
    status = 500,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(TOOLS_ENDPOINT, () => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

/**
 * Mock error for tool config endpoint
 */
export function mockToolConfigError(
  toolId: string = 'builtin-exa-web-search',
  {
    message = 'Tool not found',
    code = 'not_found',
    type = 'not_found_error',
    status = 404,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(`${TOOLS_ENDPOINT}/:toolId/config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

/**
 * Mock error for update tool config endpoint
 */
export function mockUpdateToolConfigError(
  toolId: string = 'builtin-exa-web-search',
  {
    message = 'Failed to update tool configuration',
    code = INTERNAL_SERVER_ERROR.code,
    type = INTERNAL_SERVER_ERROR.type,
    status = 500,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.put(`${TOOLS_ENDPOINT}/:toolId/config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

/**
 * Mock error for app-level enable/disable
 */
export function mockSetAppToolError(
  toolId: string = 'builtin-exa-web-search',
  method: 'put' | 'delete' = 'put',
  {
    message = 'Admin access required',
    code = 'forbidden',
    type = 'forbidden_error',
    status = 403,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  const handler = method === 'put' ? http.put : http.delete;
  return [
    handler(`${TOOLS_ENDPOINT}/:toolId/app-config`, ({ params }) => {
      if (params.toolId !== toolId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Default mocks for all tools endpoints (happy path)
 */
export function mockToolsDefault() {
  return [
    ...mockAvailableTools(),
    ...mockToolConfig(),
    ...mockUpdateToolConfig(),
    ...mockDeleteToolConfig(),
    ...mockSetAppToolEnabled(),
    ...mockSetAppToolDisabled(),
  ];
}

/**
 * Mock a tool as fully configured and enabled
 */
export function mockToolConfigured(toolId: string = 'builtin-exa-web-search') {
  const toolItem: ToolListItem = {
    ...DEFAULT_TOOL_DEFINITION,
    app_enabled: true,
    user_config: {
      enabled: true,
      has_api_key: true,
    },
  };
  toolItem.function.name = toolId;

  return [
    ...mockAvailableTools([toolItem], { stub: true }),
    ...mockToolConfig(
      toolId,
      {
        app_enabled: true,
        config: {
          tool_id: toolId,
          enabled: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      },
      { stub: true }
    ),
  ];
}

/**
 * Mock a tool as app-disabled
 */
export function mockToolAppDisabled(toolId: string = 'builtin-exa-web-search') {
  const toolItem: ToolListItem = {
    ...DEFAULT_TOOL_DEFINITION,
    app_enabled: false,
    user_config: undefined,
  };
  toolItem.function.name = toolId;

  return [
    ...mockAvailableTools([toolItem], { stub: true }),
    ...mockToolConfig(
      toolId,
      {
        app_enabled: false,
        config: {
          tool_id: toolId,
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      },
      { stub: true }
    ),
  ];
}
