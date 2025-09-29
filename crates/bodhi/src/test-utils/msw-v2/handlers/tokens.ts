/**
 * Type-safe MSW v2 handlers for API tokens endpoints using openapi-msw
 */
import { API_TOKENS_ENDPOINT, ENDPOINT_TOKEN_ID } from '@/hooks/useQuery';
import { delay } from 'msw';
import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../openapi-msw-setup';

// ============================================================================
// API Tokens - Success Handlers
// ============================================================================

/**
 * Create type-safe MSW v2 handlers for list tokens endpoint
 * Uses generated OpenAPI types directly
 */
export function mockTokens(
  {
    data = [
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
    total = 1,
    page = 1,
    page_size = 10,
    ...rest
  }: Partial<components['schemas']['PaginatedApiTokenResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(API_TOKENS_ENDPOINT, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['PaginatedApiTokenResponse'] = {
        data,
        total,
        page,
        page_size,
        ...rest,
      };

      return response(200 as const).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handlers for create token endpoint
 * Uses generated OpenAPI types directly
 */
export function mockCreateToken(
  { offline_token = 'test-token-123', ...rest }: Partial<components['schemas']['ApiTokenResponse']> = {},
  { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(API_TOKENS_ENDPOINT, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      if (delayMs) {
        await delay(delayMs);
      }
      const responseData: components['schemas']['ApiTokenResponse'] = {
        offline_token,
        ...rest,
      };

      return response(201 as const).json(responseData);
    }),
  ];
}

/**
 * Create type-safe MSW v2 handlers for update token endpoint
 * Uses generated OpenAPI types directly
 */
export function mockUpdateToken(
  tokenId: string,
  {
    id = tokenId,
    name = 'Test Token 1',
    status = 'inactive',
    token_hash = 'hash123',
    token_id = 'jwt-token-id-1',
    user_id = 'user-123',
    created_at = '2024-01-01T00:00:00Z',
    updated_at = '2024-01-01T00:00:01Z',
    ...rest
  }: Partial<components['schemas']['ApiToken']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_TOKEN_ID, async ({ params, response }) => {
      // Only respond if id matches
      if (params.id !== tokenId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const responseData: components['schemas']['ApiToken'] = {
        id,
        name,
        status,
        token_hash,
        token_id,
        user_id,
        created_at,
        updated_at,
        ...rest,
      };

      return response(200 as const).json(responseData);
    }),
  ];
}

// ============================================================================
// API Tokens - Convenience Methods
// ============================================================================

/**
 * Convenience method for tokens with default test data
 */
export function mockTokensDefault() {
  return [...mockTokens(), ...mockCreateToken(), ...mockUpdateToken('token-1')];
}

/**
 * Convenience method for empty tokens list
 */
export function mockEmptyTokensList() {
  return mockTokens({
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

// ============================================================================
// API Tokens - Error Handlers
// ============================================================================

/**
 * Error handler for list tokens endpoint
 */
export function mockTokensError(
  {
    code = 'access_denied',
    message = 'Insufficient permissions to list tokens',
    type = 'invalid_request_error',
    status = 401,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 401 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(API_TOKENS_ENDPOINT, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
 * Error handler for create token endpoint
 */
export function mockCreateTokenError(
  {
    code = 'validation_error',
    message = 'Invalid token creation request',
    type = 'invalid_request_error',
    status = 400,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(API_TOKENS_ENDPOINT, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
 * Error handler for update token endpoint
 */
export function mockUpdateTokenError(
  tokenId: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 401 | 404 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.put(ENDPOINT_TOKEN_ID, async ({ params, response }) => {
      // Only respond if id matches
      if (params.id !== tokenId) {
        return; // Pass through to next handler
      }

      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

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
 * Convenience method for token not found error
 */
export function mockTokenNotFound(tokenId: string) {
  return mockUpdateTokenError(tokenId, {
    code: 'not_found',
    message: `Token with id ${tokenId} not found`,
    type: 'not_found_error',
  });
}

/**
 * Convenience method for insufficient permissions error
 */
export function mockTokenAccessDenied() {
  return mockTokensError({
    code: 'access_denied',
    message: 'Insufficient permissions to access tokens',
    type: 'invalid_request_error',
  });
}
