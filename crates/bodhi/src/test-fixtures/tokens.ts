/**
 * Fixture factories for API token-related mock data.
 *
 * Uses openapi-generated types from @bodhiapp/ts-client for consistency with MSW handlers.
 * All factories accept optional overrides and return fresh objects per call.
 */
import type { components } from '@/test-utils/msw-v2/setup';

type TokenDetail = components['schemas']['TokenDetail'];
type PaginatedTokenResponse = components['schemas']['PaginatedTokenResponse'];
type TokenCreated = components['schemas']['TokenCreated'];

// ============================================================================
// Token Factories
// ============================================================================

/**
 * Create a mock token detail
 */
export function createMockToken(overrides?: Partial<TokenDetail>): TokenDetail {
  return {
    id: 'token-1',
    name: 'Test Token 1',
    status: 'active',
    token_prefix: 'bodhiapp_test01',
    scopes: 'scope_token_user',
    user_id: 'user-123',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

/**
 * Create a mock paginated token response
 */
export function createMockPaginatedTokens(overrides?: Partial<PaginatedTokenResponse>): PaginatedTokenResponse {
  return {
    data: [createMockToken()],
    total: 1,
    page: 1,
    page_size: 10,
    ...overrides,
  };
}

/**
 * Create an empty paginated token response
 */
export function createMockEmptyPaginatedTokens(): PaginatedTokenResponse {
  return createMockPaginatedTokens({
    data: [],
    total: 0,
  });
}

/**
 * Create a mock token creation response
 */
export function createMockTokenCreated(overrides?: Partial<TokenCreated>): TokenCreated {
  return {
    token: 'test-token-123',
    ...overrides,
  };
}
