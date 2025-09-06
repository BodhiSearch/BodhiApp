use crate::{AliasSource, ApiAlias, ModelAlias, UserAlias};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Flat enum representing all types of model aliases
/// Each variant is identified by the source field
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum Alias {
  /// User-defined local model (source: "user")
  #[serde(rename = "user")]
  User(UserAlias),
  /// Auto-discovered local model (source: "model")
  #[serde(rename = "model")]
  Model(ModelAlias),
  /// Remote API model (source: "api")
  #[serde(rename = "api")]
  Api(ApiAlias),
}

impl Alias {
  /// Check if this alias can serve the requested model
  pub fn can_serve(&self, model: &str) -> bool {
    match self {
      Alias::User(alias) => alias.alias == model,
      Alias::Model(alias) => alias.alias == model,
      Alias::Api(api_alias) => api_alias.models.contains(&model.to_string()),
    }
  }

  /// Get the alias name for this model
  pub fn alias_name(&self) -> &str {
    match self {
      Alias::User(alias) => &alias.alias,
      Alias::Model(alias) => &alias.alias,
      Alias::Api(api_alias) => &api_alias.id,
    }
  }

  /// Get the source of this alias
  pub fn source(&self) -> AliasSource {
    match self {
      Alias::User(_) => AliasSource::User,
      Alias::Model(_) => AliasSource::Model,
      Alias::Api(_) => AliasSource::Api,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{AliasSource, ModelAliasBuilder, Repo, UserAliasBuilder};
  use anyhow::Result;
  use chrono::Utc;
  use std::str::FromStr;

  #[test]
  fn test_model_alias_user_can_serve() {
    let alias = UserAliasBuilder::default()
      .alias("llama3:instruct")
      .repo(Repo::from_str("test/llama3").unwrap())
      .filename("llama3.gguf")
      .snapshot("main")
      .build()
      .unwrap();

    let model_alias = Alias::User(alias);

    assert!(model_alias.can_serve("llama3:instruct"));
    assert!(!model_alias.can_serve("other:model"));
    assert_eq!(model_alias.alias_name(), "llama3:instruct");
  }

  #[test]
  fn test_model_alias_model_can_serve() {
    let alias = ModelAliasBuilder::default()
      .alias("testalias:instruct")
      .repo(Repo::from_str("test/testalias").unwrap())
      .filename("testalias.gguf")
      .snapshot("main")
      .build()
      .unwrap();

    let model_alias = Alias::Model(alias);

    assert!(model_alias.can_serve("testalias:instruct"));
    assert!(!model_alias.can_serve("llama3:instruct"));
    assert_eq!(model_alias.alias_name(), "testalias:instruct");
    assert_eq!(model_alias.source(), AliasSource::Model);
  }

  #[test]
  fn test_model_alias_api_can_serve() {
    let api_alias = ApiAlias::new(
      "openai",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      None,
      Utc::now(),
    );

    let model_alias = Alias::Api(api_alias);

    assert!(model_alias.can_serve("gpt-4"));
    assert!(model_alias.can_serve("gpt-3.5-turbo"));
    assert!(!model_alias.can_serve("claude-3-opus"));
    assert_eq!(model_alias.alias_name(), "openai");
  }

  #[test]
  fn test_model_alias_serialization() -> Result<()> {
    // Test User variant
    let user_alias = UserAliasBuilder::default()
      .alias("llama3:instruct")
      .repo(Repo::from_str("test/llama3").unwrap())
      .filename("llama3.gguf")
      .snapshot("main")
      .build()
      .unwrap();
    let user_model = Alias::User(user_alias);

    let user_json = serde_json::to_string(&user_model)?;
    let user_deserialized: Alias = serde_json::from_str(&user_json)?;
    assert_eq!(user_model, user_deserialized);

    // Test Model variant
    let model_alias = ModelAliasBuilder::default()
      .alias("auto:model")
      .repo(Repo::from_str("test/auto").unwrap())
      .filename("auto.gguf")
      .snapshot("main")
      .build()
      .unwrap();
    let model_model = Alias::Model(model_alias);

    let model_json = serde_json::to_string(&model_model)?;
    let model_deserialized: Alias = serde_json::from_str(&model_json)?;

    // With tagged enum, Model variant should deserialize correctly
    assert_eq!(model_model, model_deserialized);
    assert!(model_deserialized.can_serve("auto:model"));
    assert_eq!(model_deserialized.alias_name(), "auto:model");
    assert_eq!(model_deserialized.source(), AliasSource::Model);

    // Test Api variant
    let api_alias = ApiAlias::new(
      "openai",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      None,
      Utc::now(),
    );
    let api_model = Alias::Api(api_alias);

    let api_json = serde_json::to_string(&api_model)?;
    let api_deserialized: Alias = serde_json::from_str(&api_json)?;
    assert_eq!(api_model, api_deserialized);

    Ok(())
  }

  #[test]
  fn test_model_alias_serde_tagged() -> Result<()> {
    // With tagged enum, the JSON includes a source field
    let api_alias = ApiAlias::new(
      "openai",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      None,
      Utc::now(),
    );
    let api_model = Alias::Api(api_alias.clone());

    // The tagged enum JSON should include the source field
    let model_json = serde_json::to_string(&api_model)?;
    assert!(model_json.contains("\"source\":\"api\""));

    // Deserializing should work correctly
    let deserialized: Alias = serde_json::from_str(&model_json)?;
    assert_eq!(api_model, deserialized);

    Ok(())
  }
}
