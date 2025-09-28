/**
 * Type-safe MSW v2 handlers for access-requests endpoints using openapi-msw
 */
import {
  ENDPOINT_ACCESS_REQUEST_APPROVE,
  ENDPOINT_ACCESS_REQUEST_REJECT,
  ENDPOINT_ACCESS_REQUESTS,
  ENDPOINT_ACCESS_REQUESTS_PENDING,
  ENDPOINT_USER_REQUEST_ACCESS,
  ENDPOINT_USER_REQUEST_STATUS,
} from '@/hooks/useAccessRequest';
import { delay } from 'msw';
import { INTERNAL_SERVER_ERROR, typedHttp, type components } from '../openapi-msw-setup';

// =============================================================================
// CORE TYPED HTTP METHODS (Success cases + Error handlers)
// =============================================================================

/**
 * Mock handler for all access requests endpoint with configurable responses
 */
export function mockAccessRequests({
  requests = [],
  total = 0,
  page = 1,
  page_size = 10,
  ...rest
}: Partial<components['schemas']['PaginatedUserAccessResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS, async ({ response: resp }) => {
      const responseData: components['schemas']['PaginatedUserAccessResponse'] = {
        requests,
        total: total || requests.length,
        page,
        page_size,
        ...rest,
      };
      return resp(200 as const).json(responseData);
    }),
  ];
}

export function mockAccessRequestsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for pending access requests endpoint with configurable responses
 */
export function mockAccessRequestsPending({
  requests = [],
  total = 0,
  page = 1,
  page_size = 10,
  ...rest
}: Partial<components['schemas']['PaginatedUserAccessResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_PENDING, async ({ response: resp }) => {
      const responseData: components['schemas']['PaginatedUserAccessResponse'] = {
        requests,
        total: total || requests.length,
        page,
        page_size,
        ...rest,
      };
      return resp(200 as const).json(responseData);
    }),
  ];
}

export function mockAccessRequestsPendingError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_PENDING, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for access request approval - success case
 * Only responds to the specified ID, returns 404 for others
 */
export function mockAccessRequestApprove(id: number) {
  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUEST_APPROVE, async ({ params, response }) => {
      // Only respond with success if ID matches
      if (params.id === id.toString()) {
        return response(200 as const).empty();
      }

      // Pass through to next handler for non-matching IDs
      return;
    }),
  ];
}

export function mockAccessRequestApproveError(
  id: number,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUEST_APPROVE, async ({ params, response }) => {
      // Only return error for matching ID
      if (params.id === id.toString()) {
        const errorData = {
          code,
          message,
          type,
          ...rest,
        };
        return response(status).json({ error: errorData });
      }

      // Pass through to next handler for non-matching IDs
      return;
    }),
  ];
}

/**
 * Mock handler for access request rejection - success case
 * Only responds to the specified ID, returns 404 for others
 */
export function mockAccessRequestReject(id: number) {
  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUEST_REJECT, async ({ params, response }) => {
      // Only respond with success if ID matches
      if (params.id === id.toString()) {
        return response(200 as const).empty();
      }

      // Pass through to next handler for non-matching IDs
      return;
    }),
  ];
}

export function mockAccessRequestRejectError(
  id: number,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}
) {
  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUEST_REJECT, async ({ params, response }) => {
      // Only return error for matching ID
      if (params.id === id.toString()) {
        const errorData = {
          code,
          message,
          type,
          ...rest,
        };
        return response(status).json({ error: errorData });
      }

      // Pass through to next handler for non-matching IDs
      return;
    }),
  ];
}

/**
 * Mock handler for user access request status endpoint
 */
export function mockUserRequestStatus({
  status = 'pending',
  username = 'user@example.com',
  created_at = '2024-01-01T00:00:00Z',
  updated_at = '2024-01-01T00:00:00Z',
  ...rest
}: Partial<components['schemas']['UserAccessStatusResponse']> = {}) {
  return [
    typedHttp.get(ENDPOINT_USER_REQUEST_STATUS, async ({ response: resp }) => {
      const responseData: components['schemas']['UserAccessStatusResponse'] = {
        status,
        username,
        created_at,
        updated_at,
        ...rest,
      };
      return resp(200 as const).json(responseData);
    }),
  ];
}

export function mockUserRequestStatusError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 500 } = {}) {
  return [
    typedHttp.get(ENDPOINT_USER_REQUEST_STATUS, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * Mock handler for creating user access request - success case
 */
export function mockUserRequestAccess(delayMs?: number) {
  return [
    typedHttp.post(ENDPOINT_USER_REQUEST_ACCESS, async ({ response }) => {
      if (delayMs) {
        await delay(delayMs);
      }
      // Success response - typically returns 201 with empty body for access requests
      return response(201 as const).empty();
    }),
  ];
}

export function mockUserRequestAccessError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
  ...rest
}: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 409 | 500 } = {}) {
  return [
    typedHttp.post(ENDPOINT_USER_REQUEST_ACCESS, async ({ response }) => {
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}

// =============================================================================
// VARIANT METHODS (Using core methods above)
// =============================================================================

/**
 * Mock handler for all access requests with default data
 */
export function mockAccessRequestsDefault() {
  const defaultRequests: components['schemas']['UserAccessRequest'][] = [
    {
      id: 1,
      user_id: '550e8400-e29b-41d4-a716-446655440001',
      username: 'user@example.com',
      status: 'pending',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
      reviewer: null,
    },
    {
      id: 2,
      user_id: '550e8400-e29b-41d4-a716-446655440002',
      username: 'approved@example.com',
      status: 'approved',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-02T00:00:00Z',
      reviewer: 'admin@example.com',
    },
    {
      id: 3,
      user_id: '550e8400-e29b-41d4-a716-446655440003',
      username: 'rejected@example.com',
      status: 'rejected',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-02T00:00:00Z',
      reviewer: 'admin@example.com',
    },
  ];

  return mockAccessRequests({
    requests: defaultRequests,
    total: defaultRequests.length,
  });
}

/**
 * Mock handler for empty access requests
 */
export function mockAccessRequestsEmpty() {
  return mockAccessRequests({
    requests: [],
    total: 0,
  });
}

/**
 * Mock handler for pending access requests with default data (only pending status)
 */
export function mockAccessRequestsPendingDefault() {
  const pendingRequests: components['schemas']['UserAccessRequest'][] = [
    {
      id: 1,
      user_id: '550e8400-e29b-41d4-a716-446655440001',
      username: 'user@example.com',
      status: 'pending',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
      reviewer: null,
    },
  ];

  return mockAccessRequestsPending({
    requests: pendingRequests,
    total: pendingRequests.length,
  });
}

/**
 * Mock handler for empty pending access requests
 */
export function mockAccessRequestsPendingEmpty() {
  return mockAccessRequestsPending({
    requests: [],
    total: 0,
  });
}

/**
 * Mock handler for user request status - pending status
 */
export function mockUserRequestStatusPending(config: { username?: string } = {}) {
  return mockUserRequestStatus({
    status: 'pending',
    username: config.username || 'user@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  });
}

/**
 * Mock handler for user request status - approved status
 */
export function mockUserRequestStatusApproved(config: { username?: string } = {}) {
  return mockUserRequestStatus({
    status: 'approved',
    username: config.username || 'approved@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  });
}

/**
 * Mock handler for user request status - rejected status
 */
export function mockUserRequestStatusRejected(config: { username?: string } = {}) {
  return mockUserRequestStatus({
    status: 'rejected',
    username: config.username || 'rejected@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  });
}
