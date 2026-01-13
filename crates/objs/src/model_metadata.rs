use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Model metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelMetadata {
  pub capabilities: ModelCapabilities,
  pub context: ContextLimits,
  pub architecture: ModelArchitecture,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chat_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelCapabilities {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub vision: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub audio: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub thinking: Option<bool>,
  pub tools: ToolCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ToolCapabilities {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub function_calling: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub structured_output: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ContextLimits {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_input_tokens: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelArchitecture {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub family: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter_count: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quantization: Option<String>,
  pub format: String,
}
