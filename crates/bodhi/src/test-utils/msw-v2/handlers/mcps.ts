/**
 * MSW v2 handlers for MCP endpoints
 */
import { http, HttpResponse } from 'msw';

import type { McpExecuteResponse, McpResponse, McpServerInfo, McpServerResponse, McpTool } from '@/hooks/useMcps';
import { BODHI_API_BASE } from '@/hooks/useQuery';

// ============================================================================
// Mock Data
// ============================================================================

export const mockMcpTool: McpTool = {
  name: 'read_wiki_structure',
  description: 'Read the structure of a wiki',
  input_schema: {
    type: 'object',
    properties: {
      repo_name: { type: 'string', description: 'Repository name' },
    },
    required: ['repo_name'],
  },
};

export const mockMcpServerInfo: McpServerInfo = {
  id: 'server-uuid-1',
  url: 'https://mcp.example.com/mcp',
  name: 'Example Server',
  enabled: true,
};

export const mockMcpServerResponse: McpServerResponse = {
  id: 'server-uuid-1',
  url: 'https://mcp.example.com/mcp',
  name: 'Example Server',
  description: 'An example MCP server',
  enabled: true,
  created_by: 'admin',
  updated_by: 'admin',
  enabled_mcp_count: 1,
  disabled_mcp_count: 0,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockMcp: McpResponse = {
  id: 'mcp-uuid-1',
  mcp_server: mockMcpServerInfo,
  slug: 'example-mcp',
  name: 'Example MCP',
  description: 'An example MCP server',
  enabled: true,
  tools_cache: [mockMcpTool],
  tools_filter: ['read_wiki_structure'],
  auth_type: 'public',
  has_auth_header_value: false,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockMcpWithHeaderAuth: McpResponse = {
  ...mockMcp,
  id: 'mcp-uuid-2',
  slug: 'header-mcp',
  name: 'Header Auth MCP',
  auth_type: 'header',
  auth_header_key: 'Authorization',
  has_auth_header_value: true,
};

// ============================================================================
// Handler Factories
// ============================================================================

export function mockListMcps(mcps: McpResponse[] = [mockMcp]) {
  return http.get(`${BODHI_API_BASE}/mcps`, () => HttpResponse.json({ mcps }));
}

export function mockGetMcp(mcp: McpResponse = mockMcp) {
  return http.get(`${BODHI_API_BASE}/mcps/:id`, () => HttpResponse.json(mcp));
}

export function mockCreateMcp(response: McpResponse = mockMcp) {
  return http.post(`${BODHI_API_BASE}/mcps`, () => HttpResponse.json(response, { status: 201 }));
}

export function mockUpdateMcp(response: McpResponse = mockMcp) {
  return http.put(`${BODHI_API_BASE}/mcps/:id`, () => HttpResponse.json(response));
}

export function mockDeleteMcp() {
  return http.delete(`${BODHI_API_BASE}/mcps/:id`, () => new HttpResponse(null, { status: 204 }));
}

export function mockListMcpServers(servers: McpServerResponse[] = [mockMcpServerResponse]) {
  return http.get(`${BODHI_API_BASE}/mcp_servers`, () => HttpResponse.json({ mcp_servers: servers }));
}

export function mockGetMcpServer(server: McpServerResponse = mockMcpServerResponse) {
  return http.get(`${BODHI_API_BASE}/mcp_servers/:id`, () => HttpResponse.json(server));
}

export function mockCreateMcpServer(response: McpServerResponse = mockMcpServerResponse) {
  return http.post(`${BODHI_API_BASE}/mcp_servers`, () => HttpResponse.json(response, { status: 201 }));
}

export function mockUpdateMcpServer(response: McpServerResponse = mockMcpServerResponse) {
  return http.put(`${BODHI_API_BASE}/mcp_servers/:id`, () => HttpResponse.json(response));
}

export function mockFetchMcpTools(tools: McpTool[] = [mockMcpTool]) {
  return http.post(`${BODHI_API_BASE}/mcps/fetch-tools`, () => HttpResponse.json({ tools }));
}

export function mockFetchMcpToolsError({
  message = 'Failed to fetch tools',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/fetch-tools`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockRefreshMcpTools(tools: McpTool[] = [mockMcpTool]) {
  return http.post(`${BODHI_API_BASE}/mcps/:id/tools/refresh`, () => HttpResponse.json({ tools }));
}

export function mockExecuteMcpTool(response: McpExecuteResponse = { result: { data: 'test' } }) {
  return http.post(`${BODHI_API_BASE}/mcps/:id/tools/:tool_name/execute`, () => HttpResponse.json(response));
}

// ============================================================================
// Error Handlers
// ============================================================================

export function mockListMcpsError({
  message = 'Failed to fetch MCPs',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.get(`${BODHI_API_BASE}/mcps`, () => HttpResponse.json({ error: { message, code, type } }, { status }));
}

export function mockCreateMcpError({
  message = 'Failed to create MCP',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps`, () => HttpResponse.json({ error: { message, code, type } }, { status }));
}

export function mockExecuteMcpToolError({
  message = 'Tool not allowed',
  code = 'bad_request',
  type = 'bad_request',
  status = 400,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/:id/tools/:tool_name/execute`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

// ============================================================================
// Default Handlers
// ============================================================================

export const mcpsHandlers = [
  mockListMcps(),
  mockGetMcp(),
  mockCreateMcp(),
  mockUpdateMcp(),
  mockDeleteMcp(),
  mockListMcpServers(),
  mockGetMcpServer(),
  mockCreateMcpServer(),
  mockUpdateMcpServer(),
  mockFetchMcpTools(),
  mockRefreshMcpTools(),
  mockExecuteMcpTool(),
];
