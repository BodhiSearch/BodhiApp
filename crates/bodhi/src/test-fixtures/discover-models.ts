/**
 * Fixture factories for the external Reference API model catalog.
 *
 * Typed by `@bodhiapp/reference-api-types` (the API's own published wire types) so fixtures stay
 * honest against the real shape. Note the v1 reality these encode: LIST rows have null
 * `max_quant_size_bytes`/`total_size_bytes`/`context_max`/`architecture` (populated only on the
 * single-model detail); `params_b` is a string; sizes are bytes; `quants` is detail-only.
 */
import type { ListModelsResponse, Model, Quant } from '@bodhiapp/reference-api-types';

/** A LIST-shaped row: summary fields present, detail-only fields null, no `quants`. */
export function createListModel(overrides?: Partial<Model>): Model {
  return {
    source: 'huggingface',
    type: 'gguf',
    namespace: 'Qwen',
    repo: 'Qwen3-Coder-32B-GGUF',
    pipeline_tag: 'text-generation',
    library: 'gguf',
    license: 'apache-2.0',
    languages: ['en', 'zh'],
    tags: ['coding', 'tool-use', 'reasoning'],
    capabilities: ['tool-use', 'reasoning'],
    specialisation: ['coding', 'reasoning'],
    quant_count: 5,
    quant_bits: [2, 3, 4, 6, 8],
    quant_methods: ['K', 'K_M', '0'],
    max_quant_size_bytes: null,
    total_size_bytes: null,
    architecture: null,
    context_max: null,
    params_b: '32',
    provider: 'Qwen',
    downloads: 443000,
    likes: 9100,
    trending_score: 96,
    created_at: '2025-08-20T00:00:00.000Z',
    last_modified: '2025-09-08T00:00:00.000Z',
    curated: false,
    owner_verified: true,
    fetched_at: '2026-06-21T11:00:00.000Z',
    ...overrides,
  };
}

export function createQuant(overrides?: Partial<Quant>): Quant {
  return {
    name: 'Q4_K_M',
    filename: 'Qwen3-Coder-32B-Q4_K_M.gguf',
    size: 18253611008,
    bits: 4,
    method: 'K_M',
    recommended: true,
    ...overrides,
  };
}

/** A DETAIL-shaped model: detail-only fields populated + a `quants` table. */
export function createDetailModel(overrides?: Partial<Model>): Model {
  return {
    ...createListModel(),
    max_quant_size_bytes: 34489280512,
    total_size_bytes: 34489280512,
    architecture: 'qwen3-moe',
    context_max: 131072,
    quants: [
      createQuant({
        name: 'Q8_0',
        filename: 'Qwen3-Coder-32B-Q8_0.gguf',
        size: 34489280512,
        bits: 8,
        method: '0',
        recommended: false,
      }),
      createQuant({
        name: 'Q4_K_M',
        filename: 'Qwen3-Coder-32B-Q4_K_M.gguf',
        size: 18253611008,
        bits: 4,
        method: 'K_M',
        recommended: true,
      }),
      createQuant({
        name: 'Q2_K',
        filename: 'Qwen3-Coder-32B-Q2_K.gguf',
        size: null,
        bits: 2,
        method: 'K',
        recommended: false,
      }),
    ],
    ...overrides,
  };
}

export function createListResponse(items: Model[], next_cursor: string | null = null): ListModelsResponse {
  return { items, next_cursor, total_estimate: null };
}

/** A small default catalog: an LLM, a multimodal repo, and a second LLM. */
export function createDefaultCatalog(): Model[] {
  return [
    createListModel(),
    createListModel({
      namespace: 'Qwen',
      repo: 'Qwen2.5-VL-7B-Instruct-GGUF',
      pipeline_tag: 'image-text-to-text',
      tags: ['vision', 'tool-use', 'chat'],
      params_b: '7',
      downloads: 380000,
      likes: 7500,
      trending_score: 88,
    }),
    createListModel({
      namespace: 'meta-llama',
      repo: 'Llama-3.3-70B-Instruct-GGUF',
      license: 'llama3.3',
      params_b: '70',
      downloads: 820000,
      likes: 14000,
      trending_score: 71,
    }),
  ];
}
