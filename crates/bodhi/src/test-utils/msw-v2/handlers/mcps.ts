/**
 * MSW v2 handlers for MCP endpoints
 */
import { http, HttpResponse } from 'msw';

import type {
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  Mcp,
  McpServerResponse,
  OAuthTokenResponse,
} from '@/hooks/mcps';
import { BODHI_API_BASE } from '@/hooks/useQuery';
import {
  createMockMcpServerInfo,
  createMockMcpServerResponse,
  createMockMcp,
  createMockMcpWithHeaderAuth,
  createMockMcpWithOAuth,
  createMockMcpWithDcr,
  createMockOAuthToken,
  createMockAuthConfigHeader,
  createMockAuthConfigOAuthPreReg,
  createMockAuthConfigOAuthDynamic,
} from '@/test-fixtures/mcps';

// ============================================================================
// Mock Data -- created via fixture factories (single source of truth)
// ============================================================================

export const mockMcpServerInfo = createMockMcpServerInfo();
export const mockMcpServerResponse: McpServerResponse = createMockMcpServerResponse();
export const mockMcp: Mcp = createMockMcp();
export const mockMcpWithHeaderAuth: Mcp = createMockMcpWithHeaderAuth();
export const mockOAuthToken: OAuthTokenResponse = createMockOAuthToken();
export const mockMcpWithOAuth: Mcp = createMockMcpWithOAuth();
export const mockMcpWithDcr: Mcp = createMockMcpWithDcr();
export const mockAuthConfigHeader: McpAuthConfigResponse = createMockAuthConfigHeader();
export const mockAuthConfigOAuthPreReg: McpAuthConfigResponse = createMockAuthConfigOAuthPreReg();
export const mockAuthConfigOAuthDynamic: McpAuthConfigResponse = createMockAuthConfigOAuthDynamic();

// ============================================================================
// Handler Factories - MCP Instance CRUD
// ============================================================================

export function mockListMcps(mcps: Mcp[] = [mockMcp]) {
  return http.get(`${BODHI_API_BASE}/mcps`, () => HttpResponse.json({ mcps }));
}

export function mockGetMcp(mcp: Mcp = mockMcp) {
  return http.get(`${BODHI_API_BASE}/mcps/:id`, () => HttpResponse.json(mcp));
}

export function mockCreateMcp(response: Mcp = mockMcp) {
  return http.post(`${BODHI_API_BASE}/mcps`, () => HttpResponse.json(response, { status: 201 }));
}

export function mockUpdateMcp(response: Mcp = mockMcp) {
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

export function mockUpdateMcpError({
  message = 'Failed to update MCP',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.put(`${BODHI_API_BASE}/mcps/:id`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockDeleteMcpError({
  message = 'Failed to delete MCP',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.delete(`${BODHI_API_BASE}/mcps/:id`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockCreateMcpServerError({
  message = 'Failed to create MCP server',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/servers`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockUpdateMcpServerError({
  message = 'Failed to update MCP server',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.put(`${BODHI_API_BASE}/mcps/servers/:id`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockOAuthLoginError({
  message = 'Failed to initiate OAuth login',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/login`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockOAuthTokenExchangeError({
  message = 'Failed to exchange OAuth token',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/auth-configs/:id/token`, () =>
    HttpResponse.json({ error: { message, code, type } }, { status })
  );
}

export function mockStandaloneDynamicRegisterError({
  message = 'Failed to register dynamic client',
  code = 'internal_server_error',
  type = 'internal_server_error',
  status = 500,
}: { message?: string; code?: string; type?: string; status?: number } = {}) {
  return http.post(`${BODHI_API_BASE}/mcps/oauth/dynamic-register`, () =>
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
];
