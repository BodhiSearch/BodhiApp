/**
 * Fixture factories for toolset-related mock data.
 *
 * Uses types from hooks/toolsets for consistency with MSW handlers.
 * All factories accept optional overrides and return fresh objects per call.
 */
import type { ToolsetResponse, ToolsetDefinition, AppToolsetConfig, ToolDefinition } from '@/hooks/toolsets';

// ============================================================================
// Tool Definition Factories
// ============================================================================

/**
 * Create a mock tool definition (function-type tool)
 */
export function createMockToolDefinition(overrides?: Partial<ToolDefinition>): ToolDefinition {
  return {
    type: 'function',
    function: {
      name: 'search',
      description: 'Search the web using Exa AI',
      parameters: {
        type: 'object',
        properties: {
          query: { type: 'string', description: 'The search query' },
        },
        required: ['query'],
      },
    },
    ...overrides,
  };
}

// ============================================================================
// Toolset Instance Factories
// ============================================================================

/**
 * Create a mock toolset response (user instance)
 */
export function createMockToolset(overrides?: Partial<ToolsetResponse>): ToolsetResponse {
  return {
    id: 'uuid-test-toolset',
    slug: 'my-exa-search',
    toolset_type: 'builtin-exa-search',
    description: 'Test toolset',
    enabled: true,
    has_api_key: true,
    tools: [createMockToolDefinition()],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

// ============================================================================
// Toolset Type Factories
// ============================================================================

/**
 * Create a mock app toolset config (admin-level toolset type config)
 */
export function createMockAppToolsetConfig(overrides?: Partial<AppToolsetConfig>): AppToolsetConfig {
  return {
    toolset_type: 'builtin-exa-search',
    name: 'Exa Web Search',
    description: 'Search the web using Exa AI',
    enabled: true,
    updated_by: 'system',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

/**
 * Create a mock toolset definition (type catalog entry)
 */
export function createMockToolsetDefinition(overrides?: Partial<ToolsetDefinition>): ToolsetDefinition {
  return {
    toolset_type: 'builtin-exa-search',
    name: 'Exa Web Search',
    description: 'Search the web using Exa AI',
    tools: [createMockToolDefinition()],
    ...overrides,
  };
}
