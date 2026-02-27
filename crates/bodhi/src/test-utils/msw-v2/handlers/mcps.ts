/**
 * MSW v2 handlers for MCP endpoints
 */
import { http, HttpResponse } from 'msw';

import type {
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  McpExecuteResponse,
  McpResponse,
  McpServerInfo,
  McpServerResponse,
  McpTool,
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

export const mockMcpWithHeaderAuth: McpResponse = {
  ...mockMcp,
  id: 'mcp-uuid-2',
  slug: 'header-mcp',
  name: 'Header Auth MCP',
  auth_type: 'header',
  auth_uuid: 'auth-header-uuid-1',
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
  auth_type: 'oauth',
  auth_uuid: 'oauth-token-uuid-1',
};

export const mockMcpWithDcr: McpResponse = {
  ...mockMcp,
  id: 'mcp-uuid-4',
  slug: 'dcr-mcp',
  name: 'DCR MCP',
  auth_type: 'oauth',
  auth_uuid: 'oauth-token-uuid-2',
};

// ============================================================================
// Mock Data - Unified Auth Configs
// ============================================================================

export const mockAuthConfigHeader: McpAuthConfigResponse = {
  id: 'auth-header-uuid-1',
  name: 'Header',
  mcp_server_id: 'server-uuid-1',
  header_key: 'Authorization',
  has_header_value: true,
  created_by: 'test-user',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  type: 'header',
};

export const mockAuthConfigOAuthPreReg: McpAuthConfigResponse = {
  id: 'oauth-config-uuid-1',
  name: 'OAuth Pre-Registered',
  mcp_server_id: 'server-uuid-1',
  registration_type: 'pre_registered',
  client_id: 'test-client-id',
  authorization_endpoint: 'https://auth.example.com/authorize',
  token_endpoint: 'https://auth.example.com/token',
  scopes: 'mcp:tools mcp:read',
  has_client_secret: true,
  has_registration_access_token: false,
  created_by: 'test-user',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  type: 'oauth',
};

export const mockAuthConfigOAuthDynamic: McpAuthConfigResponse = {
  id: 'oauth-config-dcr-uuid-1',
  name: 'OAuth Dynamic',
  mcp_server_id: 'server-uuid-1',
  registration_type: 'dynamic_registration',
  client_id: 'dcr-client-id-123',
  authorization_endpoint: 'https://auth.example.com/authorize',
  token_endpoint: 'https://auth.example.com/token',
  registration_endpoint: 'https://auth.example.com/register',
  scopes: 'mcp:tools mcp:read',
  client_id_issued_at: null,
  token_endpoint_auth_method: null,
  has_client_secret: false,
  has_registration_access_token: true,
  created_by: 'test-user',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  type: 'oauth',
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
  return http.get(`${BODHI_API_BASE}/mcps/servers`, () => HttpResponse.json({ mcp_servers: servers }));
}

export function mockGetMcpServer(server: McpServerResponse = mockMcpServerResponse) {
  return http.get(`${BODHI_API_BASE}/mcps/servers/:id`, () => HttpResponse.json(server));
}

export function mockCreateMcpServer(response: McpServerResponse = mockMcpServerResponse) {
  return http.post(`${BODHI_API_BASE}/mcps/servers`, () => HttpResponse.json(response, { status: 201 }));
}

export function mockUpdateMcpServer(response: McpServerResponse = mockMcpServerResponse) {
  return http.put(`${BODHI_API_BASE}/mcps/servers/:id`, () => HttpResponse.json(response));
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
// Handler Factories - OAuth Discovery
// ============================================================================

export function mockDiscoverAs() {
  return http.post(`${BODHI_API_BASE}/mcps/oauth/discover-as`, () =>
    HttpResponse.json({
      authorization_endpoint: 'https://auth.example.com/authorize',
      token_endpoint: 'https://auth.example.com/token',
      scopes_supported: ['mcp:tools', 'mcp:read'],
    })
  );
}

export function mockDiscoverMcp(
  response: {
    authorization_endpoint?: string;
    token_endpoint?: string;
    registration_endpoint?: string;
    scopes_supported?: string[];
    resource?: string;
    authorization_server_url?: string;
  } = {}
) {
  const defaultResponse = {
    authorization_endpoint: 'https://auth.example.com/authorize',
    token_endpoint: 'https://auth.example.com/token',
    registration_endpoint: 'https://auth.example.com/register',
    scopes_supported: ['mcp:tools', 'mcp:read'],
    resource: 'https://mcp.example.com',
    authorization_server_url: 'https://auth.example.com',
  };

  return http.post(`${BODHI_API_BASE}/mcps/oauth/discover-mcp`, () =>
    HttpResponse.json({ ...defaultResponse, ...response })
  );
}

export function mockDiscoverMcpError({
  message = 'Failed to discover MCP OAuth endpoints',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/oauth/discover-mcp`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockStandaloneDynamicRegister(
  response: {
    client_id: string;
    client_secret?: string;
    token_endpoint_auth_method?: string;
    client_id_issued_at?: number;
    registration_access_token?: string;
  } = {
    client_id: 'mock-client-id',
    client_secret: 'mock-client-secret',
    token_endpoint_auth_method: 'client_secret_post',
  }
) {
  return http.post(`${BODHI_API_BASE}/mcps/oauth/dynamic-register`, () => HttpResponse.json(response));
}

/** @deprecated Use mockDiscoverAs instead */
export function mockOAuthDiscover() {
  return mockDiscoverAs();
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
// Handler Factories - Unified Auth Configs
// ============================================================================

export function mockListAuthConfigs(response: McpAuthConfigsListResponse = { auth_configs: [mockAuthConfigHeader] }) {
  return http.get(`${BODHI_API_BASE}/mcps/auth-configs`, () => HttpResponse.json(response));
}

export function mockCreateAuthConfig(response: McpAuthConfigResponse = mockAuthConfigHeader) {
  return http.post(`${BODHI_API_BASE}/mcps/auth-configs`, () => HttpResponse.json(response, { status: 201 }));
}

export function mockGetAuthConfig(response: McpAuthConfigResponse = mockAuthConfigHeader) {
  return http.get(`${BODHI_API_BASE}/mcps/auth-configs/:configId`, () => HttpResponse.json(response));
}

export function mockDeleteAuthConfig() {
  return http.delete(`${BODHI_API_BASE}/mcps/auth-configs/:configId`, () => new HttpResponse(null, { status: 204 }));
}

export function mockCreateAuthConfigError({
  message = 'Failed to create auth config',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/auth-configs`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockDeleteAuthConfigError({
  message = 'Failed to delete auth config',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.delete(`${BODHI_API_BASE}/mcps/auth-configs/:configId`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

// ============================================================================
// Handler Factories - OAuth Login & Token Exchange (under auth-configs)
// ============================================================================

export function mockOAuthLogin() {
  return http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/login`, () =>
    HttpResponse.json({ authorization_url: 'https://auth.example.com/authorize?client_id=test&state=abc123' })
  );
}

export function mockOAuthTokenExchange(response: OAuthTokenResponse = mockOAuthToken) {
  return http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/token`, () => HttpResponse.json(response));
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

// IMPORTANT: Sub-path handlers (e.g. GET /mcps/servers, GET /mcps/auth-configs)
// must be registered BEFORE wildcard handlers (e.g. GET /mcps/:id) because MSW
// matches handlers in registration order and :id would match "servers", "auth-configs", etc.
export const mcpsHandlers = [
  mockListMcps(),
  // --- Sub-path GET routes must come before GET /mcps/:id ---
  mockListMcpServers(),
  mockGetMcpServer(),
  mockCreateMcpServer(),
  mockUpdateMcpServer(),
  mockFetchMcpTools(),
  mockDiscoverAs(),
  mockDiscoverMcp(),
  mockStandaloneDynamicRegister(),
  mockGetOAuthToken(),
  mockDeleteOAuthToken(),
  mockListAuthConfigs(),
  mockCreateAuthConfig(),
  mockGetAuthConfig(),
  mockDeleteAuthConfig(),
  mockOAuthLogin(),
  mockOAuthTokenExchange(),
  // --- Wildcard /mcps/:id routes last ---
  mockGetMcp(),
  mockCreateMcp(),
  mockUpdateMcp(),
  mockDeleteMcp(),
  mockRefreshMcpTools(),
  mockExecuteMcpTool(),
];
