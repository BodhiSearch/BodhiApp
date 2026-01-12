# Model Metadata API - Research Report

**Date**: 2026-01-12
**Research Focus**: Model metadata/capabilities APIs across AI platforms

## Executive Summary

This research explores model metadata/capabilities APIs across:
1. **llama.cpp /props** - Local GGUF model properties extraction
2. **OpenRouter** - Multi-provider gateway with rich model metadata
3. **OpenAI** - Minimal model info (no capabilities exposed)
4. **Ollama** - Local model metadata with simple capabilities array
5. **LiteLLM** - Proxy with pricing/context data in JSON format

**Goal**: Design a unified model capabilities endpoint for BodhiApp that works for both local GGUF models and remote API providers.

---

## 1. llama.cpp /props Endpoint

### Location
- **Handler**: `/tools/server/server-context.cpp` (lines 3371-3425)
- **Metadata struct**: `/tools/server/server-context.h` (lines 12-41)

### Response Schema

```json
{
  "default_generation_settings": {
    "n_ctx": 4096,
    "params": {
      "seed": -1,
      "temperature": 0.8,
      "top_k": 40,
      "top_p": 0.95,
      "min_p": 0.05,
      "repeat_penalty": 1.1,
      "presence_penalty": 0.0,
      "frequency_penalty": 0.0,
      "max_tokens": -1,
      "samplers": ["top_k", "tfs_z", "typ_p", "top_p", "min_p", "temperature"],
      "reasoning_format": "deepseek",
      "chat_format": "chatml"
    }
  },
  "total_slots": 1,
  "model_alias": "llama-3.2-1b",
  "model_path": "/models/llama-3.2-1b.Q4_K_M.gguf",
  "modalities": {
    "vision": true,
    "audio": false
  },
  "chat_template": "{% for message in messages %}...",
  "bos_token": "<|begin_of_text|>",
  "eos_token": "<|end_of_text|>",
  "build_info": "b4567-abc123",
  "is_sleeping": false,
  "endpoint_slots": true,
  "endpoint_metrics": true
}
```

### How Metadata is Extracted from GGUF

| Source | Function | Data Extracted |
|--------|----------|----------------|
| llama.h API | `llama_vocab_bos()`/`llama_vocab_eos()` | Special tokens (BOS/EOS) |
| llama.h API | `llama_model_n_ctx_train()` | Training context length |
| llama.h API | `llama_model_n_params()` | Total parameter count |
| llama.h API | `llama_model_size()` | Model size in bytes |
| llama.h API | `llama_vocab_n_tokens()` | Vocabulary size |
| llama.h API | `llama_model_chat_template()` | Jinja2 chat template |
| Multimodal ctx | `mtmd_support_vision()` | Vision projector loaded (bool) |
| Multimodal ctx | `mtmd_support_audio()` | Audio projector loaded (bool) |
| GGUF metadata | `llama_model_meta_val_str()` | Arbitrary key-value pairs from GGUF |

**Key Insight**: llama.cpp extracts metadata from loaded models at runtime. Vision/audio detection requires the multimodal projector (`--mmproj`) to be loaded.

### WebUI Usage Patterns

The llama.cpp webui uses `/props` for:
- **Modalities detection**: Enable/disable image/audio upload UI elements
- **Settings sync**: Sync default sampling parameters with server configuration
- **Model info display**: Show model name, context size, build info in UI
- **Model selector**: Filter models by capability compatibility with conversation
- **Template rendering**: Use chat template for proper prompt formatting

**Pattern**: UI fetches `/props` on load and adapts features based on model capabilities.

---

## 2. OpenRouter Models API

### Endpoint
`GET https://openrouter.ai/api/v1/models`

### Complete Schema

```json
{
  "data": [{
    "id": "anthropic/claude-sonnet-4",
    "canonical_slug": "claude-sonnet-4",
    "name": "Claude Sonnet 4",
    "hugging_face_id": null,
    "created": 1699900000,
    "description": "Anthropic's latest model...",

    "architecture": {
      "tokenizer": "claude",
      "instruct_type": "claude",
      "input_modalities": ["text", "image", "file"],
      "output_modalities": ["text"],
      "modality": "text+image->text"
    },

    "context_length": 200000,
    "top_provider": {
      "context_length": 200000,
      "max_completion_tokens": 8192,
      "is_moderated": true
    },

    "pricing": {
      "prompt": "0.000003",
      "completion": "0.000015",
      "request": "0",
      "image": "0.00048",
      "image_token": "0",
      "image_output": "0",
      "audio": "0.000024",
      "input_audio_cache": "0",
      "web_search": "0",
      "internal_reasoning": "0.000015",
      "input_cache_read": "0.0000003",
      "input_cache_write": "0.00000375",
      "discount": "0"
    },

    "supported_parameters": [
      "temperature", "top_p", "top_k", "frequency_penalty",
      "presence_penalty", "max_tokens", "stop", "tools",
      "tool_choice", "reasoning", "reasoning_effort",
      "web_search_options", "structured_outputs"
    ],

    "default_parameters": {
      "temperature": 1.0,
      "top_p": 0.999,
      "frequency_penalty": 0.0
    },

    "per_request_limits": null
  }]
}
```

### Key Features

1. **Rich modality info**:
   - `input_modalities`: text, image, file, audio, video
   - `output_modalities`: text, image, embeddings
   - Granular capability flags

2. **Detailed pricing breakdown**:
   - Per-token costs for prompt/completion
   - Image, audio pricing
   - Cache read/write costs
   - Internal reasoning tokens (for thinking models)

3. **Thinking/reasoning support**:
   - `:thinking` suffix for model variants (e.g., `deepseek/deepseek-r1:thinking`)
   - `internal_reasoning` pricing field indicates thinking capability
   - Claude 4 uses hybrid approach with `reasoning.effort` parameter

4. **Capability detection via parameters**:
   - `supported_parameters` array enables feature detection
   - Client can check for "tools", "structured_outputs", "reasoning"

5. **Provider metadata**:
   - `top_provider` shows actual context limits from best provider
   - `is_moderated` flag for content filtering

**Pattern**: OpenRouter provides comprehensive metadata inline in model list, enabling rich client-side filtering and feature detection.

### :thinking Variant System

- **Suffix-based activation**: Append `:thinking` to model ID
- **Claude 4 evolution**: Uses `reasoning.max_tokens` or `reasoning.effort` parameters instead
- **Pricing indicator**: `internal_reasoning` field shows if model supports extended thinking
- **Backward compatibility challenge**: Applications hardcoded to `:thinking` suffix broke with Claude 4 changes

---

## 3. OpenAI /v1/models API

### Current Schema (Limited)

```json
{
  "object": "list",
  "data": [{
    "id": "gpt-4",
    "object": "model",
    "created": 1686935002,
    "owned_by": "openai"
  }]
}
```

### Notable Limitations

- **No capability information exposed**: Vision, function calling, structured outputs not indicated
- **No context length**: Clients must hardcode context limits
- **No pricing**: Cost information not available via API
- **Developer requests**: Community has requested capability flags but not implemented

**Community Discussion**: [OpenAI Forum - Expose Model Capabilities](https://community.openai.com/t/expose-model-capabilities-in-the-v1-models-api-response/1314117)

**Pattern**: OpenAI assumes clients have out-of-band knowledge of model capabilities.

---

## 4. Ollama /api/show

### Endpoint
`POST /api/show` (model-specific)

### Response Schema

```json
{
  "license": "MIT",
  "template": "{{ .System }}\n{{ .Prompt }}",
  "parameters": "stop <|im_end|>\nstop <|im_start|>",
  "modified_at": "2024-01-15T10:30:00Z",

  "capabilities": ["completion", "vision"],

  "details": {
    "parent_model": "",
    "format": "gguf",
    "family": "llama",
    "families": ["llama"],
    "parameter_size": "8B",
    "quantization_level": "Q4_K_M"
  },

  "model_info": {
    "general.architecture": "llama",
    "general.parameter_count": 8000000000,
    "llama.context_length": 131072,
    "llama.attention.head_count": 32,
    "llama.block_count": 32,
    "vision.image_size": 560,
    "mm.tokens_per_image": 196
  }
}
```

### Key Features

- **Simple capabilities array**: String flags like "completion", "vision"
- **GGUF metadata passthrough**: `model_info` exposes raw GGUF metadata
- **Human-readable details**: Family, size, quantization in structured format
- **Vision-specific metadata**: Image size, tokens per image for multimodal models

**Pattern**: Ollama exposes raw GGUF metadata with minimal abstraction, plus simple capability flags.

---

## 5. LiteLLM Model Metadata

### Data Source
Static JSON file: [`model_prices_and_context_window.json`](https://github.com/BerriAI/litellm/blob/main/model_prices_and_context_window.json)

### Schema per Model

```json
{
  "gpt-4": {
    "max_tokens": 8192,
    "max_input_tokens": 128000,
    "max_output_tokens": 8192,
    "input_cost_per_token": 0.00003,
    "output_cost_per_token": 0.00006,
    "mode": "chat",
    "supports_vision": true,
    "supports_function_calling": true,
    "supports_audio_input": false,
    "supports_audio_output": false
  }
}
```

### Capability Detection Methods

- **Static lookup**: `litellm.get_supported_openai_params(model)` - Returns supported parameters
- **Specific checks**: `litellm.utils.supports_vision(model)` - Boolean capability checks
- **Runtime validation**: Warns if unsupported parameter used with `drop_params=False`

### Maintenance Approach

- **Bundled JSON**: Updated with library releases
- **Community contributions**: Users submit PRs for new models
- **Manual curation**: No automated sync from providers

**Pattern**: LiteLLM maintains a comprehensive static capability database, trading freshness for reliability.

---

## 7. BodhiApp Current State

### Existing Endpoints

| Endpoint | Purpose | Response Type |
|----------|---------|---------------|
| `GET /v1/models` | OpenAI-compatible model list | Basic model info only |
| `GET /v1/models/{id}` | Single model details | OpenAI Model format (minimal) |
| `GET /api/models` | All aliases (paginated) | UserAlias/ModelAlias/ApiAlias |
| `GET /bodhi/v1/api-models` | Remote API configurations | ApiModelResponse |

### Model Types Architecture

```rust
pub enum Alias {
  User(UserAlias),   // User-defined GGUF aliases from YAML
  Model(ModelAlias), // Auto-discovered GGUF files from filesystem
  Api(ApiAlias),     // Remote API providers (OpenAI, OpenRouter, etc.)
}
```

### Current Limitations

- ❌ No rich capability metadata exposed
- ❌ No modality information (vision/audio/thinking)
- ❌ No pricing information for cost estimation
- ❌ No context length from local GGUF models
- ❌ No unified capabilities endpoint
- ❌ Clients must hardcode model capabilities

---

## 8. Comparison Matrix

| Feature | llama.cpp | OpenRouter | Ollama | LiteLLM | OpenAI | AI Gateway |
|---------|-----------|------------|--------|---------|--------|---------|
| **Context length** | ✅ Runtime | ✅ Static | ✅ GGUF | ✅ Static | ❌ None | ❌ Internal |
| **Vision modality** | ✅ Projector | ✅ Input array | ✅ Capability | ✅ Flag | ❌ None | ❌ Internal |
| **Audio modality** | ✅ Projector | ✅ Input array | ❌ None | ✅ Flag | ❌ None | ❌ Internal |
| **Thinking/reasoning** | ✅ Format | ✅ :suffix | ❌ None | ❌ None | ❌ None | ❌ Internal |
| **Pricing info** | ❌ None | ✅ Detailed | ❌ None | ✅ Cost | ❌ None | ❌ Internal |
| **Sampling defaults** | ✅ Params | ✅ Defaults | ❌ None | ❌ None | ❌ None | ❌ Internal |
| **Chat template** | ✅ Jinja2 | ❌ None | ✅ Template | ❌ None | ❌ None | ❌ Internal |
| **Special tokens** | ✅ BOS/EOS | ❌ None | ❌ None | ❌ None | ❌ None | ❌ Internal |
| **Parameter support** | ✅ Samplers | ✅ Array | ❌ None | ✅ Check | ❌ None | ❌ Internal |
| **Model architecture** | ❌ None | ✅ Struct | ✅ GGUF | ❌ None | ❌ None | ❌ Internal |
| **Model size/params** | ✅ API | ❌ None | ✅ GGUF | ❌ None | ❌ None | ❌ Internal |
| **Data source** | Runtime | API | File | JSON | N/A | N/A |
| **Update mechanism** | Live | Cached | Parse | Release | N/A | N/A |

### Key Insights

1. **llama.cpp**: Runtime extraction, most complete for local models
2. **OpenRouter**: Most comprehensive remote model metadata
3. **Ollama**: Simple but effective, direct GGUF exposure
4. **LiteLLM**: Static but reliable, good capability coverage
5. **OpenAI**: Minimal, assumes out-of-band knowledge

### Best Practices Identified

- **Hybrid approach works best**: Static + dynamic metadata
- **Capability flags are essential**: Simple booleans for UI decisions
- **Pricing needs staleness tracking**: Costs change frequently
- **Context length is critical**: Most requested capability
- **Canonical IDs with aliases**: Enable flexible matching

---

## Sources

### Primary Research

- **llama.cpp**: Code analysis of `/tools/server/` implementation
- **llama.cpp WebUI**: Usage patterns in `/tools/server/webui/`
- **OpenRouter**: [Models API Documentation](https://openrouter.ai/docs/api/api-reference/models/get-models)
- **OpenRouter**: [Thinking Variants](https://openrouter.ai/docs/guides/routing/model-variants/thinking)
- **Ollama**: [Show Model Details API](https://docs.ollama.com/api-reference/show-model-details)
- **Ollama**: [Multimodal Models Blog](https://ollama.com/blog/multimodal-models)
- **LiteLLM**: [GitHub model_prices_and_context_window.json](https://github.com/BerriAI/litellm/blob/main/model_prices_and_context_window.json)
- **OpenAI**: [Models API Reference](https://platform.openai.com/docs/api-reference/models)
- **OpenAI Community**: [Feature Request Discussion](https://community.openai.com/t/expose-model-capabilities-in-the-v1-models-api-response/1314117)

### Web Research

- OpenRouter pricing and model catalog review (2025)
- LiteLLM proxy architecture analysis
- AI Gateway comparison articles (2025)
