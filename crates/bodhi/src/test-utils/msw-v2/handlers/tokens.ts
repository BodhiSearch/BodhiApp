/**
 * Type-safe MSW v2 handlers for API tokens endpoints using openapi-msw
 */
import { API_TOKENS_ENDPOINT } from '@/hooks/useQuery';
import { typedHttp } from '../openapi-msw-setup';
import type { components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for list tokens endpoint
 * Uses generated OpenAPI types directly
 */
export function mockListTokens(config: Partial<components['schemas']['PaginatedApiTokenResponse']> = {}) {
  return [
    typedHttp.get(API_TOKENS_ENDPOINT, ({ response }) => {
      const responseData: components['schemas']['PaginatedApiTokenResponse'] = {
        data: config.data || [
          {
            id: 'token-1',
            name: 'Test Token 1',
            status: 'active',
            token_hash: 'hash123',
            token_id: 'jwt-token-id-1',
            user_id: 'user-123',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ],
        total: config.total || 1,
        page: config.page || 1,
        page_size: config.page_size || 10,
      };
      return response(200).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handlers for create token endpoint
 * Uses generated OpenAPI types directly
 */
export function mockCreateToken(config: Partial<components['schemas']['ApiTokenResponse']> & { delay?: number } = {}) {
  return [
    typedHttp.post(API_TOKENS_ENDPOINT, ({ response }) => {
      const responseData: components['schemas']['ApiTokenResponse'] = {
        offline_token: config.offline_token || 'test-token-123',
      };
      const jsonResponse = response(201).json(responseData);

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(jsonResponse), config.delay))
        : jsonResponse;
    }),
  ];
}

/**
 * Create type-safe MSW v2 handlers for update token endpoint
 * Uses generated OpenAPI types directly
 */
export function mockUpdateToken(tokenId: string, config: Partial<components['schemas']['ApiToken']> = {}) {
  return [
    typedHttp.put('/bodhi/v1/tokens/{id}', ({ response }) => {
      const responseData: components['schemas']['ApiToken'] = {
        id: config.id || tokenId,
        name: config.name || 'Test Token 1',
        status: config.status || 'inactive',
        token_hash: config.token_hash || 'hash123',
        token_id: config.token_id || 'jwt-token-id-1',
        user_id: config.user_id || 'user-123',
        created_at: config.created_at || '2024-01-01T00:00:00Z',
        updated_at: config.updated_at || '2024-01-01T00:00:01Z',
      };
      return response(200).json(responseData);
    }),
  ];
}

/**
 * Convenience method for tokens with default test data
 */
export function mockTokensDefault() {
  return [...mockListTokens(), ...mockCreateToken(), ...mockUpdateToken('token-1')];
}

/**
 * Convenience method for empty tokens list
 */
export function mockEmptyTokensList() {
  return mockListTokens({
    data: [],
    total: 0,
    page: 1,
    page_size: 10,
  });
}

/**
 * Convenience method for create token with custom response
 */
export function mockCreateTokenWithResponse(token: Partial<components['schemas']['ApiTokenResponse']>) {
  return mockCreateToken(token);
}

/**
 * Convenience method for successful token status update
 */
export function mockUpdateTokenStatus(tokenId: string, status: 'active' | 'inactive') {
  return mockUpdateToken(tokenId, {
    status,
    updated_at: new Date().toISOString(),
  });
}

/**
 * Error handler for list tokens endpoint
 */
export function mockListTokensError(
  config: {
    status?: 401 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.get(API_TOKENS_ENDPOINT, ({ response }) => {
      return response(config.status || 401).json({
        error: {
          code: config.code || 'access_denied',
          message: config.message || 'Insufficient permissions to list tokens',
          type: config.status === 500 ? 'internal_server_error' : 'invalid_request_error',
        },
      });
    }),
  ];
}

/**
 * Error handler for create token endpoint
 */
export function mockCreateTokenError(
  config: {
    status?: 400 | 500;
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    typedHttp.post(API_TOKENS_ENDPOINT, ({ response }) => {
      const errorResponse = response(config.status || 400).json({
        error: {
          code: config.code || 'validation_error',
          message: config.message || 'Invalid token creation request',
          type: config.status === 500 ? 'internal_server_error' : 'invalid_request_error',
        },
      });

      return config.delay
        ? new Promise((resolve) => setTimeout(() => resolve(errorResponse), config.delay))
        : errorResponse;
    }),
  ];
}

/**
 * Error handler for update token endpoint
 */
export function mockUpdateTokenError(
  tokenId: string,
  config: {
    status?: 401 | 404 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    typedHttp.put('/bodhi/v1/tokens/{id}', ({ response }) => {
      const getErrorType = (status: number) => {
        if (status === 404) return 'not_found_error';
        if (status === 500) return 'internal_server_error';
        return 'invalid_request_error'; // for 401
      };

      return response(config.status || 500).json({
        error: {
          code: config.code || 'server_error',
          message: config.message || 'Test Error',
          type: getErrorType(config.status || 500),
        },
      });
    }),
  ];
}

/**
 * Convenience method for token not found error
 */
export function mockTokenNotFound(tokenId: string) {
  return mockUpdateTokenError(tokenId, {
    status: 404,
    code: 'not_found',
    message: `Token with id ${tokenId} not found`,
  });
}

/**
 * Convenience method for insufficient permissions error
 */
export function mockTokenAccessDenied() {
  return mockListTokensError({
    status: 401,
    code: 'access_denied',
    message: 'Insufficient permissions to access tokens',
  });
}
