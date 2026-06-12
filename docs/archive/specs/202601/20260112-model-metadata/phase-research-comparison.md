# Model Capability Detection - Tool Comparison

## Executive Summary

| Tool | Vision Detection | Tool Calling Detection | Other Capabilities |
|------|------------------|----------------------|-------------------|
| **Ollama** | Library categorization + model metadata | Try-and-fail (test with tools) | Browsable search categories |
| **vLLM** | GGUF metadata + mmproj files | Architecture-specific support (incremental) | HuggingFace config fallback |
| **LM Studio** | Collection tags + model metadata | Hammer badge in UI | Built-in model templates |
| **GPT4All** | No auto-detection (manual templates) | Chat template configuration | JSON metadata files (models.json) |

---

## Detailed Analysis

### Ollama

**Vision Capability Detection:**
- Models categorized by capability in Ollama library/search under "vision" category
- No programmatic detection method exposed
- Vision models (Llama 3.2-Vision, LLaVA) identified through model library browsing
- Each model self-contained with its own projection layer aligned to training

**Tool Calling Detection:**
- No automatic detection available via API
- Current approach: send tools request and handle error response if unsupported
- GitHub issue #10693 requests API mechanism with "features" field on GET /api/tags endpoint
- Currently expected to iterate through models.json or Ollama search to identify supported models

**Detection Method:**
- GET `/api/tags` endpoint returns: name, format, family, families, parameter_size, quantization_level
- No capabilities/features field currently exposed
- Model library provides browsable categorization: "tools" and "vision" categories

**References:**
- [Ollama Tool Calling Docs](https://docs.ollama.com/capabilities/tool-calling)
- [Ollama API Tags Endpoint](https://docs.ollama.com/api/tags)
- [Tool Support Detection Issue](https://github.com/ollama/ollama/issues/10693)

---

### vLLM

**Vision Capability Detection:**
- GGUF metadata extraction via `config_from_gguf()` function
- Automatic projector type detection (gemma3, llama4, etc.)
- mmproj files required for multimodal support (separate quantized vision encoder + projector)
- Falls back to HuggingFace transformers for architecture validation

**Tool Calling Detection:**
- Architecture-specific support (model architecture must be explicitly supported)
- Incremental feature addition per architecture type
- No general capability probing available
- Support status determined at model load time based on architecture

**Detection Method:**
- `gguf_utils` module performs metadata inspection
- `config_from_gguf()` extracts attention, RoPE, quantization configs
- For unsupported models: manual `--hf-config-path` override required
- Single-file GGUF limitation (multi-file requires merging with gguf-split tool)

**Limitations:**
- Assumes HuggingFace can convert GGUF metadata to config
- Tokenizer conversion from GGUF is "time-consuming and unstable" (uses base model tokenizer instead)
- GGUF support remains experimental and may conflict with other features

**References:**
- [vLLM GGUF Documentation](https://docs.vllm.ai/en/stable/features/quantization/gguf/)
- [vLLM gguf_utils API](https://docs.vllm.ai/en/v0.12.0/api/vllm/transformers_utils/gguf_utils/)

---

### LM Studio

**Vision Capability Detection:**
- Model library organized by capability tags (vision models in dedicated collection)
- Models with vision capabilities grouped in curated "Vision Models (GGUF)" collection
- No programmatic detection exposed; relies on UI collections

**Tool Calling Detection:**
- Hammer badge displayed in UI for models "trained for tool use"
- Badge indicates native tool use support and improved performance in tool scenarios
- v0.3.6 introduced badge to indicate tool use training
- Visual indicator in model selection interface

**Detection Method:**
- Model catalog with capability metadata
- Badge system in application UI
- Tool use support indicated by visual markers
- Built-in model templates handle capability-specific formatting

**References:**
- [LM Studio Tool Use Docs](https://lmstudio.ai/docs/advanced/tool-use)
- [LM Studio v0.3.6 Release Notes](https://lmstudio.ai/blog/lmstudio-v0.3.6)

---

### GPT4All

**Vision Capability Detection:**
- No automatic vision detection
- Manual configuration approach through chat templates
- No dedicated vision support in current implementation
- Collection metadata does not expose multimodal capabilities

**Tool Calling Detection:**
- No explicit tool calling detection
- Indirectly supported through proper chat templates
- Tool support depends on model being designed for it and using correct template format

**Detection Method:**
- `models.json` metadata file with model descriptions and system prompts
- Chat template configuration via `tokenizer_config.json` from HuggingFace
- Chat templates declare expected format (e.g., `|start_header_id|`, `|end_header_id|`)
- Manual template creation required for sideloaded models
- Jinja2 templating support for flexible chat formatting

**Model Metadata Sources:**
1. Built-in models come with appropriate templates
2. HuggingFace `tokenizer_config.json` for repository models
3. Manual creation for custom/sideloaded models

**References:**
- [GPT4All Chat Templates Docs](https://docs.gpt4all.io/gpt4all_desktop/chat_templates.html)
- [GPT4All Custom Models Configuration](https://github.com/nomic-ai/gpt4all/wiki/Configuring-Custom-Models)

---

## Key Patterns & Findings

### Capability Detection Strategies

1. **Library/Collection Categorization** (Ollama, LM Studio)
   - Models organized by capability tags
   - Manual curation of capability-specific collections
   - No programmatic auto-detection

2. **Metadata Introspection** (vLLM, GPT4All)
   - Parse model configuration files (config.json, tokenizer_config.json)
   - Extract capability hints from architecture and training metadata
   - Fallback to HuggingFace for standard formats

3. **Try-and-Fail Approach** (Ollama for tools)
   - Send capability request and handle response/error
   - Used when no metadata explicitly declares support

4. **Visual Indicators** (LM Studio)
   - Badge system in UI to mark capability-trained models
   - Tool use support indicated by hammer badge

### Multimodal/Vision Handling

- **GGUF Standard**: Uses separate mmproj files for vision encoders + projectors
- **Metadata**: Architecture field in GGUF metadata identifies multimodal type
- **Collections**: Dedicated HuggingFace collections for multimodal GGUF models
- **Manual Override**: Config path specification when auto-detection fails

### Common Limitations

- No universal capability detection method across tools
- Tool calling support requires explicit model training and format support
- Vision support depends on available projector files (mmproj)
- Chat templates must match training format exactly for optimal performance
- Most tools lack programmatic capability discovery APIs

### Standards & Conventions

- **GGUF Metadata**: Standard container with architecture, quantization, tokenizer info
- **HuggingFace Integration**: Default source for config conversion and metadata
- **Chat Templates**: Jinja2 format increasingly standardized (GPT4All, some others)
- **Model Metadata**: JSON files (config.json, tokenizer_config.json, models.json)

---

## Recommendations for BodhiApp Implementation

1. **Primary Detection Method**: Combine metadata parsing (config.json) with GGUF introspection
2. **Fallback Pattern**: Try-and-fail for uncertain capabilities, cache results
3. **Vision Support**: Check for mmproj files and GGUF multimodal architecture indicators
4. **Tool Support**: Examine model metadata for tool training markers and chat template format
5. **API Exposure**: Consider exposing capability metadata similar to Ollama's proposed "features" field

