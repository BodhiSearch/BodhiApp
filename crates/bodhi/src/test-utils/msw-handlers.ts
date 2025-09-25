import { rest } from 'msw';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import {
  ENDPOINT_USER_REQUEST_STATUS,
  ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_ACCESS_REQUESTS_PENDING,
  ENDPOINT_ACCESS_REQUESTS,
} from '@/hooks/useAccessRequest';
import { mockPendingRequests, mockAllRequests, createMockUserInfo } from '@/test-fixtures/access-requests';
import { mockSimpleUsersResponse } from '@/test-fixtures/users';

export interface HandlerOverrides {
  appInfo?: unknown;
  userInfo?: unknown;
  requestStatus?: unknown;
  pendingRequests?: unknown;
  allRequests?: unknown;
  users?: unknown;
  submitRequest?: unknown;
  approveRequest?: unknown;
  rejectRequest?: unknown;
  apiFormats?: unknown;
  testApiModel?: unknown;
  fetchModels?: unknown;
  createApiModel?: unknown;
  updateApiModel?: unknown;
}

export const createAccessRequestHandlers = (overrides: HandlerOverrides = {}) => [
  // App info endpoint
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => res(ctx.json(overrides.appInfo || { status: 'ready' }))),

  // User info endpoint
  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) =>
    res(ctx.json(overrides.userInfo || createMockUserInfo('resource_user')))
  ),

  // User request status
  rest.get(`*${ENDPOINT_USER_REQUEST_STATUS}`, (_, res, ctx) =>
    res(
      ctx.json(
        overrides.requestStatus || {
          status: 'pending',
          username: 'user@example.com',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        }
      )
    )
  ),

  // Submit access request
  rest.post(`*${ENDPOINT_USER_REQUEST_ACCESS}`, (_, res, ctx) =>
    res(
      ctx.json(
        overrides.submitRequest || {
          id: 1,
          user_id: '550e8400-e29b-41d4-a716-446655440000',
          status: 'pending',
          username: 'test@example.com',
          created_at: new Date().toISOString(),
        }
      )
    )
  ),

  // Get pending requests
  rest.get(`*${ENDPOINT_ACCESS_REQUESTS_PENDING}`, (_, res, ctx) =>
    res(ctx.json(overrides.pendingRequests || mockPendingRequests))
  ),

  // Get all requests
  rest.get(`*${ENDPOINT_ACCESS_REQUESTS}`, (_, res, ctx) => res(ctx.json(overrides.allRequests || mockAllRequests))),

  // Approve request
  rest.post(`*${ENDPOINT_ACCESS_REQUESTS}/:id/approve`, (_, res, ctx) => res(ctx.json(overrides.approveRequest || {}))),

  // Reject request
  rest.post(`*${ENDPOINT_ACCESS_REQUESTS}/:id/reject`, (_, res, ctx) => res(ctx.json(overrides.rejectRequest || {}))),

  // Users endpoint
  rest.get('*/bodhi/v1/users', (_, res, ctx) => {
    if (overrides.users === 'error') {
      return res(ctx.status(500), ctx.json({ error: { type: 'internal_error', message: 'Failed to fetch users' } }));
    }
    return res(ctx.json(overrides.users || mockSimpleUsersResponse));
  }),

  // Change user role
  rest.put('*/bodhi/v1/users/:userId/role', (_, res, ctx) => res(ctx.json({}))),

  // Remove user
  rest.delete('*/bodhi/v1/users/:userId', (_, res, ctx) => res(ctx.json({}))),

  // API Models endpoints
  rest.get('*/bodhi/v1/api-models/api-formats', (_, res, ctx) =>
    res(ctx.json(overrides.apiFormats || { data: ['openai'] }))
  ),

  rest.post('*/bodhi/v1/api-models/test', (_, res, ctx) =>
    res(ctx.json(overrides.testApiModel || { success: true, response: 'Test successful' }))
  ),

  rest.post('*/bodhi/v1/api-models/fetch-models', (_, res, ctx) =>
    res(ctx.json(overrides.fetchModels || { models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'] }))
  ),

  rest.post('*/bodhi/v1/api-models', (_, res, ctx) =>
    res(ctx.json(overrides.createApiModel || { id: 'test-api-model-id', api_format: 'openai' }))
  ),

  rest.put('*/bodhi/v1/api-models/:id', (_, res, ctx) =>
    res(ctx.json(overrides.updateApiModel || { id: 'test-api-model-id', api_format: 'openai' }))
  ),
];

// Create handlers for error scenarios
export const createErrorHandlers = () => [
  // App info endpoint error
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { type: 'internal_error', message: 'Failed to fetch-models app info' } }))
  ),

  // User info endpoint error
  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { type: 'internal_error', message: 'Failed to fetch-models user info' } }))
  ),

  rest.get(`*${ENDPOINT_USER_REQUEST_STATUS}`, (_, res, ctx) =>
    res(
      ctx.status(500),
      ctx.json({ error: { type: 'internal_error', message: 'Failed to fetch-models request status' } })
    )
  ),

  rest.post(`*${ENDPOINT_USER_REQUEST_ACCESS}`, (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { type: 'invalid_request_error', message: 'Request already exists' } }))
  ),

  rest.get(`*${ENDPOINT_ACCESS_REQUESTS_PENDING}`, (_, res, ctx) =>
    res(ctx.status(403), ctx.json({ error: { type: 'authentication_error', message: 'Insufficient permissions' } }))
  ),

  rest.get(`*${ENDPOINT_ACCESS_REQUESTS}`, (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { type: 'internal_error', message: 'Internal server error' } }))
  ),

  rest.post(`*${ENDPOINT_ACCESS_REQUESTS}/*/approve`, (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { type: 'invalid_request_error', message: 'Invalid request' } }))
  ),

  rest.post(`*${ENDPOINT_ACCESS_REQUESTS}/*/reject`, (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { type: 'invalid_request_error', message: 'Cannot reject request' } }))
  ),

  // Users endpoint error
  rest.get('*/bodhi/v1/users', (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { type: 'internal_error', message: 'Failed to fetch users' } }))
  ),

  // API Models endpoints errors
  rest.get('*/bodhi/v1/api-models/api-formats', (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { type: 'internal_error', message: 'Failed to fetch-models API formats' } }))
  ),

  rest.post('*/bodhi/v1/api-models/test', (_, res, ctx) =>
    res(ctx.json({ success: false, error: 'Connection test failed' }))
  ),

  rest.post('*/bodhi/v1/api-models/fetch-models', (_, res, ctx) =>
    res(ctx.status(401), ctx.json({ error: { type: 'authentication_error', message: 'Invalid API key' } }))
  ),

  rest.post('*/bodhi/v1/api-models', (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { type: 'invalid_request_error', message: 'Invalid API model data' } }))
  ),

  rest.put('*/bodhi/v1/api-models/:id', (_, res, ctx) =>
    res(ctx.status(404), ctx.json({ error: { type: 'not_found_error', message: 'API model not found' } }))
  ),
];

// Role-based handlers for testing access control
export const createRoleBasedHandlers = (userRole: string, shouldHaveAccess: boolean = true) => [
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => res(ctx.json({ status: 'ready' }))),

  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => res(ctx.json(createMockUserInfo(userRole)))),

  // Access-restricted endpoints return 403 if user doesn't have access
  rest.get(`*${ENDPOINT_ACCESS_REQUESTS_PENDING}`, (_, res, ctx) => {
    if (!shouldHaveAccess) {
      return res(
        ctx.status(403),
        ctx.json({ error: { type: 'authentication_error', message: 'Insufficient permissions' } })
      );
    }
    return res(ctx.json(mockPendingRequests));
  }),

  rest.get(`*${ENDPOINT_ACCESS_REQUESTS}`, (_, res, ctx) => {
    if (!shouldHaveAccess) {
      return res(
        ctx.status(403),
        ctx.json({ error: { type: 'authentication_error', message: 'Insufficient permissions' } })
      );
    }
    return res(ctx.json(mockAllRequests));
  }),

  rest.get('*/bodhi/v1/users', (_, res, ctx) => {
    if (!shouldHaveAccess) {
      return res(
        ctx.status(403),
        ctx.json({ error: { type: 'authentication_error', message: 'Insufficient permissions' } })
      );
    }
    return res(ctx.json(mockSimpleUsersResponse));
  }),
];

// Create handlers specifically for API models testing
export const createApiModelHandlers = (overrides: Partial<HandlerOverrides> = {}) => [
  // Standard app/user endpoints
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => res(ctx.json(overrides.appInfo || { status: 'ready' }))),
  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) =>
    res(ctx.json(overrides.userInfo || createMockUserInfo('resource_user')))
  ),

  // API Models endpoints
  rest.get('*/bodhi/v1/api-models/api-formats', (_, res, ctx) =>
    res(ctx.json(overrides.apiFormats || { data: ['openai'] }))
  ),

  rest.post('*/bodhi/v1/api-models/test', (_, res, ctx) =>
    res(ctx.json(overrides.testApiModel || { success: true, response: 'Test successful' }))
  ),

  rest.post('*/bodhi/v1/api-models/fetch-models', (_, res, ctx) =>
    res(ctx.json(overrides.fetchModels || { models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'] }))
  ),

  rest.post('*/bodhi/v1/api-models', (_, res, ctx) =>
    res(ctx.json(overrides.createApiModel || { id: 'test-api-model-id', api_format: 'openai' }))
  ),

  rest.put('*/bodhi/v1/api-models/:id', (_, res, ctx) =>
    res(ctx.json(overrides.updateApiModel || { id: 'test-api-model-id', api_format: 'openai' }))
  ),
];
