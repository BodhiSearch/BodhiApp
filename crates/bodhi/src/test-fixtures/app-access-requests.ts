import type { AccessRequestReviewResponse } from '@/hooks/useAppAccessRequests';

const REQUEST_ID = '550e8400-e29b-41d4-a716-446655440000';
const APP_CLIENT_ID = 'test-app-client';

// Draft review with tool types and instances
export const mockDraftReviewResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: 'A test third-party application',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [
        {
          id: 'instance-1',
          slug: 'my-exa-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
        {
          id: 'instance-2',
          slug: 'test-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
  mcps_info: [],
};

// Draft review with tool type but no user instances
export const mockDraftNoInstancesResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Test Application',
  app_description: 'A test third-party application',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [],
    },
  ],
  mcps_info: [],
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
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [
        {
          id: 'instance-1',
          slug: 'my-exa-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
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
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [],
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
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [],
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
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [],
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
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [
        {
          id: 'instance-1',
          slug: 'my-exa-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
  mcps_info: [],
};

// Draft with multiple tool types
export const mockDraftMultiToolResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Multi-Tool App',
  app_description: 'An app requesting multiple tool types',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_power_user',
  requested: {
    toolset_types: [{ toolset_type: 'builtin-exa-search' }, { toolset_type: 'builtin-weather' }],
    mcp_servers: [],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [
        {
          id: 'instance-1',
          slug: 'my-exa-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
    {
      toolset_type: 'builtin-weather',
      name: 'Weather Lookup',
      description: 'Get weather information',
      instances: [
        {
          id: 'instance-3',
          slug: 'my-weather-instance',
          toolset_type: 'builtin-weather',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
        {
          id: 'instance-4',
          slug: 'disabled-instance',
          toolset_type: 'builtin-weather',
          enabled: false,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
  mcps_info: [],
};

// Draft with multiple tool types where one has instances and another has none
export const mockDraftMultiToolMixedResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Mixed Tool App',
  app_description: 'An app with mixed tool availability',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    toolset_types: [{ toolset_type: 'builtin-exa-search' }, { toolset_type: 'builtin-calculator' }],
    mcp_servers: [],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [
        {
          id: 'instance-1',
          slug: 'my-exa-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
    {
      toolset_type: 'builtin-calculator',
      name: 'Calculator',
      description: 'Perform calculations',
      instances: [],
    },
  ],
  mcps_info: [],
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
    toolset_types: [],
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  tools_info: [],
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
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

// Draft with both toolsets and MCP servers
export const mockDraftMixedResourcesResponse: AccessRequestReviewResponse = {
  id: REQUEST_ID,
  app_client_id: APP_CLIENT_ID,
  app_name: 'Mixed Resources App',
  app_description: 'An app requesting both tools and MCPs',
  flow_type: 'popup',
  status: 'draft',
  requested_role: 'scope_user_user',
  requested: {
    toolset_types: [{ toolset_type: 'builtin-exa-search' }],
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  tools_info: [
    {
      toolset_type: 'builtin-exa-search',
      name: 'Exa Web Search',
      description: 'Search the web using Exa AI',
      instances: [
        {
          id: 'instance-1',
          slug: 'my-exa-instance',
          toolset_type: 'builtin-exa-search',
          enabled: true,
          has_api_key: true,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ],
    },
  ],
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
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
    toolset_types: [],
    mcp_servers: [{ url: 'https://mcp.example.com/mcp' }],
  },
  tools_info: [],
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
    toolset_types: [],
    mcp_servers: [{ url: 'https://mcp.deepwiki.com/mcp' }],
  },
  tools_info: [],
  mcps_info: [
    {
      url: 'https://mcp.deepwiki.com/mcp',
      instances: [
        {
          id: 'mcp-instance-1',
          name: 'DeepWiki',
          slug: 'deepwiki-prod',
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
