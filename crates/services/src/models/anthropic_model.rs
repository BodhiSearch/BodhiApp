use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Whether a single capability is supported by the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CapabilitySupport {
  pub supported: bool,
}

/// Supported thinking type configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ThinkingTypes {
  pub adaptive: CapabilitySupport,
  pub enabled: CapabilitySupport,
}

/// Thinking capability and supported type configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ThinkingCapability {
  pub supported: bool,
  pub types: ThinkingTypes,
}

/// Context management capability details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ContextManagementCapability {
  pub clear_thinking_20251015: Option<CapabilitySupport>,
  pub clear_tool_uses_20250919: Option<CapabilitySupport>,
  pub compact_20260112: Option<CapabilitySupport>,
}

/// Effort (reasoning_effort) capability details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct EffortCapability {
  pub high: CapabilitySupport,
  pub low: CapabilitySupport,
  pub max: CapabilitySupport,
}

/// Model capability information (Anthropic ModelCapabilities schema).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AnthropicModelCapabilities {
  pub batch: CapabilitySupport,
  pub citations: CapabilitySupport,
  pub code_execution: CapabilitySupport,
  pub context_management: ContextManagementCapability,
  pub effort: EffortCapability,
  pub image_input: CapabilitySupport,
  pub pdf_input: CapabilitySupport,
  pub structured_outputs: CapabilitySupport,
  pub thinking: ThinkingCapability,
}

/// Mirrors Anthropic's `ModelInfo` schema — full model metadata returned by
/// `GET /anthropic/v1/models` and stored alongside model IDs in `ApiAlias.models`.
///
/// **IMPORTANT**: Do NOT add `#[serde(skip_serializing_if = "Option::is_none")]` on the
/// `Option` fields below. The Anthropic API spec marks `capabilities`, `max_input_tokens`,
/// and `max_tokens` as **required** (nullable via `anyOf [T, null]`). They must serialize
/// as `null` when absent, not be omitted from the JSON output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AnthropicModel {
  pub id: String,
  pub display_name: String,
  /// RFC 3339 datetime string representing the model's release date.
  pub created_at: String,
  pub capabilities: Option<AnthropicModelCapabilities>,
  pub max_input_tokens: Option<i64>,
  pub max_tokens: Option<i64>,
  /// Always `"model"` — included for Anthropic API compatibility.
  #[serde(rename = "type")]
  pub model_type: String,
}
