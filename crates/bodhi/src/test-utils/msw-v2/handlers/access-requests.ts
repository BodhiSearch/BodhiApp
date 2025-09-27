/**
 * Type-safe MSW v2 handlers for access-requests endpoints using openapi-msw
 */
import {
  ENDPOINT_ACCESS_REQUESTS,
  ENDPOINT_ACCESS_REQUESTS_PENDING,
  ENDPOINT_USER_REQUEST_STATUS,
  ENDPOINT_USER_REQUEST_ACCESS,
} from '@/hooks/useAccessRequest';
import { typedHttp } from '../openapi-msw-setup';
import { http, HttpResponse, type components } from '../setup';

/**
 * Mock handler for all access requests endpoint with configurable responses
 *
 * @param config Configuration options
 * @param config.requests - Array of access requests to return
 * @param config.total - Total number of requests
 * @param config.page - Current page number (default: 1)
 * @param config.page_size - Page size (default: 10)
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockAccessRequests(
  config: {
    requests?: components['schemas']['UserAccessRequest'][];
    total?: number;
    page?: number;
    page_size?: number;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS, ({ response }) => {
      const responseData: components['schemas']['PaginatedUserAccessResponse'] = {
        requests: config.requests || [],
        total: config.total || config.requests?.length || 0,
        page: config.page || 1,
        page_size: config.page_size || 10,
      };
      const responseObj = response(200).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay))
        : responseObj;
    }),
  ];
}

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
 * Mock handler for pending access requests endpoint with configurable responses
 *
 * @param config Configuration options
 * @param config.requests - Array of pending access requests to return
 * @param config.total - Total number of pending requests
 * @param config.page - Current page number (default: 1)
 * @param config.page_size - Page size (default: 10)
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockAccessRequestsPending(
  config: {
    requests?: components['schemas']['UserAccessRequest'][];
    total?: number;
    page?: number;
    page_size?: number;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_PENDING, ({ response }) => {
      const responseData: components['schemas']['PaginatedUserAccessResponse'] = {
        requests: config.requests || [],
        total: config.total || config.requests?.length || 0,
        page: config.page || 1,
        page_size: config.page_size || 10,
      };
      const responseObj = response(200).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay))
        : responseObj;
    }),
  ];
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
 * Mock handler for access request approval
 *
 * @param config Configuration options
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.success - Whether the approval should succeed (default: true)
 */
export function mockAccessRequestApprove(
  config: {
    delay?: number;
    success?: boolean;
  } = {}
) {
  return [
    typedHttp.post('/bodhi/v1/access-requests/{id}/approve', async ({ request, response }) => {
      // Parse request body to validate it contains role
      const body = await request.json();

      if (config.success === false) {
        const responseObj = response(404).json({
          error: {
            code: 'approval_failed',
            message: 'Failed to approve access request',
            type: 'not_found_error',
          },
        });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay)) : responseObj;
      }

      // Success response - no content expected
      const responseObj = response(200).empty();
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay)) : responseObj;
    }),
  ];
}

/**
 * Mock handler for access request rejection
 *
 * @param config Configuration options
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.success - Whether the rejection should succeed (default: true)
 */
export function mockAccessRequestReject(
  config: {
    delay?: number;
    success?: boolean;
  } = {}
) {
  return [
    typedHttp.post('/bodhi/v1/access-requests/{id}/reject', ({ response }) => {
      if (config.success === false) {
        const responseObj = response(404).json({
          error: {
            code: 'rejection_failed',
            message: 'Failed to reject access request',
            type: 'not_found_error',
          },
        });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay)) : responseObj;
      }

      // Success response - no content expected
      const responseObj = response(200).empty();
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay)) : responseObj;
    }),
  ];
}

/**
 * Error handler for access requests endpoint
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 500)
 * @param config.code - Error code (default: 'server_error')
 * @param config.message - Error message (default: 'Server error')
 */
export function mockAccessRequestsError(
  config: {
    status?: 401 | 403;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS, ({ response }) => {
      const statusCode = config.status || 401;
      return response(statusCode).json({
        error: {
          code: config.code || 'unauthorized_error',
          message: config.message || 'Unauthorized',
          type: statusCode === 401 ? 'unauthorized_error' : 'forbidden_error',
        },
      });
    }),
  ];
}

/**
 * Error handler for access request approval
 */
export function mockAccessRequestApproveError() {
  return mockAccessRequestApprove({ success: false });
}

/**
 * Error handler for access request rejection
 */
export function mockAccessRequestRejectError() {
  return mockAccessRequestReject({ success: false });
}

/**
 * Error handler for pending access requests endpoint
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 404)
 * @param config.code - Error code (default: 'not_found')
 * @param config.message - Error message (default: 'Not found')
 */
export function mockAccessRequestsPendingError(
  config: {
    status?: 401 | 403;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_PENDING, ({ response }) => {
      const statusCode = config.status || 401;
      return response(statusCode).json({
        error: {
          code: config.code || 'unauthorized_error',
          message: config.message || 'Unauthorized',
          type: statusCode === 401 ? 'unauthorized_error' : 'forbidden_error',
        },
      });
    }),
  ];
}

/**
 * Mock handler for user access request status endpoint
 *
 * @param config Configuration options
 * @param config.status - Request status: 'pending', 'approved', 'rejected'
 * @param config.username - Username for the request
 * @param config.created_at - Creation timestamp (default: '2024-01-01T00:00:00Z')
 * @param config.updated_at - Last update timestamp (default: '2024-01-01T00:00:00Z')
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockUserRequestStatus(
  config: {
    status?: 'pending' | 'approved' | 'rejected';
    username?: string;
    created_at?: string;
    updated_at?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_USER_REQUEST_STATUS, ({ response }) => {
      const responseData: components['schemas']['UserAccessStatusResponse'] = {
        status: config.status || 'pending',
        username: config.username || 'user@example.com',
        created_at: config.created_at || '2024-01-01T00:00:00Z',
        updated_at: config.updated_at || '2024-01-01T00:00:00Z',
      };
      const responseObj = response(200).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay))
        : responseObj;
    }),
  ];
}

/**
 * Mock handler for user request status - pending status
 */
export function mockUserRequestStatusPending(config: { username?: string; delay?: number } = {}) {
  return mockUserRequestStatus({
    status: 'pending',
    username: config.username || 'user@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    delay: config.delay,
  });
}

/**
 * Mock handler for user request status - approved status
 */
export function mockUserRequestStatusApproved(config: { username?: string; delay?: number } = {}) {
  return mockUserRequestStatus({
    status: 'approved',
    username: config.username || 'approved@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
    delay: config.delay,
  });
}

/**
 * Mock handler for user request status - rejected status
 */
export function mockUserRequestStatusRejected(config: { username?: string; delay?: number } = {}) {
  return mockUserRequestStatus({
    status: 'rejected',
    username: config.username || 'rejected@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
    delay: config.delay,
  });
}

/**
 * Error handler for user access request status endpoint (404 - no request found)
 *
 * @param config Error configuration
 * @param config.status - HTTP status code (default: 404)
 * @param config.code - Error code (default: 'not_found_error')
 * @param config.message - Error message (default: 'pending access request for user not found')
 * @param config.delay - Add delay in milliseconds for testing loading states
 */
export function mockUserRequestStatusError(
  config: {
    status?: 400 | 401 | 404;
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.get(ENDPOINT_USER_REQUEST_STATUS, ({ response }) => {
      const statusCode = config.status || 404;
      let errorType: string;
      if (statusCode === 400) errorType = 'invalid_request_error';
      else if (statusCode === 401) errorType = 'unauthorized_error';
      else errorType = 'not_found_error';

      const responseObj = response(statusCode).json({
        error: {
          code: config.code || (statusCode === 404 ? 'not_found' : 'error'),
          message: config.message || 'pending access request for user not found',
          type: errorType,
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay))
        : responseObj;
    }),
  ];
}

/**
 * Mock handler for creating user access request
 *
 * @param config Configuration options
 * @param config.delay - Add delay in milliseconds for testing loading states
 * @param config.success - Whether the request creation should succeed (default: true)
 */
export function mockUserRequestAccess(
  config: {
    delay?: number;
    success?: boolean;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_USER_REQUEST_ACCESS, ({ response }) => {
      if (config.success === false) {
        const responseObj = response(409).json({
          error: {
            code: 'conflict',
            message: 'Request already exists',
            type: 'conflict_error',
          },
        });
        return config.delay ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay)) : responseObj;
      }

      // Success response - typically returns 201 with empty body for access requests
      const responseObj = response(201).empty();
      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay)) : responseObj;
    }),
  ];
}

/**
 * Error handler for creating user access request
 */
export function mockUserRequestAccessError(
  config: {
    status?: 401 | 409 | 422;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(ENDPOINT_USER_REQUEST_ACCESS, ({ response }) => {
      const statusCode = config.status || 409;
      let errorType: string;
      if (statusCode === 401) errorType = 'unauthorized_error';
      else if (statusCode === 409) errorType = 'conflict_error';
      else errorType = 'unprocessable_entity_error';

      const responseObj = response(statusCode).json({
        error: {
          code: statusCode === 409 ? 'conflict' : 'error',
          message: config.message || 'Request already exists',
          type: errorType,
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(responseObj), config.delay))
        : responseObj;
    }),
  ];
}
