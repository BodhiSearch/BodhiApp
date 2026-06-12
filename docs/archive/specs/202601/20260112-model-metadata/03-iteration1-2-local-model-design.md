# Model Metadata API - Iteration 1.2 Delta Design Specification

**Date**: 2026-01-13
**Version**: Iteration 1.2 (Local GGUF Models - Research-Driven Updates)
**Based On**: Iteration 1 Design + Research Report Findings

---

## ⚠️ SPECIFICATION STATUS

**Status**: ✅ IMPLEMENTATION COMPLETE

**Implementation Summary**:
- **Backend**: Fully implemented and committed (4 commits: 7e6339ab, 676bbd9d, df38c449, d95320a3)
- **Frontend**: Implemented (uncommitted changes in working tree)
- **E2E Tests**: Passing (7/7 tests in model-metadata.spec.mjs)

**Commits**:
1. `7e6339ab` - Research phase documentation
2. `676bbd9d` - Core GGUF metadata extraction (capabilities.rs, model_metadata.rs, test fixtures)
3. `df38c449` - Database schema and queue infrastructure (migration 0006, queue_service.rs)
4. `d95320a3` - API endpoints (routes_models_metadata.rs, DTOs, OpenAPI)

**Uncommitted Changes**:
- `crates/bodhi/src/app/ui/models/components/ModelPreviewModal.tsx` - Modal component
- `crates/bodhi/src/hooks/useModelMetadata.ts` - React hooks for metadata API
- `crates/bodhi/src/app/ui/models/page.tsx` - Updated models page with refresh UI
- `crates/lib_bodhiserver_napi/tests-js/specs/models/model-metadata.spec.mjs` - E2E tests

**What Was Implemented**:
- Nullable capability columns (NULL = unknown, false = explicitly no, true = explicitly yes)
- GGUF-first detection strategy (HF API deferred to Iteration 2)
- Expanded capability patterns (vision, audio, thinking, function_calling, structured_output)
- Research-validated quantization extraction (filename priority over metadata)
- Background queue for bulk metadata refresh
- Synchronous per-model refresh endpoint
- Frontend UI for preview modal and refresh actions

**What Was Deferred to Iteration 2**:
- HuggingFace API integration for fallback detection
- Batch query optimization for GET /models endpoint

---

## Executive Summary

### What Changed

This delta specification updates Iteration 1 design based on comprehensive research into GGUF metadata, HuggingFace API, chat templates, and industry implementations. The core changes are:

1. **Priority Shift**: GGUF metadata becomes primary source, HF API becomes fallback (was opposite)
2. **Capability Expansion**: Vision detection expanded, thinking/tools/structured_output moved from hardcoded `false` to pattern detection
3. **Quantization Flip**: Filename parsing prioritized over metadata key (research-validated)
4. **Nullable Schema**: Capability columns become nullable (NULL = unknown, false = explicitly no, true = explicitly yes)
5. **HF Integration**: New HfApiService trait for gap-filling only
6. **Test Fixtures**: Real GGUF test files replace mocked unit tests

### Why

**Research Findings:**
- GGUF metadata provides 119+ architectures, comprehensive multimodal indicators, and authoritative chat templates
- HF API is network-dependent, less reliable for local-only workflows, but provides standardized fallback
- Chat templates contain detectable patterns for tool calling, reasoning, structured output
- Industry tools (Ollama, vLLM, LM Studio) prioritize metadata introspection over API calls
- Quantization info more reliably extracted from filenames than metadata keys (research Phase 1 finding)

**Impact on Current Implementation:**
- `capabilities.rs` module expands significantly (vision patterns, adds thinking/tools/structured)
- Migration 0007 required to make capability columns nullable
- New `hf_api_service.rs` trait and implementation
- `batch_get_metadata()` needed in DbService for GET endpoint completion
- Test suite requires real GGUF fixtures (Python script to generate)

---

## Priority Shift Analysis

### Original Design (Iteration 1)

**HuggingFace API Authoritative:**
```
1. Query HF API for repo metadata (pipeline_tag, tags, config)
2. Apply HF detection rules (standardized taxonomy)
3. Fall back to GGUF metadata if HF unavailable
4. Store results in DB
```

**Rationale (Original):**
- HF provides standardized capability taxonomy
- Comprehensive model card metadata
- Consistent across all models in ecosystem

### New Design (Iteration 1.2)

**GGUF Metadata Primary:**
```
1. Extract GGUF metadata (architecture, clip.*, chat_template)
2. Apply GGUF capability detection (direct indicators)
3. For NULL/unknown capabilities only:
   a. Query HF API for repo metadata
   b. Apply HF detection rules
   c. Merge results (GGUF wins on conflicts)
4. Store results with NULL for undetected capabilities
```

**Rationale (Research-Driven):**
- GGUF metadata is embedded (offline capability, no network dependency)
- GGUF chat templates are authoritative (model-specific, not base model)
- HF API requires network access (slower, potential failures)
- Industry pattern: metadata introspection first, API fallback (vLLM, GPT4All)
- GGUF provides 119+ architecture identifiers with capability implications

### Decision Tree for HF API Calls

```
For each capability (vision, audio, thinking, tools, structured_output):

  1. Extract capability from GGUF metadata
     ├─ If result = true → Store true, skip HF API
     ├─ If result = false → Store false, skip HF API
     └─ If result = NULL → Continue to step 2

  2. Parse repo ID from model path/metadata
     ├─ If repo ID unavailable → Store NULL, done
     └─ If repo ID available → Continue to step 3

  3. Query HF API for model info
     ├─ If API call fails (offline/timeout/404) → Store NULL, done
     └─ If API call succeeds → Continue to step 4

  4. Apply HF detection rules
     ├─ If HF detects capability → Store true
     ├─ If HF detects no capability → Store false
     └─ If HF inconclusive → Store NULL
```

**Key Principle:** GGUF detection results are never overwritten by HF API. HF only fills NULL gaps.

---

## Capability Detection Updates

### Vision Capability (Expanded)

**Current Implementation (Iteration 1):**
```rust
fn has_vision(metadata: &GGUFMetadata) -> bool {
    metadata.get("clip.has_vision_encoder").as_bool().unwrap_or(false)
    || metadata.contains_key("clip.vision.image_size")
    || metadata.get("general.type").as_str() == Some("mmproj")
}
```

**Updated Implementation (Iteration 1.2):**
```rust
pub fn detect_vision(metadata: &GGUFMetadata) -> Option<bool> {
    // 1. Architecture-specific vision models
    if let Some(arch) = metadata.get("general.architecture").and_then(|v| v.as_str()) {
        match arch {
            "qwen2vl" | "qwen3vl" | "qwen3vlmoe" | "chameleon" | "cogvlm" | "minicpm" | "minicpm3" => {
                return Some(true);
            }
            _ => {}
        }
    }

    // 2. MMPROJ architecture with vision encoder
    if metadata.get("general.architecture").and_then(|v| v.as_str()) == Some("clip") {
        if metadata.get("clip.has_vision_encoder").and_then(|v| v.as_bool()) == Some(true) {
            return Some(true);
        }
    }

    // 3. Projector type presence (strong indicator)
    if metadata.contains_key("clip.projector_type") || metadata.contains_key("clip.vision.projector_type") {
        return Some(true);
    }

    // 4. Vision-specific metadata keys
    if metadata.contains_key("clip.vision.image_size")
        || metadata.contains_key("clip.vision.patch_size")
        || metadata.contains_key("clip.vision.embedding_length") {
        return Some(true);
    }

    // 5. Explicit false indicator
    if metadata.get("clip.has_vision_encoder").and_then(|v| v.as_bool()) == Some(false) {
        return Some(false);
    }

    // 6. No evidence found
    None
}
```

**Key Changes:**
- Returns `Option<bool>` instead of `bool` (nullable semantics)
- Expanded architecture list: qwen2vl, qwen3vl, qwen3vlmoe, chameleon, cogvlm, minicpm, minicpm3
- Added projector type check (clip.projector_type, clip.vision.projector_type)
- Added vision metadata keys: patch_size, embedding_length
- Explicit false detection (clip.has_vision_encoder = false)
- Returns None when no evidence found (not false by default)

### Tool Calling Capability (New)

**Current Implementation (Iteration 1):**
```rust
// Hardcoded false
tools.function_calling = false;
```

**Updated Implementation (Iteration 1.2):**
```rust
pub fn detect_tool_calling(chat_template: Option<&str>) -> Option<bool> {
    if let Some(template) = chat_template {
        // Template patterns indicating tool support
        let patterns = [
            r"\{\{\s*tools\s*\}\}",                      // {{ tools }}
            r"\{%\s*if\s+tools\s*%\}",                   // {% if tools %}
            r"\{%-?\s*if\s+tools\s*-?%\}",               // {%- if tools -%}
            r"\{%\s*for\s+tool\s+in\s+tools",            // {% for tool in tools %}
            r"'tool_calls'\s+in\s+message",              // 'tool_calls' in message
            r"message\[['\"]\s*role\s*['\"]\]\s*==\s*['\"]tool['\"]", // message['role'] == 'tool'
            r"<tool_call>",                              // Hermes-3 tag
            r"<tool_response>",                          // Hermes-3 response tag
            r"\[TOOL_CALLS\]",                           // Mistral format
            r"\[AVAILABLE_TOOLS\]",                      // Mistral tools declaration
            r"<tools>",                                  // Generic tools wrapper
            r"\{\{\s*\.Tools\s*\}\}",                    // Ollama Go template: {{ .Tools }}
            r"\{\{\s*if\s+\.Tools\s*\}\}",               // Ollama Go template: {{ if .Tools }}
            r"\{\{\s*range\s+\.Tools\s*\}\}",            // Ollama Go template: {{ range .Tools }}
            r"builtin_tools",                            // Llama 3.1 built-in
            r"tool_name",                                // Command-R format
            r"json_to_python_type",                      // Hermes-3 macro
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(template) {
                    return Some(true);
                }
            }
        }

        // If template exists but no tool patterns found
        return Some(false);
    }

    // No chat template available
    None
}
```

**Key Implementation Notes:**
- String-only regex matching (NEVER execute Jinja2 templates - security)
- Comprehensive pattern coverage: Jinja2, Ollama Go templates, model-specific tags
- Returns Some(true) if pattern detected, Some(false) if template exists but no patterns
- Returns None if no chat template metadata available (triggers HF API fallback)

**HF API Fallback:**
```rust
pub fn detect_tool_calling_hf(tags: &[String], config_chat_template: Option<&str>) -> Option<bool> {
    // 1. Check tags
    let tool_tags = [
        "function calling",
        "function-calling",
        "tool-use",
        "tool calling",
        "json mode",
        "chatml",
    ];

    for tag in tags {
        let tag_lower = tag.to_lowercase();
        if tool_tags.iter().any(|t| tag_lower.contains(t)) {
            return Some(true);
        }
    }

    // 2. If config.chat_template available, apply same regex detection
    if let Some(template) = config_chat_template {
        return detect_tool_calling(Some(template));
    }

    // No evidence
    None
}
```

### Thinking Capability (New)

**Current Implementation (Iteration 1):**
```rust
// Hardcoded false
capabilities.thinking = false;
```

**Updated Implementation (Iteration 1.2):**
```rust
pub fn detect_thinking(
    chat_template: Option<&str>,
    model_name: Option<&str>,
    hf_tags: Option<&[String]>
) -> Option<bool> {
    // 1. Chat template pattern detection (highest confidence)
    if let Some(template) = chat_template {
        let patterns = [
            r"<think>",                                  // DeepSeek-R1, QwQ tag
            r"</think>",                                 // Closing tag
            r"<reasoning>",                              // Alternative tag
            r"</reasoning>",                             // Alternative closing
            r"reasoning_content",                        // API response field
            r"<\|im_start\|>assistant\\n<think>\\n",    // QwQ specific format
            r"add_generation_prompt.*<think>",          // QwQ template pattern
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(template) {
                    return Some(true);
                }
            }
        }
    }

    // 2. Model name heuristics (medium confidence)
    if let Some(name) = model_name {
        let name_lower = name.to_lowercase();
        let reasoning_indicators = [
            "-r1", // DeepSeek-R1, Llama-3.3-70B-R1
            "qwq",
            "reasoning",
            "deepseek-r",
        ];

        for indicator in &reasoning_indicators {
            if name_lower.contains(indicator) {
                return Some(true);
            }
        }
    }

    // 3. HF API tags (low confidence, fallback only)
    if let Some(tags) = hf_tags {
        let thinking_tags = [
            "reasoning",
            "chain-of-thought",
            "cot",
            "thinking",
            "o1-style",
            "r1-style",
        ];

        for tag in tags {
            let tag_lower = tag.to_lowercase();
            if thinking_tags.iter().any(|t| tag_lower.contains(t)) {
                return Some(true);
            }
        }
    }

    // No evidence
    None
}
```

**Three-Tier Detection Strategy:**
1. **Chat Template Patterns** (Highest Confidence): `<think>` tags in template = definitive
2. **Model Name Heuristics** (Medium Confidence): Name contains "R1", "QwQ", "reasoning"
3. **HF API Tags** (Low Confidence): Tags contain reasoning-related keywords

**Key Difference from Tool Calling:** Thinking detection uses model name as secondary signal because reasoning models follow strong naming conventions (DeepSeek-R1, QwQ, Llama-R1).

### Structured Output Capability (New)

**Current Implementation (Iteration 1):**
```rust
// Hardcoded false
tools.structured_output = false;
```

**Updated Implementation (Iteration 1.2):**
```rust
pub fn detect_structured_output(chat_template: Option<&str>) -> Option<bool> {
    if let Some(template) = chat_template {
        let patterns = [
            r"response_format",                          // API parameter
            r"json_mode",                                // JSON mode indicator
            r"json_object",                              // Type specification
            r"json_schema",                              // Schema definition
            r"<schema>",                                 // Schema wrapper tag
            r"</schema>",                                // Schema closing tag
            r"\"type\"\s*:\s*\"json_object\"",          // Explicit type
            r"\"strict\"\s*:\s*true",                    // Strict mode indicator
            r"constrained.*output",                      // Constrained generation
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(template) {
                    return Some(true);
                }
            }
        }

        // Template exists but no structured output patterns
        return Some(false);
    }

    // No chat template
    None
}
```

**Note:** Structured output is closely related to tool calling (many tool-calling models also support structured output). Detection focused on `response_format`, `json_schema`, and constrained generation patterns.

### Quantization Detection (Priority Flip)

**Original Implementation (Iteration 1):**
```rust
// Metadata key first, filename fallback
let quantization = metadata.get("general.quantization_version")
    .and_then(|v| v.as_str())
    .map(String::from)
    .or_else(|| parse_quantization_from_filename(filename));
```

**Updated Implementation (Iteration 1.2):**
```rust
// Filename parsing first, metadata fallback
pub fn detect_quantization(filename: &str, metadata: &GGUFMetadata) -> Option<String> {
    // 1. Parse from filename (highest reliability per research)
    if let Some(quant) = parse_quantization_from_filename(filename) {
        return Some(quant);
    }

    // 2. Fall back to metadata key
    metadata.get("general.quantization_version")
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn parse_quantization_from_filename(filename: &str) -> Option<String> {
    let filename_lower = filename.to_lowercase();

    // Common quantization patterns in GGUF filenames
    let patterns = [
        // Standard quantization formats
        (r"q2_k", "Q2_K"),
        (r"q3_k_s", "Q3_K_S"),
        (r"q3_k_m", "Q3_K_M"),
        (r"q3_k_l", "Q3_K_L"),
        (r"q4_0", "Q4_0"),
        (r"q4_1", "Q4_1"),
        (r"q4_k_s", "Q4_K_S"),
        (r"q4_k_m", "Q4_K_M"),
        (r"q5_0", "Q5_0"),
        (r"q5_1", "Q5_1"),
        (r"q5_k_s", "Q5_K_S"),
        (r"q5_k_m", "Q5_K_M"),
        (r"q6_k", "Q6_K"),
        (r"q8_0", "Q8_0"),
        (r"f16", "F16"),
        (r"f32", "F32"),
        (r"iq[0-9]_[a-z]+", "IQ"), // IQ quantizations
    ];

    for (pattern, quant_name) in &patterns {
        if filename_lower.contains(pattern) {
            return Some(quant_name.to_string());
        }
    }

    None
}
```

**Rationale (Research Phase 1 Finding):**
> "Quantization information is more reliably extracted from filenames than metadata keys. The `general.quantization_version` key is often missing or inconsistent, while filenames follow standardized naming conventions."

**Examples:**
- `Phi-3-mini-4k-instruct-q4_k_m.gguf` → `Q4_K_M` (from filename)
- `llama-3.2-1b-instruct-q8_0.gguf` → `Q8_0` (from filename)
- `model.gguf` with `general.quantization_version = "Q4_K_M"` → `Q4_K_M` (from metadata fallback)

---

## Database Schema Changes

### Migration 0007: Nullable Capabilities

**File:** `crates/services/migrations/0007_model_metadata.up.sql`

```sql
-- Migration 0007: Make capability columns nullable
-- NULL semantics: NULL = unknown/not determined, false = explicitly no, true = explicitly yes

ALTER TABLE model_metadata
  ALTER COLUMN capabilities_vision DROP NOT NULL,
  ALTER COLUMN capabilities_audio DROP NOT NULL,
  ALTER COLUMN capabilities_thinking DROP NOT NULL,
  ALTER COLUMN capabilities_function_calling DROP NOT NULL,
  ALTER COLUMN capabilities_structured_output DROP NOT NULL;

-- Remove default values (NULL is the new default for unknown capabilities)
ALTER TABLE model_metadata
  ALTER COLUMN capabilities_vision DROP DEFAULT,
  ALTER COLUMN capabilities_audio DROP DEFAULT,
  ALTER COLUMN capabilities_thinking DROP DEFAULT,
  ALTER COLUMN capabilities_function_calling DROP DEFAULT,
  ALTER COLUMN capabilities_structured_output DROP DEFAULT;
```

**File:** `crates/services/migrations/0007_model_metadata.down.sql`

```sql
-- Migration 0007 rollback: Restore NOT NULL constraints with default false

-- Convert NULL to false for rollback compatibility
UPDATE model_metadata
  SET capabilities_vision = 0 WHERE capabilities_vision IS NULL;
UPDATE model_metadata
  SET capabilities_audio = 0 WHERE capabilities_audio IS NULL;
UPDATE model_metadata
  SET capabilities_thinking = 0 WHERE capabilities_thinking IS NULL;
UPDATE model_metadata
  SET capabilities_function_calling = 0 WHERE capabilities_function_calling IS NULL;
UPDATE model_metadata
  SET capabilities_structured_output = 0 WHERE capabilities_structured_output IS NULL;

-- Restore NOT NULL constraints and defaults
ALTER TABLE model_metadata
  ALTER COLUMN capabilities_vision SET NOT NULL,
  ALTER COLUMN capabilities_vision SET DEFAULT 0,
  ALTER COLUMN capabilities_audio SET NOT NULL,
  ALTER COLUMN capabilities_audio SET DEFAULT 0,
  ALTER COLUMN capabilities_thinking SET NOT NULL,
  ALTER COLUMN capabilities_thinking SET DEFAULT 0,
  ALTER COLUMN capabilities_function_calling SET NOT NULL,
  ALTER COLUMN capabilities_function_calling SET DEFAULT 0,
  ALTER COLUMN capabilities_structured_output SET NOT NULL,
  ALTER COLUMN capabilities_structured_output SET DEFAULT 0;
```

### Null Semantics

**Three-Value Logic:**

| Value | Meaning | API Serialization | DB Storage |
|-------|---------|-------------------|------------|
| `NULL` | Unknown/not determined | Omit field | NULL |
| `false` | Explicitly does not have capability | `"vision": false` | 0 |
| `true` | Explicitly has capability | `"vision": true` | 1 |

**Detection Flow:**

```rust
pub struct ModelCapabilities {
    pub vision: Option<bool>,          // NULL = unknown
    pub audio: Option<bool>,           // NULL = unknown
    pub thinking: Option<bool>,        // NULL = unknown
    pub tools: ToolCapabilities,
}

pub struct ToolCapabilities {
    pub function_calling: Option<bool>, // NULL = unknown
    pub structured_output: Option<bool>, // NULL = unknown
}

// Serialization with null omission for API responses
#[derive(Serialize)]
pub struct ModelCapabilitiesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<bool>,

    pub tools: ToolCapabilitiesResponse,
}
```

**API Examples:**

```json
// Example 1: All capabilities detected
{
  "capabilities": {
    "vision": true,
    "audio": false,
    "thinking": false,
    "tools": {
      "function_calling": true,
      "structured_output": true
    }
  }
}

// Example 2: Some capabilities unknown (fields omitted)
{
  "capabilities": {
    "vision": false,
    "audio": false,
    "tools": {
      "function_calling": false
    }
  }
  // thinking, tools.structured_output omitted = unknown
}

// Example 3: No metadata extracted yet
// No "capabilities" field in response at all
```

**Query Examples:**

```sql
-- Find models with vision capability
SELECT * FROM model_metadata WHERE capabilities_vision = 1;

-- Find models where vision is unknown
SELECT * FROM model_metadata WHERE capabilities_vision IS NULL;

-- Find models that explicitly do NOT have vision
SELECT * FROM model_metadata WHERE capabilities_vision = 0;

-- Find models that might have vision (true or unknown)
SELECT * FROM model_metadata WHERE capabilities_vision IS NULL OR capabilities_vision = 1;

-- Find models with complete metadata (no NULL capabilities)
SELECT * FROM model_metadata
WHERE capabilities_vision IS NOT NULL
  AND capabilities_audio IS NOT NULL
  AND capabilities_thinking IS NOT NULL
  AND capabilities_function_calling IS NOT NULL
  AND capabilities_structured_output IS NOT NULL;
```

---

## HuggingFace API Integration

### HfApiService Trait

**File:** `crates/services/src/hf_api_service.rs`

```rust
use crate::objs::{Repo, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Service for querying HuggingFace Hub API
#[async_trait]
pub trait HfApiService: Send + Sync {
    /// Get model information from HuggingFace Hub
    async fn get_model_info(&self, repo: &Repo) -> Result<HfModelInfo>;
}

/// HuggingFace model information relevant for capability detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfModelInfo {
    /// Model repository ID (e.g., "microsoft/phi-3-mini-4k-instruct")
    pub id: String,

    /// Pipeline tag (e.g., "text-generation", "image-text-to-text")
    pub pipeline_tag: Option<String>,

    /// Model tags (user-provided, may include capability indicators)
    pub tags: Vec<String>,

    /// Model card metadata (from YAML frontmatter)
    pub card_data: Option<ModelCardData>,

    /// Model config (from config.json)
    pub config: Option<ModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardData {
    pub language: Option<Vec<String>>,
    pub license: Option<String>,
    pub datasets: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub pipeline_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub architectures: Option<Vec<String>>,
    pub model_type: Option<String>,
    pub chat_template: Option<serde_json::Value>, // String or object
}

/// HTTP-based HuggingFace API client
pub struct HttpHfApiService {
    client: reqwest::Client,
    base_url: String,
}

impl HttpHfApiService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://huggingface.co".to_string(),
        }
    }
}

#[async_trait]
impl HfApiService for HttpHfApiService {
    async fn get_model_info(&self, repo: &Repo) -> Result<HfModelInfo> {
        let url = format!(
            "{}/api/models/{}/{}?expand=cardData,config",
            self.base_url,
            repo.namespace(),
            repo.repo_name()
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", "BodhiApp/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(crate::error::Error::HfApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let info: HfModelInfo = response.json().await?;
        Ok(info)
    }
}
```

### When to Call HF API

**Decision Logic:**

```rust
pub async fn extract_metadata_with_hf_fallback(
    metadata: &GGUFMetadata,
    filename: &str,
    repo: &Repo,
    hf_service: Option<&dyn HfApiService>,
) -> ModelMetadata {
    // 1. Extract from GGUF
    let mut capabilities = extract_capabilities_from_gguf(metadata);
    let architecture = extract_architecture_from_gguf(metadata, filename);
    let context = extract_context_from_gguf(metadata);

    // 2. Check for NULL/unknown capabilities
    let needs_hf_fallback = capabilities.vision.is_none()
        || capabilities.audio.is_none()
        || capabilities.thinking.is_none()
        || capabilities.tools.function_calling.is_none()
        || capabilities.tools.structured_output.is_none();

    // 3. Only query HF API if needed and available
    if needs_hf_fallback {
        if let Some(hf) = hf_service {
            match hf.get_model_info(repo).await {
                Ok(hf_info) => {
                    // Fill gaps only (GGUF wins on conflicts)
                    capabilities = merge_capabilities(capabilities, &hf_info);
                }
                Err(e) => {
                    // Log error but don't fail - NULL capabilities acceptable
                    log::warn!("HF API fallback failed for {}: {}", repo, e);
                }
            }
        }
    }

    ModelMetadata {
        capabilities,
        architecture,
        context,
    }
}

fn merge_capabilities(
    gguf_caps: ModelCapabilities,
    hf_info: &HfModelInfo,
) -> ModelCapabilities {
    ModelCapabilities {
        // Only use HF if GGUF returned None
        vision: gguf_caps.vision.or_else(|| detect_vision_hf(hf_info)),
        audio: gguf_caps.audio.or_else(|| detect_audio_hf(hf_info)),
        thinking: gguf_caps.thinking.or_else(|| detect_thinking_hf(hf_info)),
        tools: ToolCapabilities {
            function_calling: gguf_caps.tools.function_calling
                .or_else(|| detect_tool_calling_hf(hf_info)),
            structured_output: gguf_caps.tools.structured_output
                .or_else(|| detect_structured_output_hf(hf_info)),
        },
    }
}
```

**Key Principle:** HF API is only called when:
1. GGUF detection returns `None` for at least one capability
2. HfApiService is available (may be None in tests or offline mode)
3. Repo information is available

### HF API Detection Rules

**Vision Detection:**
```rust
fn detect_vision_hf(hf_info: &HfModelInfo) -> Option<bool> {
    // 1. Pipeline tag (highest confidence)
    if let Some(tag) = &hf_info.pipeline_tag {
        let vision_tags = [
            "image-text-to-text",
            "image-to-text",
            "visual-question-answering",
            "document-question-answering",
            "video-text-to-text",
            "any-to-any",
        ];

        if vision_tags.contains(&tag.as_str()) {
            return Some(true);
        }
    }

    // 2. Model architecture (from config.json)
    if let Some(config) = &hf_info.config {
        if let Some(archs) = &config.architectures {
            for arch in archs {
                let arch_lower = arch.to_lowercase();
                if arch_lower.contains("llava")
                    || arch_lower.contains("vision")
                    || arch_lower.contains("qwen2vl")
                    || arch_lower.contains("cogvlm") {
                    return Some(true);
                }
            }
        }
    }

    // 3. Tags array (lower confidence)
    let vision_keywords = ["vision", "multimodal", "image-text-to-text", "llava", "clip"];
    for tag in &hf_info.tags {
        let tag_lower = tag.to_lowercase();
        if vision_keywords.iter().any(|k| tag_lower.contains(k)) {
            return Some(true);
        }
    }

    // No evidence found
    None
}
```

**Tool Calling Detection:**
```rust
fn detect_tool_calling_hf(hf_info: &HfModelInfo) -> Option<bool> {
    // 1. Tags (primary for tool calling)
    let tool_keywords = [
        "function calling",
        "function-calling",
        "tool-use",
        "tool calling",
        "json mode",
        "chatml",
    ];

    for tag in &hf_info.tags {
        let tag_lower = tag.to_lowercase();
        if tool_keywords.iter().any(|k| tag_lower.contains(k)) {
            return Some(true);
        }
    }

    // 2. Config chat template (if available)
    if let Some(config) = &hf_info.config {
        if let Some(template_value) = &config.chat_template {
            // Extract template string
            let template_str = match template_value {
                serde_json::Value::String(s) => Some(s.as_str()),
                serde_json::Value::Object(obj) => {
                    obj.get("template").and_then(|v| v.as_str())
                }
                _ => None,
            };

            if let Some(template) = template_str {
                return detect_tool_calling(Some(template));
            }
        }
    }

    None
}
```

**Thinking Detection:**
```rust
fn detect_thinking_hf(hf_info: &HfModelInfo) -> Option<bool> {
    // 1. Model name heuristics
    let name_lower = hf_info.id.to_lowercase();
    let reasoning_indicators = ["-r1", "qwq", "reasoning", "deepseek-r"];

    for indicator in &reasoning_indicators {
        if name_lower.contains(indicator) {
            return Some(true);
        }
    }

    // 2. Tags
    let thinking_keywords = [
        "reasoning",
        "chain-of-thought",
        "cot",
        "thinking",
        "o1-style",
        "r1-style",
    ];

    for tag in &hf_info.tags {
        let tag_lower = tag.to_lowercase();
        if thinking_keywords.iter().any(|k| tag_lower.contains(k)) {
            return Some(true);
        }
    }

    // 3. Config chat template
    if let Some(config) = &hf_info.config {
        if let Some(template_value) = &config.chat_template {
            let template_str = match template_value {
                serde_json::Value::String(s) => Some(s.as_str()),
                serde_json::Value::Object(obj) => {
                    obj.get("template").and_then(|v| v.as_str())
                }
                _ => None,
            };

            if let Some(template) = template_str {
                let (has_thinking, _) = detect_thinking(Some(template), Some(&hf_info.id), None);
                return has_thinking;
            }
        }
    }

    None
}
```

### Merge Logic Decision Matrix

| Capability | GGUF Result | HF Result | Final Result | Rationale |
|-----------|-------------|-----------|--------------|-----------|
| vision | true | true | true | Agreement |
| vision | true | false | **true** | GGUF wins (embedded metadata authoritative) |
| vision | true | None | true | HF inconclusive, use GGUF |
| vision | false | true | **false** | GGUF wins (explicit false) |
| vision | false | false | false | Agreement |
| vision | false | None | false | HF inconclusive, use GGUF |
| vision | None | true | true | HF fills gap |
| vision | None | false | false | HF fills gap |
| vision | None | None | **None** | Both inconclusive, store NULL |

**Implementation:**
```rust
fn merge_capability(gguf: Option<bool>, hf: Option<bool>) -> Option<bool> {
    // GGUF always wins when present
    gguf.or(hf)
}
```

**Simple Rule:** GGUF result is never overwritten. HF only fills `None` gaps via `or()` operator.

---

## GET Endpoint Completion

### Batch Query Implementation

**File:** `crates/services/src/db/service.rs`

```rust
use std::collections::HashMap;
use crate::objs::{AliasSource, ModelMetadata, Result};

impl DbService {
    /// Batch query metadata for multiple aliases
    pub async fn batch_get_metadata(
        &self,
        aliases: &[(AliasSource, String)],
    ) -> Result<HashMap<(AliasSource, String), ModelMetadata>> {
        if aliases.is_empty() {
            return Ok(HashMap::new());
        }

        // Build SQL query with IN clause for efficient batch lookup
        let placeholders = aliases.iter().enumerate()
            .map(|(i, _)| format!("(${},${})", i * 2 + 1, i * 2 + 2))
            .collect::<Vec<_>>()
            .join(",");

        let query_str = format!(
            r#"
            SELECT
                source,
                alias,
                capabilities_vision,
                capabilities_audio,
                capabilities_thinking,
                capabilities_function_calling,
                capabilities_structured_output,
                context_max_input_tokens,
                context_max_output_tokens,
                architecture
            FROM model_metadata
            WHERE (source, alias) IN ({})
            "#,
            placeholders
        );

        // Bind parameters
        let mut query = sqlx::query_as::<_, ModelMetadataRow>(&query_str);
        for (source, alias) in aliases {
            query = query.bind(source.to_string()).bind(alias);
        }

        // Execute query
        let rows = query.fetch_all(&self.pool).await?;

        // Convert to HashMap
        let mut result = HashMap::new();
        for row in rows {
            let key = (row.source.parse()?, row.alias.clone());
            let metadata = ModelMetadata::from_db_row(&row)?;
            result.insert(key, metadata);
        }

        Ok(result)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ModelMetadataRow {
    source: String,
    alias: String,
    capabilities_vision: Option<i64>,
    capabilities_audio: Option<i64>,
    capabilities_thinking: Option<i64>,
    capabilities_function_calling: Option<i64>,
    capabilities_structured_output: Option<i64>,
    context_max_input_tokens: Option<i64>,
    context_max_output_tokens: Option<i64>,
    architecture: Option<String>, // JSON
}
```

### List Aliases Endpoint Update

**File:** `crates/routes_app/src/routes_models.rs`

```rust
pub async fn list_aliases_handler(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<ListAliasesParams>,
) -> Result<Json<PaginatedAliasResponse>> {
    // 1. Get aliases (existing logic)
    let aliases = app_state.db_service
        .list_aliases(params.page, params.page_size, params.sort, params.sort_order)
        .await?;

    let total = app_state.db_service.count_aliases().await?;

    // 2. Batch query metadata for all aliases
    let alias_keys: Vec<_> = aliases.iter()
        .map(|a| (a.source(), a.alias().to_string()))
        .collect();

    let metadata_map = app_state.db_service
        .batch_get_metadata(&alias_keys)
        .await?;

    // 3. Attach metadata to each alias
    let enriched_data = aliases.into_iter()
        .map(|alias| {
            let key = (alias.source(), alias.alias().to_string());
            let metadata = metadata_map.get(&key).cloned();

            // Convert to response DTO with optional metadata
            match alias {
                AliasWithSource::User(user_alias) => {
                    AliasResponseWithSource::User(UserAliasResponse {
                        source: AliasSource::User,
                        alias: user_alias.alias,
                        repo: user_alias.repo,
                        filename: user_alias.filename,
                        snapshot: user_alias.snapshot,
                        request_params: Some(user_alias.request_params),
                        context_params: Some(user_alias.context_params),
                        metadata, // Optional
                    })
                }
                AliasWithSource::Model(model_alias) => {
                    AliasResponseWithSource::Model(ModelAliasResponse {
                        source: AliasSource::Model,
                        alias: model_alias.alias,
                        repo: model_alias.repo,
                        filename: model_alias.filename,
                        snapshot: model_alias.snapshot,
                        metadata, // Optional
                    })
                }
                AliasWithSource::Api(api_alias) => {
                    AliasResponseWithSource::Api(ApiAliasResponse {
                        source: AliasSource::Api,
                        id: api_alias.id,
                        api_format: api_alias.api_format,
                        metadata: None, // Iteration 2
                    })
                }
            }
        })
        .collect();

    Ok(Json(PaginatedAliasResponse {
        data: enriched_data,
        total,
        page: params.page,
        page_size: params.page_size,
    }))
}
```

**Key Changes:**
1. Batch query metadata after retrieving aliases (single DB query vs N+1)
2. Create lookup map by (source, alias) key
3. Attach metadata to response DTOs as `Option<ModelMetadata>`
4. Serialization automatically omits metadata field when None

**Performance:**
- Old: 1 query for aliases + N queries for metadata (N+1 problem)
- New: 1 query for aliases + 1 batch query for metadata (2 queries total)
- Improvement: O(N) → O(1) database round trips

---

## Test Fixture Specification

### Required GGUF Test Files

Generate 6 minimal GGUF files with specific metadata configurations for comprehensive testing:

| # | Filename | Architecture | Context | Vision | Audio | Tools | Thinking | Structured | Key Metadata |
|---|----------|--------------|---------|--------|-------|-------|----------|------------|--------------|
| 1 | `llama-plain.gguf` | llama | 4096 | false | false | false | false | false | Standard llama keys only, no multimodal |
| 2 | `qwen-vision.gguf` | qwen2vl | 8192 | true | false | false | false | false | `clip.has_vision_encoder=true`, `clip.vision.image_size=448` |
| 3 | `phi-tools.gguf` | phi3 | 4096 | false | false | true | false | true | `chat_template` with `{{ tools }}` and `json_schema` |
| 4 | `deepseek-thinking.gguf` | deepseek | 16384 | false | false | false | true | false | `chat_template` with `<think>` tags, model name suffix "R1" |
| 5 | `mistral-audio.gguf` | mistral | 8192 | false | true | false | false | false | `clip.has_audio_encoder=true`, `clip.audio.embedding_length=1024` |
| 6 | `llava-multimodal.gguf` | llava | 4096 | true | false | true | false | true | Vision + tools combined, multiple capability patterns |

### Python Generation Script

**File:** `crates/objs/tests/fixtures/generate_test_gguf.py`

```python
#!/usr/bin/env python3
"""
Generate minimal GGUF test fixtures for capability detection testing.
Requires: pip install gguf numpy
"""

import struct
import numpy as np
from pathlib import Path
from gguf import GGUFWriter, GGUFValueType

def create_minimal_tensor_data():
    """Create minimal valid tensor data"""
    return np.zeros((32, 32), dtype=np.float32)

def create_llama_plain():
    """Model 1: Plain llama model with no special capabilities"""
    writer = GGUFWriter("llama-plain.gguf", "llama")

    # Basic metadata
    writer.add_name("llama-test-plain")
    writer.add_architecture("llama")
    writer.add_file_type(1)  # Q4_0
    writer.add_context_length(4096)
    writer.add_embedding_length(256)
    writer.add_block_count(8)
    writer.add_head_count(8)
    writer.add_layer_norm_rms_eps(1e-5)

    # Tokenizer (minimal)
    writer.add_tokenizer_model("llama")
    writer.add_token_list(["<unk>", "<s>", "</s>"])
    writer.add_token_scores([0.0, 0.0, 0.0])
    writer.add_token_types([3, 1, 2])

    # Add minimal tensor
    writer.add_tensor("output.weight", create_minimal_tensor_data())

    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()
    print("✓ Generated llama-plain.gguf")

def create_qwen_vision():
    """Model 2: Qwen2VL with vision capabilities"""
    writer = GGUFWriter("qwen-vision.gguf", "qwen2vl")

    # Basic metadata
    writer.add_name("qwen2vl-test")
    writer.add_architecture("qwen2vl")
    writer.add_file_type(1)
    writer.add_context_length(8192)
    writer.add_embedding_length(256)
    writer.add_block_count(8)
    writer.add_head_count(8)

    # Vision encoder metadata
    writer.add_bool("clip.has_vision_encoder", True)
    writer.add_uint32("clip.vision.image_size", 448)
    writer.add_uint32("clip.vision.patch_size", 14)
    writer.add_uint32("clip.vision.embedding_length", 1024)
    writer.add_string("clip.projector_type", "resampler")

    # Tokenizer
    writer.add_tokenizer_model("qwen2")
    writer.add_token_list(["<|endoftext|>", "<|im_start|>", "<|im_end|>"])
    writer.add_token_scores([0.0, 0.0, 0.0])
    writer.add_token_types([3, 1, 2])

    writer.add_tensor("output.weight", create_minimal_tensor_data())

    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()
    print("✓ Generated qwen-vision.gguf")

def create_phi_tools():
    """Model 3: Phi3 with tool calling and structured output"""
    writer = GGUFWriter("phi-tools.gguf", "phi3")

    # Basic metadata
    writer.add_name("phi3-tools-test")
    writer.add_architecture("phi3")
    writer.add_file_type(1)
    writer.add_context_length(4096)
    writer.add_embedding_length(256)
    writer.add_block_count(8)
    writer.add_head_count(8)

    # Chat template with tool calling patterns
    chat_template = """
{%- for message in messages %}
    {%- if message['role'] == 'system' %}
        <|system|>{{ message['content'] }}<|end|>
    {%- elif message['role'] == 'user' %}
        <|user|>{{ message['content'] }}<|end|>
    {%- elif message['role'] == 'assistant' %}
        <|assistant|>{{ message['content'] }}<|end|>
    {%- elif message['role'] == 'tool' %}
        <|tool|>{{ message['content'] }}<|end|>
    {%- endif %}
{%- endfor %}
{%- if tools %}
<tools>
{%- for tool in tools %}
{{ tool | tojson }}
{%- endfor %}
</tools>
{%- endif %}
{%- if response_format and response_format.type == 'json_schema' %}
<schema>{{ response_format.json_schema | tojson }}</schema>
{%- endif %}
<|assistant|>
""".strip()

    writer.add_string("tokenizer.chat_template", chat_template)
    writer.add_tokenizer_model("gpt2")
    writer.add_token_list(["<|system|>", "<|user|>", "<|assistant|>", "<|end|>", "<|tool|>", "<tools>", "</tools>", "<schema>"])
    writer.add_token_scores([0.0] * 8)
    writer.add_token_types([1] * 8)

    writer.add_tensor("output.weight", create_minimal_tensor_data())

    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()
    print("✓ Generated phi-tools.gguf")

def create_deepseek_thinking():
    """Model 4: DeepSeek with thinking/reasoning capability"""
    writer = GGUFWriter("deepseek-thinking.gguf", "deepseek")

    # Basic metadata with R1 suffix in name
    writer.add_name("deepseek-test-R1")
    writer.add_architecture("deepseek")
    writer.add_file_type(1)
    writer.add_context_length(16384)
    writer.add_embedding_length(256)
    writer.add_block_count(8)
    writer.add_head_count(8)

    # Chat template with thinking tags
    chat_template = """
{%- for message in messages %}
    {%- if message['role'] == 'system' %}
        <|system|>{{ message['content'] }}<|end|>
    {%- elif message['role'] == 'user' %}
        <|user|>{{ message['content'] }}<|end|>
    {%- elif message['role'] == 'assistant' %}
        <|assistant|>
        {%- if add_generation_prompt and loop.last %}
            <think>
        {%- else %}
            {{ message['content'] }}
        {%- endif %}
        <|end|>
    {%- endif %}
{%- endfor %}
{%- if add_generation_prompt %}
<|assistant|><think>
{%- endif %}
""".strip()

    writer.add_string("tokenizer.chat_template", chat_template)
    writer.add_tokenizer_model("gpt2")
    writer.add_token_list(["<|system|>", "<|user|>", "<|assistant|>", "<|end|>", "<think>", "</think>"])
    writer.add_token_scores([0.0] * 6)
    writer.add_token_types([1] * 6)

    writer.add_tensor("output.weight", create_minimal_tensor_data())

    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()
    print("✓ Generated deepseek-thinking.gguf")

def create_mistral_audio():
    """Model 5: Mistral with audio input capability"""
    writer = GGUFWriter("mistral-audio.gguf", "mistral")

    # Basic metadata
    writer.add_name("mistral-audio-test")
    writer.add_architecture("mistral")
    writer.add_file_type(1)
    writer.add_context_length(8192)
    writer.add_embedding_length(256)
    writer.add_block_count(8)
    writer.add_head_count(8)

    # Audio encoder metadata
    writer.add_bool("clip.has_audio_encoder", True)
    writer.add_uint32("clip.audio.num_mel_bins", 80)
    writer.add_uint32("clip.audio.embedding_length", 1024)
    writer.add_uint32("clip.audio.block_count", 12)

    # Tokenizer
    writer.add_tokenizer_model("gpt2")
    writer.add_token_list(["<s>", "</s>", "<unk>"])
    writer.add_token_scores([0.0, 0.0, 0.0])
    writer.add_token_types([1, 2, 3])

    writer.add_tensor("output.weight", create_minimal_tensor_data())

    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()
    print("✓ Generated mistral-audio.gguf")

def create_llava_multimodal():
    """Model 6: LLaVA with vision + tool calling"""
    writer = GGUFWriter("llava-multimodal.gguf", "llava")

    # Basic metadata
    writer.add_name("llava-multimodal-test")
    writer.add_architecture("llava")
    writer.add_file_type(1)
    writer.add_context_length(4096)
    writer.add_embedding_length(256)
    writer.add_block_count(8)
    writer.add_head_count(8)

    # Vision metadata
    writer.add_bool("clip.has_vision_encoder", True)
    writer.add_bool("clip.has_llava_projector", True)
    writer.add_uint32("clip.vision.image_size", 336)
    writer.add_uint32("clip.vision.patch_size", 14)
    writer.add_string("clip.projector_type", "mlp")

    # Chat template with tools and structured output
    chat_template = """
{%- for message in messages %}
    USER: {{ message['content'] }}
    {%- if message['role'] == 'tool' %}
    TOOL_RESPONSE: {{ message['content'] }}
    {%- endif %}
{%- endfor %}
{%- if tools %}
AVAILABLE_TOOLS:
{{ tools | tojson }}
{%- endif %}
{%- if response_format %}
RESPONSE_FORMAT: {{ response_format | tojson }}
{%- endif %}
ASSISTANT:
""".strip()

    writer.add_string("tokenizer.chat_template", chat_template)
    writer.add_tokenizer_model("gpt2")
    writer.add_token_list(["USER:", "ASSISTANT:", "TOOL_RESPONSE:", "AVAILABLE_TOOLS:"])
    writer.add_token_scores([0.0] * 4)
    writer.add_token_types([1] * 4)

    writer.add_tensor("output.weight", create_minimal_tensor_data())

    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()
    print("✓ Generated llava-multimodal.gguf")

def main():
    print("Generating GGUF test fixtures...")
    print()

    create_llama_plain()
    create_qwen_vision()
    create_phi_tools()
    create_deepseek_thinking()
    create_mistral_audio()
    create_llava_multimodal()

    print()
    print("All test fixtures generated successfully!")
    print(f"Output directory: {Path.cwd()}")

if __name__ == "__main__":
    main()
```

**Usage:**
```bash
cd crates/objs/tests/fixtures/
python3 generate_test_gguf.py
```

**Generated Files:**
- `llama-plain.gguf` (~100KB)
- `qwen-vision.gguf` (~100KB)
- `phi-tools.gguf` (~100KB)
- `deepseek-thinking.gguf` (~100KB)
- `mistral-audio.gguf` (~100KB)
- `llava-multimodal.gguf` (~100KB)

---

## Implementation Checklist

**Status**: ✅ Core implementation complete (2026-01-13), GGUF-only detection (HF API deferred to Iteration 2)

### Phase 1: Core Capability Detection ✅ COMPLETED

**Commit**: `676bbd9d` - feat(objs): add GGUF model metadata extraction and capabilities detection

**File: `crates/objs/src/gguf/capabilities.rs`**
- [x] Expand `detect_vision()` with architecture patterns, projector types
- [x] Add `detect_audio()` with encoder flags and metadata keys
- [x] Add `detect_thinking()` with chat template regex, model name heuristics
- [x] Add `detect_tool_calling()` with comprehensive template patterns (17 patterns)
- [x] Add `detect_structured_output()` with response_format patterns (8 patterns)
- [x] Update `detect_quantization()` to prioritize filename parsing
- [x] Add regex patterns with `once_cell::sync::Lazy<Regex>` for static compilation

**File: `crates/objs/src/model_metadata.rs`**
- [x] Update `ModelCapabilities` to use `Option<bool>` for all fields
- [x] Update `ToolCapabilities` to use `Option<bool>` for all fields
- [x] Add serialization logic to skip None values in API responses (`#[serde(skip_serializing_if = "Option::is_none")]`)
- [x] Update `extract_metadata()` to use new detection functions

**File: `crates/objs/src/repo.rs` (additional)**
- [x] Add `namespace()` getter method
- [x] Add `repo_name()` getter method

### Phase 2: Database Schema ✅ COMPLETED

**Commit**: `df38c449` - feat(services): add model metadata storage and queue infrastructure

**File: `crates/services/migrations/0006_model_metadata.up.sql`**
- [x] Made capability columns nullable (removed `NOT NULL DEFAULT 0`)
- [x] Changed from BOOLEAN to INTEGER type for SQLite compatibility

**File: `crates/services/src/db/objs.rs`**
- [x] Update SQL queries to handle nullable capability columns
- [x] Update `ModelMetadataRow` struct with `Option<i64>` types
- [x] Update `from_db_row()` conversion logic

**File: `crates/services/src/db/service.rs`**
- [x] Implement `upsert_model_metadata()` for storing extracted metadata
- [x] Implement `get_metadata_by_file()` for single model lookup
- [x] Implement `list_model_metadata()` for listing all metadata
- [x] Add comprehensive test coverage with factory pattern

### Phase 3: HF API Integration ⏸️ DEFERRED TO ITERATION 2

**Scope Decision**: User explicitly chose "Deferred to iteration 2" for when to call HF API

**File: `crates/services/src/hf_api_service.rs`**
- [ ] ⏸️ Define `HfApiService` trait
- [ ] ⏸️ Define `HfModelInfo`, `ModelCardData`, `ModelConfig` structs
- [ ] ⏸️ Implement `HttpHfApiService` with reqwest client
- [ ] ⏸️ Add error handling for network failures, 404s, rate limits
- [ ] ⏸️ Add caching layer (optional, future enhancement)

**File: `crates/objs/src/gguf/capabilities.rs` (HF fallback)**
- [ ] ⏸️ Add `detect_vision_hf()` with pipeline_tag, architecture, tags
- [ ] ⏸️ Add `detect_audio_hf()` with pipeline_tag patterns
- [ ] ⏸️ Add `detect_tool_calling_hf()` with tags and config.chat_template
- [ ] ⏸️ Add `detect_thinking_hf()` with model name, tags
- [ ] ⏸️ Add `merge_capabilities()` with GGUF-wins logic

### Phase 4: Queue Service & API Endpoints ✅ COMPLETED

**Commits**: `df38c449`, `d95320a3`

**File: `crates/services/src/queue_service.rs`**
- [x] Implement `InMemoryQueue` with `QueueProducer`/`QueueConsumer` traits
- [x] Define `RefreshTask` enum (RefreshAll, RefreshSingle)
- [x] Implement `extract_and_store_metadata()` function
- [x] Implement `RefreshWorker` for background task processing
- [x] Add `bool_to_i64()` helper function for Option<bool> → Option<i64> conversion
- [x] Handle Option<bool> types from detection functions

**File: `crates/routes_app/src/routes_models_metadata.rs`**
- [x] POST /bodhi/v1/models/refresh - Queue metadata refresh for all models (202 Accepted)
- [x] POST /bodhi/v1/models/{id}/refresh - Sync metadata extraction for single model (200 OK)
- [x] GET /bodhi/v1/queue - Queue status endpoint (idle/processing)
- [x] Add `RefreshParams`, `RefreshResponse`, `QueueStatusResponse` DTOs

**File: `crates/routes_app/src/routes_models.rs`**
- [x] Add `metadata` field to `UserAliasResponse` and `ModelAliasResponse`
- [x] Attach metadata to existing model alias responses via DB query

**File: `crates/routes_app/src/openapi.rs`**
- [x] Add new endpoint schemas to OpenAPI spec
- [x] Regenerate TypeScript client types (`ts-client/`)

### Phase 5: Batch Query Optimization ⏸️ DEFERRED TO ITERATION 2

**Status**: Deferred - current implementation uses per-alias queries (acceptable for small model counts)

**File: `crates/services/src/db/service.rs`**
- [ ] ⏸️ Implement `batch_get_metadata()` with IN clause query
- [ ] ⏸️ Optimize for large model counts (>50)

### Phase 6: Test Fixtures ✅ COMPLETED

**Commit**: `676bbd9d`

**File: `crates/objs/tests/scripts/test_data_capabilities.py`**
- [x] Create Python script with gguf-py library
- [x] Implement 6 generator functions (llama-plain, qwen-vision, phi-tools, deepseek-thinking, mistral-audio, llava-multimodal)
- [x] Set specific metadata keys per specification
- [x] Generate minimal but valid GGUF files

**File: `crates/objs/tests/scripts/generate_e2e_test_gguf.py`**
- [x] Generate E2E test GGUF files with HuggingFace cache structure
- [x] Create symlink structure matching real HF cache layout
- [x] Output to `crates/lib_bodhiserver_napi/tests-js/data/test-gguf/`

**File: `crates/objs/src/gguf/capabilities.rs` (test module)**
- [x] Add unit tests for all detection functions
- [x] Test vision detection with qwen-vision.gguf, llava-multimodal.gguf
- [x] Test audio detection with mistral-audio.gguf
- [x] Test tool calling detection with phi-tools.gguf, llava-multimodal.gguf
- [x] Test thinking detection with deepseek-thinking.gguf
- [x] Test structured output detection patterns
- [x] Test plain model returns Some(false) for all capabilities (llama-plain.gguf)
- [x] Add pattern-based unit tests for regex validation

**Test Results**: All backend tests passing

### Phase 7: Frontend Implementation ✅ COMPLETED (Uncommitted)

**Files (uncommitted changes in working tree)**:

**File: `crates/bodhi/src/app/ui/models/components/ModelPreviewModal.tsx`**
- [x] Create modal component for displaying model metadata
- [x] Show basic info: alias, repo, filename, snapshot, source
- [x] Show capabilities with Supported/Not supported badges
- [x] Show context limits (max input/output tokens)
- [x] Show architecture info (family, parameter count, quantization)
- [x] Handle API aliases vs local model aliases (type guards)

**File: `crates/bodhi/src/hooks/useModelMetadata.ts`**
- [x] `useRefreshAllMetadata()` - POST /bodhi/v1/models/refresh
- [x] `useRefreshSingleMetadata()` - POST /bodhi/v1/models/{id}/refresh
- [x] `useQueueStatus()` - GET /bodhi/v1/queue
- [x] Import types from `@bodhiapp/ts-client`
- [x] URL encoding for aliases with special characters

**File: `crates/bodhi/src/app/ui/models/page.tsx`**
- [x] Add "Refresh All" button with disabled state during processing
- [x] Add per-row refresh button for individual models
- [x] Add preview button to open ModelPreviewModal
- [x] Implement queue polling with cleanup on unmount
- [x] Query invalidation after refresh completion
- [x] Toast notifications for success/error states

**File: `crates/bodhi/src/lib/utils.ts`**
- [x] Add `hasModelMetadata()` type guard for safe property access

**File: `crates/bodhi/src/test-utils/msw-v2/handlers/models.ts`**
- [x] `mockRefreshAllMetadata()` - Mock bulk refresh endpoint
- [x] `mockQueueStatus()` - Mock queue status endpoint
- [x] `mockRefreshSingleMetadata()` - Mock single model refresh

**File: `crates/bodhi/src/app/ui/models/page.test.tsx`**
- [x] Unit tests for refresh button interactions
- [x] Unit tests for preview modal open/close

### Phase 8: E2E Testing ✅ COMPLETED (Uncommitted)

**Files (uncommitted changes in working tree)**:

**File: `crates/lib_bodhiserver_napi/tests-js/specs/models/model-metadata.spec.mjs`**
- [x] Single journey test pattern (login → refresh → verify all models)
- [x] Test "Refresh All" button disabled state during processing
- [x] Test per-row refresh with toast notifications
- [x] Verify capabilities for all 6 test models:
  - llama-plain: no capabilities
  - qwen-vision: vision=true
  - phi-tools: function_calling=true
  - deepseek-thinking: thinking=true
  - mistral-audio: audio=true
  - llava-multimodal: vision=true, function_calling=true
- [x] Test expectations corrected for TOOLS_TEMPLATE (structured_output=false)

**File: `crates/lib_bodhiserver_napi/tests-js/pages/ModelsListPage.mjs`**
- [x] Add `clickRefreshAll()`, `clickRefreshButton()` methods
- [x] Add `clickPreviewButton()`, `closePreviewModal()` methods
- [x] Add `waitForQueueIdle()` with toast verification
- [x] Add `verifyPreviewBasicInfo()`, `verifyPreviewCapability()` methods

**Test Results**: 7/7 E2E tests passing (1 flow test with 9 steps)

### Phase 9: Documentation 🔄 IN PROGRESS

**File: `ai-docs/specs/20260112-model-metadata/03-iteration1-2-local-model-design.md`**
- [x] Update specification status to IMPLEMENTATION COMPLETE
- [x] Document commit history and implementation phases
- [x] Update implementation checklist with actual progress
- [ ] Add examples of API responses with nullable semantics

**File: `crates/objs/src/gguf/capabilities.rs`**
- [ ] Add comprehensive doc comments for all detection functions
- [ ] Document return value semantics (Some(true), Some(false), None)
- [ ] Document security considerations (NEVER execute templates)

---

## Testing Strategy

### Unit Tests (Real Fixtures)

**File: `crates/objs/tests/gguf_capabilities_test.rs`**

```rust
use objs::gguf::{GGUFMetadata, capabilities};
use std::path::Path;

#[test]
fn test_llama_plain_no_capabilities() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/llama-plain.gguf").unwrap();

    assert_eq!(capabilities::detect_vision(&metadata), Some(false));
    assert_eq!(capabilities::detect_audio(&metadata), Some(false));
    assert_eq!(capabilities::detect_thinking(
        metadata.get_chat_template(),
        Some("llama-test-plain"),
        None
    ), Some(false));
    assert_eq!(capabilities::detect_tool_calling(
        metadata.get_chat_template()
    ), Some(false)); // Template exists but no patterns
}

#[test]
fn test_qwen_vision_capability() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/qwen-vision.gguf").unwrap();

    // Architecture-based detection
    assert_eq!(metadata.get_architecture(), Some("qwen2vl"));
    assert_eq!(capabilities::detect_vision(&metadata), Some(true));

    // Vision metadata present
    assert_eq!(metadata.get_u32("clip.vision.image_size"), Some(448));
    assert_eq!(metadata.get_bool("clip.has_vision_encoder"), Some(true));
}

#[test]
fn test_phi_tool_calling_detection() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/phi-tools.gguf").unwrap();
    let chat_template = metadata.get_chat_template();

    // Tool calling patterns
    assert_eq!(capabilities::detect_tool_calling(chat_template), Some(true));
    assert!(chat_template.unwrap().contains("{{ tools }}"));
    assert!(chat_template.unwrap().contains("<tools>"));

    // Structured output patterns
    assert_eq!(capabilities::detect_structured_output(chat_template), Some(true));
    assert!(chat_template.unwrap().contains("response_format"));
    assert!(chat_template.unwrap().contains("json_schema"));
}

#[test]
fn test_deepseek_thinking_detection() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/deepseek-thinking.gguf").unwrap();
    let chat_template = metadata.get_chat_template();
    let model_name = metadata.get_name(); // Should be "deepseek-test-R1"

    // Template pattern
    assert_eq!(capabilities::detect_thinking(chat_template, model_name, None), Some(true));
    assert!(chat_template.unwrap().contains("<think>"));

    // Model name heuristic
    assert!(model_name.unwrap().contains("R1"));
}

#[test]
fn test_mistral_audio_detection() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/mistral-audio.gguf").unwrap();

    assert_eq!(capabilities::detect_audio(&metadata), Some(true));
    assert_eq!(metadata.get_bool("clip.has_audio_encoder"), Some(true));
    assert_eq!(metadata.get_u32("clip.audio.num_mel_bins"), Some(80));
}

#[test]
fn test_llava_multimodal_detection() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/llava-multimodal.gguf").unwrap();
    let chat_template = metadata.get_chat_template();

    // Vision
    assert_eq!(capabilities::detect_vision(&metadata), Some(true));
    assert_eq!(metadata.get_bool("clip.has_vision_encoder"), Some(true));

    // Tool calling
    assert_eq!(capabilities::detect_tool_calling(chat_template), Some(true));
    assert!(chat_template.unwrap().contains("AVAILABLE_TOOLS"));

    // Structured output
    assert_eq!(capabilities::detect_structured_output(chat_template), Some(true));
    assert!(chat_template.unwrap().contains("RESPONSE_FORMAT"));
}

#[test]
fn test_quantization_filename_priority() {
    let metadata = GGUFMetadata::from_file("tests/fixtures/phi-tools.gguf").unwrap();

    // Filename: phi-tools.gguf (no quantization in name)
    // Metadata: general.quantization_version = "Q4_0"
    let quant = capabilities::detect_quantization("phi-tools.gguf", &metadata);
    assert_eq!(quant, Some("Q4_0".to_string())); // From metadata fallback

    // Filename with quantization
    let quant2 = capabilities::detect_quantization("model-q5_k_m.gguf", &metadata);
    assert_eq!(quant2, Some("Q5_K_M".to_string())); // From filename wins
}
```

### Integration Tests

**File: `crates/integration-tests/tests/metadata_hf_fallback.rs`**

```rust
use services::{HfApiService, HfModelInfo};
use objs::{Repo, ModelMetadata};
use std::sync::Arc;
use async_trait::async_trait;

// Mock HF API service for testing
struct MockHfApiService {
    responses: HashMap<String, HfModelInfo>,
}

#[async_trait]
impl HfApiService for MockHfApiService {
    async fn get_model_info(&self, repo: &Repo) -> Result<HfModelInfo> {
        self.responses.get(&repo.to_string())
            .cloned()
            .ok_or_else(|| Error::NotFound("Mock repo not found".to_string()))
    }
}

#[tokio::test]
async fn test_hf_fallback_fills_gaps() {
    // GGUF metadata with NULL thinking capability
    let gguf_metadata = GGUFMetadata::from_file("tests/fixtures/llama-plain.gguf").unwrap();

    // Mock HF API response with thinking tag
    let mut mock_responses = HashMap::new();
    mock_responses.insert(
        "test/llama-plain".to_string(),
        HfModelInfo {
            id: "test/llama-plain".to_string(),
            pipeline_tag: Some("text-generation".to_string()),
            tags: vec!["reasoning".to_string()],
            card_data: None,
            config: None,
        },
    );

    let hf_service = Arc::new(MockHfApiService { responses: mock_responses });

    // Extract with fallback
    let repo = Repo::try_from("test/llama-plain").unwrap();
    let metadata = extract_metadata_with_hf_fallback(
        &gguf_metadata,
        "llama-plain.gguf",
        &repo,
        Some(hf_service.as_ref()),
    ).await.unwrap();

    // GGUF detection returns Some(false) for thinking
    // HF API should NOT override (GGUF wins)
    assert_eq!(metadata.capabilities.thinking, Some(false));
}

#[tokio::test]
async fn test_gguf_wins_on_conflicts() {
    // Simulate conflict: GGUF says true, HF says false
    // Expected: GGUF wins

    // This test validates the merge_capabilities() logic
}

#[tokio::test]
async fn test_hf_offline_graceful_degradation() {
    // HF API unavailable (offline mode)
    // Expected: NULL capabilities remain NULL, no error thrown
}
```

### API Endpoint Tests

**File: `crates/integration-tests/tests/routes_models_metadata.rs`**

```rust
#[tokio::test]
async fn test_list_models_with_metadata() {
    let app = test_app().await;

    // Refresh metadata for test model
    let response = app.post("/bodhi/v1/models/refresh")
        .header("Authorization", "Bearer admin-token")
        .send()
        .await;
    assert_eq!(response.status(), 202);

    // Wait for background processing
    tokio::time::sleep(Duration::from_secs(2)).await;

    // List models
    let response = app.get("/bodhi/v1/models").send().await;
    assert_eq!(response.status(), 200);

    let body: PaginatedAliasResponse = response.json().await;

    // Find model with metadata
    let model_with_metadata = body.data.iter()
        .find(|alias| alias.metadata().is_some())
        .expect("At least one model should have metadata");

    let metadata = model_with_metadata.metadata().unwrap();

    // Verify nullable fields
    if metadata.capabilities.vision.is_some() {
        // Vision field present (true or false)
        assert!(metadata.capabilities.vision == Some(true)
            || metadata.capabilities.vision == Some(false));
    }
}

#[tokio::test]
async fn test_batch_query_performance() {
    let app = test_app().await;

    // Create 50 test models
    for i in 0..50 {
        app.create_test_alias(&format!("test-model-{}", i)).await;
    }

    // Measure query time
    let start = Instant::now();
    let response = app.get("/bodhi/v1/models?page_size=50").send().await;
    let duration = start.elapsed();

    assert_eq!(response.status(), 200);

    // Should complete in <100ms (batch query vs N+1)
    assert!(duration < Duration::from_millis(100));
}
```

---

## Cross-References to Research Report

### GGUF Metadata (Section 1)
- **Architecture Enum:** 119+ architectures → `detect_vision()` architecture patterns
- **CLIP Metadata Keys:** Vision/audio encoder flags → `detect_vision()`, `detect_audio()`
- **Projector Types:** 10 projector types → `clip.projector_type` check in vision detection
- **Context Length Pattern:** `{arch}.context_length` → existing implementation unchanged

### HuggingFace API (Section 2)
- **Pipeline Tag Taxonomy:** 57 tags → `detect_vision_hf()`, `detect_audio_hf()`
- **Model Card Tags:** Capability keywords → `detect_tool_calling_hf()`, `detect_thinking_hf()`
- **Chat Template Indicators:** Special tokens → HF fallback for template extraction

### Chat Template Analysis (Section 3)
- **Tool Calling Patterns:** 15+ template patterns → `detect_tool_calling()` regex list
- **Thinking Patterns:** `<think>` tags → `detect_thinking()` primary detection
- **Structured Output Patterns:** `response_format`, `json_schema` → `detect_structured_output()`
- **Security Considerations:** GGUF-SSTI vulnerability → string-only regex, NEVER execute templates

### Industry Comparison (Section 4)
- **Detection Strategy:** Metadata introspection first → GGUF-first priority shift
- **Ollama:** Try-and-fail approach → NOT adopted (slow, poor UX)
- **vLLM:** GGUF metadata extraction → Validated our approach
- **LM Studio:** Visual indicators → Future UI enhancement

### Recommended Strategy (Section 5.1-5.2)
- **Layered Detection:** GGUF → HF → Template → Manual → Implemented in this spec
- **GGUF-first Rationale:** Offline capability, authoritative templates → Priority shift justification
- **Confidence Scoring:** Multi-source confidence → Deferred to future enhancement (nullable semantics sufficient for iteration 1.2)

---

## Migration Path from Iteration 1

### Code Changes Required

1. **Migration 0007:** Run database migration to make capability columns nullable
2. **capabilities.rs:** Expand vision, add thinking/tools/structured detection functions
3. **model_metadata.rs:** Update types to `Option<bool>`
4. **hf_api_service.rs:** New file with trait and HTTP implementation
5. **queue_service.rs:** Add HF fallback logic in worker
6. **db/service.rs:** Add `batch_get_metadata()` method
7. **routes_models.rs:** Update list endpoint to use batch query

### Data Migration

**No data loss:** Existing metadata remains valid. Migration 0007 preserves all data:
- Existing `1` values remain `1` (true)
- Existing `0` values remain `0` (false)
- New records may have `NULL` (unknown)

**Gradual enrichment:** Re-run refresh jobs to populate new capabilities (thinking, tools, structured_output).

### Testing Migration

1. Generate GGUF test fixtures: `python3 crates/objs/tests/fixtures/generate_test_gguf.py`
2. Run unit tests: `cargo test -p objs gguf_capabilities`
3. Run integration tests: `cargo test -p integration-tests metadata`
4. Run full test suite: `make test`

---

## Future Enhancements (Not in Iteration 1.2)

### Confidence Scoring (Deferred)
- Multi-source confidence aggregation
- User feedback for correction
- Model-specific confidence overrides

### HF API Caching (Deferred)
- Local cache of HF API responses
- TTL-based invalidation
- Offline mode with stale cache

### Automatic Refresh (Deferred)
- Periodic background refresh for all models
- Snapshot change detection with automatic re-extraction
- WebSocket/SSE progress notifications

### Advanced Template Analysis (Deferred)
- Parse Jinja2 AST in sandboxed environment
- Extract variable references programmatically
- Detect malicious patterns with security scoring

---

## References

**Research Report:** `ai-docs/specs/20260112-model-metadata/research-report.md`

**Original Design:** `ai-docs/specs/20260112-model-metadata/02-iteration1-design.md`

**Research Phases:**
- `ai-docs/specs/20260112-model-metadata/research-phase-1-gguf-spec.md`
- `ai-docs/specs/20260112-model-metadata/research-phase-2-hf-api.md`
- `ai-docs/specs/20260112-model-metadata/research-phase-3-chat-templates.md`
- `ai-docs/specs/20260112-model-metadata/research-phase-4-industry.md`

**External References:**
- [llama.cpp GGUF constants](https://github.com/ggml-org/llama.cpp/blob/master/gguf-py/gguf/constants.py)
- [HuggingFace API documentation](https://huggingface.co/docs/hub/en/api)
- [GGUF-SSTI Security Research](https://research.jfrog.com/model-threats/gguf-ssti/)

---

## Pending Items for Future Iterations

### Iteration 2 (Deferred Items)

1. **HuggingFace API Integration** (Phase 3)
   - Define `HfApiService` trait and `HttpHfApiService` implementation
   - Implement HF API detection functions (`detect_vision_hf`, `detect_tool_calling_hf`, etc.)
   - Add `merge_capabilities()` with GGUF-wins conflict resolution
   - Handle network failures, 404s, rate limits gracefully

2. **Batch Query Optimization** (Phase 5)
   - Implement `batch_get_metadata()` with SQL IN clause
   - Optimize GET /bodhi/v1/models for large model counts (>50)

3. **Documentation** (Phase 9)
   - Add comprehensive doc comments to `capabilities.rs` functions
   - Document return value semantics (Some(true), Some(false), None)
   - Document security considerations (NEVER execute Jinja2 templates)
   - Add API response examples with nullable capability fields

### Future Enhancements (Not Scoped)

- **Confidence Scoring**: Multi-source confidence aggregation with user feedback
- **HF API Caching**: Local cache with TTL-based invalidation
- **Automatic Refresh**: Periodic background refresh with snapshot change detection
- **Advanced Template Analysis**: Sandboxed Jinja2 AST parsing

---

**Document Version:** 1.1
**Last Updated:** 2026-01-13
**Status:** ✅ Implementation Complete (core features), ⏸️ Some items deferred to Iteration 2
