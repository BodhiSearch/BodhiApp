# HuggingFace Hub API Research Report - Model Capability Detection

**Research Date:** 2026-01-12
**Purpose:** Investigate HuggingFace Hub API for detecting model capabilities (multimodal, tool-calling, reasoning) for local model metadata extraction

## Table of Contents

1. [API Endpoint Documentation](#api-endpoint-documentation)
2. [Pipeline Tag Taxonomy](#pipeline-tag-taxonomy)
3. [Model Card Tags and Metadata](#model-card-tags-and-metadata)
4. [Model Info API Response Examples](#model-info-api-response-examples)
5. [HuggingFace Cache Structure](#huggingface-cache-structure)
6. [Capability Detection Strategy](#capability-detection-strategy)
7. [References](#references)

---

## API Endpoint Documentation

### GET /api/models/{namespace}/{repo}

**Base URL:** `https://huggingface.co/api/models/{namespace}/{repo}`

**Note:** The official OpenAPI specification at `https://huggingface.co/.well-known/openapi.json` does not include complete documentation for this endpoint. However, the endpoint is functional and returns comprehensive model metadata.

#### Query Parameters

Based on [HfApi Client documentation](https://huggingface.co/docs/huggingface_hub/package_reference/hf_api):

| Parameter | Type | Description |
|-----------|------|-------------|
| `expand` | list[str] | List of properties to include in response. Cannot be used with `full`, `cardData`, or `fetch_config` |
| `cardData` | bool | Whether to include model card metadata (carbon emissions, metrics, training datasets) |
| `full` | bool | Return all available properties |

#### Expand Parameter Values (for models)

- `author` - Model author/organization
- `cardData` - Model card YAML frontmatter metadata
- `citation` - Citation information
- `createdAt` - Creation timestamp
- `disabled` - Whether model is disabled
- `description` - Model description
- `downloads` - Download count (recent)
- `downloadsAllTime` - Total download count
- `gated` - Whether model requires access approval
- `lastModified` - Last modification timestamp
- `likes` - Number of likes
- `paperswithcode_id` - Papers With Code identifier
- `private` - Whether model is private
- `siblings` - Model files
- `sha` - Commit hash
- `tags` - All model tags
- `trendingScore` - Trending score
- `usedStorage` - Storage usage
- `resourceGroup` - Resource group information
- `xetEnabled` - Whether Xet storage is enabled

#### Response Schema (Core Fields)

```json
{
  "_id": "string (internal ID)",
  "id": "string (full repo ID: namespace/repo)",
  "modelId": "string (same as id)",
  "pipeline_tag": "string (task type)",
  "library_name": "string (e.g., transformers, peft, gguf)",
  "tags": ["string"],
  "author": "string",
  "downloads": "number",
  "downloadsAllTime": "number",
  "likes": "number",
  "createdAt": "ISO 8601 timestamp",
  "lastModified": "ISO 8601 timestamp",
  "private": "boolean",
  "gated": "boolean | string",
  "disabled": "boolean",
  "cardData": {
    "language": ["string"],
    "license": "string",
    "datasets": ["string"],
    "metrics": ["string"],
    "base_model": "string | string[]",
    "tags": ["string"],
    "pipeline_tag": "string"
  },
  "siblings": [
    {
      "rfilename": "string (relative file path)",
      "size": "number (bytes)",
      "lfs": {
        "oid": "string",
        "size": "number",
        "pointerSize": "number"
      }
    }
  ],
  "config": {
    "architectures": ["string"],
    "model_type": "string",
    "auto_model": "string",
    "chat_template": "string | object"
  }
}
```

---

## Pipeline Tag Taxonomy

**Source:** [HuggingFace Tasks Documentation](https://huggingface.co/docs/hub/en/models-tasks), [huggingface.js tasks/index.ts](https://github.com/huggingface/huggingface.js/blob/main/packages/tasks/src/tasks/index.ts)

Pipeline tags (also called task types) describe the "shape" of each model's API (inputs and outputs). They determine which widget is displayed on the model page and which Inference API to use.

### Complete Pipeline Tag List (57 total)

#### Natural Language Processing (NLP)

| Pipeline Tag | Description | Capability Relevance |
|--------------|-------------|---------------------|
| `text-generation` | Generate text from a prompt | Standard text models |
| `text2text-generation` | Text-to-text generation | Seq2seq models |
| `text-classification` | Classify text into categories | Sentiment, classification |
| `token-classification` | Classify individual tokens (NER, POS) | Named entity recognition |
| `fill-mask` | Fill in masked tokens | BERT-style models |
| `question-answering` | Answer questions from context | QA systems |
| `summarization` | Summarize long text | Summary generation |
| `translation` | Translate between languages | Translation models |
| `sentence-similarity` | Compare sentence similarity | Embedding models |
| `text-ranking` | Rank text relevance | Search, retrieval |
| `text-retrieval` | Retrieve relevant documents | RAG, search |
| `zero-shot-classification` | Classify without training data | Zero-shot NLP |
| `multiple-choice` | Multiple choice question answering | Multiple choice tasks |
| `conversational` | Multi-turn conversation | Chat models |
| `table-question-answering` | Answer questions from tables | Tabular QA |
| `table-to-text` | Generate text from tables | Table understanding |
| `tabular-to-text` | Convert tabular data to text | Data to text |

#### Audio

| Pipeline Tag | Description | Capability Relevance |
|--------------|-------------|---------------------|
| `automatic-speech-recognition` | Speech to text | **Audio input** |
| `audio-classification` | Classify audio content | Audio understanding |
| `audio-to-audio` | Transform audio | Audio processing |
| `text-to-speech` | Generate speech from text | TTS |
| `text-to-audio` | Generate audio from text | Audio generation |
| `audio-text-to-text` | Audio + text input, text output | **Multimodal (audio+text)** |
| `voice-activity-detection` | Detect voice activity | Audio processing |

#### Computer Vision

| Pipeline Tag | Description | Capability Relevance |
|--------------|-------------|---------------------|
| `image-classification` | Classify images | Image understanding |
| `image-segmentation` | Segment image regions | Image analysis |
| `image-feature-extraction` | Extract image features | Image embeddings |
| `object-detection` | Detect objects in images | Object recognition |
| `zero-shot-image-classification` | Classify images without training | Zero-shot vision |
| `zero-shot-object-detection` | Detect objects without training | Zero-shot detection |
| `depth-estimation` | Estimate depth from images | 3D understanding |
| `keypoint-detection` | Detect keypoints in images | Pose, landmarks |
| `mask-generation` | Generate segmentation masks | Segmentation |
| `video-classification` | Classify video content | Video understanding |

#### Multimodal

| Pipeline Tag | Description | Capability Relevance |
|--------------|-------------|---------------------|
| `image-text-to-text` | Image + text input, text output | **Vision language models (VLMs)** |
| `image-to-text` | Generate text from images | **Vision models** |
| `visual-question-answering` | Answer questions about images | **Vision + QA** |
| `document-question-answering` | Answer questions from documents | **Document understanding** |
| `text-to-image` | Generate images from text | Image generation |
| `image-to-image` | Transform images | Image editing |
| `image-text-to-image` | Image + text to image | Multimodal generation |
| `image-text-to-video` | Image + text to video | Video generation |
| `image-to-video` | Image to video | Video synthesis |
| `text-to-video` | Text to video | Video generation |
| `video-text-to-text` | Video + text to text | **Video understanding** |
| `visual-document-retrieval` | Retrieve documents visually | Document search |
| `video-to-video` | Transform videos | Video processing |

#### 3D and Other Modalities

| Pipeline Tag | Description | Capability Relevance |
|--------------|-------------|---------------------|
| `text-to-3d` | Generate 3D models from text | 3D generation |
| `image-to-3d` | Generate 3D models from images | 3D reconstruction |
| `any-to-any` | Any modality to any modality | **Universal multimodal** |

#### Specialized Tasks

| Pipeline Tag | Description | Capability Relevance |
|--------------|-------------|---------------------|
| `feature-extraction` | Extract embeddings | Embedding models |
| `graph-ml` | Graph machine learning | Graph neural networks |
| `reinforcement-learning` | RL models | RL agents |
| `robotics` | Robotics models | Robot control |
| `time-series-forecasting` | Forecast time series | Time series |
| `tabular-classification` | Classify tabular data | Tabular ML |
| `tabular-regression` | Regress on tabular data | Tabular ML |
| `unconditional-image-generation` | Generate images unconditionally | Image synthesis |
| `other` | Other/unknown tasks | Unclassified |

### Capability Mapping from Pipeline Tags

| Capability | Pipeline Tags |
|-----------|---------------|
| **Vision/Multimodal** | `image-text-to-text`, `image-to-text`, `visual-question-answering`, `document-question-answering`, `image-classification`, `object-detection`, `video-text-to-text`, `any-to-any` |
| **Audio** | `automatic-speech-recognition`, `audio-classification`, `audio-text-to-text`, `audio-to-audio`, `text-to-speech`, `text-to-audio` |
| **Tool/Function Calling** | *Not detected via pipeline_tag* - requires tag/metadata analysis |
| **Reasoning** | *Not detected via pipeline_tag* - requires tag/metadata analysis |

---

## Model Card Tags and Metadata

**Sources:**
- [Model Cards Documentation](https://huggingface.co/docs/hub/en/model-cards)
- [Create and Share Model Cards](https://huggingface.co/docs/huggingface_hub/en/guides/model-cards)

### YAML Frontmatter Structure

Model cards include YAML metadata at the top of the `README.md` file:

```yaml
---
language:
  - en
  - es
tags:
  - vision
  - multimodal
  - function-calling
  - tool-use
  - reasoning
  - chain-of-thought
  - custom-tag
license: mit
datasets:
  - dataset-namespace/dataset-name
base_model: meta-llama/Llama-3.1-8B
library_name: transformers
pipeline_tag: image-text-to-text
---
```

### Key Metadata Fields for Capability Detection

| Field | Type | Description | Capability Relevance |
|-------|------|-------------|---------------------|
| `pipeline_tag` | string | Task type | Primary capability indicator |
| `tags` | string[] | Free-form tags | Custom capability tags |
| `library_name` | string | Framework/library | `peft`, `gguf`, `transformers`, etc. |
| `base_model` | string or string[] | Base model(s) | Inheritance from base model |
| `language` | string[] | Supported languages | Multilingual support |
| `datasets` | string[] | Training datasets | Dataset-based capability hints |

### Common Capability Tags (Observed in Practice)

#### Vision/Multimodal Tags

- `vision`
- `multimodal`
- `image-to-text`
- `image-text-to-text`
- `visual-question-answering`
- `llava`
- `clip`
- `vit` (Vision Transformer)

#### Tool/Function Calling Tags

- `function calling` (with space)
- `function-calling` (with hyphen)
- `tool-use`
- `tool calling`
- `json mode`
- `structured output`
- `chatml` (often indicates tool support)

#### Reasoning Tags

- `reasoning`
- `chain-of-thought`
- `cot`
- `thinking`
- `o1-style` / `o1`
- `r1-style` / `r1`

#### Other Relevant Tags

- `instruct` - Instruction-following
- `chat` - Chat/conversation optimized
- `conversational` - Dialogue models
- `finetune` - Fine-tuned model
- `rlhf` - Reinforcement learning from human feedback
- `dpo` - Direct preference optimization

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

---

## Model Info API Response Examples

### Example 1: Multimodal Model (llava-hf/llava-1.5-7b-hf)

**Request:** `GET https://huggingface.co/api/models/llava-hf/llava-1.5-7b-hf`

**Response (Key Fields):**

```json
{
  "_id": "658fd0ce6e5c9039006d7e31",
  "id": "llava-hf/llava-1.5-7b-hf",
  "pipeline_tag": "image-to-text",
  "library_name": "transformers",
  "tags": [
    "transformers",
    "safetensors",
    "llava",
    "image-to-text",
    "vision",
    "image-text-to-text",
    "conversational",
    "en"
  ],
  "config": {
    "architectures": ["LlavaForConditionalGeneration"],
    "model_type": "llava",
    "auto_model": "AutoModelForVision2Seq"
  },
  "cardData": {
    "language": ["en"],
    "pipeline_tag": "image-to-text",
    "tags": ["vision", "image-text-to-text"],
    "datasets": ["liuhaotian/LLaVA-Instruct-150K"]
  },
  "downloads": 941581,
  "likes": 328
}
```

**Capability Detection:**
- **Vision:** ✅ (pipeline_tag: `image-to-text`, tags: `vision`, `image-text-to-text`, architecture: `LlavaForConditionalGeneration`)
- **Tool Calling:** ❌
- **Reasoning:** ❌

---

### Example 2: Function Calling Model (NousResearch/Hermes-3-Llama-3.1-8B)

**Request:** `GET https://huggingface.co/api/models/NousResearch/Hermes-3-Llama-3.1-8B`

**Response (Key Fields):**

```json
{
  "_id": "66a5de99a28bc058db44d78f",
  "id": "NousResearch/Hermes-3-Llama-3.1-8B",
  "pipeline_tag": "text-generation",
  "library_name": "transformers",
  "tags": [
    "transformers",
    "safetensors",
    "llama",
    "text-generation",
    "Llama-3",
    "instruct",
    "finetune",
    "chatml",
    "gpt4",
    "synthetic data",
    "distillation",
    "function calling",
    "json mode",
    "axolotl",
    "roleplaying",
    "chat",
    "conversational",
    "en"
  ],
  "config": {
    "architectures": ["LlamaForCausalLM"],
    "model_type": "llama",
    "chat_template": [
      {
        "name": "default",
        "template": "..."
      },
      {
        "name": "tool_use",
        "template": "...function calling AI model...<tool_call>...</tool_call>"
      }
    ]
  },
  "downloads": 47272,
  "likes": 381
}
```

**Capability Detection:**
- **Vision:** ❌
- **Tool Calling:** ✅ (tags: `function calling`, `json mode`, `chatml`, chat_template: `tool_use` with `<tool_call>` tags)
- **Reasoning:** ❌

---

### Example 3: Reasoning Model (deepseek-ai/DeepSeek-R1)

**Request:** `GET https://huggingface.co/api/models/deepseek-ai/DeepSeek-R1`

**Response (Key Fields):**

```json
{
  "_id": "679f2a9e8f332d66ccfb17a1",
  "id": "deepseek-ai/DeepSeek-R1",
  "pipeline_tag": "text-generation",
  "library_name": "transformers",
  "tags": [
    "transformers",
    "safetensors",
    "deepseek_v3",
    "text-generation",
    "conversational",
    "custom_code",
    "arxiv:2501.12948",
    "license:mit",
    "text-generation-inference",
    "endpoints_compatible",
    "fp8",
    "region:us"
  ],
  "config": {
    "architectures": ["DeepseekV3ForCausalLM"],
    "model_type": "deepseek_v3",
    "chat_template": "...{% if add_generation_prompt and not ns.is_tool %}{{'<｜Assistant｜><think>\\n'}}{% endif %}..."
  },
  "cardData": {
    "license": "mit",
    "pipeline_tag": "text-generation",
    "arxiv": "2501.12948"
  },
  "downloads": 363588,
  "likes": 12946
}
```

**Capability Detection:**
- **Vision:** ❌
- **Tool Calling:** ❌ (no function calling tags, but chat template suggests tool support with `ns.is_tool`)
- **Reasoning:** ✅ (model name contains "R1", chat_template includes `<think>` tag, ArXiv paper 2501.12948 describes reasoning capabilities)

---

## HuggingFace Cache Structure

**Source:** [Understand caching](https://huggingface.co/docs/huggingface_hub/en/guides/manage-cache)

### Default Cache Location

- **Linux/Mac:** `~/.cache/huggingface/hub`
- **Windows:** `%USERPROFILE%\.cache\huggingface\hub`
- **Environment Variables:**
  - `HF_HOME` - Base cache directory
  - `HUGGINGFACE_HUB_CACHE` - Direct cache path
  - `HF_HUB_CACHE` - Alternative cache path

### Cache Directory Structure

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
│       │   ├── config.json -> ../../blobs/{sha256-hash}
│       │   ├── model.safetensors -> ../../blobs/{sha256-hash}
│       │   └── tokenizer.json -> ../../blobs/{sha256-hash}
│       └── {commit-hash-2}/
│           └── ...
├── datasets--{namespace}--{repo}/
│   └── ...
└── spaces--{namespace}--{repo}/
    └── ...
```

### Repository Naming Convention

**Format:** `{type}--{namespace}--{repo}` or `{type}--{repo}`

**Examples:**
- `models--meta-llama--Llama-3.1-8B` → `meta-llama/Llama-3.1-8B`
- `models--bert-base-cased` → `bert-base-cased` (no namespace)
- `datasets--google--fleurs` → `google/fleurs`

### Parsing Repository from Cache Path

**Algorithm:**

1. Extract directory name from path
2. Split by `--` delimiter
3. Parse components:
   - First component: repo type (`models`, `datasets`, `spaces`)
   - Remaining components: namespace (if present) and repo name
4. Reconstruct repo ID: `{namespace}/{repo}` or just `{repo}`

**Python Example:**

```python
from pathlib import Path

def parse_hf_cache_path(cache_path: str) -> tuple[str, str]:
    """
    Parse HuggingFace cache path to extract repo type and repo ID.

    Args:
        cache_path: Path like "~/.cache/huggingface/hub/models--meta-llama--Llama-3.1-8B"

    Returns:
        Tuple of (repo_type, repo_id)
        e.g., ("models", "meta-llama/Llama-3.1-8B")
    """
    dir_name = Path(cache_path).name
    parts = dir_name.split("--")

    repo_type = parts[0]  # "models", "datasets", or "spaces"

    if len(parts) == 2:
        # No namespace: models--bert-base-cased
        repo_id = parts[1]
    elif len(parts) == 3:
        # With namespace: models--meta-llama--Llama-3.1-8B
        repo_id = f"{parts[1]}/{parts[2]}"
    else:
        # Handle edge cases with more dashes in repo name
        repo_id = "/".join(parts[1:])

    return repo_type, repo_id

# Example usage
cache_path = "~/.cache/huggingface/hub/models--meta-llama--Llama-3.1-8B"
repo_type, repo_id = parse_hf_cache_path(cache_path)
# Returns: ("models", "meta-llama/Llama-3.1-8B")
```

### Snapshots and Revisions

- Each `snapshots/{commit-hash}/` directory represents a specific revision
- Files in snapshots are symlinks to deduplicated blobs
- `refs/main` contains the current commit hash for the main branch
- Multiple revisions can share blob files (deduplication)

### Local Model Detection from hf_home

**Strategy:**

1. Scan `{hf_home}/models--*` directories
2. Parse directory names to extract repo IDs
3. Check `refs/main` for current revision
4. Read `snapshots/{revision}/config.json` for model metadata
5. Query HuggingFace API with repo ID to get full metadata
6. Cache metadata locally to avoid repeated API calls

---

## Capability Detection Strategy

### Detection Priority (Highest to Lowest)

1. **Pipeline Tag** - Most reliable, standardized
2. **Model Architecture** - From config.json `architectures` field
3. **Tags Array** - User-provided, may be incomplete
4. **Chat Template** - Inspect for tool/reasoning patterns
5. **Model Name** - Pattern matching (e.g., "R1", "vision", "tool")
6. **Base Model Metadata** - Inherit from base model

### Capability Detection Rules

#### Vision/Multimodal Detection

**Positive Indicators (Priority Order):**

1. **pipeline_tag:**
   - `image-text-to-text`
   - `image-to-text`
   - `visual-question-answering`
   - `document-question-answering`
   - `video-text-to-text`
   - `any-to-any`

2. **Architecture patterns:**
   - `Llava*` (LlavaForConditionalGeneration)
   - `*VisionEncoder*`
   - `Qwen2VL*`
   - `*Vision2Seq`

3. **Tags:**
   - `vision`
   - `multimodal`
   - `image-text-to-text`
   - `llava`
   - `clip`

#### Audio Detection

**Positive Indicators:**

1. **pipeline_tag:**
   - `automatic-speech-recognition`
   - `audio-classification`
   - `audio-text-to-text`
   - `audio-to-audio`

2. **Tags:**
   - `audio`
   - `speech`
   - `whisper`

#### Tool/Function Calling Detection

**Positive Indicators:**

1. **Tags (exact match):**
   - `function calling` (with space)
   - `function-calling`
   - `tool-use`
   - `tool calling`
   - `json mode`
   - `chatml` (high correlation)

2. **Chat Template:**
   - Contains `tool_use` template name
   - Contains `<tool_call>` tags
   - Contains `<function>` tags
   - Template includes `tools` parameter

3. **Model Name Patterns:**
   - Contains "Hermes"
   - Contains "tool"
   - Contains "function"

#### Reasoning Detection

**Positive Indicators:**

1. **Model Name Patterns:**
   - Contains "R1" or "r1"
   - Contains "DeepSeek-R"
   - Contains "QwQ"
   - Contains "reasoning"

2. **Chat Template:**
   - Contains `<think>` tags
   - Contains "reasoning mode"
   - Template has thinking/non-thinking mode switch

3. **Tags:**
   - `reasoning`
   - `chain-of-thought`
   - `cot`
   - `thinking`

### Recommended Detection Implementation

```rust
pub struct ModelCapabilities {
    pub vision: bool,
    pub audio: bool,
    pub tool_calling: bool,
    pub reasoning: bool,
}

impl ModelCapabilities {
    pub fn detect(
        pipeline_tag: Option<&str>,
        tags: &[String],
        architectures: &[String],
        chat_template: Option<&str>,
        model_id: &str,
    ) -> Self {
        let vision = Self::detect_vision(pipeline_tag, tags, architectures);
        let audio = Self::detect_audio(pipeline_tag, tags);
        let tool_calling = Self::detect_tool_calling(tags, chat_template);
        let reasoning = Self::detect_reasoning(tags, chat_template, model_id);

        Self { vision, audio, tool_calling, reasoning }
    }

    fn detect_vision(
        pipeline_tag: Option<&str>,
        tags: &[String],
        architectures: &[String],
    ) -> bool {
        // Priority 1: Pipeline tag
        if let Some(tag) = pipeline_tag {
            if matches!(tag,
                "image-text-to-text" | "image-to-text" | "visual-question-answering" |
                "document-question-answering" | "video-text-to-text" | "any-to-any"
            ) {
                return true;
            }
        }

        // Priority 2: Architecture
        if architectures.iter().any(|arch| {
            arch.contains("Llava") || arch.contains("Vision") || arch.contains("Qwen2VL")
        }) {
            return true;
        }

        // Priority 3: Tags
        tags.iter().any(|tag| {
            matches!(tag.as_str(),
                "vision" | "multimodal" | "image-text-to-text" | "llava" | "clip"
            )
        })
    }

    fn detect_audio(pipeline_tag: Option<&str>, tags: &[String]) -> bool {
        // Priority 1: Pipeline tag
        if let Some(tag) = pipeline_tag {
            if matches!(tag,
                "automatic-speech-recognition" | "audio-classification" |
                "audio-text-to-text" | "audio-to-audio"
            ) {
                return true;
            }
        }

        // Priority 2: Tags
        tags.iter().any(|tag| {
            matches!(tag.as_str(), "audio" | "speech" | "whisper")
        })
    }

    fn detect_tool_calling(tags: &[String], chat_template: Option<&str>) -> bool {
        // Priority 1: Tags
        if tags.iter().any(|tag| {
            matches!(tag.as_str(),
                "function calling" | "function-calling" | "tool-use" |
                "tool calling" | "json mode" | "chatml"
            )
        }) {
            return true;
        }

        // Priority 2: Chat template
        if let Some(template) = chat_template {
            let template_lower = template.to_lowercase();
            return template_lower.contains("tool_use") ||
                   template_lower.contains("<tool_call>") ||
                   template_lower.contains("<function>");
        }

        false
    }

    fn detect_reasoning(
        tags: &[String],
        chat_template: Option<&str>,
        model_id: &str,
    ) -> bool {
        // Priority 1: Model name patterns
        let model_lower = model_id.to_lowercase();
        if model_lower.contains("-r1") ||
           model_lower.contains("deepseek-r") ||
           model_lower.contains("qwq") ||
           model_lower.contains("reasoning") {
            return true;
        }

        // Priority 2: Chat template
        if let Some(template) = chat_template {
            let template_lower = template.to_lowercase();
            if template_lower.contains("<think>") ||
               template_lower.contains("reasoning mode") {
                return true;
            }
        }

        // Priority 3: Tags
        tags.iter().any(|tag| {
            matches!(tag.as_str(),
                "reasoning" | "chain-of-thought" | "cot" | "thinking"
            )
        })
    }
}
```

---

## References

### Official Documentation

1. [HuggingFace Tasks Documentation](https://huggingface.co/docs/hub/en/models-tasks) - Pipeline tag taxonomy
2. [Model Cards](https://huggingface.co/docs/hub/en/model-cards) - Model card metadata structure
3. [HfApi Client](https://huggingface.co/docs/huggingface_hub/package_reference/hf_api) - API query parameters
4. [Understand caching](https://huggingface.co/docs/huggingface_hub/en/guides/manage-cache) - Cache directory structure

### Source Code

5. [huggingface.js tasks/index.ts](https://github.com/huggingface/huggingface.js/blob/main/packages/tasks/src/tasks/index.ts) - Complete pipeline tag definitions
6. [huggingface_hub hf_api.py](https://github.com/huggingface/huggingface_hub/blob/main/src/huggingface_hub/hf_api.py) - Python API client
7. [hub-docs modelcard.md](https://github.com/huggingface/hub-docs/blob/main/docs/hub/models-tasks.md) - Model card specification

### Examples and Collections

8. [Function Calling Models Collection](https://huggingface.co/collections/MarketAgents/function-calling-models-tool-use-6760677e7a05c491c232dee7) - Curated function calling models
9. [NVIDIA Reasoning Models](https://huggingface.co/blog/nvidia/open-reasoning-models) - Reasoning capabilities overview
10. [Qwen Models](https://huggingface.co/Qwen) - Examples of vision + tool + reasoning models

### API Endpoints (Tested)

11. `GET https://huggingface.co/api/models/llava-hf/llava-1.5-7b-hf` - Multimodal example
12. `GET https://huggingface.co/api/models/NousResearch/Hermes-3-Llama-3.1-8B` - Function calling example
13. `GET https://huggingface.co/api/models/deepseek-ai/DeepSeek-R1` - Reasoning example
14. `GET https://huggingface.co/.well-known/openapi.json` - OpenAPI specification (partial)

---

**Report Generated:** 2026-01-12
**Next Steps:** Implement capability detection logic based on this research in `crates/objs/src/model_metadata.rs`
