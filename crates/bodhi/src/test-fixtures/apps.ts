import type {
  AppAccessSummary,
  ConsentAppInfo,
  ConsentContextResponse,
  ConsentPriorGrant,
  ListAppAccessResponse,
} from '@/hooks/apps';

const APP_CLIENT_ID = 'test-app-client';
const REDIRECT_URI = 'https://myapp.example.com/callback';

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

type ConsentOk = Extract<ConsentContextResponse, { result: 'ok' }>;

export const mockConsentAppInfo: ConsentAppInfo = {
  client_id: APP_CLIENT_ID,
  name: 'Test Application',
  description: 'A test third-party application',
  redirect_uri: REDIRECT_URI,
};

/** Defaults: both sections requested, role User, can_approve, no prior grant. */
export const mockConsentOk = (overrides: Partial<ConsentOk> = {}): ConsentContextResponse => ({
  result: 'ok',
  app: mockConsentAppInfo,
  scope: { role: 'scope_user_user', llms: true, mcps: true, passthrough: [] },
  prior_grant: null,
  can_approve: true,
  ...overrides,
});

export const mockConsentOkPowerUser: ConsentContextResponse = mockConsentOk({
  scope: { role: 'scope_user_power_user', llms: true, mcps: true, passthrough: [] },
});

export const mockConsentRoleOnly: ConsentContextResponse = mockConsentOk({
  scope: { role: 'scope_user_user', llms: false, mcps: false, passthrough: [] },
});

export const mockConsentBlocked: ConsentContextResponse = mockConsentOk({ can_approve: false });

export const mockConsentErrorInApp: ConsentContextResponse = {
  result: 'error',
  error: 'invalid_request',
  error_description: 'redirect_uri does not match the registered redirect URIs',
  redirect_url: null,
};

export const MOCK_ERROR_REDIRECT_URL = `${REDIRECT_URI}?error=invalid_scope&error_description=unknown+scope&error_source=bodhi&state=xyz789`;

export const mockConsentErrorRedirect: ConsentContextResponse = {
  result: 'error',
  error: 'invalid_scope',
  error_description: 'unknown scope',
  redirect_url: MOCK_ERROR_REDIRECT_URL,
};

const priorGrant = (source: ConsentPriorGrant['source']): ConsentPriorGrant => ({
  id: 'prior-grant-1',
  approved_role: 'scope_user_user',
  approved: {
    version: '1' as const,
    models_list: true,
    models_access: { type: 'specific', ids: ['model-a'] },
    mcps_list: true,
    mcps: [],
    mcps_access: { type: 'specific', ids: ['mcp-x'] },
  },
  source,
});

export const mockConsentPriorGrantExplicit: ConsentContextResponse = mockConsentOk({
  prior_grant: priorGrant('explicit'),
});

export const mockConsentPriorGrantLatest: ConsentContextResponse = mockConsentOk({
  prior_grant: priorGrant('latest'),
});

export const MOCK_REDIRECT_URI = REDIRECT_URI;
