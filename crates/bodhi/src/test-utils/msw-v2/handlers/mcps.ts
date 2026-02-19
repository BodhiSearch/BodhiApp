/**
 * MSW v2 handlers for MCP endpoints
 */
import { http, HttpResponse } from 'msw';

import type {
  AuthHeaderResponse,
  McpExecuteResponse,
  McpResponse,
  McpServerInfo,
  McpServerResponse,
  McpTool,
  OAuthConfigResponse,
  OAuthConfigsListResponse,
  OAuthTokenResponse,
} from '@/hooks/useMcps';
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
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockAuthHeader: AuthHeaderResponse = {
  id: 'auth-header-uuid-1',
  header_key: 'Authorization',
  has_header_value: true,
  created_by: 'test-user',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockMcpWithHeaderAuth: McpResponse = {
  ...mockMcp,
  id: 'mcp-uuid-2',
  slug: 'header-mcp',
  name: 'Header Auth MCP',
  auth_type: 'header',
  auth_uuid: 'auth-header-uuid-1',
};

export const mockOAuthConfig: OAuthConfigResponse = {
  id: 'oauth-config-uuid-1',
  mcp_server_id: 'server-uuid-1',
  client_id: 'test-client-id',
  authorization_endpoint: 'https://auth.example.com/authorize',
  token_endpoint: 'https://auth.example.com/token',
  scopes: 'mcp:tools mcp:read',
  has_client_secret: true,
  created_by: 'test-user',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockOAuthToken: OAuthTokenResponse = {
  id: 'oauth-token-uuid-1',
  mcp_oauth_config_id: 'oauth-config-uuid-1',
  scopes_granted: 'mcp:tools mcp:read',
  expires_at: Math.floor(Date.now() / 1000) + 3600,
  has_access_token: true,
  has_refresh_token: true,
  created_by: 'test-user',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockMcpWithOAuth: McpResponse = {
  ...mockMcp,
  id: 'mcp-uuid-3',
  slug: 'oauth-mcp',
  name: 'OAuth MCP',
  auth_type: 'oauth-pre-registered',
  auth_uuid: 'oauth-token-uuid-1',
};

// ============================================================================
// Handler Factories - MCP Instance CRUD
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

// ============================================================================
// Handler Factories - MCP Servers
// ============================================================================

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

// ============================================================================
// Handler Factories - Auth Headers CRUD
// ============================================================================

export function mockCreateAuthHeader(response: AuthHeaderResponse = mockAuthHeader) {
  return http.post(`${BODHI_API_BASE}/mcps/auth-headers`, () => HttpResponse.json(response, { status: 201 }));
}

export function mockGetAuthHeader(response: AuthHeaderResponse = mockAuthHeader) {
  return http.get(`${BODHI_API_BASE}/mcps/auth-headers/:id`, () => HttpResponse.json(response));
}

export function mockUpdateAuthHeader(response: AuthHeaderResponse = mockAuthHeader) {
  return http.put(`${BODHI_API_BASE}/mcps/auth-headers/:id`, () => HttpResponse.json(response));
}

export function mockDeleteAuthHeader() {
  return http.delete(`${BODHI_API_BASE}/mcps/auth-headers/:id`, () => new HttpResponse(null, { status: 204 }));
}

export function mockGetAuthHeaderNotFound() {
  return http.get(`${BODHI_API_BASE}/mcps/auth-headers/:id`, () =>
    HttpResponse.json({ error: { message: 'Not found', code: 'not_found', type: 'not_found' } }, { status: 404 })
  );
}

// ============================================================================
// Handler Factories - Tools
// ============================================================================

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
// Handler Factories - OAuth Config CRUD (nested under mcp-servers)
// ============================================================================

export function mockListOAuthConfigs(response: OAuthConfigsListResponse = { oauth_configs: [mockOAuthConfig] }) {
  return http.get(`${BODHI_API_BASE}/mcp-servers/:serverId/oauth-configs`, () => HttpResponse.json(response));
}

export function mockCreateOAuthConfig(response: OAuthConfigResponse = mockOAuthConfig) {
  return http.post(`${BODHI_API_BASE}/mcp-servers/:serverId/oauth-configs`, () =>
    HttpResponse.json(response, { status: 201 })
  );
}

export function mockGetOAuthConfig(response: OAuthConfigResponse = mockOAuthConfig) {
  return http.get(`${BODHI_API_BASE}/mcp-servers/:serverId/oauth-configs/:id`, () => HttpResponse.json(response));
}

export function mockOAuthLogin() {
  return http.post(`${BODHI_API_BASE}/mcp-servers/:serverId/oauth-configs/:id/login`, () =>
    HttpResponse.json({ authorization_url: 'https://auth.example.com/authorize?client_id=test&state=abc123' })
  );
}

export function mockOAuthTokenExchange(response: OAuthTokenResponse = mockOAuthToken) {
  return http.post(`${BODHI_API_BASE}/mcp-servers/:serverId/oauth-configs/:id/token`, () =>
    HttpResponse.json(response)
  );
}

export function mockOAuthDiscover() {
  return http.post(`${BODHI_API_BASE}/mcps/oauth/discover`, () =>
    HttpResponse.json({
      authorization_endpoint: 'https://auth.example.com/authorize',
      token_endpoint: 'https://auth.example.com/token',
      scopes_supported: ['mcp:tools', 'mcp:read'],
    })
  );
}

// ============================================================================
// Handler Factories - OAuth Token CRUD
// ============================================================================

export function mockGetOAuthToken(response: OAuthTokenResponse = mockOAuthToken) {
  return http.get(`${BODHI_API_BASE}/mcps/oauth-tokens/:tokenId`, () => HttpResponse.json(response));
}

export function mockDeleteOAuthToken() {
  return http.delete(`${BODHI_API_BASE}/mcps/oauth-tokens/:tokenId`, () => new HttpResponse(null, { status: 204 }));
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

export function mockCreateOAuthConfigError({
  message = 'Failed to create OAuth config',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcp-servers/:serverId/oauth-configs`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockDeleteOAuthTokenError({
  message = 'Failed to delete token',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.delete(`${BODHI_API_BASE}/mcps/oauth-tokens/:tokenId`, () =>
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
  mockCreateAuthHeader(),
  mockGetAuthHeader(),
  mockUpdateAuthHeader(),
  mockDeleteAuthHeader(),
  mockFetchMcpTools(),
  mockRefreshMcpTools(),
  mockExecuteMcpTool(),
  mockListOAuthConfigs(),
  mockCreateOAuthConfig(),
  mockGetOAuthConfig(),
  mockOAuthLogin(),
  mockOAuthTokenExchange(),
  mockOAuthDiscover(),
  mockGetOAuthToken(),
  mockDeleteOAuthToken(),
];
