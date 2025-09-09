import { rest } from 'msw';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import {
  ENDPOINT_USER_REQUEST_STATUS,
  ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_ACCESS_REQUESTS_PENDING,
  ENDPOINT_ACCESS_REQUESTS,
} from '@/hooks/useAccessRequest';
import {
  mockPendingRequests,
  mockAllRequests,
  mockUserAccessStatusNone,
  createMockUserInfo,
} from '@/test-fixtures/access-requests';
import { mockUsersResponse } from '@/test-fixtures/users';

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
    res(ctx.json(overrides.requestStatus || mockUserAccessStatusNone))
  ),

  // Submit access request
  rest.post(`*${ENDPOINT_USER_REQUEST_ACCESS}`, (_, res, ctx) =>
    res(
      ctx.json(
        overrides.submitRequest || {
          id: 1,
          status: 'pending',
          email: 'test@example.com',
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

  // Users endpoint (placeholder - returns mock data since not implemented yet)
  rest.get('*/users', (_, res, ctx) => res(ctx.json(overrides.users || mockUsersResponse))),

  // Change user role (placeholder)
  rest.put('*/users/:userId/role', (_, res, ctx) => res(ctx.json({}))),

  // Remove user (placeholder)
  rest.delete('*/users/:userId', (_, res, ctx) => res(ctx.json({}))),
];

// Create handlers for error scenarios
export const createErrorHandlers = () => [
  rest.get(`*${ENDPOINT_USER_REQUEST_STATUS}`, (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { message: 'Failed to fetch request status' } }))
  ),

  rest.post(`*${ENDPOINT_USER_REQUEST_ACCESS}`, (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { message: 'Request already exists' } }))
  ),

  rest.get(`*${ENDPOINT_ACCESS_REQUESTS_PENDING}`, (_, res, ctx) =>
    res(ctx.status(403), ctx.json({ error: { message: 'Insufficient permissions' } }))
  ),

  rest.get(`*${ENDPOINT_ACCESS_REQUESTS}`, (_, res, ctx) =>
    res(ctx.status(500), ctx.json({ error: { message: 'Internal server error' } }))
  ),

  rest.post(`*${ENDPOINT_ACCESS_REQUESTS}/*/approve`, (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { message: 'Invalid request' } }))
  ),

  rest.post(`*${ENDPOINT_ACCESS_REQUESTS}/*/reject`, (_, res, ctx) =>
    res(ctx.status(400), ctx.json({ error: { message: 'Cannot reject request' } }))
  ),
];

// Role-based handlers for testing access control
export const createRoleBasedHandlers = (userRole: string, shouldHaveAccess: boolean = true) => [
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => res(ctx.json({ status: 'ready' }))),

  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => res(ctx.json(createMockUserInfo(userRole)))),

  // Access-restricted endpoints return 403 if user doesn't have access
  rest.get(`*${ENDPOINT_ACCESS_REQUESTS_PENDING}`, (_, res, ctx) => {
    if (!shouldHaveAccess) {
      return res(ctx.status(403), ctx.json({ error: { message: 'Insufficient permissions' } }));
    }
    return res(ctx.json(mockPendingRequests));
  }),

  rest.get(`*${ENDPOINT_ACCESS_REQUESTS}`, (_, res, ctx) => {
    if (!shouldHaveAccess) {
      return res(ctx.status(403), ctx.json({ error: { message: 'Insufficient permissions' } }));
    }
    return res(ctx.json(mockAllRequests));
  }),

  rest.get('*/users', (_, res, ctx) => {
    if (!shouldHaveAccess) {
      return res(ctx.status(403), ctx.json({ error: { message: 'Insufficient permissions' } }));
    }
    return res(ctx.json(mockUsersResponse));
  }),
];
