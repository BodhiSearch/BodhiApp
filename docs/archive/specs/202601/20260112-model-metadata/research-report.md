# GGUF Model Metadata & Capability Detection Research Report

## Executive Summary

This comprehensive research synthesizes findings from four research phases investigating model capability detection for BodhiApp's model metadata system. The research addresses three key sources of model capability information: GGUF format metadata, HuggingFace Hub API, chat template patterns, and comparison with industry implementations (Ollama, vLLM, LM Studio, GPT4All).

**Key Findings:**
- GGUF metadata provides direct capability indicators (119+ architectures, multimodal encoders, projector types)
- HuggingFace API offers standardized pipeline tags and model card metadata for fallback detection
- Chat templates contain pattern-matchable capability indicators (tool calling, reasoning, structured output)
- Industry tools use complementary strategies: metadata introspection, library categorization, and try-and-fail approaches
- A layered detection strategy combining GGUF metadata (primary) with HF API (fallback) enables robust capability detection

---

## 1. GGUF Metadata Specification

### Overview

GGUF (GGML Universal Format) is the standard container format for local LLM inference. Based on llama.cpp implementation, GGUF metadata contains comprehensive capability indicators through architecture identifiers, multimodal metadata keys, and context length specifications.

**Reference Sources:**
- [llama.cpp constants.py](https://github.com/ggml-org/llama.cpp/blob/master/gguf-py/gguf/constants.py)
- [llama.cpp convert_hf_to_gguf.py](https://github.com/ggml-org/llama.cpp/blob/master/convert_hf_to_gguf.py)
- [Multimodal GGUFs Collection](https://huggingface.co/collections/ggml-org/multimodal-ggufs-68244e01ff1f39e5bebeeedc)

### MODEL_ARCH Enum - 119+ Architectures

Complete architecture list from llama.cpp with major categories:

**Language Models (General Purpose):**
- `llama`, `llama4` - LLaMA family
- `qwen`, `qwen2`, `qwen3` - Qwen series
- `falcon`, `falcon-h1` - Falcon family
- `mistral3` - Mistral 3
- `phi2`, `phi3`, `phimoe` - Phi series
- `gemma`, `gemma2`, `gemma3` - Gemma series
- `deepseek`, `deepseek2` - DeepSeek series

**Vision-Language Models:**
- `qwen2vl`, `qwen3vl`, `qwen3vlmoe` - Qwen vision-language models
- `chameleon` - Chameleon multimodal
- `cogvlm` - CogVLM (vision-language)
- `minicpm`, `minicpm3` - MiniCPM with vision

**Specialized Models:**
- `llama-embed`, `gemma-embedding`, `pangu-embedded` - Embedding models
- `rwkv6`, `rwkv7`, `mamba`, `mamba2`, `jamba` - State-space models
- `t5`, `t5encoder` - T5 and encoder variants
- `bert`, `modern-bert`, `nomic-bert` - BERT family
- `starcoder`, `starcoder2`, `codeshell` - Code models

**Audio/Multimodal:**
- `lfm2`, `lfm2moe` - Audio models
- `clip` - Vision/audio encoder dummy architecture

**Total: 119+ supported architectures as of January 2026**

### CLIP/Multimodal Metadata Keys

#### General CLIP Architecture Keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.projector_type` | string | Type of projector (mlp, ldp, ldpv2, resampler, glm_edge, merger, gemma3n, gemma3, qwen3vl, cogvlm, pixtral) |
| `clip.has_vision_encoder` | bool | Model includes vision encoder |
| `clip.has_audio_encoder` | bool | Model includes audio encoder |
| `clip.has_llava_projector` | bool | Model uses LLaVA-style projector |

#### Vision Encoder Metadata Keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.vision.image_size` | u32 | Input image size (e.g., 448) |
| `clip.vision.patch_size` | u32 | Vision patch size (e.g., 14) |
| `clip.vision.embedding_length` | u32 | Vision embedding dimension |
| `clip.vision.projection_dim` | u32 | Projection output dimension |
| `clip.vision.block_count` | u32 | Number of vision transformer blocks |
| `clip.vision.image_mean` | arr[f32,3] | Image normalization mean values |
| `clip.vision.image_std` | arr[f32,3] | Image normalization std values |
| `clip.vision.attention.head_count` | u32 | Number of attention heads |
| `clip.vision.projector.scale_factor` | u32 | Projector scaling factor |

#### Audio Encoder Metadata Keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.audio.num_mel_bins` | u32 | Number of mel spectrogram bins |
| `clip.audio.embedding_length` | u32 | Audio embedding dimension |
| `clip.audio.projection_dim` | u32 | Projection output dimension |
| `clip.audio.block_count` | u32 | Number of audio transformer blocks |
| `clip.audio.attention.head_count` | u32 | Number of attention heads |

#### Vision Projector Types

| Enum | Description |
|------|-------------|
| MLP | Multi-layer perceptron projector (LLaVA-style) |
| LDP | Linear-Depth Pooling |
| LDPV2 | Linear-Depth Pooling v2 |
| RESAMPLER | Perceiver Resampler (MiniCPM-V) |
| GLM_EDGE | GLM Edge projector |
| MERGER | Merging projector |
| GEMMA3N | Gemma 3N projector |
| QWEN3VL | Qwen3-VL projector |
| COGVLM | CogVLM projector |
| PIXTRAL | Pixtral projector |

### Architecture-Specific Context Length Keys

Pattern: `{arch}.context_length` where `{arch}` is the architecture identifier.

**Sample Mappings:**
| Architecture | Context Key | Example Value |
|--------------|------------|---------------|
| llama | `llama.context_length` | 4096 |
| qwen2 | `qwen2.context_length` | 32768 |
| qwen2vl | `qwen2vl.context_length` | 131072 |
| deepseek2 | `deepseek2.context_length` | 163840 |
| command-r | `command-r.context_length` | 131072 |
| glm4 | `glm4.context_length` | 131072 |

**Note:** Stateful models (RWKV, Mamba) don't use traditional context windows.

### Real-World Multimodal GGUF Examples

#### Example 1: MiniCPM-V-4 GGUF (Vision Model)

Language model metadata:
```
general.architecture = minicpm
minicpm.context_length = 32768
```

Vision encoder metadata (mmproj file):
```
general.architecture = clip
clip.has_vision_encoder = true
clip.has_minicpmv_projector = true
clip.minicpmv_version = 5
clip.projector_type = resampler
clip.vision.image_size = 448
clip.vision.patch_size = 14
```

#### Example 2: Qwen2.5-Omni-7B GGUF (Any-to-Any Multimodal)

**Capabilities:**
- Text input ✅
- Audio input ✅
- Image input ✅
- Audio generation ❌

**Metadata:**
```
general.architecture = qwen2vl
qwen2vl.context_length = 131072
```

Compatible with llama-server and llama-mtmd-cli for multimodal inference.

#### Example 3: Pixtral-12B GGUF (Vision Model)

**Metadata:**
```
general.architecture = llama
llama.context_length = 4096
clip.projector_type = pixtral
```

**Available Quantizations:** Q2_K (4.79 GB), Q4_K_M (7.48 GB), Q8_0 (13 GB), F16 (24.5 GB)

### Multimodal Model Collections

#### ggml-org Multimodal GGUFs Collection

**Vision Models:** Mistral-Small-3.1-24B-Instruct, moondream2, pixtral-12b

**Audio Models:** ultravox-v0_5 (1B and 8B variants)

**Vision + Audio Models:** Qwen2.5-Omni-3B/7B, Voxtral-Mini-3B, LFM2-Audio-1.5B

All models compatible with **llama-server** and **llama-mtmd-cli**.

### Conversion Patterns from HuggingFace to GGUF

**Vision Encoder Handling:**
```python
# Extract config
def get_vision_config(self) -> dict[str, Any] | None:
    config_name = "vision_config" if not self.is_mistral_format else "vision_encoder"
    return self.global_config.get(config_name)

# Set metadata
if self.has_vision_encoder:
    self.gguf_writer.add_clip_has_vision_encoder(True)
    self.gguf_writer.add_vision_projection_dim(self.n_embd_text)
    self.gguf_writer.add_vision_image_mean(image_mean)
    self.gguf_writer.add_vision_image_std(image_std)
```

**Multimodal Tensor Detection:**
```python
vision_prefixes = [
    "vision_encoder.",
    "vision_language_adapter.",
    "patch_merger.",
    "audio_encoder.",
]

is_multimodal_tensor = "vision_tower" in name \
    or "vision_model" in name \
    or "audio_tower" in name \
    or any(name.startswith(prefix) for prefix in vision_prefixes)
```

### Capability Detection Strategy (GGUF-First)

**For Vision Capabilities:**
1. Architecture-specific vision models: `qwen2vl`, `qwen3vl`, `qwen3vlmoe`, `chameleon`, `cogvlm`
2. MMPROJ architecture: `general.architecture == "clip"` with `clip.has_vision_encoder == true`
3. Projector type presence: Non-null `clip.projector_type` or `clip.vision.projector_type`
4. Vision-specific metadata: Presence of `clip.vision.image_size` or `clip.vision.patch_size`

**For Audio Capabilities:**
1. MMPROJ architecture: `general.architecture == "clip"` with `clip.has_audio_encoder == true`
2. Audio-specific metadata: Presence of `clip.audio.num_mel_bins` or `clip.audio.embedding_length`
3. Architecture hints: `lfm2`, `lfm2moe` architectures

**For Mixed Modality (Vision + Audio):**
1. Both flags present: `clip.has_vision_encoder == true` AND `clip.has_audio_encoder == true`
2. Qwen Omni models: `qwen2vl` architecture with audio metadata
3. Separate projector types: Both `clip.vision.projector_type` and `clip.audio.projector_type` present

### Key Implementation Insights

1. **Architecture determines context key:** Always use `{arch}.context_length` pattern
2. **MMPROJ is dummy arch:** `clip` architecture is special - used for vision/audio encoders only
3. **Separate files common:** Many models split language model and vision encoder into separate GGUF files
4. **Projector types matter:** Different projector architectures affect inference performance
5. **Boolean flags are critical:** `has_vision_encoder`, `has_audio_encoder`, `has_llava_projector` determine capabilities
6. **Image preprocessing varies:** `image_mean` and `image_std` are model-specific and required for correct inference

---

## 2. HuggingFace API Reference

### Overview

HuggingFace Hub API provides standardized model metadata through REST endpoints and configuration files. The API is authoritative for capability detection when GGUF metadata is incomplete or unavailable.

**Reference Sources:**
- [HuggingFace Tasks Documentation](https://huggingface.co/docs/hub/en/models-tasks)
- [HfApi Client Documentation](https://huggingface.co/docs/huggingface_hub/package_reference/hf_api)
- [Model Cards Documentation](https://huggingface.co/docs/hub/en/model-cards)

### API Endpoint: GET /api/models/{namespace}/{repo}

**Base URL:** `https://huggingface.co/api/models/{namespace}/{repo}`

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `expand` | list[str] | Properties to include in response (author, cardData, downloads, tags, etc.) |
| `cardData` | bool | Include model card metadata |
| `full` | bool | Return all available properties |

**Core Response Fields:**
```json
{
  "id": "string (full repo ID)",
  "pipeline_tag": "string (task type)",
  "library_name": "string",
  "tags": ["string"],
  "downloads": "number",
  "likes": "number",
  "cardData": {
    "language": ["string"],
    "license": "string",
    "datasets": ["string"],
    "tags": ["string"],
    "pipeline_tag": "string"
  },
  "config": {
    "architectures": ["string"],
    "model_type": "string",
    "chat_template": "string | object"
  }
}
```

### Pipeline Tag Taxonomy (57 Total)

**Capability-Relevant Pipeline Tags:**

| Capability | Pipeline Tags |
|-----------|---------------|
| **Vision/Multimodal** | `image-text-to-text`, `image-to-text`, `visual-question-answering`, `document-question-answering`, `video-text-to-text`, `any-to-any` |
| **Audio Input** | `automatic-speech-recognition`, `audio-classification`, `audio-text-to-text` |
| **Audio Output** | `text-to-speech`, `text-to-audio` |
| **Text Generation** | `text-generation`, `conversational` |

### Model Card Tags and Metadata

**YAML Frontmatter Structure (README.md):**
```yaml
---
pipeline_tag: image-text-to-text
library_name: transformers
tags:
  - vision
  - multimodal
  - function-calling
  - tool-use
  - reasoning
  - chain-of-thought
base_model: meta-llama/Llama-3.1-8B
language:
  - en
datasets:
  - dataset-namespace/dataset-name
license: mit
---
```

**Common Capability Tags:**

- **Vision/Multimodal:** `vision`, `multimodal`, `image-to-text`, `image-text-to-text`, `visual-question-answering`, `llava`, `clip`
- **Tool/Function Calling:** `function calling`, `function-calling`, `tool-use`, `tool calling`, `json mode`, `chatml`
- **Reasoning:** `reasoning`, `chain-of-thought`, `cot`, `thinking`, `o1-style`, `r1-style`
- **Other:** `instruct`, `chat`, `conversational`, `finetune`, `rlhf`, `dpo`

### Chat Template Indicators

Models with tool/function calling often include special chat templates. Check the `config` object for:

```json
{
  "chat_template": {
    "name": "tool_use",
    "template": "...function calling AI model..."
  }
}
```

Or look for chat template strings containing:
- `<tool_call>` / `</tool_call>`
- `<function>` / `</function>`
- `tools` parameter in template
- `<think>` / `</think>` (reasoning models)

### API Response Examples

**Example 1: LLaVA (Multimodal Model)**

```json
{
  "id": "llava-hf/llava-1.5-7b-hf",
  "pipeline_tag": "image-to-text",
  "tags": ["vision", "image-text-to-text", "conversational"],
  "config": {
    "architectures": ["LlavaForConditionalGeneration"],
    "model_type": "llava"
  }
}
```

**Detected Capabilities:**
- Vision: ✅ (pipeline_tag, architecture, tags)

**Example 2: Hermes-3 (Function Calling Model)**

```json
{
  "id": "NousResearch/Hermes-3-Llama-3.1-8B",
  "pipeline_tag": "text-generation",
  "tags": ["function calling", "json mode", "chatml"],
  "config": {
    "chat_template": {
      "name": "tool_use",
      "template": "...function calling AI model...<tool_call>...</tool_call>"
    }
  }
}
```

**Detected Capabilities:**
- Tool Calling: ✅ (tags, chat_template)

**Example 3: DeepSeek-R1 (Reasoning Model)**

```json
{
  "id": "deepseek-ai/DeepSeek-R1",
  "pipeline_tag": "text-generation",
  "config": {
    "chat_template": "...{% if add_generation_prompt and not ns.is_tool %}{{'<｜Assistant｜><think>\\n'}}{% endif %}..."
  }
}
```

**Detected Capabilities:**
- Reasoning: ✅ (chat_template includes `<think>` tag)

### HuggingFace Cache Structure

**Default Cache Location:**
- Linux/Mac: `~/.cache/huggingface/hub`
- Windows: `%USERPROFILE%\.cache\huggingface\hub`

**Cache Directory Structure:**
```
~/.cache/huggingface/hub/
├── models--{namespace}--{repo}/
│   ├── blobs/
│   │   ├── {sha256-hash-1}
│   │   └── {sha256-hash-2}
│   ├── refs/
│   │   └── main
│   └── snapshots/
│       ├── {commit-hash-1}/
│       │   ├── config.json
│       │   └── model.safetensors
│       └── {commit-hash-2}/
│           └── ...
```

**Repository Naming Convention:**
`{type}--{namespace}--{repo}` → `models--meta-llama--Llama-3.1-8B` → `meta-llama/Llama-3.1-8B`

### Capability Detection Strategy (HF API)

**Detection Priority (Highest to Lowest):**
1. **Pipeline Tag** - Most reliable, standardized
2. **Model Architecture** - From config.json `architectures` field
3. **Tags Array** - User-provided, may be incomplete
4. **Chat Template** - Inspect for tool/reasoning patterns
5. **Model Name** - Pattern matching
6. **Base Model Metadata** - Inherit from base model

**Vision/Multimodal Detection:**
1. Pipeline tag: `image-text-to-text`, `image-to-text`, `visual-question-answering`, `video-text-to-text`, `any-to-any`
2. Architecture: `Llava*`, `*VisionEncoder*`, `Qwen2VL*`, `*Vision2Seq`
3. Tags: `vision`, `multimodal`, `image-text-to-text`, `llava`, `clip`

**Audio Detection:**
1. Pipeline tag: `automatic-speech-recognition`, `audio-classification`, `audio-text-to-text`, `audio-to-audio`
2. Tags: `audio`, `speech`, `whisper`

**Tool/Function Calling Detection:**
1. Tags: `function calling`, `function-calling`, `tool-use`, `tool calling`, `json mode`, `chatml`
2. Chat Template: Contains `tool_use`, `<tool_call>`, `<function>`, `tools` parameter
3. Model Name: Contains "Hermes", "tool", "function"

**Reasoning Detection:**
1. Model Name: Contains "R1", "DeepSeek-R", "QwQ", "reasoning"
2. Chat Template: Contains `<think>`, `<reasoning>`, "reasoning mode"
3. Tags: `reasoning`, `chain-of-thought`, `cot`, `thinking`

---

## 3. Chat Template Analysis

### Overview

Chat templates are Jinja2 (HuggingFace) or Go templates (Ollama) that format model inputs. Beyond standard conversation formatting, templates contain detectable patterns for tool calling, reasoning, and structured output capabilities.

### Tool Calling Patterns

#### Jinja Template Variables

**Primary Detection Pattern:**
- `{{ tools }}` - Variable reference to tools array
- `{% if tools %}` - Conditional check for tools existence
- `{% for tool in tools %}` - Iteration over tools
- `'tool_calls' in message` - Tool call response handling
- `message['role'] == 'tool'` - Tool response role check

**Standard API Format:**
```json
{
  "type": "function",
  "function": {
    "name": "multiply",
    "description": "A function that multiplies two numbers",
    "parameters": {
      "type": "object",
      "properties": {
        "a": {"type": "number"},
        "b": {"type": "number"}
      },
      "required": ["a", "b"]
    }
  }
}
```

#### Model-Specific Tool Patterns

**Llama 3.1 Built-in Tools:**
- `<|python_tag|>` - Indicates built-in tool execution
- `builtin_tools` variable reference
- Tool names: `wolfram_alpha`, `brave_search`, `code_interpreter`

**Hermes-3 Tool Patterns:**
```
<|im_start|>system
You are a function calling AI model. You are provided with function signatures within <tools></tools> XML tags...
<tools>{function_definitions}</tools>
...<|im_end|>
```

Special Tokens:
- Token 128002: `<tool_call>`
- Token 128003: `<tool_response>`

**Command-R Tool Patterns:**
```json
[
    {
        "tool_name": "internet_search",
        "parameters": {"query": "..."}
    }
]
```

**Mistral/Pixtral Tool Patterns:**
```
[AVAILABLE_TOOLS] [{...}] [/AVAILABLE_TOOLS]
[TOOL_CALLS] [{...}]
[TOOL_RESULTS] [...] [/TOOL_RESULTS]
```

### Thinking/Reasoning Patterns

#### DeepSeek-R1 Patterns

**Primary Pattern:**
```
<think>
{reasoning_process}
</think>
{final_answer}
```

**Key Observations:**
- Models may bypass thinking pattern for certain queries
- Reasoning returned in separate `reasoning_content` field via API
- Temperature: 0.5-0.7 recommended
- Force models to start with `<think>\n` for thorough reasoning

#### QwQ Patterns

**Chat Template Integration:**
```jinja
{%- if add_generation_prompt %}
    {{- '<|im_start|>assistant\n<think>\n' }}
{%- endif %}
```

**Key Characteristics:**
- Automatically adds `<think>` block when `add_generation_prompt=True`
- Model does NOT output the first `<think>` token (normal behavior)
- Thinking excluded from chat history in multi-turn conversations
- Temperature: 0.6, TopP: 0.95

#### OpenAI o1-style Patterns

- Models use internal "reasoning tokens" or "thinking tokens"
- Not visible via API but occupy context window space
- Billed as output tokens
- Developer message role: `{"role": "developer", "content": "..."}`

### Structured Output Patterns

**JSON Mode Indicators:**
- `response_format` - API parameter
- `"type": "json_object"` - JSON mode indicator
- `json_schema` - Schema specification
- `<schema>` / `</schema>` - Schema wrapper

**System Prompt Patterns:**
- "You are a helpful assistant that responds in JSON format"
- "Respond only with valid JSON matching the following schema"

**Schema Constraints:**
- `pattern` - Regex constraint
- `minLength`, `maxLength` - String constraints
- `format` - Format specification (email, date-time)
- `strict: true` - Strict schema enforcement

**Command-R Grounded Generation:**
```
The <co: 0>Emperor Penguin</co: 0> is the <co: 0>tallest</co: 0> penguin.
```

### Ollama Template Patterns

**Key Difference:** Ollama uses Go templates instead of Jinja2.

**Tool Support Pattern:**
```go
{{ if .Tools }}
    Available tools:
    {{ range .Tools }}
        - {{ .Function.Name }}: {{ .Function.Description }}
    {{ end }}
{{ end }}
```

**Context Variables:**
- `$.Tools` - Dollar sign notation
- `.Tools` - Current context tools
- `.ToolCalls` - Tool call information

**Automatic Detection:** If a template uses `.Tools`, the model is marked as supporting tool calling.

### Safe Regex Detection Patterns

**Jinja Tool Patterns (Rust):**
```rust
r"\{\{\s*tools\s*\}\}"                    // {{ tools }}
r"\{%\s*if\s+tools\s"                     // {% if tools %}
r"\{%\s*for\s+tool\s+in\s+tools"          // {% for tool in tools %}
r"tool_calls"                             // General keyword
r"message\[['\"]\s*role\s*['\"]]\s*==\s*['\"]tool['\"]"  // role check
```

**Ollama Go Template Patterns:**
```rust
r"\{\{\s*\.Tools\s*\}\}"                  // {{ .Tools }}
r"\{\{\s*if\s+\.Tools\s*\}\}"             // {{ if .Tools }}
r"\{\{\s*range\s+\.Tools\s*\}\}"          // {{ range .Tools }}
```

**Reasoning Detection:**
```rust
r"<think>"                                // Opening tag
r"</think>"                               // Closing tag
r"reasoning_content"                      // Response field
r"think\s+step\s+by\s+step"               // Instruction
```

**Structured Output Detection:**
```rust
r"response_format"                        // API parameter
r"json_object"                            // Type value
r"json_schema"                            // Schema specification
r"<schema>"                               // Schema wrapper
r"\"pattern\"\s*:"                        // Regex constraint
```

### Security Considerations

**Important:** From [JFrog GGUF-SSTI research](https://research.jfrog.com/model-threats/gguf-ssti/):

> "Malicious Jinja2 templates can be inserted into a GGUF model's chat_template metadata parameter and execute shell commands if loaded in an unsandboxed environment."

**Safe Analysis Approach:**
1. NEVER execute or render the template
2. Use regex pattern matching only
3. Extract template as string from GGUF metadata
4. Scan for capability indicators
5. Flag suspicious patterns separately

**Dangerous Patterns to Detect:**
```rust
r"__class__"                              // Python introspection
r"\bos\b|\bsubprocess\b"                  // Shell execution
r"\beval\b|\bexec\b"                      // Code evaluation
r"__import__"                             // Dynamic imports
r"\.popen\b"                              // Process execution
```

### Real Template Examples

**Hermes-3 Default Template (abbreviated):**
```jinja
{{bos_token}}{% for message in messages %}...{{'<|im_start|>' + message['role'] + '\n' + message['content'] + '<|im_end|>' + '\n'}}{% endfor %}{% if add_generation_prompt %}{{ '<|im_start|>assistant\n' }}{% endif %}
```

**Hermes-3 Tool Use Template:**
- Defines `json_to_python_type()` macro for type conversion
- Handles tool definitions in `<tools></tools>` blocks
- Wraps tool calls in `<tool_call>` XML tags
- Returns tool responses in `<tool_response>` tags

**QwQ Reasoning Template:**
```jinja
{%- if add_generation_prompt %}
    {{- '<|im_start|>assistant\n<think>\n' }}
{%- endif %}
```

---

## 4. Industry Comparison

### Overview

This section synthesizes capability detection approaches from four leading local LLM tools: Ollama, vLLM, LM Studio, and GPT4All.

### Ollama

**Vision Capability Detection:**
- Models categorized by capability in Ollama library/search under "vision" category
- No programmatic detection method exposed
- Library browsing identifies vision models (Llama 3.2-Vision, LLaVA)
- Each model self-contained with its own projection layer

**Tool Calling Detection:**
- No automatic detection available via API
- Current approach: send tools request and handle error response if unsupported
- GitHub issue #10693 requests API mechanism with "features" field
- Expected to iterate through models.json or Ollama search

**Detection Method:**
- GET `/api/tags` endpoint returns: name, format, family, families, parameter_size, quantization_level
- No capabilities/features field currently exposed
- Model library provides browsable categorization: "tools" and "vision" categories

**References:**
- [Ollama Tool Calling Docs](https://docs.ollama.com/capabilities/tool-calling)
- [Tool Support Detection Issue](https://github.com/ollama/ollama/issues/10693)

### vLLM

**Vision Capability Detection:**
- GGUF metadata extraction via `config_from_gguf()` function
- Automatic projector type detection (gemma3, llama4, etc.)
- mmproj files required for multimodal support
- Falls back to HuggingFace transformers for architecture validation

**Tool Calling Detection:**
- Architecture-specific support (must be explicitly supported)
- Incremental feature addition per architecture type
- No general capability probing available
- Support status determined at model load time

**Detection Method:**
- `gguf_utils` module performs metadata inspection
- `config_from_gguf()` extracts attention, RoPE, quantization configs
- For unsupported models: manual `--hf-config-path` override required
- Single-file GGUF limitation (multi-file requires gguf-split tool)

**Limitations:**
- Assumes HuggingFace can convert GGUF metadata to config
- Tokenizer conversion from GGUF is "time-consuming and unstable"
- GGUF support remains experimental

**References:**
- [vLLM GGUF Documentation](https://docs.vllm.ai/en/stable/features/quantization/gguf/)

### LM Studio

**Vision Capability Detection:**
- Model library organized by capability tags
- Models with vision capabilities grouped in dedicated "Vision Models (GGUF)" collection
- No programmatic detection exposed; relies on UI collections

**Tool Calling Detection:**
- Hammer badge displayed in UI for models "trained for tool use"
- Badge indicates native tool use support and improved performance
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

### GPT4All

**Vision Capability Detection:**
- No automatic vision detection
- Manual configuration approach through chat templates
- No dedicated vision support in current implementation

**Tool Calling Detection:**
- No explicit tool calling detection
- Indirectly supported through proper chat templates
- Tool support depends on model design and template format

**Detection Method:**
- `models.json` metadata file with model descriptions
- Chat template configuration via `tokenizer_config.json` from HuggingFace
- Jinja2 templating support for flexible chat formatting
- Manual template creation required for sideloaded models

**Model Metadata Sources:**
1. Built-in models come with appropriate templates
2. HuggingFace `tokenizer_config.json` for repository models
3. Manual creation for custom/sideloaded models

**References:**
- [GPT4All Chat Templates Docs](https://docs.gpt4all.io/gpt4all_desktop/chat_templates.html)

### Comparative Analysis

| Strategy | Tools | Strengths | Weaknesses |
|----------|-------|-----------|-----------|
| **Library Categorization** | Ollama, LM Studio | Manual curation, high accuracy | No programmatic detection, labor-intensive |
| **Metadata Introspection** | vLLM, GPT4All | Automatic, scalable | Requires config parsing, incomplete data |
| **Try-and-Fail** | Ollama (tools) | Definitive detection | Slow, affects UX |
| **Visual Indicators** | LM Studio | User-friendly, discoverable | Limited to UI, not API-accessible |

### Key Patterns & Findings

**Capability Detection Strategies:**
1. Library/Collection Categorization (Ollama, LM Studio) - Manual curation, high accuracy
2. Metadata Introspection (vLLM, GPT4All) - Automatic, scalable
3. Try-and-Fail Approach (Ollama for tools) - Definitive but slow
4. Visual Indicators (LM Studio) - User-friendly but limited

**Multimodal/Vision Handling:**
- GGUF Standard: Uses separate mmproj files for vision encoders + projectors
- Metadata: Architecture field in GGUF metadata identifies multimodal type
- Collections: Dedicated HuggingFace collections for multimodal GGUF models
- Manual Override: Config path specification when auto-detection fails

**Common Limitations:**
- No universal capability detection method across tools
- Tool calling support requires explicit model training
- Vision support depends on available projector files
- Chat templates must match training format exactly
- Most tools lack programmatic capability discovery APIs

---

## 5. Recommended Implementation Strategy

### 5.1 Detection Priority Algorithm

**Layered Detection with Fallback Chain:**

```
1. GGUF Metadata (Highest Priority)
   ├─ Vision: Check {arch}.context_length + clip.has_vision_encoder
   ├─ Audio: Check clip.has_audio_encoder + audio metadata keys
   ├─ Tool Calling: Inspect chat_template string (regex patterns)
   └─ Reasoning: Inspect chat_template string (regex patterns)
         ↓ (If incomplete/missing)
2. HuggingFace API (Secondary)
   ├─ Query HF API for repo metadata
   ├─ Extract pipeline_tag, tags, chat_template
   └─ Apply HF detection rules
         ↓ (If still uncertain)
3. Chat Template Pattern Matching
   └─ Parse extracted chat_template for capability markers
         ↓ (If still unresolved)
4. Manual Override/User Input
   └─ Allow user to specify capabilities explicitly
```

### 5.2 GGUF-first vs HF-first Strategy

**Recommendation: GGUF-First with HF Fallback**

**Rationale:**
- GGUF metadata is embedded in local files (offline capability)
- HuggingFace API requires network access (slower, potential failures)
- Chat templates in GGUF are authoritative (model-specific)
- HF API provides standardized fallback for conversions/base models

**Implementation Flow:**

```
For each model:
  1. Extract GGUF metadata
  2. Read chat_template as string (don't execute)
  3. Apply GGUF capability detection (architectures, multimodal keys)
  4. If GGUF detection incomplete:
     a. Parse repo ID from GGUF or user input
     b. Query HF API with repo ID
     c. Extract pipeline_tag, tags, architectures
     d. Apply HF detection rules
  5. Cache detection results in database
  6. Provide user UI for capability overrides
```

### 5.3 Implementation Checklist

**Phase 1: GGUF Metadata Extraction**

- [ ] Read GGUF model file and extract metadata dictionary
- [ ] Implement architecture identifier parsing (`general.architecture`)
- [ ] Extract context length using pattern `{arch}.context_length`
- [ ] Detect multimodal indicators:
  - [ ] Check `clip.has_vision_encoder` (bool)
  - [ ] Check `clip.has_audio_encoder` (bool)
  - [ ] Extract `clip.projector_type` (string)
  - [ ] Collect vision-specific keys: `clip.vision.*`
  - [ ] Collect audio-specific keys: `clip.audio.*`

**Phase 2: Chat Template Pattern Detection**

- [ ] Extract `chat_template` from GGUF metadata as raw string
- [ ] Implement regex-based detection patterns (NO Jinja execution):
  - [ ] Tool calling: `{{ tools }}`, `{% if tools %}`, `tool_calls`, `<tool_call>`
  - [ ] Reasoning: `<think>`, `</think>`, `reasoning_content`
  - [ ] Structured output: `response_format`, `json_schema`, `<schema>`
  - [ ] Model-specific: Hermes (`<tool_call>`), Llama (`builtin_tools`), Command-R (`tool_name`)
- [ ] Implement multi-pattern confidence scoring
- [ ] Flag dangerous patterns separately (security audit)

**Phase 3: HuggingFace API Integration**

- [ ] Parse repo ID from model path or metadata
- [ ] Implement HF API client:
  - [ ] GET `/api/models/{repo_id}?expand=cardData`
  - [ ] Cache responses locally (avoid rate limits)
  - [ ] Handle errors gracefully (offline fallback)
- [ ] Extract capability-relevant fields:
  - [ ] `pipeline_tag` (primary indicator)
  - [ ] `tags` array (secondary indicator)
  - [ ] `architectures` from config
  - [ ] `chat_template` from config (for pattern analysis)

**Phase 4: Capability Detection Logic**

- [ ] Implement vision detection:
  - [ ] GGUF: Architecture + multimodal keys
  - [ ] HF: pipeline_tag + tags + architecture patterns
  - [ ] Combine results with OR logic
- [ ] Implement audio detection:
  - [ ] GGUF: Audio encoder flags + metadata keys
  - [ ] HF: Audio-related pipeline tags
- [ ] Implement tool calling detection:
  - [ ] Chat template regex (primary)
  - [ ] HF tags (secondary)
  - [ ] Confidence scoring
- [ ] Implement reasoning detection:
  - [ ] Chat template regex (primary)
  - [ ] Model name patterns (secondary)
  - [ ] HF tags (tertiary)

**Phase 5: Database Schema & Storage**

- [ ] Create `model_capabilities` table:
  ```sql
  CREATE TABLE model_capabilities (
    id UUID PRIMARY KEY,
    model_id UUID FOREIGN KEY,
    has_vision BOOLEAN,
    has_audio_input BOOLEAN,
    has_audio_output BOOLEAN,
    has_tool_calling BOOLEAN,
    has_reasoning BOOLEAN,
    tool_calling_confidence FLOAT,  -- 0.0-1.0
    reasoning_confidence FLOAT,      -- 0.0-1.0
    detected_via VARCHAR,            -- 'gguf', 'hf_api', 'template', 'manual'
    detected_at TIMESTAMP,
    updated_at TIMESTAMP
  );
  ```
- [ ] Implement detection result caching
- [ ] Add user override mechanism

**Phase 6: API Exposure**

- [ ] Expose capabilities in model info endpoint:
  ```json
  {
    "id": "model-id",
    "name": "Model Name",
    "capabilities": {
      "vision": true,
      "audio_input": false,
      "tool_calling": true,
      "reasoning": false
    },
    "capability_sources": {
      "vision": "gguf_metadata",
      "tool_calling": "chat_template"
    }
  }
  ```
- [ ] Allow capability overrides via admin API
- [ ] Provide capability detection transparency (sources)

**Phase 7: Testing & Validation**

- [ ] Test GGUF parsing:
  - [ ] Vision model (Pixtral, Qwen2VL)
  - [ ] Audio model (LFM2)
  - [ ] Mixed modality (Qwen2.5-Omni)
  - [ ] Text-only model (Llama)
- [ ] Test HF API fallback:
  - [ ] Known vision models (LLaVA)
  - [ ] Function calling models (Hermes)
  - [ ] Reasoning models (DeepSeek-R1)
  - [ ] Text-only models
- [ ] Test template pattern matching:
  - [ ] False positive mitigation
  - [ ] Edge case handling
  - [ ] Security pattern detection
- [ ] Integration testing:
  - [ ] End-to-end capability detection
  - [ ] Offline vs online modes
  - [ ] Caching behavior

### 5.4 Rust Implementation Skeleton

**Capability Detection Module:**

```rust
pub struct ModelCapabilities {
    pub vision: bool,
    pub audio_input: bool,
    pub audio_output: bool,
    pub tool_calling: bool,
    pub reasoning: bool,
    pub tool_calling_confidence: f32,
    pub reasoning_confidence: f32,
}

impl ModelCapabilities {
    pub fn from_gguf(metadata: &GgufMetadata) -> Self {
        let vision = Self::detect_vision_gguf(metadata);
        let audio_input = Self::detect_audio_input_gguf(metadata);
        let audio_output = Self::detect_audio_output_gguf(metadata);

        let chat_template = metadata.get_string("tokenizer.chat_template");
        let tool_calling = Self::detect_tool_calling_template(chat_template);
        let (reasoning, confidence) = Self::detect_reasoning_template(chat_template);

        Self {
            vision,
            audio_input,
            audio_output,
            tool_calling,
            reasoning,
            tool_calling_confidence: 0.8,
            reasoning_confidence: confidence,
        }
    }

    pub fn from_hf_api(model_info: &HfModelInfo) -> Self {
        let vision = Self::detect_vision_hf(&model_info.pipeline_tag, &model_info.tags);
        let audio_input = Self::detect_audio_input_hf(&model_info.pipeline_tag, &model_info.tags);
        let tool_calling = Self::detect_tool_calling_hf(&model_info.tags, &model_info.config.chat_template);
        let reasoning = Self::detect_reasoning_hf(&model_info.tags, &model_info.model_id);

        Self {
            vision,
            audio_input,
            audio_output: false,  // Less commonly detected
            tool_calling,
            reasoning,
            tool_calling_confidence: 0.7,
            reasoning_confidence: 0.6,
        }
    }

    fn detect_vision_gguf(metadata: &GgufMetadata) -> bool {
        // Architecture-specific
        if matches!(metadata.general_architecture.as_str(), "qwen2vl" | "qwen3vl" | "chameleon" | "cogvlm") {
            return true;
        }

        // MMPROJ indicators
        if metadata.general_architecture == "clip" && metadata.get_bool("clip.has_vision_encoder") == Some(true) {
            return true;
        }

        // Projector type present
        if metadata.get_string("clip.projector_type").is_some() {
            return true;
        }

        // Vision-specific keys
        if metadata.get_u32("clip.vision.image_size").is_some() {
            return true;
        }

        false
    }

    fn detect_tool_calling_template(template: Option<&str>) -> bool {
        if let Some(t) = template {
            let patterns = [
                r"\{\{\s*tools\s*\}\}",
                r"\{%\s*if\s+tools\s",
                r"tool_calls",
                r"<tool_call>",
                r"<tool_response>",
                r"\[AVAILABLE_TOOLS\]",
                r"\{\{\s*\.Tools\s*\}\}",  // Ollama
            ];

            for pattern in &patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(t) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn detect_reasoning_template(template: Option<&str>) -> (bool, f32) {
        if let Some(t) = template {
            let patterns = [
                (r"<think>", 0.9),
                (r"<reasoning>", 0.8),
                (r"reasoning_content", 0.8),
                (r"<\|im_start\|>assistant\\n<think>", 0.95),
            ];

            for (pattern, confidence) in &patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(t) {
                        return (true, *confidence);
                    }
                }
            }
        }
        (false, 0.0)
    }
}
```

---

## 6. References

### GGUF Specification & Conversion

1. [llama.cpp constants.py](https://github.com/ggml-org/llama.cpp/blob/master/gguf-py/gguf/constants.py) - Complete metadata key definitions
2. [llama.cpp convert_hf_to_gguf.py](https://github.com/ggml-org/llama.cpp/blob/master/convert_hf_to_gguf.py) - Conversion patterns
3. [Multimodal GGUFs Collection](https://huggingface.co/collections/ggml-org/multimodal-ggufs-68244e01ff1f39e5bebeeedc) - Official multimodal models
4. [GGUF Metadata Guide](https://opus4i.com/gguf) - GGUF vs HF metadata comparison
5. [GGUF File Format Docs](https://deepwiki.com/ggml-org/llama.cpp/6.1-gguf-file-format) - Technical documentation

### HuggingFace API & Metadata

6. [HuggingFace Tasks Documentation](https://huggingface.co/docs/hub/en/models-tasks) - Pipeline tag taxonomy
7. [Model Cards](https://huggingface.co/docs/hub/en/model-cards) - Model card metadata structure
8. [HfApi Client](https://huggingface.co/docs/huggingface_hub/package_reference/hf_api) - API query parameters
9. [Understand caching](https://huggingface.co/docs/huggingface_hub/en/guides/manage-cache) - Cache directory structure
10. [huggingface.js tasks/index.ts](https://github.com/huggingface/huggingface.js/blob/main/packages/tasks/src/tasks/index.ts) - Complete pipeline tag definitions

### Chat Templates & Capability Detection

11. [HuggingFace Chat Templating](https://huggingface.co/docs/transformers/main/en/chat_templating_writing) - Writing a chat template
12. [Llama 3.1 Prompt Formats](https://www.llama.com/docs/model-cards-and-prompt-formats/llama3_1/) - Llama format specification
13. [Hermes-3 Model Card](https://huggingface.co/NousResearch/Hermes-3-Llama-3.1-8B) - Function calling implementation
14. [Command-R+ Model Card](https://huggingface.co/CohereForAI/c4ai-command-r-plus) - Tool use and grounded generation
15. [DeepSeek-R1 Repository](https://github.com/deepseek-ai/DeepSeek-R1) - Reasoning model implementation
16. [QwQ Repository](https://github.com/QwenLM/QwQ) - Reasoning model with thinking tags

### Industry Tools & Standards

17. [Ollama Tool Calling Docs](https://docs.ollama.com/capabilities/tool-calling) - Ollama tool support
18. [vLLM GGUF Documentation](https://docs.vllm.ai/en/stable/features/quantization/gguf/) - vLLM GGUF handling
19. [LM Studio Tool Use Docs](https://lmstudio.ai/docs/advanced/tool-use) - LM Studio capabilities
20. [GPT4All Chat Templates](https://docs.gpt4all.io/gpt4all_desktop/chat_templates.html) - GPT4All template system

### Model Collections & Examples

21. [Vision Models (GGUF) - LM Studio](https://huggingface.co/collections/lmstudio-ai/vision-models-gguf-6577e1ce821f439498ced0c1) - Curated vision models
22. [NexaAI Multimodal Collection](https://huggingface.co/collections/NexaAI/multimodal-gguf) - Additional examples
23. [Vision Language Models 2025](https://huggingface.co/blog/vlms-2025) - Recent VLM developments
24. [Qwen2.5-Omni GGUF](https://huggingface.co/ggml-org/Qwen2.5-Omni-7B-GGUF) - Any-to-any multimodal example
25. [pixtral-12b-GGUF](https://huggingface.co/ggml-org/pixtral-12b-GGUF) - Vision model example
26. [MiniCPM-V GGUF Discussion](https://github.com/OpenBMB/MiniCPM-V/issues/957) - Real metadata examples

### Security & Additional Resources

27. [JFrog GGUF-SSTI Vulnerability](https://research.jfrog.com/model-threats/gguf-ssti/) - Template security analysis
28. [vLLM Tool Calling](https://docs.vllm.ai/en/latest/features/tool_calling/) - vLLM tool support
29. [vLLM Structured Outputs](https://docs.vllm.ai/en/latest/features/structured_outputs/) - Output formatting
30. [Mistral Function Calling](https://docs.mistral.ai/capabilities/function_calling/) - Mistral tool format

---

## Appendix: Detection Confidence Scoring

**Recommended Scoring System:**

```rust
pub struct CapabilityConfidence {
    pub vision: f32,           // 0.0-1.0
    pub audio: f32,            // 0.0-1.0
    pub tool_calling: f32,     // 0.0-1.0
    pub reasoning: f32,        // 0.0-1.0
}

impl CapabilityConfidence {
    pub fn combine_detections(gguf: &ModelCapabilities, hf: &ModelCapabilities) -> Self {
        // If GGUF is certain, trust it (0.9+ confidence)
        // Otherwise, fallback to HF with lower confidence

        Self {
            vision: if gguf.vision { 0.95 } else { hf.vision as f32 * 0.7 },
            audio: if gguf.audio_input { 0.95 } else { hf.audio_input as f32 * 0.7 },
            tool_calling: gguf.tool_calling_confidence.max(hf.tool_calling_confidence * 0.8),
            reasoning: gguf.reasoning_confidence.max(hf.reasoning_confidence * 0.8),
        }
    }

    pub fn threshold_enabled(&self, capability: &str, threshold: f32) -> bool {
        match capability {
            "vision" => self.vision >= threshold,
            "audio" => self.audio >= threshold,
            "tool_calling" => self.tool_calling >= threshold,
            "reasoning" => self.reasoning >= threshold,
            _ => false,
        }
    }
}
```

---

**Report Generated:** 2026-01-12
**Research Scope:** GGUF metadata, HuggingFace API, chat templates, industry comparison
**Next Steps:** Implement capability detection logic based on this research in crates/objs/src/model_metadata.rs
