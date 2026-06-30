import type { AccessRequestReviewResponse, AppAccessSummary, ListAppAccessResponse } from '@/hooks/apps';

const REQUEST_ID = '550e8400-e29b-41d4-a716-446655440000';
const APP_CLIENT_ID = 'test-app-client';

export const mockAppAccessSummary: AppAccessSummary = {
  id: 'app-grant-1',
  app_client_id: 'research-copilot',
  app_name: 'Research Copilot',
  app_description: 'An app that summarises research papers',
  status: 'approved',
  approved_role: 'scope_user_user',
  models: { type: 'specific', list: true, ids: ['gpt-4o'] },
  mcps: { type: 'specific', list: false, ids: ['mcp-instance-1'] },
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-02T00:00:00Z',
};

export const mockAppAccessSummaryAll: AppAccessSummary = {
  id: 'app-grant-2',
  app_client_id: 'notes-agent',
  app_name: 'Notes Agent',
  app_description: null,
  status: 'approved',
  approved_role: 'scope_user_power_user',
  models: { type: 'all', list: true },
  mcps: { type: 'all', list: true },
  created_at: '2024-01-03T00:00:00Z',
  updated_at: '2024-01-03T00:00:00Z',
};

export const mockAppAccessList: ListAppAccessResponse = {
  data: [mockAppAccessSummary, mockAppAccessSummaryAll],
};

export const mockAppAccessListEmpty: ListAppAccessResponse = { data: [] };

export const mockAppAccessRevoked: AppAccessSummary = { ...mockAppAccessSummary, status: 'revoked' };

export const mockDraftReviewResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: 'A test third-party application',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

export const mockDraftNoInstancesResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: 'A test third-party application',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.example.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.example.com/mcp',
      instances: [],
    },
  ],
};

export const mockApprovedReviewResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: 'A test third-party application',
  flow_type: 'popup',
  status: 'approved',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [],
  },
  mcps_info: [],
};

export const mockFailedReviewResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: null,
  flow_type: 'redirect',
  status: 'failed',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [],
  },
  mcps_info: [],
};

export const mockExpiredReviewResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: null,
  flow_type: 'redirect',
  status: 'expired',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [],
  },
  mcps_info: [],
};

export const mockDeniedReviewResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: null,
  flow_type: 'redirect',
  status: 'denied',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [],
  },
  mcps_info: [],
};

export const mockDraftRedirectResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Redirect App',
  app_description: 'An app using redirect flow',
  flow_type: 'redirect',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

export const mockDraftMultiToolResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Multi-MCP App',
  app_description: 'An app requesting multiple MCP servers',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_power_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }, { url: 'https://mcp.weather.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
    {
      url: 'https://mcp.weather.com/mcp',
      instances: [
        {
          id: 'mcp-instance-2',
          name: 'Weather',
          slug: 'weather-prod',
          path: '/mcp/weather-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-2',
            url: 'https://mcp.weather.com/mcp',
            name: 'Weather MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

export const mockDraftMultiToolMixedResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Mixed MCP App',
  app_description: 'An app with mixed MCP availability',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }, { url: 'https://mcp.calculator.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
    {
      url: 'https://mcp.calculator.com/mcp',
      instances: [],
    },
  ],
};

export const mockDraftMcpResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'MCP App',
  app_description: 'An app requesting MCP access',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

// Draft where the user's instances span multiple server URLs: an exact-URL match (sorted first)
// plus a non-matching instance reached via a gateway. Mirrors the backend's all-instances payload.
export const mockDraftMcpCrossUrlResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'MCP App',
  app_description: 'An app requesting MCP access',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
        {
          id: 'mcp-instance-gw',
          name: 'DeepWiki via Gateway',
          slug: 'deepwiki-gateway',
          path: '/mcp/deepwiki-gateway',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-gw',
            url: 'https://gateway.composio.dev/deepwiki/mcp',
            name: 'Composio Gateway',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

export const mockDraftMixedResourcesResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Mixed Resources App',
  app_description: 'An app requesting MCPs',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

export const mockDraftMcpNoInstancesResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'MCP No Instances App',
  app_description: 'An app requesting MCP with no instances',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.example.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.example.com/mcp',
      instances: [],
    },
  ],
};

/** Draft request where the app asks for the model + MCP grant controls (all four
 *  UI-driver flags on) plus a by-url MCP. Drives the AI Models + Connected Tools sections. */
export const mockDraftWithGrantFlagsResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Grants App',
  app_description: 'An app requesting model and MCP grant controls',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    version: '1' as const,
    models_list: true,
    models_access: true,
    mcps_list: true,
    mcps_access: true,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};
export const mockDraftReviewResponsePowerUser: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Power User App',
  app_description: 'An app requesting power user access',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_power_user',
  requested: {
    version: '1' as const,
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
          path: '/mcp/deepwiki-prod',
          enabled: true,
          mcp_server: {
            id: 'mcp-server-1',
            url: 'https://mcp.deepwiki.com/mcp',
            name: 'DeepWiki MCP',
            enabled: true,
          },
          auth_type: 'public',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
};

export const MOCK_REQUEST_ID = REQUEST_ID;
export const MOCK_APP_CLIENT_ID = APP_CLIENT_ID;
