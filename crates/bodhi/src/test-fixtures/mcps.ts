/**
 * Fixture factories for MCP-related mock data.
 *
 * Uses types from hooks/mcps for consistency with MSW handlers.
 * All factories accept optional overrides and return fresh objects per call.
 */
import type {
  McpAuthConfigResponse,
  McpAuthConfigsListResponse,
  Mcp,
  McpServerInfo,
  McpServerResponse,
  OAuthTokenResponse,
} from '@/hooks/mcps';

// Extract discriminated union variants for type-safe overrides
type HeaderAuthConfig = Extract<McpAuthConfigResponse, { type: 'header' }>;
type OAuthAuthConfig = Extract<McpAuthConfigResponse, { type: 'oauth' }>;

// ============================================================================
// MCP Server Factories
// ============================================================================

/**
 * Create a mock MCP server info (lightweight reference)
 */
export function createMockMcpServerInfo(overrides?: Partial<McpServerInfo>): McpServerInfo {
  return {
    id: 'server-uuid-1',
    url: 'https://mcp.example.com/mcp',
    name: 'Example Server',
    enabled: true,
    ...overrides,
  };
}

/**
 * Create a mock MCP server response (full server details)
 */
export function createMockMcpServerResponse(overrides?: Partial<McpServerResponse>): McpServerResponse {
  return {
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
    ...overrides,
  };
}

// ============================================================================
// MCP Instance Factories
// ============================================================================

/**
 * Create a mock MCP instance (public auth)
 */
export function createMockMcp(overrides?: Partial<Mcp>): Mcp {
  return {
    id: 'mcp-uuid-1',
    mcp_server: createMockMcpServerInfo(),
    slug: 'example-mcp',
    name: 'Example MCP',
    description: 'An example MCP server',
    enabled: true,
    mcp_endpoint: '/bodhi/v1/apps/mcps/mcp-uuid-1/mcp',
    auth_type: 'public',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

/**
 * Create a mock MCP instance with header auth
 */
export function createMockMcpWithHeaderAuth(overrides?: Partial<Mcp>): Mcp {
  return createMockMcp({
    id: 'mcp-uuid-2',
    slug: 'header-mcp',
    name: 'Header Auth MCP',
    mcp_endpoint: '/bodhi/v1/apps/mcps/mcp-uuid-2/mcp',
    auth_type: 'header',
    auth_config_id: 'auth-header-uuid-1',
    ...overrides,
  });
}

/**
 * Create a mock MCP instance with OAuth auth (pre-registered)
 */
export function createMockMcpWithOAuth(overrides?: Partial<Mcp>): Mcp {
  return createMockMcp({
    id: 'mcp-uuid-3',
    slug: 'oauth-mcp',
    name: 'OAuth MCP',
    mcp_endpoint: '/bodhi/v1/apps/mcps/mcp-uuid-3/mcp',
    auth_type: 'oauth',
    auth_config_id: 'oauth-config-uuid-1',
    ...overrides,
  });
}

/**
 * Create a mock MCP instance with OAuth DCR auth
 */
export function createMockMcpWithDcr(overrides?: Partial<Mcp>): Mcp {
  return createMockMcp({
    id: 'mcp-uuid-4',
    slug: 'dcr-mcp',
    name: 'DCR MCP',
    mcp_endpoint: '/bodhi/v1/apps/mcps/mcp-uuid-4/mcp',
    auth_type: 'oauth',
    auth_config_id: 'oauth-config-dcr-uuid-1',
    ...overrides,
  });
}

// ============================================================================
// OAuth Token Factories
// ============================================================================

/**
 * Create a mock OAuth token response
 */
export function createMockOAuthToken(overrides?: Partial<OAuthTokenResponse>): OAuthTokenResponse {
  return {
    id: 'oauth-token-uuid-1',
    mcp_id: 'mcp-uuid-3',
    auth_config_id: 'oauth-config-uuid-1',
    scopes_granted: 'mcp:tools mcp:read',
    expires_at: Math.floor(Date.now() / 1000) + 3600,
    has_refresh_token: true,
    user_id: 'test-user',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

// ============================================================================
// Auth Config Factories
// ============================================================================

/**
 * Create a mock header auth config
 */
export function createMockAuthConfigHeader(overrides?: Partial<HeaderAuthConfig>): HeaderAuthConfig {
  return {
    id: 'auth-header-uuid-1',
    name: 'Header',
    mcp_server_id: 'server-uuid-1',
    created_by: 'admin',
    entries: [{ id: 'entry-1', param_type: 'header', param_key: 'Authorization' }],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    type: 'header',
    ...overrides,
  };
}

/**
 * Create a mock OAuth pre-registered auth config
 */
export function createMockAuthConfigOAuthPreReg(overrides?: Partial<OAuthAuthConfig>): OAuthAuthConfig {
  return {
    id: 'oauth-config-uuid-1',
    name: 'OAuth Pre-Registered',
    mcp_server_id: 'server-uuid-1',
    created_by: 'admin',
    registration_type: 'pre_registered',
    client_id: 'test-client-id',
    authorization_endpoint: 'https://auth.example.com/authorize',
    token_endpoint: 'https://auth.example.com/token',
    scopes: 'mcp:tools mcp:read',
    has_client_secret: true,
    has_registration_access_token: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    type: 'oauth',
    ...overrides,
  };
}

/**
 * Create a mock OAuth dynamic registration auth config
 */
export function createMockAuthConfigOAuthDynamic(overrides?: Partial<OAuthAuthConfig>): OAuthAuthConfig {
  return {
    id: 'oauth-config-dcr-uuid-1',
    name: 'OAuth Dynamic',
    mcp_server_id: 'server-uuid-1',
    created_by: 'admin',
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
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    type: 'oauth',
    ...overrides,
  };
}

/**
 * Create a mock auth configs list response
 */
export function createMockAuthConfigsList(overrides?: Partial<McpAuthConfigsListResponse>): McpAuthConfigsListResponse {
  return {
    auth_configs: [createMockAuthConfigHeader()],
    ...overrides,
  };
}
