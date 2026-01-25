/**
 * MSW v2 handlers for toolsets endpoints - instance-based architecture
 */
import { http, HttpResponse } from 'msw';

import { BODHI_API_BASE } from '@/hooks/useQuery';
import type { ToolsetResponse, ToolsetTypeResponse, AppToolsetConfigResponse } from '@/hooks/useToolsets';

// ============================================================================
// Mock Data
// ============================================================================

export const mockToolset: ToolsetResponse = {
  id: 'uuid-test-toolset',
  name: 'my-exa-search',
  scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
  scope: 'scope_toolset-builtin-exa-web-search',
  description: 'Test toolset',
  enabled: true,
  has_api_key: true,
  tools: [
    {
      type: 'function',
      function: {
        name: 'search',
        description: 'Search the web using Exa AI',
        parameters: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'The search query' },
          },
          required: ['query'],
        },
      },
    },
  ],
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockToolsetTypes = [
  {
    scope: 'scope_toolset-builtin-exa-web-search',
    scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
    enabled: true,
    updated_by: 'system',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

export const mockType: ToolsetTypeResponse = {
  scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
  scope: 'scope_toolset-builtin-exa-web-search',
  name: 'Exa Web Search',
  description: 'Search the web using Exa AI',
  app_enabled: true,
  tools: [
    {
      type: 'function',
      function: {
        name: 'search',
        description: 'Search the web using Exa AI',
        parameters: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'The search query' },
          },
          required: ['query'],
        },
      },
    },
  ],
};

// ============================================================================
// Handler Factories
// ============================================================================

/**
 * Mock GET /bodhi/v1/toolsets - List user's toolsets
 */
export function mockListToolsets(toolsets: ToolsetResponse[] = [mockToolset], toolset_types = mockToolsetTypes) {
  return http.get(`${BODHI_API_BASE}/toolsets`, () => HttpResponse.json({ toolsets, toolset_types }));
}

/**
 * Mock GET /bodhi/v1/toolsets/:id - Get single toolset
 */
export function mockGetToolset(toolset: ToolsetResponse = mockToolset) {
  return http.get(`${BODHI_API_BASE}/toolsets/:id`, () => HttpResponse.json(toolset));
}

/**
 * Mock POST /bodhi/v1/toolsets - Create new toolset
 */
export function mockCreateToolset(response: ToolsetResponse = mockToolset) {
  return http.post(`${BODHI_API_BASE}/toolsets`, () => HttpResponse.json(response, { status: 201 }));
}

/**
 * Mock PUT /bodhi/v1/toolsets/:id - Update toolset
 */
export function mockUpdateToolset(response: ToolsetResponse = mockToolset) {
  return http.put(`${BODHI_API_BASE}/toolsets/:id`, () => HttpResponse.json(response));
}

/**
 * Mock DELETE /bodhi/v1/toolsets/:id - Delete toolset
 */
export function mockDeleteToolset() {
  return http.delete(`${BODHI_API_BASE}/toolsets/:id`, () => new HttpResponse(null, { status: 204 }));
}

/**
 * Mock GET /bodhi/v1/toolset_types - List toolset types
 */
export function mockListTypes(types: ToolsetTypeResponse[] = [mockType]) {
  return http.get(`${BODHI_API_BASE}/toolset_types`, () => HttpResponse.json({ types }));
}

/**
 * Mock PUT /bodhi/v1/toolset_types/:scope/app-config - Enable toolset type (admin)
 */
export function mockEnableType(response?: AppToolsetConfigResponse) {
  const defaultResponse: AppToolsetConfigResponse = {
    scope: 'scope_toolset-builtin-exa-web-search',
    scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
    enabled: true,
    updated_by: 'admin123',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  };
  return http.put(`${BODHI_API_BASE}/toolset_types/:scope/app-config`, () =>
    HttpResponse.json(response || defaultResponse)
  );
}

/**
 * Mock DELETE /bodhi/v1/toolset_types/:scope/app-config - Disable toolset type (admin)
 */
export function mockDisableType(response?: AppToolsetConfigResponse) {
  const defaultResponse: AppToolsetConfigResponse = {
    scope: 'scope_toolset-builtin-exa-web-search',
    scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
    enabled: false,
    updated_by: 'admin123',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  };
  return http.delete(`${BODHI_API_BASE}/toolset_types/:scope/app-config`, () =>
    HttpResponse.json(response || defaultResponse)
  );
}

// ============================================================================
// Error Handlers
// ============================================================================

/**
 * Mock error for toolsets list endpoint
 */
export function mockListToolsetsError({
  message = 'Failed to fetch toolsets',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.get(`${BODHI_API_BASE}/toolsets`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

/**
 * Mock error for get toolset endpoint
 */
export function mockGetToolsetError({
  message = 'Toolset not found',
  code = 'not_found',
  type = 'not_found_error',
  status = 404,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.get(`${BODHI_API_BASE}/toolsets/:id`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

/**
 * Mock error for create toolset endpoint
 */
export function mockCreateToolsetError({
  message = 'Failed to create toolset',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/toolsets`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

/**
 * Mock error for update toolset endpoint
 */
export function mockUpdateToolsetError({
  message = 'Failed to update toolset',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.put(`${BODHI_API_BASE}/toolsets/:id`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

// ============================================================================
// Default Handlers
// ============================================================================

export const toolsetsHandlers = [
  mockListToolsets(),
  mockGetToolset(),
  mockCreateToolset(),
  mockUpdateToolset(),
  mockDeleteToolset(),
  mockListTypes(),
  mockEnableType(),
  mockDisableType(),
];
