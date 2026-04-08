import type { AccessRequestReviewResponse } from '@/hooks/apps';

const REQUEST_ID = '550e8400-e29b-41d4-a716-446655440000';
const APP_CLIENT_ID = 'test-app-client';

// Draft review with MCP servers
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

// Draft review with no MCP instances
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

// Already approved
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

// Failed status
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

// Expired status
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

// Denied status
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

// Draft with redirect flow (for testing redirect behavior)
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

// Draft with multiple MCP servers
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

// Draft with mixed MCP availability (one with instances, one without)
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

// Draft with MCP servers requested
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

// Draft with both toolsets and MCP servers (now MCP-only)
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

// Draft with MCP but no instances available
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

// Draft review with power_user requested_role (for testing role downgrade)
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

// Convenience constant for test IDs
export const MOCK_REQUEST_ID = REQUEST_ID;
export const MOCK_APP_CLIENT_ID = APP_CLIENT_ID;
