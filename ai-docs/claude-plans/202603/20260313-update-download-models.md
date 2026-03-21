# Plan: Update Model Catalog for March 2026 Release

## Context

The last BodhiApp release was in January 2026. Since then, major model releases have shifted the landscape — most notably the Qwen 3.5 family (Feb 16, 2026), GLM-4.7-Flash (Jan 2026), and continued Gemma 3 availability. The current catalog contains several models that are now outclassed on benchmarks, file size, and context window. This update refreshes the setup wizard's model recommendations for 32GB Mac M-series users.

## File to Modify

`crates/bodhi/src/app/ui/setup/download-models/data.ts`

## Chat Models: Final Catalog (6 models)

### 1. Qwen3.5-35B-A3B — ⭐ Best Overall (premium)
**Replaces: Nemotron Nano 3 30B**

```typescript
{
  id: 'qwen3.5-35b-a3b',
  name: 'Qwen3.5 35B-A3B',
  repo: 'unsloth/Qwen3.5-35B-A3B-GGUF',
  filename: 'Qwen3.5-35B-A3B-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '22GB',
  parameters: '35B (3B active)',
  category: 'chat',
  tier: 'premium',
  badge: '⭐ Best Overall',
  ratings: { quality: 5, speed: 4.5, specialization: 5 },
  benchmarks: {},
  contextWindow: '262K',
  memoryEstimate: '~22GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/unsloth/Qwen3.5-35B-A3B-GGUF',
  tooltipContent: {
    strengths: ['MoE architecture (3B active of 35B)', 'Beats GPT-OSS-120B on GPQA Diamond', 'Natively multimodal'],
    useCase: 'Best all-around model for general tasks with excellent efficiency',
    researchNotes: "Alibaba's Feb 2026 flagship MoE. 262K context. Outperforms models 13x its size on key benchmarks.",
  },
}
```

### 2. Qwen3.5-27B — ⭐ Best Dense (premium)
**Replaces: Qwen3 32B**

```typescript
{
  id: 'qwen3.5-27b',
  name: 'Qwen3.5 27B',
  repo: 'unsloth/Qwen3.5-27B-GGUF',
  filename: 'Qwen3.5-27B-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '16.7GB',
  parameters: '27B',
  category: 'chat',
  tier: 'premium',
  badge: '⭐ Best Dense',
  ratings: { quality: 5, speed: 4, specialization: 5 },
  benchmarks: {},
  contextWindow: '262K',
  memoryEstimate: '~16.7GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/unsloth/Qwen3.5-27B-GGUF',
  tooltipContent: {
    strengths: ['Dense architecture — all 27B params active', 'Highest per-token reasoning density', '262K context window'],
    useCase: 'Deep reasoning tasks where every parameter contributes — math, coding, analysis',
    researchNotes: 'Feb 2026. Dense model complements MoE variants. Major upgrade from Qwen3-32B with smaller footprint.',
  },
}
```

### 3. Phi-4 Reasoning — ⭐ Best Reasoning (premium)
**No change** — kept as-is from current catalog.

### 4. Qwen3.5-9B — Best Value (specialized)
**Replaces: GPT-OSS 20B**

```typescript
{
  id: 'qwen3.5-9b',
  name: 'Qwen3.5 9B',
  repo: 'unsloth/Qwen3.5-9B-GGUF',
  filename: 'Qwen3.5-9B-Q8_0.gguf',
  quantization: 'Q8_0',
  size: '9.5GB',
  parameters: '9B',
  category: 'chat',
  tier: 'specialized',
  badge: 'Best Value',
  ratings: { quality: 4.5, speed: 5, specialization: 4.5 },
  benchmarks: { humanEval: 81.7 },
  contextWindow: '262K',
  memoryEstimate: '~9.5GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/unsloth/Qwen3.5-9B-GGUF',
  tooltipContent: {
    strengths: ['Beats previous-gen 30B models', 'Matches GPT-OSS-120B on GPQA Diamond (81.7)', 'Natively multimodal'],
    useCase: 'Fast inference, running multiple models, or resource-constrained setups',
    researchNotes: 'Mar 2026 small series. 9B params that outperform models 13x larger. 262K context.',
  },
}
```

### 5. GLM-4.7-Flash — Multimodal Latest (specialized)
**Replaces: GLM-4.6V Flash**

```typescript
{
  id: 'glm-4.7-flash',
  name: 'GLM-4.7 Flash',
  repo: 'ggml-org/GLM-4.7-Flash-GGUF',
  filename: 'GLM-4.7-Flash-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '18.1GB',
  parameters: '30B (3.6B active)',
  category: 'chat',
  tier: 'specialized',
  badge: 'Multimodal Latest',
  ratings: { quality: 4.5, speed: 4, specialization: 5 },
  benchmarks: {},
  contextWindow: '200K',
  memoryEstimate: '~18.1GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/ggml-org/GLM-4.7-Flash-GGUF',
  tooltipContent: {
    strengths: ['MoE 30B with 3.6B active params', '200K context with MLA', 'Native vision and tool calling'],
    useCase: 'Vision-language tasks, tool use, and agentic workflows',
    researchNotes: "Zhipu AI's Jan 2026 model. Successor to GLM-4.6V with 200K context and improved benchmarks.",
  },
}
```

### 6. Gemma 3 27B — Google Multimodal (specialized)
**NEW addition**

```typescript
{
  id: 'gemma-3-27b',
  name: 'Gemma 3 27B',
  repo: 'ggml-org/gemma-3-27b-it-GGUF',
  filename: 'gemma-3-27b-it-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '16.5GB',
  parameters: '27B',
  category: 'chat',
  tier: 'specialized',
  badge: 'Google Multimodal',
  ratings: { quality: 4.5, speed: 4, specialization: 5 },
  benchmarks: {},
  contextWindow: '128K',
  memoryEstimate: '~16.5GB',
  license: 'Gemma License',
  licenseUrl: 'https://huggingface.co/ggml-org/gemma-3-27b-it-GGUF',
  tooltipContent: {
    strengths: ['Google QAT quantization (54% less quality loss)', 'Native vision support', 'Outperforms Gemini 1.5 Pro'],
    useCase: 'Multimodal tasks with Google ecosystem. Strong alternative to Qwen for diverse vendor coverage.',
    researchNotes: 'Google DeepMind. QAT preserves near-BF16 quality at 3x less memory. 128K context.',
  },
}
```

## Embedding Models: No Changes

All 4 embedding models remain as-is:
1. Qwen3 Embedding 8B — ⭐ Top Choice (premium)
2. BGE-M3 — ⭐ Best Multilingual (premium)
3. Nomic Embed v1.5 — Most Efficient (specialized)
4. BGE Large EN v1.5 — Strong English (specialized)

## Ordering

Chat models ordered: premium tier first (sorted by descending capability), then specialized tier (sorted by descending capability):
1. Qwen3.5-35B-A3B (premium, MoE flagship)
2. Qwen3.5-27B (premium, dense flagship)
3. Phi-4 Reasoning (premium, reasoning specialist)
4. Qwen3.5-9B (specialized, best value)
5. GLM-4.7-Flash (specialized, multimodal)
6. Gemma 3 27B (specialized, Google multimodal)

## Files to Modify

- `crates/bodhi/src/app/ui/setup/download-models/data.ts` — main catalog data
- `crates/bodhi/src/app/ui/setup/download-models/page.test.tsx` — references Nemotron/GPT-OSS by name, repo, and filename in 5+ locations (lines 121-122, 132, 134, 156-158, 238-239). Must update to new model names.

## Implementation Steps

1. Replace the 6 entries in `chatModelsCatalog` array in `data.ts`
2. Keep `embeddingModelsCatalog` array unchanged
3. Verify exact GGUF filenames from HuggingFace repos during implementation (navigate to repo file listings)
4. Update `page.test.tsx`: replace all Nemotron references with Qwen3.5-35B-A3B equivalents, GPT-OSS references with Qwen3.5-9B equivalents

## Verification

1. `cd crates/bodhi && npm test` — run component tests
2. `cd crates/bodhi && npm run dev` — visual check of setup wizard
3. Verify all HuggingFace repo URLs resolve (spot check 2-3)
4. Check that model cards render correctly with new badge text lengths
