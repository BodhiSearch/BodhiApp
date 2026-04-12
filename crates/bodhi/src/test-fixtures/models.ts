/**
 * Fixture factories for model-related mock data.
 *
 * Uses openapi-generated types from @bodhiapp/ts-client for consistency with MSW handlers.
 * All factories accept optional overrides and return fresh objects per call.
 */
import type { components } from '@/test-utils/msw-v2/setup';

type AliasResponse = components['schemas']['AliasResponse'];
type UserAliasResponse = components['schemas']['UserAliasResponse'];
type ModelAliasResponse = components['schemas']['ModelAliasResponse'];
type ApiAliasResponse = components['schemas']['ApiAliasResponse'];
type PaginatedAliasResponse = components['schemas']['PaginatedAliasResponse'];
type RefreshResponse = components['schemas']['RefreshResponse'];
type QueueStatusResponse = components['schemas']['QueueStatusResponse'];

// ============================================================================
// Individual Model Factories
// ============================================================================

/**
 * Create a user-source alias (local GGUF model)
 */
export function createMockUserAlias(overrides?: Partial<UserAliasResponse>): UserAliasResponse {
  return {
    source: 'user',
    id: 'test-uuid-1',
    alias: 'test-model',
    repo: 'test-repo',
    filename: 'test-file.bin',
    snapshot: 'abc123',
    request_params: {},
    context_params: [],
    model_params: {},
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

/**
 * Create an OpenAI ApiModel fixture
 */
export function createMockOpenAIModel(id: string, overrides?: Record<string, unknown>) {
  return {
    id,
    object: 'model' as const,
    created: 0,
    owned_by: 'openai',
    provider: 'openai' as const,
    ...overrides,
  };
}

/**
 * Create an Anthropic ApiModel fixture
 */
export function createMockAnthropicModel(id: string, overrides?: Record<string, unknown>) {
  return {
    id,
    display_name: id,
    created_at: '2024-01-01T00:00:00Z',
    type: 'model' as const,
    provider: 'anthropic' as const,
    ...overrides,
  };
}

/**
 * Create an API-source alias (remote API model)
 */
export function createMockApiAlias(overrides?: Partial<ApiAliasResponse>): ApiAliasResponse {
  return {
    source: 'api',
    id: 'test-api-model',
    api_format: 'openai',
    base_url: 'https://api.openai.com/v1',
    has_api_key: true,
    models: [createMockOpenAIModel('gpt-4'), createMockOpenAIModel('gpt-3.5-turbo')],
    forward_all_with_prefix: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

/**
 * Create a source model alias (model file reference)
 */
export function createMockModelAlias(overrides?: Partial<ModelAliasResponse>): ModelAliasResponse {
  return {
    source: 'model',
    alias: 'test-model',
    repo: 'test-repo',
    filename: 'test-file.bin',
    snapshot: 'abc123',
    ...overrides,
  };
}

// ============================================================================
// Paginated Response Factories
// ============================================================================

/**
 * Create a paginated models list response
 */
export function createMockPaginatedModels(overrides?: Partial<PaginatedAliasResponse>): PaginatedAliasResponse {
  return {
    data: [createMockUserAlias()],
    page: 1,
    page_size: 30,
    total: 1,
    ...overrides,
  };
}

/**
 * Create an empty paginated models list response
 */
export function createMockEmptyPaginatedModels(): PaginatedAliasResponse {
  return createMockPaginatedModels({
    data: [],
    total: 0,
  });
}

/**
 * Create a paginated models list with an API model
 */
export function createMockPaginatedApiModels(overrides?: Partial<PaginatedAliasResponse>): PaginatedAliasResponse {
  return createMockPaginatedModels({
    data: [createMockApiAlias()],
    ...overrides,
  });
}

/**
 * Create a paginated models list with a source model
 */
export function createMockPaginatedSourceModels(overrides?: Partial<PaginatedAliasResponse>): PaginatedAliasResponse {
  return createMockPaginatedModels({
    data: [createMockModelAlias() as AliasResponse],
    ...overrides,
  });
}

// ============================================================================
// Refresh & Queue Factories
// ============================================================================

/**
 * Create a refresh response
 */
export function createMockRefreshResponse(overrides?: Partial<RefreshResponse>): RefreshResponse {
  return {
    num_queued: 'all',
    alias: null,
    ...overrides,
  };
}

/**
 * Create a queue status response
 */
export function createMockQueueStatus(overrides?: Partial<QueueStatusResponse>): QueueStatusResponse {
  return {
    status: 'idle',
    ...overrides,
  };
}
