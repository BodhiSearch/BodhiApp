import { BODHI_API_BASE } from '@/hooks/constants';

export const mcpKeys = {
  all: ['mcps'] as const,
  lists: () => [...mcpKeys.all, 'list'] as const,
  list: (params?: { enabled?: boolean }) => {
    const base = mcpKeys.lists();
    return params?.enabled !== undefined ? ([...base, String(params.enabled)] as const) : base;
  },
  details: () => [...mcpKeys.all, 'detail'] as const,
  detail: (id: string) => [...mcpKeys.details(), id] as const,
};

export const mcpServerKeys = {
  all: ['mcp_servers'] as const,
  lists: () => [...mcpServerKeys.all, 'list'] as const,
  list: (params?: { enabled?: boolean }) => {
    const base = mcpServerKeys.lists();
    return params?.enabled !== undefined ? ([...base, String(params.enabled)] as const) : base;
  },
  details: () => [...mcpServerKeys.all, 'detail'] as const,
  detail: (id: string) => [...mcpServerKeys.details(), id] as const,
};

export const authConfigKeys = {
  all: ['auth-configs'] as const,
  lists: () => [...authConfigKeys.all, 'list'] as const,
  list: (serverId?: string) => (serverId ? ([...authConfigKeys.lists(), serverId] as const) : authConfigKeys.lists()),
  details: () => [...authConfigKeys.all, 'detail'] as const,
  detail: (id: string) => [...authConfigKeys.details(), id] as const,
};

export const oauthTokenKeys = {
  all: ['oauth-tokens'] as const,
  detail: (tokenId: string) => [...oauthTokenKeys.all, tokenId] as const,
};

// Endpoint constants (consolidated from hook files)
export const MCPS_ENDPOINT = `${BODHI_API_BASE}/mcps`;
export const MCPS_FETCH_TOOLS_ENDPOINT = `${BODHI_API_BASE}/mcps/fetch-tools`;
export const MCP_SERVERS_ENDPOINT = `${BODHI_API_BASE}/mcps/servers`;
export const MCPS_AUTH_CONFIGS_ENDPOINT = `${BODHI_API_BASE}/mcps/auth-configs`;
export const MCPS_OAUTH_TOKENS_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth-tokens`;
export const MCPS_OAUTH_DISCOVER_MCP_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth/discover-mcp`;
export const MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE_ENDPOINT = `${BODHI_API_BASE}/mcps/oauth/dynamic-register`;
