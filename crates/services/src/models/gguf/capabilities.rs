use crate::models::gguf::GGUFMetadata;
use crate::models::{
  ContextLimits, ModelArchitecture, ModelCapabilities, ModelMetadata, ToolCapabilities,
};
use once_cell::sync::Lazy;
use regex::Regex;

// =============================================================================
// Constants
// =============================================================================

/// Vision-capable architectures (research-report.md Section 1)
const VISION_ARCHITECTURES: &[&str] = &[
  "qwen2vl",
  "qwen3vl",
  "qwen3vlmoe",
  "chameleon",
  "cogvlm",
  "minicpm",
  "minicpm3",
  "llava",
];

/// Audio-capable architectures (research-report.md Section 1)
const AUDIO_ARCHITECTURES: &[&str] = &["lfm2", "lfm2moe"];

/// Thinking/reasoning model name indicators (research-report.md Section 3)
const THINKING_NAME_INDICATORS: &[&str] = &["-r1", "qwq", "reasoning", "deepseek-r"];

/// Quantization patterns for filename parsing (research-report.md Phase 1)
/// Ordered by specificity (longer patterns first to avoid false matches)
const QUANTIZATION_PATTERNS: &[(&str, &str)] = &[
  ("q5_k_m", "Q5_K_M"),
  ("q5_k_s", "Q5_K_S"),
  ("q4_k_m", "Q4_K_M"),
  ("q4_k_s", "Q4_K_S"),
  ("q3_k_l", "Q3_K_L"),
  ("q3_k_m", "Q3_K_M"),
  ("q3_k_s", "Q3_K_S"),
  ("q8_0", "Q8_0"),
  ("q6_k", "Q6_K"),
  ("q5_1", "Q5_1"),
  ("q5_0", "Q5_0"),
  ("q4_1", "Q4_1"),
  ("q4_0", "Q4_0"),
  ("q2_k", "Q2_K"),
  ("f16", "F16"),
  ("f32", "F32"),
];

// =============================================================================
// Regex Patterns (Compiled Once)
// =============================================================================

/// Tool calling patterns from research-report.md Section 3
static TOOL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
  vec![
    Regex::new(r"\{\{\s*tools\s*\}\}").unwrap(), // {{ tools }}
    Regex::new(r"\{%\s*if\s+tools\s").unwrap(),  // {% if tools
    Regex::new(r"\{%-?\s*if\s+tools\s*-?%\}").unwrap(), // {%- if tools -%}
    Regex::new(r"\{%\s*for\s+tool\s+in\s+tools").unwrap(), // {% for tool in tools
    Regex::new(r"'tool_calls'\s+in\s+message").unwrap(), // 'tool_calls' in message
    Regex::new(r#"message\[["']role["']\]\s*==\s*["']tool["']"#).unwrap(), // message['role'] == 'tool'
    Regex::new(r"<tool_call>").unwrap(),                                   // Hermes-3
    Regex::new(r"<tool_response>").unwrap(),                               // Hermes-3 response
    Regex::new(r"\[TOOL_CALLS\]").unwrap(),                                // Mistral
    Regex::new(r"\[AVAILABLE_TOOLS\]").unwrap(),                           // Mistral tools
    Regex::new(r"<tools>").unwrap(),                                       // Generic wrapper
    Regex::new(r"\{\{\s*\.Tools\s*\}\}").unwrap(),                         // Ollama {{ .Tools }}
    Regex::new(r"\{\{\s*if\s+\.Tools\s*\}\}").unwrap(),                    // Ollama {{ if .Tools }}
    Regex::new(r"\{\{\s*range\s+\.Tools\s*\}\}").unwrap(), // Ollama {{ range .Tools }}
    Regex::new(r"builtin_tools").unwrap(),                 // Llama 3.1
    Regex::new(r"tool_name").unwrap(),                     // Command-R
    Regex::new(r"json_to_python_type").unwrap(),           // Hermes-3 macro
  ]
});

/// Thinking/reasoning patterns from research-report.md Section 3
static THINKING_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
  vec![
    Regex::new(r"<think>").unwrap(),
    Regex::new(r"</think>").unwrap(),
    Regex::new(r"<reasoning>").unwrap(),
    Regex::new(r"</reasoning>").unwrap(),
    Regex::new(r"reasoning_content").unwrap(),
    Regex::new(r"<\|im_start\|>assistant\\n<think>\\n").unwrap(), // QwQ specific
  ]
});

/// Structured output patterns from research-report.md Section 3
static STRUCTURED_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
  vec![
    Regex::new(r"response_format").unwrap(),
    Regex::new(r"json_mode").unwrap(),
    Regex::new(r"json_object").unwrap(),
    Regex::new(r"json_schema").unwrap(),
    Regex::new(r"<schema>").unwrap(),
    Regex::new(r"</schema>").unwrap(),
    Regex::new(r#""type"\s*:\s*"json_object""#).unwrap(),
    Regex::new(r#""strict"\s*:\s*true"#).unwrap(),
  ]
});

// =============================================================================
// Public API
// =============================================================================

/// Extract complete model metadata from GGUF file
pub fn extract_metadata(metadata: &GGUFMetadata, filename: &str) -> ModelMetadata {
  ModelMetadata {
    capabilities: extract_capabilities(metadata),
    context: extract_context(metadata),
    architecture: extract_architecture(metadata, filename),
    chat_template: get_chat_template(metadata),
  }
}

/// Extract model capabilities from GGUF metadata
pub fn extract_capabilities(metadata: &GGUFMetadata) -> ModelCapabilities {
  let chat_template = get_chat_template(metadata);
  let model_name = get_model_name(metadata);

  ModelCapabilities {
    vision: detect_vision(metadata),
    audio: detect_audio(metadata),
    thinking: detect_thinking(chat_template.as_deref(), model_name.as_deref()),
    tools: ToolCapabilities {
      function_calling: detect_tool_calling(chat_template.as_deref()),
      structured_output: detect_structured_output(chat_template.as_deref()),
    },
  }
}

// =============================================================================
// Capability Detection (Research-Driven)
// =============================================================================

/// Detect vision support from GGUF metadata
///
/// Vision support indicators (research-report.md Section 1):
/// 1. Architecture-specific: qwen2vl, qwen3vl, chameleon, cogvlm, minicpm, llava
/// 2. CLIP architecture with vision encoder flag
/// 3. Projector type presence (clip.projector_type)
/// 4. Vision-specific keys (clip.vision.image_size, patch_size, embedding_length)
///
/// Returns:
/// - Some(true): Definite vision capability
/// - Some(false): Definite no vision capability
/// - None: Unknown (parse error only, defaults to Some(false))
pub fn detect_vision(metadata: &GGUFMetadata) -> Option<bool> {
  // 1. Architecture-specific vision models
  if let Some(arch) = metadata
    .get("general.architecture")
    .and_then(|v| v.as_str().ok())
  {
    if VISION_ARCHITECTURES.contains(&arch) {
      return Some(true);
    }
  }

  // 2. CLIP architecture with vision encoder
  if metadata
    .get("general.architecture")
    .and_then(|v| v.as_str().ok())
    == Some("clip")
  {
    if let Some(true) = metadata
      .get("clip.has_vision_encoder")
      .and_then(|v| v.as_bool().ok())
    {
      return Some(true);
    }
  }

  // 3. Projector type presence
  if metadata.contains_key("clip.projector_type")
    || metadata.contains_key("clip.vision.projector_type")
  {
    return Some(true);
  }

  // 4. Vision-specific metadata keys
  if metadata.contains_key("clip.vision.image_size")
    || metadata.contains_key("clip.vision.patch_size")
    || metadata.contains_key("clip.vision.embedding_length")
  {
    return Some(true);
  }

  // 5. Explicit false indicator
  if let Some(false) = metadata
    .get("clip.has_vision_encoder")
    .and_then(|v| v.as_bool().ok())
  {
    return Some(false);
  }

  // 6. No CLIP keys = no vision capability
  Some(false)
}

/// Detect audio support from GGUF metadata
///
/// Audio support indicators (research-report.md Section 1):
/// 1. Audio encoder flag (clip.has_audio_encoder)
/// 2. Audio-specific keys (clip.audio.num_mel_bins, embedding_length, block_count)
/// 3. Audio architecture hints (lfm2, lfm2moe)
pub fn detect_audio(metadata: &GGUFMetadata) -> Option<bool> {
  // 1. Audio encoder flag
  if let Some(true) = metadata
    .get("clip.has_audio_encoder")
    .and_then(|v| v.as_bool().ok())
  {
    return Some(true);
  }

  // 2. Audio-specific metadata keys
  if metadata.contains_key("clip.audio.num_mel_bins")
    || metadata.contains_key("clip.audio.embedding_length")
    || metadata.contains_key("clip.audio.block_count")
  {
    return Some(true);
  }

  // 3. Audio architecture hints
  if let Some(arch) = metadata
    .get("general.architecture")
    .and_then(|v| v.as_str().ok())
  {
    if AUDIO_ARCHITECTURES.contains(&arch) {
      return Some(true);
    }
  }

  Some(false)
}

/// Detect tool calling support from chat template
///
/// Detects patterns indicating tool/function calling support (research-report.md Section 3).
/// Uses comprehensive regex patterns covering Jinja2, Ollama Go templates, and model-specific tags.
///
/// Returns:
/// - Some(true): Tool calling patterns detected
/// - Some(false): Template exists but no patterns OR no template
pub fn detect_tool_calling(chat_template: Option<&str>) -> Option<bool> {
  let Some(template) = chat_template else {
    return Some(false); // No template = no tool support
  };

  for pattern in TOOL_PATTERNS.iter() {
    if pattern.is_match(template) {
      return Some(true);
    }
  }

  Some(false)
}

/// Detect thinking/reasoning support from chat template and model name
///
/// Three-tier detection strategy (research-report.md Section 3):
/// 1. Chat template patterns (highest confidence): <think> tags
/// 2. Model name heuristics (medium confidence): R1, QwQ, reasoning
///
/// Returns:
/// - Some(true): Thinking capability detected
/// - Some(false): No thinking capability detected
pub fn detect_thinking(chat_template: Option<&str>, model_name: Option<&str>) -> Option<bool> {
  // 1. Chat template patterns (highest priority)
  if let Some(template) = chat_template {
    for pattern in THINKING_PATTERNS.iter() {
      if pattern.is_match(template) {
        return Some(true);
      }
    }
  }

  // 2. Model name heuristics
  if let Some(name) = model_name {
    let name_lower = name.to_lowercase();

    for indicator in THINKING_NAME_INDICATORS {
      if name_lower.contains(indicator) {
        return Some(true);
      }
    }
  }

  Some(false)
}

/// Detect structured output support from chat template
///
/// Detects patterns indicating structured output/JSON schema support (research-report.md Section 3).
///
/// Returns:
/// - Some(true): Structured output patterns detected
/// - Some(false): Template exists but no patterns OR no template
pub fn detect_structured_output(chat_template: Option<&str>) -> Option<bool> {
  let Some(template) = chat_template else {
    return Some(false);
  };

  for pattern in STRUCTURED_PATTERNS.iter() {
    if pattern.is_match(template) {
      return Some(true);
    }
  }

  Some(false)
}

// =============================================================================
// Context Extraction
// =============================================================================

/// Extract context limits from GGUF metadata
///
/// Context length is stored in architecture-specific keys:
/// - {arch}.context_length (e.g., "llama.context_length")
pub fn extract_context(metadata: &GGUFMetadata) -> ContextLimits {
  let max_input_tokens = get_context_length(metadata);

  ContextLimits {
    max_input_tokens,
    max_output_tokens: None, // GGUF doesn't specify max output tokens
  }
}

/// Get context length from architecture-specific key
fn get_context_length(metadata: &GGUFMetadata) -> Option<u64> {
  // Get architecture name
  let arch = metadata
    .get("general.architecture")
    .and_then(|v| v.as_str().ok())?;

  // Build architecture-specific key
  let key = format!("{}.context_length", arch);

  // Try to get value as different integer types
  if let Some(value) = metadata.get(&key) {
    if let Ok(v) = value.as_u64() {
      return Some(v);
    }
    if let Ok(v) = value.as_u32() {
      return Some(v as u64);
    }
    if let Ok(v) = value.as_i64() {
      return Some(v as u64);
    }
    if let Ok(v) = value.as_i32() {
      return Some(v as u64);
    }
  }

  None
}

// =============================================================================
// Architecture Extraction
// =============================================================================

/// Extract model architecture information from GGUF metadata
pub fn extract_architecture(metadata: &GGUFMetadata, filename: &str) -> ModelArchitecture {
  ModelArchitecture {
    family: get_architecture_family(metadata),
    parameter_count: get_parameter_count(metadata),
    quantization: detect_quantization(filename, metadata),
    format: "gguf".to_string(),
  }
}

/// Get model architecture family (e.g., "llama", "phi", "mistral")
fn get_architecture_family(metadata: &GGUFMetadata) -> Option<String> {
  metadata
    .get("general.architecture")
    .and_then(|v| v.as_str().ok())
    .map(|s| s.to_string())
}

/// Get total parameter count
fn get_parameter_count(metadata: &GGUFMetadata) -> Option<u64> {
  if let Some(value) = metadata.get("general.parameter_count") {
    if let Ok(v) = value.as_u64() {
      return Some(v);
    }
    if let Ok(v) = value.as_i64() {
      return Some(v as u64);
    }
  }
  None
}

/// Detect quantization method with filename-first priority (research finding)
///
/// Priority (research-report.md Phase 1 finding):
/// 1. Parse from filename (highest reliability)
/// 2. Fall back to metadata key (general.quantization_version)
///
/// Filename parsing is more reliable than metadata keys which are often missing or inconsistent.
pub fn detect_quantization(filename: &str, metadata: &GGUFMetadata) -> Option<String> {
  // 1. Parse from filename (highest reliability per research)
  if let Some(quant) = parse_quantization_from_filename(filename) {
    return Some(quant);
  }

  // 2. Fall back to metadata key
  metadata
    .get("general.quantization_version")
    .and_then(|v| v.as_str().ok())
    .map(|s| s.to_string())
}

/// Parse quantization from filename (research-validated patterns)
fn parse_quantization_from_filename(filename: &str) -> Option<String> {
  let filename_lower = filename.to_lowercase();

  for (pattern, quant_name) in QUANTIZATION_PATTERNS {
    if filename_lower.contains(pattern) {
      return Some(quant_name.to_string());
    }
  }

  None
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Extract chat template from GGUF metadata
pub fn get_chat_template(metadata: &GGUFMetadata) -> Option<String> {
  metadata
    .get("tokenizer.chat_template")
    .and_then(|v| v.as_str().ok())
    .map(|s| s.to_string())
}

/// Extract model name from GGUF metadata
fn get_model_name(metadata: &GGUFMetadata) -> Option<String> {
  metadata
    .get("general.name")
    .and_then(|v| v.as_str().ok())
    .map(|s| s.to_string())
}
