use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Gemini `Model` schema (see `openapi-gemini.json`).
#[derive(
  Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sea_orm::FromJsonQueryResult,
)]
#[serde(rename_all = "camelCase")]
pub struct GeminiModel {
  pub name: String,
  #[serde(default)]
  pub version: Option<String>,
  #[serde(default)]
  pub display_name: Option<String>,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default)]
  pub input_token_limit: Option<i64>,
  #[serde(default)]
  pub output_token_limit: Option<i64>,
  #[serde(default)]
  pub supported_generation_methods: Vec<String>,
  #[serde(default)]
  pub temperature: Option<f32>,
  #[serde(default)]
  pub max_temperature: Option<f32>,
  #[serde(default)]
  pub top_p: Option<f32>,
  #[serde(default)]
  pub top_k: Option<i32>,
  #[serde(default)]
  pub thinking: Option<bool>,
}

impl GeminiModel {
  pub fn model_id(&self) -> &str {
    self.name.strip_prefix("models/").unwrap_or(&self.name)
  }
}

#[cfg(test)]
mod tests {
  use super::GeminiModel;

  #[test]
  fn test_gemini_model_serde_roundtrip() {
    let json = r#"{
      "name": "models/gemini-2.5-flash",
      "displayName": "Gemini 2.5 Flash",
      "version": "001",
      "inputTokenLimit": 1000000,
      "outputTokenLimit": 8192,
      "supportedGenerationMethods": ["generateContent"],
      "maxTemperature": 2.0,
      "thinking": false
    }"#;

    let model: GeminiModel = serde_json::from_str(json).expect("deserialize");
    assert_eq!("models/gemini-2.5-flash", model.name);
    assert_eq!(Some("001".to_string()), model.version);
    assert_eq!(Some("Gemini 2.5 Flash".to_string()), model.display_name);
    assert_eq!(Some(1000000), model.input_token_limit);
    assert_eq!(Some(8192), model.output_token_limit);
    assert_eq!(vec!["generateContent"], model.supported_generation_methods);
    assert_eq!(Some(2.0), model.max_temperature);
    assert_eq!(Some(false), model.thinking);

    let serialized = serde_json::to_value(&model).expect("serialize");
    assert_eq!(
      "models/gemini-2.5-flash",
      serialized["name"].as_str().unwrap()
    );
    assert_eq!("001", serialized["version"].as_str().unwrap());
    let json_missing_version = r#"{"name": "models/gemini-2.5-flash"}"#;
    let m: GeminiModel = serde_json::from_str(json_missing_version).expect("deserialize");
    assert_eq!(None, m.version);
    assert!(
      serialized.get("baseModelId").is_none(),
      "baseModelId must not be serialized"
    );
  }

  #[test]
  fn test_gemini_model_id_strips_prefix() {
    let with_prefix = GeminiModel {
      name: "models/gemini-2.5-flash".to_string(),
      version: Some("001".to_string()),
      display_name: None,
      description: None,
      input_token_limit: None,
      output_token_limit: None,
      supported_generation_methods: vec![],
      temperature: None,
      max_temperature: None,
      top_p: None,
      top_k: None,
      thinking: None,
    };
    assert_eq!("gemini-2.5-flash", with_prefix.model_id());

    let without_prefix = GeminiModel {
      name: "gemini-2.5-flash".to_string(),
      ..with_prefix
    };
    assert_eq!("gemini-2.5-flash", without_prefix.model_id());
  }
}
