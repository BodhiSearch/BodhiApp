/**
 * Fixture factories for the API-model catalog (`/api/v1/catalog/models*`).
 *
 * Typed by `@bodhiapp/reference-api-types`. Values mirror the real dev-api shapes: flat fields
 * (`context_limit`, `pricing.input_per_m`, `caps[]`, `modalities_in/out`); `status: null` means
 * Stable (the UI synthesizes the label); facets are global value arrays (the full available set).
 */
import type { ModelDetailResponse, ModelLite, ModelsListResponse } from '@bodhiapp/reference-api-types';

export function createModelLite(overrides?: Partial<ModelLite>): ModelLite {
  return {
    slug: 'anthropic',
    model_id: 'claude-sonnet-4.5',
    name: 'Claude Sonnet 4.5',
    family: 'claude',
    context_limit: 200000,
    output_limit: 64000,
    pricing: { input_per_m: 3, output_per_m: 15, cache_read_per_m: 0.3, cache_write_per_m: 3.75 },
    caps: ['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision'],
    status: null,
    open_weights: false,
    modalities_in: ['text', 'image'],
    modalities_out: ['text'],
    provider_count: 4,
    release_date: '2025-09-29',
    last_updated: '2025-10-15',
    ...overrides,
  };
}

export function createModelsListResponse(
  items: ModelLite[] = createDefaultCatalogModels(),
  overrides?: Partial<ModelsListResponse>
): ModelsListResponse {
  return {
    items,
    page: 1,
    page_size: 30,
    total: items.length,
    facets: {
      capability: ['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision'],
      modality: ['text', 'audio', 'image', 'video', 'pdf'],
      status: ['stable', 'alpha', 'beta', 'deprecated'],
      provider: ['nano-gpt', 'kilo', 'openrouter', 'vercel', 'anthropic', 'openai'],
      family: ['claude', 'gpt', 'gemini', 'llama', 'deepseek-v3'],
      open_weights: ['open', 'closed'],
    },
    ...overrides,
  };
}

export function createDefaultCatalogModels(): ModelLite[] {
  return [
    createModelLite(),
    createModelLite({
      slug: 'openai',
      model_id: 'gpt-5',
      name: 'GPT-5',
      family: 'gpt',
      context_limit: 400000,
      output_limit: 128000,
      pricing: { input_per_m: 1.25, output_per_m: 10, cache_read_per_m: 0.125, cache_write_per_m: null },
      provider_count: 3,
    }),
    createModelLite({
      slug: 'meta-llama',
      model_id: 'llama-3.3-70b-instruct',
      name: 'Llama 3.3 70B Instruct',
      family: 'llama',
      context_limit: 131072,
      output_limit: 32768,
      pricing: { input_per_m: 0, output_per_m: 0, cache_read_per_m: null, cache_write_per_m: null },
      caps: ['tool_call'],
      open_weights: true,
      modalities_in: ['text'],
      modalities_out: ['text'],
      provider_count: 8,
      status: 'deprecated',
    }),
  ];
}

export function createModelDetail(overrides?: Partial<ModelDetailResponse>): ModelDetailResponse {
  return {
    slug: 'anthropic',
    model_id: 'claude-sonnet-4.5',
    name: 'Claude Sonnet 4.5',
    family: 'claude',
    status: null,
    reasoning: true,
    tool_call: true,
    structured_output: true,
    attachment: true,
    open_weights: false,
    temperature: true,
    reasoning_options: null,
    context_limit: 200000,
    output_limit: 64000,
    modalities_in: ['text', 'image'],
    modalities_out: ['text'],
    release_date: '2025-09-29',
    last_updated: '2025-10-15',
    knowledge_cutoff: '2025-03',
    pricing: {
      currency: 'USD',
      input_per_m: 3,
      output_per_m: 15,
      cache_read_per_m: 0.3,
      cache_write_per_m: 3.75,
      reasoning_per_m: null,
      input_audio_per_m: null,
      output_audio_per_m: null,
      pricing_source: 'modelsdev',
    },
    license: null,
    links: null,
    weights: null,
    benchmarks: null,
    served_by: [
      {
        slug: 'anthropic',
        name: 'Anthropic',
        logo_url: '/api/v1/catalog/logos/anthropic.svg',
        base_url: 'https://api.anthropic.com/v1',
        pricing: { input_per_m: 3, output_per_m: 15, cache_read_per_m: 0.3, cache_write_per_m: 3.75 },
      },
      {
        slug: 'openrouter',
        name: 'OpenRouter',
        logo_url: '/api/v1/catalog/logos/openrouter.svg',
        base_url: 'https://openrouter.ai/api/v1',
        pricing: { input_per_m: 3.3, output_per_m: 16.5, cache_read_per_m: null, cache_write_per_m: null },
      },
    ],
    bridge: {
      api_format: 'anthropic',
      base_url: 'https://api.anthropic.com/v1',
      base_url_source: 'modelsdev_api',
      base_url_requires_substitution: false,
    },
    ...overrides,
  };
}
