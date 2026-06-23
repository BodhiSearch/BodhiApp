/**
 * Fixture factories for the API-provider catalog (`/api/v1/catalog/providers*`).
 *
 * Typed by `@bodhiapp/reference-api-types` (the API's own published wire types). Values mirror the
 * real dev-api shapes: `logo_url` is a relative path that currently 404s (UI falls back to a
 * monogram); `api_base_url` is null for native providers; facets are page-recomputed counts.
 */
import type {
  ProviderDetailResponse,
  ProviderListResponse,
  ProviderModelRow,
  ProviderModelsResponse,
  ProviderSummary,
} from '@bodhiapp/reference-api-types';

export function createProviderSummary(overrides?: Partial<ProviderSummary>): ProviderSummary {
  return {
    slug: 'nano-gpt',
    name: 'NanoGPT',
    logo_url: '/api/v1/catalog/logos/nano-gpt.svg',
    model_count: 617,
    rank: 1,
    api_base_url: 'https://nano-gpt.com/api/v1',
    provider_shape: 'openai-compatible',
    api_format_hint: 'openai',
    capabilities_summary: ['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision'],
    pricing_summary: { min_in_per_m: 0, min_out_per_m: 0 },
    ...overrides,
  };
}

export function createProviderListResponse(
  items: ProviderSummary[] = createDefaultProviders(),
  overrides?: Partial<ProviderListResponse>
): ProviderListResponse {
  return {
    items,
    page: 1,
    page_size: 30,
    total: items.length,
    facets: {
      capability: { reasoning: 80, tool_call: 90, structured_output: 60, attachment: 70, vision: 65 },
      api_format: { openai: 70, anthropic: 8, gemini: 6, other: 12 },
    },
    ...overrides,
  };
}

export function createDefaultProviders(): ProviderSummary[] {
  return [
    createProviderSummary(),
    createProviderSummary({
      slug: 'openrouter',
      name: 'OpenRouter',
      logo_url: '/api/v1/catalog/logos/openrouter.svg',
      model_count: 338,
      rank: 3,
      api_base_url: 'https://openrouter.ai/api/v1',
      provider_shape: 'openrouter',
    }),
    createProviderSummary({
      slug: 'vercel',
      name: 'Vercel AI Gateway',
      logo_url: '/api/v1/catalog/logos/vercel.svg',
      model_count: 281,
      rank: 4,
      api_base_url: null,
      provider_shape: 'native',
      api_format_hint: 'other',
    }),
  ];
}

export function createProviderDetail(overrides?: Partial<ProviderDetailResponse>): ProviderDetailResponse {
  return {
    slug: 'nano-gpt',
    name: 'NanoGPT',
    logo_url: '/api/v1/catalog/logos/nano-gpt.svg',
    model_count: 617,
    env: ['NANO_GPT_API_KEY'],
    npm: '@ai-sdk/openai-compatible',
    doc_url: 'https://docs.nano-gpt.com',
    api_base_url: 'https://nano-gpt.com/api/v1',
    provider_shape: 'openai-compatible',
    bridge: {
      api_format: 'openai',
      base_url: 'https://nano-gpt.com/api/v1',
      base_url_source: 'modelsdev_api',
      base_url_requires_substitution: false,
    },
    ...overrides,
  };
}

export function createProviderModelRow(overrides?: Partial<ProviderModelRow>): ProviderModelRow {
  return {
    model_id: 'anthropic/claude-sonnet-4.5',
    name: 'Claude Sonnet 4.5',
    caps: ['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision'],
    context_limit: 200000,
    output_limit: 64000,
    pricing: { input_per_m: 3, output_per_m: 15, cache_read_per_m: 0.3, cache_write_per_m: 3.75 },
    status: null,
    modalities_in: ['text', 'image'],
    modalities_out: ['text'],
    ...overrides,
  };
}

export function createProviderModelsResponse(
  items: ProviderModelRow[] = [
    createProviderModelRow(),
    createProviderModelRow({
      model_id: 'openai/gpt-5',
      name: 'GPT-5',
      context_limit: 400000,
      output_limit: 128000,
      pricing: { input_per_m: 1.25, output_per_m: 10, cache_read_per_m: 0.125, cache_write_per_m: null },
    }),
  ]
): ProviderModelsResponse {
  return { items, total: items.length };
}
