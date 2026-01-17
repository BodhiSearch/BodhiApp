/**
 * MSW v2 handlers for toolsets endpoints
 */
import {
  AppToolsetConfigResponse,
  EnhancedToolsetConfigResponse,
  ExecuteToolsetRequest,
  ListToolsetsResponse,
  ToolsetExecutionResponse,
  ToolsetWithTools,
} from '@bodhiapp/ts-client';

import { TOOLSETS_ENDPOINT } from '@/hooks/useToolsets';

import { http, HttpResponse, INTERNAL_SERVER_ERROR } from '../setup';

// ============================================================================
// Default Test Data
// ============================================================================

const DEFAULT_TOOLSET_WITH_TOOLS: ToolsetWithTools = {
  toolset_id: 'builtin-exa-web-search',
  name: 'Exa Web Search',
  description: 'Search the web using Exa AI',
  app_enabled: true,
  user_config: undefined,
  tools: [
    {
      type: 'function',
      function: {
        name: 'search',
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
    },
  ],
};

// ============================================================================
// Toolsets List Handlers
// ============================================================================

/**
 * Mock handler for GET /toolsets (list all available toolsets)
 */
export function mockAvailableToolsets(
  toolsets: ToolsetWithTools[] = [DEFAULT_TOOLSET_WITH_TOOLS],
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(TOOLSETS_ENDPOINT, () => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: ListToolsetsResponse = { toolsets };
      return HttpResponse.json(responseData);
    }),
  ];
}

// ============================================================================
// Toolset Config Handlers
// ============================================================================

/**
 * Mock handler for GET /toolsets/:toolsetId/config
 */
export function mockToolsetConfig(
  toolsetId: string = 'builtin-exa-web-search',
  config: Partial<EnhancedToolsetConfigResponse> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(`${TOOLSETS_ENDPOINT}/:toolsetId/config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: EnhancedToolsetConfigResponse = {
        toolset_id: toolsetId,
        app_enabled: config.app_enabled ?? true,
        config: {
          toolset_id: toolsetId,
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
 * Mock handler for PUT /toolsets/:toolsetId/config (update toolset config)
 */
export function mockUpdateToolsetConfig(
  toolsetId: string = 'builtin-exa-web-search',
  responseConfig: Partial<EnhancedToolsetConfigResponse> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.put(`${TOOLSETS_ENDPOINT}/:toolsetId/config`, async ({ params, request }) => {
      if (params.toolsetId !== toolsetId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const body = (await request.json()) as { enabled: boolean; api_key?: string };
      const responseData: EnhancedToolsetConfigResponse = {
        toolset_id: toolsetId,
        app_enabled: responseConfig.app_enabled ?? true,
        config: {
          toolset_id: toolsetId,
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
 * Mock handler for DELETE /toolsets/:toolsetId/config (delete toolset config / clear API key)
 */
export function mockDeleteToolsetConfig(
  toolsetId: string = 'builtin-exa-web-search',
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.delete(`${TOOLSETS_ENDPOINT}/:toolsetId/config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
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
 * Mock handler for PUT /toolsets/:toolsetId/app-config (enable app toolset)
 */
export function mockSetAppToolsetEnabled(
  toolsetId: string = 'builtin-exa-web-search',
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.put(`${TOOLSETS_ENDPOINT}/:toolsetId/app-config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: AppToolsetConfigResponse = {
        toolset_id: toolsetId,
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
 * Mock handler for DELETE /toolsets/:toolsetId/app-config (disable app toolset)
 */
export function mockSetAppToolsetDisabled(
  toolsetId: string = 'builtin-exa-web-search',
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.delete(`${TOOLSETS_ENDPOINT}/:toolsetId/app-config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: AppToolsetConfigResponse = {
        toolset_id: toolsetId,
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
// Tool Execution Handlers
// ============================================================================

/**
 * Mock handler for POST /toolsets/:toolsetId/execute/:method (execute tool method)
 */
export function mockToolsetExecute(
  toolsetId: string = 'builtin-exa-web-search',
  method: string = 'search',
  responseOverride?: Partial<ToolsetExecutionResponse> | ((req: ExecuteToolsetRequest) => ToolsetExecutionResponse),
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.post(`${TOOLSETS_ENDPOINT}/:toolsetId/execute/:method`, async ({ params, request }) => {
      if (params.toolsetId !== toolsetId || params.method !== method) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const body = (await request.json()) as ExecuteToolsetRequest;

      let responseData: ToolsetExecutionResponse;
      if (typeof responseOverride === 'function') {
        responseData = responseOverride(body);
      } else {
        responseData = {
          tool_call_id: body.tool_call_id,
          result: { success: true, data: 'Mock result' },
          ...responseOverride,
        };
      }

      return HttpResponse.json(responseData);
    }),
  ];
}

/**
 * Mock handler for tool execution that returns an error
 */
export function mockToolsetExecuteError(
  toolsetId: string = 'builtin-exa-web-search',
  method: string = 'search',
  { errorMessage = 'Tool execution failed', status = 500 }: { errorMessage?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.post(`${TOOLSETS_ENDPOINT}/:toolsetId/execute/:method`, async ({ params, request }) => {
      if (params.toolsetId !== toolsetId || params.method !== method) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const body = (await request.json()) as ExecuteToolsetRequest;

      const responseData: ToolsetExecutionResponse = {
        tool_call_id: body.tool_call_id,
        error: errorMessage,
      };

      return HttpResponse.json(responseData, { status });
    }),
  ];
}

// ============================================================================
// Error Handlers
// ============================================================================

/**
 * Mock error for toolsets list endpoint
 */
export function mockAvailableToolsetsError(
  {
    message = 'Failed to fetch toolsets',
    code = INTERNAL_SERVER_ERROR.code,
    type = INTERNAL_SERVER_ERROR.type,
    status = 500,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(TOOLSETS_ENDPOINT, () => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

/**
 * Mock error for toolset config endpoint
 */
export function mockToolsetConfigError(
  toolsetId: string = 'builtin-exa-web-search',
  {
    message = 'Toolset not found',
    code = 'not_found',
    type = 'not_found_error',
    status = 404,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.get(`${TOOLSETS_ENDPOINT}/:toolsetId/config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

/**
 * Mock error for update toolset config endpoint
 */
export function mockUpdateToolsetConfigError(
  toolsetId: string = 'builtin-exa-web-search',
  {
    message = 'Failed to update toolset configuration',
    code = INTERNAL_SERVER_ERROR.code,
    type = INTERNAL_SERVER_ERROR.type,
    status = 500,
  }: { message?: string; code?: string; type?: string; status?: number } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    http.put(`${TOOLSETS_ENDPOINT}/:toolsetId/config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return HttpResponse.json({ error: { message, code, type } }, { status });
    }),
  ];
}

/**
 * Mock error for app-level enable/disable
 */
export function mockSetAppToolsetError(
  toolsetId: string = 'builtin-exa-web-search',
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
    handler(`${TOOLSETS_ENDPOINT}/:toolsetId/app-config`, ({ params }) => {
      if (params.toolsetId !== toolsetId) return;
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
 * Default mocks for all toolsets endpoints (happy path)
 */
export function mockToolsetsDefault() {
  return [
    ...mockAvailableToolsets(),
    ...mockToolsetConfig(),
    ...mockUpdateToolsetConfig(),
    ...mockDeleteToolsetConfig(),
    ...mockSetAppToolsetEnabled(),
    ...mockSetAppToolsetDisabled(),
  ];
}

/**
 * Mock a toolset as fully configured and enabled
 */
export function mockToolsetConfigured(toolsetId: string = 'builtin-exa-web-search') {
  const toolsetItem: ToolsetWithTools = {
    ...DEFAULT_TOOLSET_WITH_TOOLS,
    toolset_id: toolsetId,
    app_enabled: true,
    user_config: {
      enabled: true,
      has_api_key: true,
    },
  };

  return [
    ...mockAvailableToolsets([toolsetItem], { stub: true }),
    ...mockToolsetConfig(
      toolsetId,
      {
        app_enabled: true,
        config: {
          toolset_id: toolsetId,
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
 * Mock a toolset as app-disabled
 */
export function mockToolsetAppDisabled(toolsetId: string = 'builtin-exa-web-search') {
  const toolsetItem: ToolsetWithTools = {
    ...DEFAULT_TOOLSET_WITH_TOOLS,
    toolset_id: toolsetId,
    app_enabled: false,
    user_config: undefined,
  };

  return [
    ...mockAvailableToolsets([toolsetItem], { stub: true }),
    ...mockToolsetConfig(
      toolsetId,
      {
        app_enabled: false,
        config: {
          toolset_id: toolsetId,
          enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      },
      { stub: true }
    ),
  ];
}
