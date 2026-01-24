# Plan: Update Model Catalog with Latest GGUF Models

## Summary

Update `crates/bodhi/src/app/ui/setup/download-models/data.ts` with newer, more capable models released since October 2025. Target: 32GB+ Mac users, ggml-org repos preferred with trusted fallbacks.

## Research Sources

- [AIME 2025 Leaderboard](https://llm-stats.com/benchmarks/aime-2025) - Math reasoning benchmarks
- [MTEB Leaderboard](https://huggingface.co/spaces/mteb/leaderboard) - Embedding benchmarks
- [Open LLM Leaderboard](https://huggingface.co/spaces/open-llm-leaderboard/open_llm_leaderboard) - General benchmarks
- [LocalLLM Guide](https://localllm.in/blog/best-local-llm-for-coding-2025) - Local inference recommendations
- [Quantization Guide](https://enclaveai.app/blog/2025/11/12/practical-quantization-guide-iphone-mac-gguf/) - Q4_K_M vs Q8_0 analysis

## Key Benchmark Findings

| Model | AIME 2025 | MATH 500 | HumanEval | MTEB |
|-------|-----------|----------|-----------|------|
| GLM-4.7 | 95.7% | - | - | - |
| Phi-4-Reasoning | - | 89.96% | - | - |
| Qwen3-32B | 92.3% | - | 91.0% | - |
| DeepSeek R1 Distill 14B | - | 93.90% | - | - |
| Nemotron-Nano-3 | - | - | Leader on LiveCodeBench | - |
| Qwen3-Embedding-8B | - | - | - | 70.58 (#1) |

## Final Model Lineup

### Chat Models (6) - Provider Diversity: NVIDIA, Microsoft, Alibaba, Mistral, Zhipu, OpenAI

| # | New Model | Badge | Quantization | Size | Repo | Why Best |
|---|-----------|-------|--------------|------|------|----------|
| 1 | **Nemotron-Nano-3 30B-A3B** | ⭐ Best Overall | Q4_K_M | 24.5GB | ggml-org/Nemotron-Nano-3-30B-A3B-GGUF | NVIDIA MoE (3B active), coding leader on LiveCodeBench |
| 2 | **Phi-4-Reasoning 14B** | ⭐ Best Reasoning | Q8_0 | 15.6GB | bartowski/microsoft_Phi-4-reasoning-GGUF | Most robust reasoning (82.2% robustness), competes with 5-50x larger models |
| 3 | **Qwen3-32B** | ⭐ Best Advanced | Q4_K_M | 19.8GB | ggml-org/Qwen3-32B-GGUF | 92.3% AIME25, 91% HumanEval, trained on 36T tokens |
| 4 | **Mistral Small 3.2 24B** | Long Context | Q4_K_M | 14.3GB | unsloth/Mistral-Small-3.2-24B-Instruct-2506-GGUF | 128K context, vision support, fits single RTX 4090 |
| 5 | **GLM-4.6V-Flash 9B** | Multimodal Latest | Q8_0 | 10GB | ggml-org/GLM-4.6V-Flash-GGUF | Zhipu's vision model, Dec 2025 release, native tool calling |
| 6 | **GPT-OSS 20B** | OpenAI Open-Weight | Q4_K_M | 12.1GB | ggml-org/gpt-oss-20b-GGUF | OpenAI's first open-weight, 42 tok/s, perfect logic scores |

### Embedding Models (4)

| # | New Model | Badge | Quantization | Size | Repo | Why Best |
|---|-----------|-------|--------------|------|------|----------|
| 1 | **Qwen3-Embedding-8B** | ⭐ Top Choice | Q8_0 | 8.05GB | Qwen/Qwen3-Embedding-8B-GGUF | #1 MTEB multilingual (70.58), 100+ languages |
| 2 | **BGE-M3** | ⭐ Best Multilingual | Q4_K_M | 438MB | gpustack/bge-m3-GGUF | Multi-functionality (dense, sparse, multi-vector), 8K context |
| 3 | **Nomic-Embed-v1.5** | Most Efficient | Q8_0 | 274MB | nomic-ai/nomic-embed-text-v1.5-GGUF | 137M params, Matryoshka learning (64-768 dims) |
| 4 | **BGE-Large-EN-v1.5** | Strong English | Q4_K_M | 208MB | mradermacher/bge-large-en-v1.5-GGUF | English-optimized, 84.7% accuracy, 1024 dimensions |

## Implementation Steps

### Phase update-data: Update data.ts

**File:** `crates/bodhi/src/app/ui/setup/download-models/data.ts`

#### Chat Models Changes:

1. **Replace Qwen2.5-14B with Nemotron-Nano-3 30B-A3B:**
```typescript
{
  id: 'nemotron-nano-3-30b',
  name: 'Nemotron Nano 3 30B',
  repo: 'ggml-org/Nemotron-Nano-3-30B-A3B-GGUF',
  filename: 'Nemotron-Nano-3-30B-A3B-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '24.5GB',
  parameters: '30B (3B active)',
  category: 'chat',
  tier: 'premium',
  badge: '⭐ Best Overall',
  ratings: { quality: 5, speed: 4.5, specialization: 5 },
  benchmarks: { liveCodeBench: 'Leader' },
  contextWindow: '128K',
  memoryEstimate: '~24.5GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/ggml-org/Nemotron-Nano-3-30B-A3B-GGUF',
  tooltipContent: {
    strengths: ['NVIDIA MoE architecture (3B active params)', 'Coding benchmark leader', 'Excellent efficiency'],
    useCase: 'Best all-around model for general tasks, exceptional coding capabilities',
    researchNotes: 'NVIDIA\'s mixture-of-experts model with only 3B activated parameters. Leads LiveCodeBench.'
  }
}
```

2. **Update Phi-4 to Phi-4-Reasoning:**
```typescript
{
  id: 'phi-4-reasoning',
  name: 'Phi-4 Reasoning',
  repo: 'bartowski/microsoft_Phi-4-reasoning-GGUF',
  filename: 'Phi-4-reasoning-Q8_0.gguf',
  quantization: 'Q8_0',
  size: '15.6GB',
  parameters: '14B',
  ...
  benchmarks: { mathRobustness: '82.2%' },
  tooltipContent: {
    strengths: ['Most robust reasoning model', 'Competes with 5-50x larger models', 'Exceptional math capabilities'],
    useCase: 'Ideal for complex reasoning, mathematical problems, and tasks requiring consistency',
    researchNotes: 'Microsoft\'s reasoning-optimized SLM. 82.2% robustness on linguistic variation tests.'
  }
}
```

3. **Replace Qwen2.5-32B with Qwen3-32B:**
```typescript
{
  id: 'qwen3-32b',
  name: 'Qwen3 32B',
  repo: 'ggml-org/Qwen3-32B-GGUF',
  filename: 'Qwen3-32B-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '19.8GB',
  ...
  benchmarks: { aime25: '92.3%', humanEval: '91.0%' },
  tooltipContent: {
    strengths: ['40% better than Qwen2.5', 'Trained on 36T tokens', '92.3% AIME 2025'],
    useCase: 'Advanced tasks requiring maximum capability, strong math/coding',
    researchNotes: 'Major upgrade from Qwen2.5. 36T tokens (2x predecessor). 15% fewer hallucinations.'
  }
}
```

4. **Replace Mistral NeMo with Mistral Small 3.2:**
```typescript
{
  id: 'mistral-small-3.2',
  name: 'Mistral Small 3.2',
  repo: 'unsloth/Mistral-Small-3.2-24B-Instruct-2506-GGUF',
  filename: 'Mistral-Small-3.2-24B-Instruct-2506-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '14.3GB',
  parameters: '24B',
  ...
  contextWindow: '128K',
  tooltipContent: {
    strengths: ['128K context window', 'Vision understanding support', 'Fits single RTX 4090 or 32GB Mac'],
    useCase: 'Best for long documents, multimodal tasks, and vision-language applications',
    researchNotes: 'June 2025 release. State-of-the-art vision understanding with long context.'
  }
}
```

5. **Replace Gemma 3 with GLM-4.6V-Flash:**
```typescript
{
  id: 'glm-4.6v-flash',
  name: 'GLM-4.6V Flash',
  repo: 'ggml-org/GLM-4.6V-Flash-GGUF',
  filename: 'GLM-4.6V-Flash-Q8_0.gguf',
  quantization: 'Q8_0',
  size: '10GB',
  parameters: '9B',
  ...
  badge: 'Multimodal Latest',
  tooltipContent: {
    strengths: ['Native vision support', 'Native tool calling', 'December 2025 release', '128K context'],
    useCase: 'Best for vision-language tasks, tool use, and multimodal applications',
    researchNotes: 'Zhipu AI\'s latest vision model. 128K context with native tool calling support.'
  }
}
```

6. **Keep GPT-OSS 20B** - No changes needed (already optimal)

#### Embedding Models Changes:

1. **Upgrade Qwen3-Embedding-4B to 8B:**
```typescript
{
  id: 'qwen3-embedding-8b',
  name: 'Qwen3 Embedding 8B',
  repo: 'Qwen/Qwen3-Embedding-8B-GGUF',
  filename: 'Qwen3-Embedding-8B-Q8_0.gguf',
  quantization: 'Q8_0',
  size: '8.05GB',
  parameters: '8B',
  ...
  benchmarks: { mteb: 70.58 },
  tooltipContent: {
    strengths: ['#1 on MTEB multilingual leaderboard', '100+ languages', 'Upgraded from 4B'],
    useCase: 'Best overall embedding model for RAG applications and semantic search',
    researchNotes: '#1 MTEB multilingual (70.58 as of Jan 2026). Significant upgrade from 4B variant.'
  }
}
```

2. **Keep BGE-M3, Nomic-Embed-v1.5, BGE-Large-EN-v1.5** - Already optimal

## Verification

1. **TypeScript build:** `cd crates/bodhi && npm run build`
2. **Unit tests:** `cd crates/bodhi && npm test`
3. **Visual verification:** `cd crates/bodhi && npm run dev` - verify download-models page renders
4. **Repo validation:** Manually verify each HuggingFace URL exists and files are available:
   - https://huggingface.co/ggml-org/Nemotron-Nano-3-30B-A3B-GGUF
   - https://huggingface.co/bartowski/microsoft_Phi-4-reasoning-GGUF
   - https://huggingface.co/ggml-org/Qwen3-32B-GGUF
   - https://huggingface.co/unsloth/Mistral-Small-3.2-24B-Instruct-2506-GGUF
   - https://huggingface.co/ggml-org/GLM-4.6V-Flash-GGUF
   - https://huggingface.co/Qwen/Qwen3-Embedding-8B-GGUF

## Provider Diversity Summary

| Provider | Model | Category |
|----------|-------|----------|
| NVIDIA | Nemotron-Nano-3 | Best Overall |
| Microsoft | Phi-4-Reasoning | Best Reasoning |
| Alibaba/Qwen | Qwen3-32B | Best Advanced |
| Mistral AI | Mistral Small 3.2 | Long Context |
| Zhipu AI/zai | GLM-4.6V-Flash | Multimodal Latest |
| OpenAI | GPT-OSS 20B | OpenAI Open-Weight |

All 6 chat models from different providers ✓
