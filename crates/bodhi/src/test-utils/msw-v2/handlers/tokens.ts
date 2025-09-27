/**
 * Type-safe MSW v2 handlers for API tokens endpoints using patterns inspired by openapi-msw
 */
import { API_TOKENS_ENDPOINT } from '@/hooks/useQuery';
import { http, HttpResponse, type components } from '../setup';

/**
 * Create type-safe MSW v2 handlers for list tokens endpoint
 * Uses generated OpenAPI types directly
 */
export function mockListTokens(config: Partial<components['schemas']['PaginatedApiTokenResponse']> = {}) {
  return [
    http.get(API_TOKENS_ENDPOINT, () => {
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
      return HttpResponse.json(responseData, { status: 200 });
    }),
  ];
}

/**
 * Create type-safe MSW v2 handlers for create token endpoint
 * Uses generated OpenAPI types directly
 */
export function mockCreateToken(config: Partial<components['schemas']['ApiTokenResponse']> & { delay?: number } = {}) {
  return [
    http.post(API_TOKENS_ENDPOINT, () => {
      const responseData: components['schemas']['ApiTokenResponse'] = {
        offline_token: config.offline_token || 'test-token-123',
      };
      const response = HttpResponse.json(responseData, { status: 201 });

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Create type-safe MSW v2 handlers for update token endpoint
 * Uses generated OpenAPI types directly
 */
export function mockUpdateToken(tokenId: string, config: Partial<components['schemas']['ApiToken']> = {}) {
  return [
    http.put(`${API_TOKENS_ENDPOINT}/${tokenId}`, () => {
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
      return HttpResponse.json(responseData, { status: 200 });
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
    status?: 403 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.get(API_TOKENS_ENDPOINT, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'access_denied',
            message: config.message || 'Insufficient permissions to list tokens',
          },
        },
        { status: config.status || 403 }
      );
    }),
  ];
}

/**
 * Error handler for create token endpoint
 */
export function mockCreateTokenError(
  config: {
    status?: 400 | 422 | 500;
    code?: string;
    message?: string;
    delay?: number;
  } = {}
) {
  return [
    http.post(API_TOKENS_ENDPOINT, () => {
      const response = HttpResponse.json(
        {
          error: {
            code: config.code || 'validation_error',
            message: config.message || 'Invalid token creation request',
          },
        },
        { status: config.status || 400 }
      );

      return config.delay ? new Promise((resolve) => setTimeout(() => resolve(response), config.delay)) : response;
    }),
  ];
}

/**
 * Error handler for update token endpoint
 */
export function mockUpdateTokenError(
  tokenId: string,
  config: {
    status?: 400 | 404 | 500;
    code?: string;
    message?: string;
  } = {}
) {
  return [
    http.put(`${API_TOKENS_ENDPOINT}/${tokenId}`, () => {
      return HttpResponse.json(
        {
          error: {
            code: config.code || 'server_error',
            message: config.message || 'Test Error',
          },
        },
        { status: config.status || 500 }
      );
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
    status: 403,
    code: 'access_denied',
    message: 'Insufficient permissions to access tokens',
  });
}
